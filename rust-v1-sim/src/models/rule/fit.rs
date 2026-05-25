use std::{collections::HashMap, fs, path::PathBuf};

use super::{
    floquet::{
        default_rule_figure8_source_curve_file, load_rule_figure8_source_curves,
        rule_figure8_wave_number_normalization, rule_floquet_evaluation,
        rule_floquet_mode_defaults, rule_source_betas_for_modes,
        source_beta_values_to_model_cycles,
    },
    sweep::{rule_sweep_grid_defaults, rule_sweep_grid_details},
};
use crate::*;
pub(crate) fn rule_fit_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/rule-2011-fit-search.json");
    let mut grid = rule_sweep_grid_defaults("dense");
    let mut parameter_set = "rule_fig8_source_like";
    let mut curve_refine_steps = 48usize;
    let mut curve_refine_tolerance = 1.0e-6;
    let mut source_curve_file: Option<PathBuf> = None;
    let mut source_curve_comparison_enabled = true;
    let mut source_beta_per_model_cycle = RULE_FIGURE8_SOURCE_BETA_PER_MODEL_CYCLE;
    let mut max_trials = 25usize;
    let mut periods_override: Option<Vec<f64>> = None;
    let mut amplitudes_override: Option<Vec<f64>> = None;
    let mut stim_i_fractions_override: Option<Vec<f64>> = None;
    let mut period_min: Option<f64> = None;
    let mut period_max: Option<f64> = None;
    let mut period_steps: Option<usize> = None;
    let mut amplitude_min: Option<f64> = None;
    let mut amplitude_max: Option<f64> = None;
    let mut amplitude_steps: Option<usize> = None;
    let mut stim_i_fraction_min: Option<f64> = None;
    let mut stim_i_fraction_max: Option<f64> = None;
    let mut stim_i_fraction_steps: Option<usize> = None;
    let mut mode_cycles = rule_floquet_mode_defaults();
    let mut mode_min: Option<f64> = None;
    let mut mode_max: Option<f64> = None;
    let mut mode_steps: Option<usize> = None;
    let mut source_beta_modes_override: Option<Vec<f64>> = None;
    let mut source_beta_min: Option<f64> = None;
    let mut source_beta_max: Option<f64> = None;
    let mut source_beta_steps: Option<usize> = None;
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--max-trials" => {
                max_trials = parse_clamped_usize(
                    iter.next().ok_or("--max-trials requires a value")?,
                    1,
                    101,
                )?;
            }
            "--preset-grid" | "--grid" => {
                grid =
                    rule_sweep_grid_defaults(iter.next().ok_or("--preset-grid requires a value")?);
            }
            "--rule-parameter-set" | "--parameter-set" => {
                parameter_set = parse_rule_parameter_set(
                    iter.next().ok_or("--rule-parameter-set requires a value")?,
                )?;
            }
            "--curve-refine-steps" => {
                curve_refine_steps = parse_clamped_usize(
                    iter.next().ok_or("--curve-refine-steps requires a value")?,
                    1,
                    128,
                )?;
            }
            "--curve-refine-tolerance" => {
                curve_refine_tolerance = parse_clamped_f64(
                    iter.next()
                        .ok_or("--curve-refine-tolerance requires a value")?,
                    1.0e-12,
                    1.0e-2,
                )?;
            }
            "--source-curve-file" => {
                source_curve_file = Some(PathBuf::from(
                    iter.next().ok_or("--source-curve-file requires a value")?,
                ));
            }
            "--no-source-curve-comparison" => {
                source_curve_comparison_enabled = false;
            }
            "--figure8-source-beta-per-model-cycle"
            | "--source-beta-per-model-cycle"
            | "--figure8-beta-scale" => {
                source_beta_per_model_cycle = parse_clamped_f64(
                    iter.next()
                        .ok_or("--figure8-source-beta-per-model-cycle requires a value")?,
                    1.0e-6,
                    8.0,
                )?;
            }
            "--figure8-model-cycles-per-source-beta" | "--model-cycles-per-source-beta" => {
                let cycles_per_source_beta = parse_clamped_f64(
                    iter.next()
                        .ok_or("--figure8-model-cycles-per-source-beta requires a value")?,
                    0.01,
                    1.0e6,
                )?;
                source_beta_per_model_cycle = 1.0 / cycles_per_source_beta;
            }
            "--periods" => {
                periods_override = Some(parse_f64_csv(
                    iter.next().ok_or("--periods requires a value")?,
                    20.0,
                    180.0,
                )?);
            }
            "--period-min" => {
                period_min = Some(parse_clamped_f64(
                    iter.next().ok_or("--period-min requires a value")?,
                    20.0,
                    180.0,
                )?);
            }
            "--period-max" => {
                period_max = Some(parse_clamped_f64(
                    iter.next().ok_or("--period-max requires a value")?,
                    20.0,
                    180.0,
                )?);
            }
            "--period-steps" => {
                period_steps = Some(parse_clamped_usize(
                    iter.next().ok_or("--period-steps requires a value")?,
                    1,
                    61,
                )?);
            }
            "--amplitudes" => {
                amplitudes_override = Some(parse_f64_csv(
                    iter.next().ok_or("--amplitudes requires a value")?,
                    0.0,
                    1.5,
                )?);
            }
            "--amplitude-min" => {
                amplitude_min = Some(parse_clamped_f64(
                    iter.next().ok_or("--amplitude-min requires a value")?,
                    0.0,
                    1.5,
                )?);
            }
            "--amplitude-max" => {
                amplitude_max = Some(parse_clamped_f64(
                    iter.next().ok_or("--amplitude-max requires a value")?,
                    0.0,
                    1.5,
                )?);
            }
            "--amplitude-steps" => {
                amplitude_steps = Some(parse_clamped_usize(
                    iter.next().ok_or("--amplitude-steps requires a value")?,
                    1,
                    41,
                )?);
            }
            "--stim-i-fractions" | "--inhibitory-drive" => {
                stim_i_fractions_override = Some(parse_f64_csv(
                    iter.next().ok_or("--stim-i-fractions requires a value")?,
                    0.0,
                    1.0,
                )?);
            }
            "--stim-i-fraction-min" | "--inhibitory-drive-min" => {
                stim_i_fraction_min = Some(parse_clamped_f64(
                    iter.next()
                        .ok_or("--stim-i-fraction-min requires a value")?,
                    0.0,
                    1.0,
                )?);
            }
            "--stim-i-fraction-max" | "--inhibitory-drive-max" => {
                stim_i_fraction_max = Some(parse_clamped_f64(
                    iter.next()
                        .ok_or("--stim-i-fraction-max requires a value")?,
                    0.0,
                    1.0,
                )?);
            }
            "--stim-i-fraction-steps" | "--inhibitory-drive-steps" => {
                stim_i_fraction_steps = Some(parse_clamped_usize(
                    iter.next()
                        .ok_or("--stim-i-fraction-steps requires a value")?,
                    1,
                    21,
                )?);
            }
            "--modes" | "--mode-cycles" => {
                mode_cycles =
                    parse_f64_csv(iter.next().ok_or("--modes requires a value")?, 0.5, 32.0)?;
            }
            "--mode-min" => {
                mode_min = Some(parse_clamped_f64(
                    iter.next().ok_or("--mode-min requires a value")?,
                    0.05,
                    32.0,
                )?);
            }
            "--mode-max" => {
                mode_max = Some(parse_clamped_f64(
                    iter.next().ok_or("--mode-max requires a value")?,
                    0.05,
                    32.0,
                )?);
            }
            "--mode-steps" => {
                mode_steps = Some(parse_clamped_usize(
                    iter.next().ok_or("--mode-steps requires a value")?,
                    1,
                    257,
                )?);
            }
            "--source-beta-modes" | "--figure8-source-beta-modes" => {
                source_beta_modes_override = Some(parse_f64_csv(
                    iter.next().ok_or("--source-beta-modes requires a value")?,
                    0.0,
                    8.0,
                )?);
            }
            "--source-beta-min" | "--figure8-source-beta-min" => {
                source_beta_min = Some(parse_clamped_f64(
                    iter.next().ok_or("--source-beta-min requires a value")?,
                    0.0,
                    8.0,
                )?);
            }
            "--source-beta-max" | "--figure8-source-beta-max" => {
                source_beta_max = Some(parse_clamped_f64(
                    iter.next().ok_or("--source-beta-max requires a value")?,
                    0.0,
                    8.0,
                )?);
            }
            "--source-beta-steps" | "--figure8-source-beta-steps" => {
                source_beta_steps = Some(parse_clamped_usize(
                    iter.next().ok_or("--source-beta-steps requires a value")?,
                    1,
                    257,
                )?);
            }
            "--no-trim-warmup" => {
                raw.insert("trim_warmup".to_string(), "false".to_string());
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    apply_rule_parameter_set(&mut raw, parameter_set);
    let model_domain_points = raw_usize(&raw, "n")
        .map(|n| n.clamp(32, 96))
        .unwrap_or(grid.n);
    let wave_number_normalization =
        rule_figure8_wave_number_normalization(source_beta_per_model_cycle, model_domain_points);

    if let Some(source_betas) = source_beta_modes_override {
        mode_cycles = source_beta_values_to_model_cycles(source_betas, wave_number_normalization);
    } else if source_beta_min.is_some() || source_beta_max.is_some() || source_beta_steps.is_some()
    {
        let current_source_betas =
            rule_source_betas_for_modes(&mode_cycles, wave_number_normalization);
        mode_cycles = source_beta_values_to_model_cycles(
            linspace_values(
                source_beta_min.unwrap_or_else(|| *current_source_betas.first().unwrap_or(&0.0)),
                source_beta_max.unwrap_or_else(|| *current_source_betas.last().unwrap_or(&1.0)),
                source_beta_steps.unwrap_or(mode_cycles.len()),
            ),
            wave_number_normalization,
        );
    } else if mode_min.is_some() || mode_max.is_some() || mode_steps.is_some() {
        mode_cycles = linspace_values(
            mode_min.unwrap_or_else(|| *mode_cycles.first().unwrap_or(&0.5)),
            mode_max.unwrap_or_else(|| *mode_cycles.last().unwrap_or(&4.0)),
            mode_steps.unwrap_or(mode_cycles.len()),
        );
    }
    if let Some(periods) = periods_override {
        grid.periods = periods;
    } else if period_min.is_some() || period_max.is_some() || period_steps.is_some() {
        grid.periods = linspace_values(
            period_min.unwrap_or_else(|| *grid.periods.first().unwrap_or(&40.0)),
            period_max.unwrap_or_else(|| *grid.periods.last().unwrap_or(&160.0)),
            period_steps.unwrap_or(grid.periods.len()),
        );
    }
    if let Some(amplitudes) = amplitudes_override {
        grid.amplitudes = amplitudes;
    } else if amplitude_min.is_some() || amplitude_max.is_some() || amplitude_steps.is_some() {
        grid.amplitudes = linspace_values(
            amplitude_min.unwrap_or_else(|| *grid.amplitudes.first().unwrap_or(&0.4)),
            amplitude_max.unwrap_or_else(|| *grid.amplitudes.last().unwrap_or(&1.2)),
            amplitude_steps.unwrap_or(grid.amplitudes.len()),
        );
    }
    if let Some(stim_i_fractions) = stim_i_fractions_override {
        grid.stim_i_fractions = stim_i_fractions;
    } else if stim_i_fraction_min.is_some()
        || stim_i_fraction_max.is_some()
        || stim_i_fraction_steps.is_some()
    {
        grid.stim_i_fractions = linspace_values(
            stim_i_fraction_min.unwrap_or_else(|| *grid.stim_i_fractions.first().unwrap_or(&0.0)),
            stim_i_fraction_max.unwrap_or_else(|| *grid.stim_i_fractions.last().unwrap_or(&0.0)),
            stim_i_fraction_steps.unwrap_or(grid.stim_i_fractions.len()),
        );
    }

    let source_curve_file = if source_curve_comparison_enabled {
        Some(source_curve_file.unwrap_or_else(|| default_rule_figure8_source_curve_file(&out)))
    } else {
        None
    };
    let source_curves = match source_curve_file.as_ref() {
        Some(path) if path.exists() => Some(load_rule_figure8_source_curves(path)?),
        _ => None,
    };
    let curve_refinement = RuleFloquetCurveRefinement {
        method: "bisection_on_beta_sign_change",
        tolerance: curve_refine_tolerance,
        max_steps: curve_refine_steps,
    };

    let trial_specs = default_rule_figure8_fit_trials(&raw, max_trials);
    let mut trials = trial_specs
        .iter()
        .map(|spec| {
            rule_figure8_fit_trial(RuleFigure8FitTrialConfig {
                spec,
                baseline_raw: &raw,
                grid: &grid,
                mode_cycles: &mode_cycles,
                curve_refinement,
                wave_number_normalization,
                source_curves: source_curves.as_ref(),
                source_curve_file: source_curve_file.as_ref(),
            })
        })
        .collect::<Vec<_>>();
    trials.sort_by(|a, b| {
        trial_score(a)
            .unwrap_or(f64::INFINITY)
            .total_cmp(&trial_score(b).unwrap_or(f64::INFINITY))
    });
    let best_trial_id = trials.first().and_then(|trial| {
        trial
            .source_curve_comparison
            .fit_objective
            .as_ref()
            .map(|_| trial.trial_id.clone())
    });
    let best_score = trials.first().and_then(trial_score);
    let report = RuleFigure8FitSearchReport {
        format: "rule-2011-figure8-fit-search-v2",
        model_family: MODEL_FAMILY_RULE,
        source_key: "rule-2011",
        status: if best_score.is_some() {
            "scored-parameter-trials"
        } else {
            "source-comparison-unavailable"
        },
        note: "Small one-parameter-at-a-time Rule Figure 8 calibration search. Trials are scored against digitized source curves with the configured zero-offset domain beta normalization; identity, scale-only, and affine generated-beta-to-source-beta fits remain diagnostics, and the baseline preset is not modified by this report.",
        baseline_parameter_set: parameter_set,
        source_curve_file: source_curve_file
            .as_ref()
            .map(|path| path.display().to_string()),
        wave_number_normalization,
        grid: rule_sweep_grid_details(&grid),
        mode_source_betas: rule_source_betas_for_modes(&mode_cycles, wave_number_normalization),
        mode_cycles,
        curve_refinement,
        trial_count: trials.len(),
        best_trial_id,
        best_score,
        trials,
    };
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} trials={} best_score={}",
        out.display(),
        report.trial_count,
        report
            .best_score
            .map(|score| format!("{score:.6}"))
            .unwrap_or_else(|| "n/a".to_string())
    );
    Ok(())
}

fn default_rule_figure8_fit_trials(
    baseline_raw: &HashMap<String, String>,
    max_trials: usize,
) -> Vec<RuleFigure8FitTrialSpec> {
    let mut trials = vec![RuleFigure8FitTrialSpec {
        trial_id: "baseline".to_string(),
        parameter_changes: BTreeMap::new(),
    }];
    for scan in rule_figure8_fit_parameter_scans() {
        let Some(base_value) = raw_f64(baseline_raw, scan.key) else {
            continue;
        };
        for factor in scan.factors {
            let value = (base_value * factor).clamp(scan.min, scan.max);
            if (value - base_value).abs() <= 1.0e-9 {
                continue;
            }
            let mut parameter_changes = BTreeMap::new();
            parameter_changes.insert(scan.key.to_string(), format_rule_fit_value(value));
            trials.push(RuleFigure8FitTrialSpec {
                trial_id: format!(
                    "{}_x{}",
                    scan.key.trim_start_matches("rule_"),
                    format_rule_fit_factor(factor)
                ),
                parameter_changes,
            });
            if trials.len() >= max_trials {
                return trials;
            }
        }
    }
    trials
}

fn rule_figure8_fit_parameter_scans() -> Vec<RuleFigure8FitParameterScan> {
    vec![
        RuleFigure8FitParameterScan {
            key: "rule_tau_e_ms",
            factors: [0.85, 1.15],
            min: 2.0,
            max: 80.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_tau_i_ms",
            factors: [0.85, 1.15],
            min: 2.0,
            max: 120.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_aee",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 30.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_aei",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 30.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_aie",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 30.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_aii",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 30.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_sigma_e",
            factors: [0.85, 1.15],
            min: 0.25,
            max: 20.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_sigma_i",
            factors: [0.85, 1.15],
            min: 0.25,
            max: 25.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_theta_e",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 10.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_theta_i",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 10.0,
        },
        RuleFigure8FitParameterScan {
            key: "rule_stim_threshold",
            factors: [0.9, 1.1],
            min: 0.0,
            max: 0.98,
        },
    ]
}

fn rule_figure8_fit_trial(config: RuleFigure8FitTrialConfig<'_>) -> RuleFigure8FitTrial {
    let mut trial_raw = config.baseline_raw.clone();
    for (key, value) in &config.spec.parameter_changes {
        trial_raw.insert(key.clone(), value.clone());
    }
    let evaluation = rule_floquet_evaluation(RuleFloquetEvaluationConfig {
        raw: &trial_raw,
        grid: config.grid,
        mode_cycles: config.mode_cycles,
        curve_refinement: config.curve_refinement,
        wave_number_normalization: config.wave_number_normalization,
        source_curves: config.source_curves,
        source_curve_file: config.source_curve_file,
    });
    let boundary_curve_point_count = evaluation
        .boundary_curves
        .iter()
        .map(|curve| curve.point_count)
        .sum::<usize>();
    let status = if evaluation.source_curve_comparison.fit_objective.is_some() {
        "scored"
    } else {
        evaluation.source_curve_comparison.status
    };
    RuleFigure8FitTrial {
        trial_id: config.spec.trial_id.clone(),
        status,
        parameter_changes: config.spec.parameter_changes.clone(),
        parameter_values: tracked_rule_fit_parameter_values(&trial_raw),
        floquet_point_count: evaluation.points.len(),
        boundary_candidate_count: evaluation.boundary_candidates.len(),
        boundary_curve_count: evaluation.boundary_curves.len(),
        boundary_curve_point_count,
        source_curve_comparison: evaluation.source_curve_comparison,
    }
}

fn trial_score(trial: &RuleFigure8FitTrial) -> Option<f64> {
    trial
        .source_curve_comparison
        .fit_objective
        .as_ref()
        .map(|objective| objective.score)
}

fn tracked_rule_fit_parameter_values(raw: &HashMap<String, String>) -> BTreeMap<String, String> {
    [
        "rule_tau_e_ms",
        "rule_tau_i_ms",
        "rule_aee",
        "rule_aei",
        "rule_aie",
        "rule_aii",
        "rule_sigma_e",
        "rule_sigma_i",
        "rule_theta_e",
        "rule_theta_i",
        "rule_stim_threshold",
        "rule_stim_smoothing",
    ]
    .into_iter()
    .filter_map(|key| raw.get(key).map(|value| (key.to_string(), value.clone())))
    .collect()
}

fn raw_f64(raw: &HashMap<String, String>, key: &str) -> Option<f64> {
    raw.get(key)?.parse::<f64>().ok()
}

fn format_rule_fit_value(value: f64) -> String {
    let mut out = format!("{value:.6}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

fn format_rule_fit_factor(factor: f64) -> String {
    format!("{factor:.2}").replace('.', "p")
}
