use std::{collections::HashMap, fs, path::PathBuf};

use super::sweep::{rule_sweep_grid_defaults, rule_sweep_grid_details, rule_sweep_params};
use super::{
    rule_gaussian_kernel, rule_rest_state, rule_sigmoid, rule_stimulus, RuleGaussianKernel,
};
use crate::*;
pub(crate) fn rule_floquet_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/rule-2011-floquet.json");
    let mut grid = rule_sweep_grid_defaults("dense");
    let mut parameter_set = "rule_fig8_source_like";
    let mut curve_refine_steps = 48usize;
    let mut curve_refine_tolerance = 1.0e-6;
    let mut source_curve_file: Option<PathBuf> = None;
    let mut source_curve_comparison_enabled = true;
    let mut source_beta_per_model_cycle = RULE_FIGURE8_SOURCE_BETA_PER_MODEL_CYCLE;
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

    let evaluation = rule_floquet_evaluation(RuleFloquetEvaluationConfig {
        raw: &raw,
        grid: &grid,
        mode_cycles: &mode_cycles,
        curve_refinement,
        wave_number_normalization,
        source_curves: source_curves.as_ref(),
        source_curve_file: source_curve_file.as_ref(),
    });
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let point_count = evaluation.points.len();
    let boundary_count = evaluation.boundary_candidates.len();
    let curve_count = evaluation.boundary_curves.len();
    let report = RuleFloquetCalibrationReport {
        format: "rule-2011-floquet-calibration-v5",
        model_family: MODEL_FAMILY_RULE,
        source_key: "rule-2011",
        parameter_set,
        status: "figure8-refined-beta-boundary-curves",
        note: "Homogeneous-orbit monodromy grid for Rule et al. 2011 Figure 8 style diagnostics. The mode grid resolves true +1/-1 sign-change crossings and refines beta-axis roots into source-axis boundary curves; the curves are still numerical calibration targets, not a final published-figure reproduction.",
        source_axes: RuleFloquetSourceAxes {
            x_axis: "forcing_period",
            x_units: "ms",
            x_secondary_axis: "stimulus_frequency",
            x_secondary_units: "Hz",
            y_axis: "wave_number",
            y_units: "radians_per_domain",
            y_secondary_axis: "beta_cycles",
            y_secondary_units: "cycles_per_domain",
        },
        wave_number_normalization,
        curve_refinement,
        source_curve_comparison: evaluation.source_curve_comparison,
        grid: rule_sweep_grid_details(&grid),
        mode_source_betas: rule_source_betas_for_modes(&mode_cycles, wave_number_normalization),
        mode_cycles,
        points: evaluation.points,
        boundary_candidates: evaluation.boundary_candidates,
        boundary_curves: evaluation.boundary_curves,
    };
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} floquet_points={point_count} boundary_candidates={boundary_count} boundary_curves={curve_count}",
        out.display()
    );
    Ok(())
}

pub(crate) fn default_rule_figure8_source_curve_file(out: &Path) -> PathBuf {
    out.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("source-curves")
        .join("rule-2011-fig8-source-curves.json")
}

pub(crate) fn rule_figure8_wave_number_normalization(
    source_beta_per_model_cycle: f64,
    model_domain_points: usize,
) -> RuleFigure8WaveNumberNormalization {
    let scale = source_beta_per_model_cycle.max(1.0e-12);
    RuleFigure8WaveNumberNormalization {
        model: "zero_offset_domain_scale",
        decision: "source_beta = source_beta_per_model_cycle * model_beta_cycles",
        internal_wave_number_formula: "q_cell = 2*pi*model_beta_cycles/model_domain_points",
        source_beta_per_model_cycle: scale,
        model_cycles_per_source_beta: 1.0 / scale,
        source_beta_offset: 0.0,
        model_domain_points,
        note: "The affine beta fit remains diagnostic only; the calibrated source-axis comparison uses a zero-offset domain scale because beta=0 must map to zero wave number.",
    }
}

pub(crate) fn rule_source_beta_for_model_cycles(
    beta_cycles: f64,
    normalization: RuleFigure8WaveNumberNormalization,
) -> f64 {
    normalization.source_beta_per_model_cycle * beta_cycles + normalization.source_beta_offset
}

pub(crate) fn rule_model_cycles_for_source_beta(
    source_beta: f64,
    normalization: RuleFigure8WaveNumberNormalization,
) -> f64 {
    (source_beta - normalization.source_beta_offset) * normalization.model_cycles_per_source_beta
}

pub(crate) fn rule_source_betas_for_modes(
    mode_cycles: &[f64],
    normalization: RuleFigure8WaveNumberNormalization,
) -> Vec<f64> {
    mode_cycles
        .iter()
        .map(|cycles| rule_source_beta_for_model_cycles(*cycles, normalization))
        .collect()
}

pub(crate) fn source_beta_values_to_model_cycles(
    values: Vec<f64>,
    normalization: RuleFigure8WaveNumberNormalization,
) -> Vec<f64> {
    values
        .into_iter()
        .map(|source_beta| rule_model_cycles_for_source_beta(source_beta, normalization))
        .collect()
}

pub(crate) fn rule_floquet_evaluation(
    config: RuleFloquetEvaluationConfig<'_>,
) -> RuleFloquetEvaluation {
    let mut points = Vec::new();
    for period in &config.grid.periods {
        for amplitude in &config.grid.amplitudes {
            for stim_i_fraction in &config.grid.stim_i_fractions {
                let params = rule_sweep_params(
                    config.raw,
                    config.grid,
                    *period,
                    *amplitude,
                    *stim_i_fraction,
                );
                points.push(rule_floquet_grid_point_for(params, config.mode_cycles));
            }
        }
    }

    let boundary_candidates = rule_floquet_boundary_candidates(
        &points,
        &config.grid.periods,
        &config.grid.amplitudes,
        &config.grid.stim_i_fractions,
    );
    let mut boundary_curves = rule_floquet_beta_boundary_curves(
        &points,
        config.raw,
        config.grid,
        config.curve_refinement.tolerance,
        config.curve_refinement.max_steps,
    );
    let source_curve_comparison = apply_rule_figure8_source_comparison(
        &mut boundary_curves,
        config.wave_number_normalization,
        config.source_curves,
        config.source_curve_file,
    );

    RuleFloquetEvaluation {
        points,
        boundary_candidates,
        boundary_curves,
        source_curve_comparison,
    }
}

pub(crate) fn rule_floquet_mode_defaults() -> Vec<f64> {
    linspace_values(0.5, 4.0, 15)
}

#[derive(Clone, Copy, Debug)]
struct RuleOrbitStep {
    dt: f64,
    time_ms: f64,
    ue: f64,
    ui: f64,
}

pub(crate) fn rule_floquet_report(params: FrameParams, mode_cycles: &[f64]) -> RuleFloquetReport {
    let (orbit_params, orbit, steps) = rule_floquet_orbit_steps(params);
    let modes = mode_cycles
        .iter()
        .map(|cycles| rule_floquet_mode(orbit_params, &steps, *cycles))
        .collect::<Vec<_>>();
    let strongest_mode = modes
        .iter()
        .copied()
        .max_by(|a, b| a.max_abs_multiplier.total_cmp(&b.max_abs_multiplier))
        .unwrap_or(RuleFloquetMode {
            beta_cycles: 0.0,
            wave_number_radians: 0.0,
            multiplier_1_real: 0.0,
            multiplier_1_imag: 0.0,
            multiplier_2_real: 0.0,
            multiplier_2_imag: 0.0,
            max_abs_multiplier: 0.0,
            monodromy_trace: 0.0,
            monodromy_determinant: 0.0,
            plus_condition: 1.0,
            minus_condition: 1.0,
            determinant_condition: 1.0,
            crossing_hint: "no-modes",
        });
    let plus_crossing_modes = modes
        .iter()
        .filter(|mode| mode.crossing_hint == "plus_one_to_one")
        .map(|mode| mode.beta_cycles)
        .collect();
    let minus_crossing_modes = modes
        .iter()
        .filter(|mode| mode.crossing_hint == "minus_period_doubling")
        .map(|mode| mode.beta_cycles)
        .collect();

    RuleFloquetReport {
        period_ms: orbit_params.rule_stim_period_ms,
        amplitude: orbit_params.rule_stim_amplitude,
        stim_i_fraction: orbit_params.rule_stim_i_fraction,
        orbit,
        modes,
        strongest_mode,
        plus_crossing_modes,
        minus_crossing_modes,
    }
}

fn rule_floquet_orbit_steps(
    params: FrameParams,
) -> (FrameParams, RuleOrbitSummary, Vec<RuleOrbitStep>) {
    let mut orbit_params = params;
    orbit_params.rule_seed_strength = 0.0;
    orbit_params.n = orbit_params.n.max(32);
    let period = orbit_params.rule_stim_period_ms.max(1.0e-9);
    let dt = orbit_params.preview_step.min(0.1).clamp(0.02, 0.1);
    let (mut ue, mut ui) = rule_rest_state(orbit_params);
    let mut time_ms = 0.0;
    let warmup_end = period * 14.0;
    while time_ms + 1.0e-12 < warmup_end {
        let step = dt.min(warmup_end - time_ms);
        (ue, ui) = rule_homogeneous_step(orbit_params, ue, ui, time_ms, step);
        time_ms += step;
    }

    let mut steps = Vec::new();
    let orbit_end = warmup_end + period;
    while time_ms + 1.0e-12 < orbit_end {
        let step = dt.min(orbit_end - time_ms);
        steps.push(RuleOrbitStep {
            dt: step,
            time_ms,
            ue,
            ui,
        });
        (ue, ui) = rule_homogeneous_step(orbit_params, ue, ui, time_ms, step);
        time_ms += step;
    }

    let orbit = summarize_rule_orbit(period, &steps);
    (orbit_params, orbit, steps)
}

pub(crate) fn rule_floquet_grid_point_for(
    params: FrameParams,
    mode_cycles: &[f64],
) -> RuleFloquetGridPoint {
    let report = rule_floquet_report(params, mode_cycles);
    let plus_margin = max_floquet_margin(&report.modes, "plus_one_to_one");
    let minus_margin = max_floquet_margin(&report.modes, "minus_period_doubling");
    let complex_margin = max_floquet_margin(&report.modes, "unstable_complex");
    let crossing_hint = if minus_margin > 0.0 {
        "minus_period_doubling"
    } else if plus_margin > 0.0 {
        "plus_one_to_one"
    } else if complex_margin > 0.0 {
        "unstable_complex"
    } else {
        "stable"
    };

    RuleFloquetGridPoint {
        period_ms: report.period_ms,
        amplitude: report.amplitude,
        stim_i_fraction: report.stim_i_fraction,
        dominant_beta_cycles: report.strongest_mode.beta_cycles,
        max_abs_multiplier: report.strongest_mode.max_abs_multiplier,
        crossing_hint,
        plus_margin,
        minus_margin,
        complex_margin,
        orbit: report.orbit,
        modes: report.modes,
    }
}

pub(crate) fn rule_floquet_boundary_candidates(
    points: &[RuleFloquetGridPoint],
    periods: &[f64],
    amplitudes: &[f64],
    stim_i_fractions: &[f64],
) -> Vec<RuleFloquetBoundaryCandidate> {
    let periods = sorted_unique_f64(periods);
    let amplitudes = sorted_unique_f64(amplitudes);
    let stim_i_fractions = sorted_unique_f64(stim_i_fractions);
    let mut candidates = Vec::new();
    for stim_i_fraction in &stim_i_fractions {
        for amplitude in &amplitudes {
            for pair in periods.windows(2) {
                if let (Some(from), Some(to)) = (
                    find_rule_floquet_point(points, pair[0], *amplitude, *stim_i_fraction),
                    find_rule_floquet_point(points, pair[1], *amplitude, *stim_i_fraction),
                ) {
                    candidates.extend(rule_floquet_boundary_between(from, to, "period"));
                }
            }
        }
        for period in &periods {
            for pair in amplitudes.windows(2) {
                if let (Some(from), Some(to)) = (
                    find_rule_floquet_point(points, *period, pair[0], *stim_i_fraction),
                    find_rule_floquet_point(points, *period, pair[1], *stim_i_fraction),
                ) {
                    candidates.extend(rule_floquet_boundary_between(from, to, "amplitude"));
                }
            }
        }
    }
    candidates.extend(rule_floquet_beta_boundary_candidates(points));
    candidates.extend(rule_floquet_nearest_boundary_candidates(points, 6));
    candidates.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| a.period_ms.total_cmp(&b.period_ms))
            .then_with(|| a.amplitude.total_cmp(&b.amplitude))
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    candidates
}

fn sorted_unique_f64(values: &[f64]) -> Vec<f64> {
    let mut values = values.to_vec();
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup_by(|a, b| (*a - *b).abs() < 1.0e-9);
    values
}

fn find_rule_floquet_point(
    points: &[RuleFloquetGridPoint],
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
) -> Option<&RuleFloquetGridPoint> {
    points.iter().find(|point| {
        (point.period_ms - period_ms).abs() < 1.0e-6
            && (point.amplitude - amplitude).abs() < 1.0e-6
            && (point.stim_i_fraction - stim_i_fraction).abs() < 1.0e-6
    })
}

fn rule_floquet_boundary_between(
    from: &RuleFloquetGridPoint,
    to: &RuleFloquetGridPoint,
    axis: &'static str,
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for from_mode in &from.modes {
        let Some(to_mode) = to
            .modes
            .iter()
            .find(|mode| (mode.beta_cycles - from_mode.beta_cycles).abs() < 1.0e-6)
        else {
            continue;
        };
        for kind in KINDS {
            let margin_from = floquet_mode_margin(from_mode, kind);
            let margin_to = floquet_mode_margin(to_mode, kind);
            if !margin_from.is_finite() || !margin_to.is_finite() {
                continue;
            }
            let crosses =
                (margin_from <= 0.0 && margin_to > 0.0) || (margin_to <= 0.0 && margin_from > 0.0);
            if !crosses {
                continue;
            }
            let denom = margin_to - margin_from;
            let t = if denom.abs() < 1.0e-12 {
                0.5
            } else {
                (-margin_from / denom).clamp(0.0, 1.0)
            };
            candidates.push(RuleFloquetBoundaryCandidate {
                kind,
                evidence: "sign_change",
                beta_cycles: from_mode.beta_cycles,
                axis,
                period_ms: from.period_ms + (to.period_ms - from.period_ms) * t,
                amplitude: from.amplitude + (to.amplitude - from.amplitude) * t,
                stim_i_fraction: from.stim_i_fraction
                    + (to.stim_i_fraction - from.stim_i_fraction) * t,
                from_period_ms: from.period_ms,
                from_amplitude: from.amplitude,
                from_beta_cycles: from_mode.beta_cycles,
                to_period_ms: to.period_ms,
                to_amplitude: to.amplitude,
                to_beta_cycles: to_mode.beta_cycles,
                margin_from,
                margin_to,
                confidence: (margin_to - margin_from).abs().min(1.0),
            });
        }
    }
    candidates
}

fn rule_floquet_beta_boundary_candidates(
    points: &[RuleFloquetGridPoint],
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for point in points {
        let mut modes = point.modes.clone();
        modes.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        for pair in modes.windows(2) {
            let from_mode = pair[0];
            let to_mode = pair[1];
            for kind in KINDS {
                let margin_from = floquet_mode_margin(&from_mode, kind);
                let margin_to = floquet_mode_margin(&to_mode, kind);
                if !margin_from.is_finite() || !margin_to.is_finite() {
                    continue;
                }
                let crosses = (margin_from <= 0.0 && margin_to > 0.0)
                    || (margin_to <= 0.0 && margin_from > 0.0);
                if !crosses {
                    continue;
                }
                let denom = margin_to - margin_from;
                let t = if denom.abs() < 1.0e-12 {
                    0.5
                } else {
                    (-margin_from / denom).clamp(0.0, 1.0)
                };
                candidates.push(RuleFloquetBoundaryCandidate {
                    kind,
                    evidence: "sign_change",
                    beta_cycles: from_mode.beta_cycles
                        + (to_mode.beta_cycles - from_mode.beta_cycles) * t,
                    axis: "beta",
                    period_ms: point.period_ms,
                    amplitude: point.amplitude,
                    stim_i_fraction: point.stim_i_fraction,
                    from_period_ms: point.period_ms,
                    from_amplitude: point.amplitude,
                    from_beta_cycles: from_mode.beta_cycles,
                    to_period_ms: point.period_ms,
                    to_amplitude: point.amplitude,
                    to_beta_cycles: to_mode.beta_cycles,
                    margin_from,
                    margin_to,
                    confidence: (margin_to - margin_from).abs().min(1.0),
                });
            }
        }
    }
    candidates
}

fn rule_floquet_beta_boundary_curves(
    points: &[RuleFloquetGridPoint],
    raw: &HashMap<String, String>,
    grid: &RuleSweepGridConfig,
    tolerance: f64,
    max_steps: usize,
) -> Vec<RuleFloquetBoundaryCurve> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut refined_points = Vec::new();
    for point in points {
        let params = rule_sweep_params(
            raw,
            grid,
            point.period_ms,
            point.amplitude,
            point.stim_i_fraction,
        );
        let (orbit_params, _, steps) = rule_floquet_orbit_steps(params);
        let mut modes = point.modes.clone();
        modes.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        for pair in modes.windows(2) {
            let from_mode = pair[0];
            let to_mode = pair[1];
            for kind in KINDS {
                let margin_from = floquet_mode_margin(&from_mode, kind);
                let margin_to = floquet_mode_margin(&to_mode, kind);
                if !floquet_margins_cross(margin_from, margin_to) {
                    continue;
                }
                if let Some(refined) = refine_rule_floquet_beta_boundary(
                    orbit_params,
                    &steps,
                    kind,
                    from_mode.beta_cycles,
                    to_mode.beta_cycles,
                    tolerance,
                    max_steps,
                ) {
                    refined_points.push(refined);
                }
            }
        }
    }
    rule_floquet_boundary_curves_from_points(refined_points)
}

fn refine_rule_floquet_beta_boundary(
    params: FrameParams,
    steps: &[RuleOrbitStep],
    kind: &'static str,
    beta_low: f64,
    beta_high: f64,
    tolerance: f64,
    max_steps: usize,
) -> Option<RuleFloquetBoundaryCurvePoint> {
    let eval = |beta: f64| floquet_mode_margin(&rule_floquet_mode(params, steps, beta), kind);
    let (beta_cycles, margin, iterations) =
        refine_scalar_sign_change(beta_low, beta_high, tolerance, max_steps, eval)?;
    let bracket_low_beta_cycles = beta_low.min(beta_high);
    let bracket_high_beta_cycles = beta_low.max(beta_high);
    Some(RuleFloquetBoundaryCurvePoint {
        kind,
        branch_label: rule_floquet_branch_label(kind),
        branch_periodicity: rule_floquet_branch_periodicity(kind),
        axis: "beta",
        period_ms: params.rule_stim_period_ms,
        stimulus_frequency_hz: 1000.0 / params.rule_stim_period_ms.max(1.0e-9),
        amplitude: params.rule_stim_amplitude,
        stim_i_fraction: params.rule_stim_i_fraction,
        beta_cycles,
        wave_number_radians: rule_wave_number_for_cycles(beta_cycles, params.n),
        bracket_low_beta_cycles,
        bracket_high_beta_cycles,
        bracket_width_beta_cycles: bracket_high_beta_cycles - bracket_low_beta_cycles,
        margin,
        condition_value: -margin,
        iterations,
        residual_abs: margin.abs(),
    })
}

pub(crate) fn rule_floquet_branch_label(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "-1 period-doubling",
        "plus_one_to_one" => "+1 one-to-one",
        "unstable_complex" => "complex unit-circle",
        _ => "unknown",
    }
}

pub(crate) fn rule_floquet_branch_periodicity(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "2T",
        "plus_one_to_one" => "T",
        "unstable_complex" => "complex",
        _ => "unknown",
    }
}

pub(crate) fn refine_scalar_sign_change<F>(
    low: f64,
    high: f64,
    tolerance: f64,
    max_iterations: usize,
    mut eval: F,
) -> Option<(f64, f64, usize)>
where
    F: FnMut(f64) -> f64,
{
    let mut lo = low.min(high);
    let mut hi = low.max(high);
    let mut f_lo = eval(lo);
    let f_hi = eval(hi);
    if !floquet_margins_cross(f_lo, f_hi) {
        return None;
    }
    let mut best_x = lo;
    let mut best_f = f_lo;
    for iteration in 1..=max_iterations {
        let mid = 0.5 * (lo + hi);
        let f_mid = eval(mid);
        if f_mid.abs() < best_f.abs() {
            best_x = mid;
            best_f = f_mid;
        }
        if f_mid.abs() <= tolerance || (hi - lo).abs() <= tolerance {
            return Some((best_x, best_f, iteration));
        }
        if floquet_margins_cross(f_lo, f_mid) {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }
    Some((best_x, best_f, max_iterations))
}

fn floquet_margins_cross(from: f64, to: f64) -> bool {
    from.is_finite() && to.is_finite() && ((from <= 0.0 && to > 0.0) || (to <= 0.0 && from > 0.0))
}

fn rule_floquet_boundary_curves_from_points(
    mut points: Vec<RuleFloquetBoundaryCurvePoint>,
) -> Vec<RuleFloquetBoundaryCurve> {
    points.sort_by(|a, b| {
        a.kind
            .cmp(b.kind)
            .then_with(|| a.stim_i_fraction.total_cmp(&b.stim_i_fraction))
            .then_with(|| a.amplitude.total_cmp(&b.amplitude))
            .then_with(|| a.period_ms.total_cmp(&b.period_ms))
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    let mut curves: Vec<RuleFloquetBoundaryCurve> = Vec::new();
    let mut index = 0usize;
    while index < points.len() {
        let start = index;
        let key = (
            points[index].kind,
            points[index].amplitude,
            points[index].stim_i_fraction,
        );
        while index < points.len()
            && points[index].kind == key.0
            && (points[index].amplitude - key.1).abs() < 1.0e-9
            && (points[index].stim_i_fraction - key.2).abs() < 1.0e-9
        {
            index += 1;
        }
        let branches = split_rule_floquet_boundary_branches(points[start..index].to_vec());
        for (branch_index, branch_points) in branches.into_iter().enumerate() {
            if let Some(point) = branch_points.first().copied() {
                curves.push(RuleFloquetBoundaryCurve {
                    curve_id: format!(
                        "{}-branch-{:02}-amp-{:.3}-stim-i-{:.3}",
                        point.kind,
                        branch_index + 1,
                        point.amplitude,
                        point.stim_i_fraction
                    ),
                    kind: point.kind,
                    branch_label: format!("{} branch {}", point.branch_label, branch_index + 1),
                    branch_periodicity: point.branch_periodicity,
                    axis: "beta",
                    source_axis: "wave_number_vs_forcing_period",
                    amplitude: point.amplitude,
                    stim_i_fraction: point.stim_i_fraction,
                    point_count: 0,
                    period_min_ms: point.period_ms,
                    period_max_ms: point.period_ms,
                    beta_min_cycles: point.beta_cycles,
                    beta_max_cycles: point.beta_cycles,
                    wave_number_min_radians: point.wave_number_radians,
                    wave_number_max_radians: point.wave_number_radians,
                    mean_residual_abs: 0.0,
                    max_residual_abs: 0.0,
                    mean_bracket_width_beta_cycles: 0.0,
                    max_bracket_width_beta_cycles: 0.0,
                    mean_period_gap_ms: 0.0,
                    max_period_gap_ms: 0.0,
                    continuity_score: 0.0,
                    fit: empty_rule_floquet_curve_fit(),
                    source_comparison: RuleFloquetBoundarySourceComparison::missing(),
                    points: branch_points,
                });
            }
        }
    }

    for curve in &mut curves {
        curve.points.sort_by(|a, b| {
            a.period_ms
                .total_cmp(&b.period_ms)
                .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
        });
        curve.point_count = curve.points.len();
        if let Some(first) = curve.points.first() {
            curve.period_min_ms = first.period_ms;
            curve.period_max_ms = first.period_ms;
            curve.beta_min_cycles = first.beta_cycles;
            curve.beta_max_cycles = first.beta_cycles;
            curve.wave_number_min_radians = first.wave_number_radians;
            curve.wave_number_max_radians = first.wave_number_radians;
        }
        for point in &curve.points {
            curve.period_min_ms = curve.period_min_ms.min(point.period_ms);
            curve.period_max_ms = curve.period_max_ms.max(point.period_ms);
            curve.beta_min_cycles = curve.beta_min_cycles.min(point.beta_cycles);
            curve.beta_max_cycles = curve.beta_max_cycles.max(point.beta_cycles);
            curve.wave_number_min_radians =
                curve.wave_number_min_radians.min(point.wave_number_radians);
            curve.wave_number_max_radians =
                curve.wave_number_max_radians.max(point.wave_number_radians);
        }
        update_rule_floquet_curve_quality(curve);
        curve.fit = fit_rule_floquet_boundary_curve(&curve.points);
    }
    curves
}

fn split_rule_floquet_boundary_branches(
    mut points: Vec<RuleFloquetBoundaryCurvePoint>,
) -> Vec<Vec<RuleFloquetBoundaryCurvePoint>> {
    points.sort_by(|a, b| {
        a.period_ms
            .total_cmp(&b.period_ms)
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    let mut branches: Vec<Vec<RuleFloquetBoundaryCurvePoint>> = Vec::new();
    let mut index = 0usize;
    while index < points.len() {
        let period = points[index].period_ms;
        let start = index;
        while index < points.len() && (points[index].period_ms - period).abs() < 1.0e-9 {
            index += 1;
        }
        let mut period_points = points[start..index].to_vec();
        period_points.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        let mut available = (0..branches.len()).collect::<Vec<_>>();
        for point in period_points {
            if available.is_empty() {
                branches.push(vec![point]);
                continue;
            }
            let best_available_index = available
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let a_last = branches[**a]
                        .last()
                        .map(|last| last.beta_cycles)
                        .unwrap_or(0.0);
                    let b_last = branches[**b]
                        .last()
                        .map(|last| last.beta_cycles)
                        .unwrap_or(0.0);
                    (a_last - point.beta_cycles)
                        .abs()
                        .total_cmp(&(b_last - point.beta_cycles).abs())
                })
                .map(|(available_index, _)| available_index)
                .unwrap_or(0);
            let branch_index = available.remove(best_available_index);
            branches[branch_index].push(point);
        }
    }
    branches.sort_by(|a, b| {
        let a_first = a.first().map(|point| point.beta_cycles).unwrap_or(0.0);
        let b_first = b.first().map(|point| point.beta_cycles).unwrap_or(0.0);
        a_first.total_cmp(&b_first)
    });
    branches
}

pub(crate) fn empty_rule_floquet_curve_fit() -> RuleFloquetBoundaryCurveFit {
    RuleFloquetBoundaryCurveFit {
        model: "polynomial_period_to_wave_number",
        degree: 0,
        x_axis: "forcing_period_ms",
        y_axis: "wave_number_radians",
        x_origin_ms: 0.0,
        x_scale_ms: 1.0,
        coefficients: vec![0.0],
        rms_residual: 0.0,
        max_abs_residual: 0.0,
    }
}

fn fit_rule_floquet_boundary_curve(
    points: &[RuleFloquetBoundaryCurvePoint],
) -> RuleFloquetBoundaryCurveFit {
    if points.is_empty() {
        return empty_rule_floquet_curve_fit();
    }

    let x_origin_ms = points.iter().map(|point| point.period_ms).sum::<f64>() / points.len() as f64;
    let x_scale_ms = points
        .iter()
        .map(|point| (point.period_ms - x_origin_ms).abs())
        .fold(0.0, f64::max)
        .max(1.0);
    let requested_degree = if points.len() >= 3 {
        2usize
    } else if points.len() >= 2 {
        1usize
    } else {
        0usize
    };

    let coefficients = polynomial_fit_normalized(
        points
            .iter()
            .map(|point| {
                (
                    (point.period_ms - x_origin_ms) / x_scale_ms,
                    point.wave_number_radians,
                )
            })
            .collect::<Vec<_>>()
            .as_slice(),
        requested_degree,
    )
    .unwrap_or_else(|| {
        vec![
            points
                .iter()
                .map(|point| point.wave_number_radians)
                .sum::<f64>()
                / points.len() as f64,
        ]
    });
    let degree = coefficients.len().saturating_sub(1);
    let residuals = points
        .iter()
        .map(|point| {
            let x = (point.period_ms - x_origin_ms) / x_scale_ms;
            polynomial_value(&coefficients, x) - point.wave_number_radians
        })
        .collect::<Vec<_>>();
    let rms_residual = (residuals
        .iter()
        .map(|residual| residual * residual)
        .sum::<f64>()
        / residuals.len().max(1) as f64)
        .sqrt();
    let max_abs_residual = residuals
        .iter()
        .map(|residual| residual.abs())
        .fold(0.0, f64::max);

    RuleFloquetBoundaryCurveFit {
        model: "polynomial_period_to_wave_number",
        degree,
        x_axis: "forcing_period_ms",
        y_axis: "wave_number_radians",
        x_origin_ms,
        x_scale_ms,
        coefficients,
        rms_residual,
        max_abs_residual,
    }
}

fn polynomial_fit_normalized(points: &[(f64, f64)], degree: usize) -> Option<Vec<f64>> {
    let terms = degree + 1;
    if points.len() < terms {
        return None;
    }
    let mut matrix = vec![vec![0.0; terms]; terms];
    let mut rhs = vec![0.0; terms];
    for (x, y) in points {
        let mut powers = vec![1.0; terms * 2];
        for i in 1..powers.len() {
            powers[i] = powers[i - 1] * x;
        }
        for row in 0..terms {
            rhs[row] += y * powers[row];
            for col in 0..terms {
                matrix[row][col] += powers[row + col];
            }
        }
    }
    solve_linear_system(matrix, rhs)
}

fn solve_linear_system(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    for col in 0..n {
        let pivot =
            (col..n).max_by(|a, b| matrix[*a][col].abs().total_cmp(&matrix[*b][col].abs()))?;
        if matrix[pivot][col].abs() < 1.0e-12 {
            return None;
        }
        matrix.swap(col, pivot);
        rhs.swap(col, pivot);
        let pivot_value = matrix[col][col];
        for value in matrix[col].iter_mut().take(n).skip(col) {
            *value /= pivot_value;
        }
        rhs[col] /= pivot_value;
        let pivot_row = matrix[col].clone();
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = matrix[row][col];
            for (cell, pivot_cell) in matrix[row]
                .iter_mut()
                .zip(pivot_row.iter())
                .take(n)
                .skip(col)
            {
                *cell -= factor * *pivot_cell;
            }
            rhs[row] -= factor * rhs[col];
        }
    }
    Some(rhs)
}

fn polynomial_value(coefficients: &[f64], x: f64) -> f64 {
    coefficients
        .iter()
        .rev()
        .fold(0.0, |acc, coefficient| acc * x + coefficient)
}

fn update_rule_floquet_curve_quality(curve: &mut RuleFloquetBoundaryCurve) {
    let point_count = curve.points.len();
    if point_count == 0 {
        return;
    }

    curve.mean_residual_abs = curve
        .points
        .iter()
        .map(|point| point.residual_abs)
        .sum::<f64>()
        / point_count as f64;
    curve.max_residual_abs = curve
        .points
        .iter()
        .map(|point| point.residual_abs)
        .fold(0.0, f64::max);
    curve.mean_bracket_width_beta_cycles = curve
        .points
        .iter()
        .map(|point| point.bracket_width_beta_cycles)
        .sum::<f64>()
        / point_count as f64;
    curve.max_bracket_width_beta_cycles = curve
        .points
        .iter()
        .map(|point| point.bracket_width_beta_cycles)
        .fold(0.0, f64::max);

    let gaps = curve
        .points
        .windows(2)
        .map(|pair| (pair[1].period_ms - pair[0].period_ms).abs())
        .collect::<Vec<_>>();
    if gaps.is_empty() {
        curve.mean_period_gap_ms = 0.0;
        curve.max_period_gap_ms = 0.0;
        curve.continuity_score = 1.0;
        return;
    }

    curve.mean_period_gap_ms = gaps.iter().sum::<f64>() / gaps.len() as f64;
    curve.max_period_gap_ms = gaps.iter().copied().fold(0.0, f64::max);
    let span = (curve.period_max_ms - curve.period_min_ms).abs();
    curve.continuity_score = if span <= 1.0e-9 {
        1.0
    } else {
        (1.0 - (curve.max_period_gap_ms / span).clamp(0.0, 1.0)).clamp(0.0, 1.0)
    };
}

fn rule_floquet_nearest_boundary_candidates(
    points: &[RuleFloquetGridPoint],
    limit_per_kind: usize,
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for kind in KINDS {
        let mut scored = Vec::new();
        for point in points {
            for mode in &point.modes {
                let margin = floquet_mode_margin(mode, kind);
                if !margin.is_finite() {
                    continue;
                }
                scored.push((
                    margin.abs(),
                    RuleFloquetBoundaryCandidate {
                        kind,
                        evidence: "nearest_margin",
                        beta_cycles: mode.beta_cycles,
                        axis: "nearest",
                        period_ms: point.period_ms,
                        amplitude: point.amplitude,
                        stim_i_fraction: point.stim_i_fraction,
                        from_period_ms: point.period_ms,
                        from_amplitude: point.amplitude,
                        from_beta_cycles: mode.beta_cycles,
                        to_period_ms: point.period_ms,
                        to_amplitude: point.amplitude,
                        to_beta_cycles: mode.beta_cycles,
                        margin_from: margin,
                        margin_to: margin,
                        confidence: (1.0 - margin.abs().min(1.0)).max(0.0),
                    },
                ));
            }
        }
        scored.sort_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then_with(|| b.1.confidence.total_cmp(&a.1.confidence))
                .then_with(|| a.1.period_ms.total_cmp(&b.1.period_ms))
                .then_with(|| a.1.amplitude.total_cmp(&b.1.amplitude))
        });
        candidates.extend(
            scored
                .into_iter()
                .take(limit_per_kind)
                .map(|(_, candidate)| candidate),
        );
    }
    candidates
}

fn max_floquet_margin(modes: &[RuleFloquetMode], kind: &'static str) -> f64 {
    modes
        .iter()
        .map(|mode| floquet_mode_margin(mode, kind))
        .fold(f64::NEG_INFINITY, f64::max)
}

pub(crate) fn floquet_mode_margin(mode: &RuleFloquetMode, kind: &'static str) -> f64 {
    match kind {
        "minus_period_doubling" => -mode.minus_condition,
        "plus_one_to_one" => -mode.plus_condition,
        "unstable_complex" => -mode.determinant_condition,
        _ => mode.max_abs_multiplier - 1.0,
    }
}

fn rule_homogeneous_step(
    params: FrameParams,
    ue: f64,
    ui: f64,
    time_ms: f64,
    dt: f64,
) -> (f64, f64) {
    let (input_e, input_i) = rule_homogeneous_inputs(params, ue, ui, time_ms);
    let target_e = rule_sigmoid(input_e);
    let target_i = rule_sigmoid(input_i);
    (
        (ue + (dt / params.rule_tau_e_ms) * (-ue + target_e)).clamp(0.0, 1.0),
        (ui + (dt / params.rule_tau_i_ms) * (-ui + target_i)).clamp(0.0, 1.0),
    )
}

fn rule_homogeneous_inputs(params: FrameParams, ue: f64, ui: f64, time_ms: f64) -> (f64, f64) {
    let stim = params.rule_stim_amplitude * rule_stimulus(params, time_ms);
    (
        params.rule_aee * ue - params.rule_aie * ui - params.rule_theta_e + stim,
        params.rule_aei * ue - params.rule_aii * ui - params.rule_theta_i
            + params.rule_stim_i_fraction * stim,
    )
}

fn summarize_rule_orbit(period_ms: f64, steps: &[RuleOrbitStep]) -> RuleOrbitSummary {
    if steps.is_empty() {
        return RuleOrbitSummary {
            period_ms,
            samples: 0,
            e_min: 0.0,
            e_max: 0.0,
            e_mean: 0.0,
            i_min: 0.0,
            i_max: 0.0,
            i_mean: 0.0,
        };
    }
    let mut e_min = f64::INFINITY;
    let mut e_max = f64::NEG_INFINITY;
    let mut e_sum = 0.0;
    let mut i_min = f64::INFINITY;
    let mut i_max = f64::NEG_INFINITY;
    let mut i_sum = 0.0;
    for step in steps {
        e_min = e_min.min(step.ue);
        e_max = e_max.max(step.ue);
        e_sum += step.ue;
        i_min = i_min.min(step.ui);
        i_max = i_max.max(step.ui);
        i_sum += step.ui;
    }
    RuleOrbitSummary {
        period_ms,
        samples: steps.len(),
        e_min,
        e_max,
        e_mean: e_sum / steps.len() as f64,
        i_min,
        i_max,
        i_mean: i_sum / steps.len() as f64,
    }
}

fn rule_floquet_mode(
    params: FrameParams,
    steps: &[RuleOrbitStep],
    beta_cycles: f64,
) -> RuleFloquetMode {
    let gain_e = rule_kernel_mode_gain(params.rule_sigma_e, beta_cycles, 0.0, params.n);
    let gain_i = rule_kernel_mode_gain(params.rule_sigma_i, beta_cycles, 0.0, params.n);
    let mut m00 = 1.0;
    let mut m01 = 0.0;
    let mut m10 = 0.0;
    let mut m11 = 1.0;

    for step in steps {
        let (input_e, input_i) = rule_homogeneous_inputs(params, step.ue, step.ui, step.time_ms);
        let slope_e = rule_sigmoid_derivative(input_e);
        let slope_i = rule_sigmoid_derivative(input_i);
        let j00 = (-1.0 + slope_e * params.rule_aee * gain_e) / params.rule_tau_e_ms;
        let j01 = (-slope_e * params.rule_aie * gain_i) / params.rule_tau_e_ms;
        let j10 = (slope_i * params.rule_aei * gain_e) / params.rule_tau_i_ms;
        let j11 = (-1.0 - slope_i * params.rule_aii * gain_i) / params.rule_tau_i_ms;
        let a00 = 1.0 + step.dt * j00;
        let a01 = step.dt * j01;
        let a10 = step.dt * j10;
        let a11 = 1.0 + step.dt * j11;
        let next00 = a00 * m00 + a01 * m10;
        let next01 = a00 * m01 + a01 * m11;
        let next10 = a10 * m00 + a11 * m10;
        let next11 = a10 * m01 + a11 * m11;
        m00 = next00;
        m01 = next01;
        m10 = next10;
        m11 = next11;
    }

    floquet_mode_from_matrix(
        beta_cycles,
        rule_wave_number_for_cycles(beta_cycles, params.n),
        m00,
        m01,
        m10,
        m11,
    )
}

pub(crate) fn floquet_mode_from_matrix(
    beta_cycles: f64,
    wave_number_radians: f64,
    m00: f64,
    m01: f64,
    m10: f64,
    m11: f64,
) -> RuleFloquetMode {
    let trace = m00 + m11;
    let determinant = m00 * m11 - m01 * m10;
    let discriminant = trace * trace - 4.0 * determinant;
    let (l1_real, l1_imag, l2_real, l2_imag) = if discriminant >= 0.0 {
        let root = discriminant.sqrt();
        ((trace + root) * 0.5, 0.0, (trace - root) * 0.5, 0.0)
    } else {
        let real = trace * 0.5;
        let imag = (-discriminant).sqrt() * 0.5;
        (real, imag, real, -imag)
    };
    let abs_1 = (l1_real * l1_real + l1_imag * l1_imag).sqrt();
    let abs_2 = (l2_real * l2_real + l2_imag * l2_imag).sqrt();
    let plus_condition = 1.0 - trace + determinant;
    let minus_condition = 1.0 + trace + determinant;
    let determinant_condition = 1.0 - determinant;
    let crossing_hint = if minus_condition < 0.0 {
        "minus_period_doubling"
    } else if plus_condition < 0.0 {
        "plus_one_to_one"
    } else if determinant_condition < 0.0 {
        "unstable_complex"
    } else {
        "stable"
    };
    RuleFloquetMode {
        beta_cycles,
        wave_number_radians,
        multiplier_1_real: l1_real,
        multiplier_1_imag: l1_imag,
        multiplier_2_real: l2_real,
        multiplier_2_imag: l2_imag,
        max_abs_multiplier: abs_1.max(abs_2),
        monodromy_trace: trace,
        monodromy_determinant: determinant,
        plus_condition,
        minus_condition,
        determinant_condition,
        crossing_hint,
    }
}

fn rule_sigmoid_derivative(input: f64) -> f64 {
    let value = rule_sigmoid(input);
    value * (1.0 - value)
}

fn rule_kernel_mode_gain(sigma: f64, beta_cycles: f64, angle: f64, n: usize) -> f64 {
    let kernel = rule_gaussian_kernel(sigma);
    let q = rule_wave_number_for_cycles(beta_cycles, n);
    let qx = q * angle.cos();
    let qy = q * angle.sin();
    rule_kernel_1d_gain(&kernel, qx) * rule_kernel_1d_gain(&kernel, qy)
}

pub(crate) fn rule_wave_number_for_cycles(beta_cycles: f64, n: usize) -> f64 {
    2.0 * PI * beta_cycles / n as f64
}

fn rule_kernel_1d_gain(kernel: &RuleGaussianKernel, q: f64) -> f64 {
    kernel
        .weights
        .iter()
        .enumerate()
        .map(|(index, weight)| {
            let offset = index as isize - kernel.radius as isize;
            weight * (q * offset as f64).cos()
        })
        .sum()
}

pub(crate) fn load_rule_figure8_source_curves(
    path: &Path,
) -> Result<RuleFigure8SourceCurves, Box<dyn std::error::Error>> {
    let body = fs::read_to_string(path)?;
    let source = serde_json::from_str::<RuleFigure8SourceCurves>(&body)?;
    if source.format != "rule-2011-figure8-source-curves-v1" {
        return Err(format!(
            "unexpected Rule Figure 8 source curve format: {}",
            source.format
        )
        .into());
    }
    if source.source_key != "rule-2011" {
        return Err(format!("unexpected Rule Figure 8 source key: {}", source.source_key).into());
    }
    if source.figure != "Figure 8C" {
        return Err(format!("unexpected Rule Figure 8 source figure: {}", source.figure).into());
    }
    Ok(source)
}

pub(crate) fn apply_rule_figure8_source_comparison(
    curves: &mut [RuleFloquetBoundaryCurve],
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    source_curves: Option<&RuleFigure8SourceCurves>,
    source_curve_file: Option<&PathBuf>,
) -> RuleFloquetSourceCurveComparisonSummary {
    let Some(source) = source_curves else {
        for curve in &mut *curves {
            curve.source_comparison = if source_curve_file.is_some() {
                RuleFloquetBoundarySourceComparison::missing()
            } else {
                RuleFloquetBoundarySourceComparison::disabled()
            };
        }
        let status = if source_curve_file.is_some() {
            "source-curve-file-missing"
        } else {
            "source-curve-comparison-disabled"
        };
        return RuleFloquetSourceCurveComparisonSummary {
            status,
            source_curve_file: source_curve_file.map(|path| path.display().to_string()),
            source_curve_count: 0,
            compared_curve_count: 0,
            mean_rms_wave_number_error: None,
            max_rms_wave_number_error: None,
            domain_beta_mapping: None,
            raw_beta_mapping: None,
            scale_only_beta_mapping: None,
            affine_beta_mapping: None,
            fit_objective: None,
        };
    };

    let mut rms_values = Vec::new();
    for curve in &mut *curves {
        curve.source_comparison =
            best_rule_source_curve_comparison(curve, &source.curves, wave_number_normalization);
        if let Some(rms) = curve.source_comparison.rms_wave_number_error {
            rms_values.push(rms);
        }
    }
    let compared_curve_count = rms_values.len();
    let mean_rms_wave_number_error =
        (!rms_values.is_empty()).then(|| rms_values.iter().sum::<f64>() / rms_values.len() as f64);
    let max_rms_wave_number_error = rms_values.iter().copied().reduce(f64::max);
    let samples = rule_figure8_comparison_samples(curves, &source.curves);
    let domain_beta_mapping = rule_beta_axis_mapping(
        "domain_normalized",
        &samples,
        wave_number_normalization.source_beta_per_model_cycle,
        wave_number_normalization.source_beta_offset,
    );
    let raw_beta_mapping = rule_beta_axis_mapping("identity", &samples, 1.0, 0.0);
    let scale_only_beta_mapping = fit_rule_beta_scale_only_mapping(&samples);
    let affine_beta_mapping = fit_rule_beta_affine_mapping(&samples);
    let fit_objective = rule_figure8_fit_objective(
        curves,
        &source.curves,
        domain_beta_mapping.as_ref(),
        raw_beta_mapping.as_ref(),
        scale_only_beta_mapping.as_ref(),
        affine_beta_mapping.as_ref(),
    );

    RuleFloquetSourceCurveComparisonSummary {
        status: if compared_curve_count > 0 {
            "compared"
        } else {
            "no-overlapping-source-curves"
        },
        source_curve_file: source_curve_file.map(|path| path.display().to_string()),
        source_curve_count: source.curves.len(),
        compared_curve_count,
        mean_rms_wave_number_error,
        max_rms_wave_number_error,
        domain_beta_mapping,
        raw_beta_mapping,
        scale_only_beta_mapping,
        affine_beta_mapping,
        fit_objective,
    }
}

fn best_rule_source_curve_comparison(
    curve: &RuleFloquetBoundaryCurve,
    source_curves: &[RuleFigure8SourceCurve],
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
) -> RuleFloquetBoundarySourceComparison {
    source_curves
        .iter()
        .filter(|source| source.kind == curve.kind)
        .filter_map(|source| compare_rule_source_curve(curve, source, wave_number_normalization))
        .min_by(|a, b| {
            a.rms_wave_number_error
                .unwrap_or(f64::INFINITY)
                .total_cmp(&b.rms_wave_number_error.unwrap_or(f64::INFINITY))
        })
        .unwrap_or_else(RuleFloquetBoundarySourceComparison::no_overlap)
}

fn compare_rule_source_curve(
    curve: &RuleFloquetBoundaryCurve,
    source: &RuleFigure8SourceCurve,
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
) -> Option<RuleFloquetBoundarySourceComparison> {
    let mut source_points = source.points.clone();
    source_points.sort_by(|a, b| a.period_ms.total_cmp(&b.period_ms));
    let source_min = source_points.first()?.period_ms;
    let source_max = source_points.last()?.period_ms;
    let mut errors = Vec::new();
    let mut matched_periods = Vec::new();
    for point in &curve.points {
        if point.period_ms < source_min || point.period_ms > source_max {
            continue;
        }
        let Some(source_wave) =
            interpolate_rule_source_wave_number(&source_points, point.period_ms)
        else {
            continue;
        };
        let normalized_beta =
            rule_source_beta_for_model_cycles(point.beta_cycles, wave_number_normalization);
        errors.push(normalized_beta - source_wave);
        matched_periods.push(point.period_ms);
    }
    if errors.is_empty() {
        return None;
    }
    let mean_abs_wave_number_error =
        errors.iter().map(|error| error.abs()).sum::<f64>() / errors.len() as f64;
    let rms_wave_number_error =
        (errors.iter().map(|error| error * error).sum::<f64>() / errors.len() as f64).sqrt();
    let max_abs_wave_number_error = errors.iter().map(|error| error.abs()).fold(0.0, f64::max);
    Some(RuleFloquetBoundarySourceComparison {
        status: "compared",
        source_curve_id: Some(source.curve_id.clone()),
        source_branch_label: Some(source.branch_label.clone()),
        overlap_point_count: errors.len(),
        period_overlap_min_ms: matched_periods.iter().copied().reduce(f64::min),
        period_overlap_max_ms: matched_periods.iter().copied().reduce(f64::max),
        mean_abs_wave_number_error: Some(mean_abs_wave_number_error),
        rms_wave_number_error: Some(rms_wave_number_error),
        max_abs_wave_number_error: Some(max_abs_wave_number_error),
    })
}

fn interpolate_rule_source_wave_number(
    points: &[RuleFigure8SourcePoint],
    period_ms: f64,
) -> Option<f64> {
    for pair in points.windows(2) {
        let from = pair[0];
        let to = pair[1];
        if period_ms < from.period_ms || period_ms > to.period_ms {
            continue;
        }
        let span = (to.period_ms - from.period_ms).abs();
        if span <= 1.0e-9 {
            return Some(from.wave_number_beta);
        }
        let t = (period_ms - from.period_ms) / (to.period_ms - from.period_ms);
        return Some(from.wave_number_beta + t * (to.wave_number_beta - from.wave_number_beta));
    }
    None
}

fn rule_figure8_comparison_samples(
    curves: &[RuleFloquetBoundaryCurve],
    source_curves: &[RuleFigure8SourceCurve],
) -> Vec<RuleFigure8ComparisonSample> {
    let mut samples = Vec::new();
    for curve in curves {
        let Some(source_curve_id) = curve.source_comparison.source_curve_id.as_deref() else {
            continue;
        };
        let Some(source) = source_curves
            .iter()
            .find(|candidate| candidate.curve_id == source_curve_id)
        else {
            continue;
        };
        let mut source_points = source.points.clone();
        source_points.sort_by(|a, b| a.period_ms.total_cmp(&b.period_ms));
        let Some(source_min) = source_points.first().map(|point| point.period_ms) else {
            continue;
        };
        let Some(source_max) = source_points.last().map(|point| point.period_ms) else {
            continue;
        };
        for point in &curve.points {
            if point.period_ms < source_min || point.period_ms > source_max {
                continue;
            }
            if let Some(source_beta) =
                interpolate_rule_source_wave_number(&source_points, point.period_ms)
            {
                samples.push(RuleFigure8ComparisonSample {
                    generated_beta_cycles: point.beta_cycles,
                    source_beta,
                });
            }
        }
    }
    samples
}

fn rule_beta_axis_mapping(
    model: &'static str,
    samples: &[RuleFigure8ComparisonSample],
    scale: f64,
    offset: f64,
) -> Option<RuleFloquetBetaAxisMapping> {
    if samples.is_empty() {
        return None;
    }
    let mut abs_sum = 0.0;
    let mut square_sum = 0.0;
    let mut max_abs = 0.0;
    for sample in samples {
        let predicted = scale * sample.generated_beta_cycles + offset;
        let error = predicted - sample.source_beta;
        let abs = error.abs();
        abs_sum += abs;
        square_sum += error * error;
        max_abs = f64::max(max_abs, abs);
    }
    let sample_count = samples.len();
    Some(RuleFloquetBetaAxisMapping {
        model,
        generated_axis: "generated_beta_cycles",
        source_axis: "source_figure_beta",
        scale,
        offset,
        sample_count,
        mean_abs_error: abs_sum / sample_count as f64,
        rms_error: (square_sum / sample_count as f64).sqrt(),
        max_abs_error: max_abs,
    })
}

fn fit_rule_beta_scale_only_mapping(
    samples: &[RuleFigure8ComparisonSample],
) -> Option<RuleFloquetBetaAxisMapping> {
    let denominator = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles * sample.generated_beta_cycles)
        .sum::<f64>();
    if denominator.abs() <= 1.0e-12 {
        return rule_beta_axis_mapping("scale_only", samples, 1.0, 0.0);
    }
    let numerator = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles * sample.source_beta)
        .sum::<f64>();
    rule_beta_axis_mapping("scale_only", samples, numerator / denominator, 0.0)
}

fn fit_rule_beta_affine_mapping(
    samples: &[RuleFigure8ComparisonSample],
) -> Option<RuleFloquetBetaAxisMapping> {
    if samples.is_empty() {
        return None;
    }
    let sample_count = samples.len() as f64;
    let mean_x = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles)
        .sum::<f64>()
        / sample_count;
    let mean_y = samples.iter().map(|sample| sample.source_beta).sum::<f64>() / sample_count;
    let variance_x = samples
        .iter()
        .map(|sample| {
            let dx = sample.generated_beta_cycles - mean_x;
            dx * dx
        })
        .sum::<f64>();
    if variance_x.abs() <= 1.0e-12 {
        return rule_beta_axis_mapping("affine", samples, 1.0, mean_y - mean_x);
    }
    let covariance_xy = samples
        .iter()
        .map(|sample| {
            let dx = sample.generated_beta_cycles - mean_x;
            let dy = sample.source_beta - mean_y;
            dx * dy
        })
        .sum::<f64>();
    let scale = covariance_xy / variance_x;
    let offset = mean_y - scale * mean_x;
    rule_beta_axis_mapping("affine", samples, scale, offset)
}

fn rule_figure8_fit_objective(
    curves: &[RuleFloquetBoundaryCurve],
    source_curves: &[RuleFigure8SourceCurve],
    domain_mapping: Option<&RuleFloquetBetaAxisMapping>,
    raw_mapping: Option<&RuleFloquetBetaAxisMapping>,
    scale_only_mapping: Option<&RuleFloquetBetaAxisMapping>,
    affine_mapping: Option<&RuleFloquetBetaAxisMapping>,
) -> Option<RuleFigure8FitObjective> {
    let raw = raw_mapping?;
    let domain = domain_mapping.unwrap_or(raw);
    let scale_only = scale_only_mapping.unwrap_or(raw);
    let affine = affine_mapping.unwrap_or(raw);
    let compared_curves = curves
        .iter()
        .filter(|curve| curve.source_comparison.status == "compared")
        .collect::<Vec<_>>();
    let matched_source_curve_count = compared_curves
        .iter()
        .filter_map(|curve| curve.source_comparison.source_curve_id.as_deref())
        .collect::<HashSet<_>>()
        .len();
    let source_curve_count = source_curves.len();
    let compared_curve_count = compared_curves.len();
    let generated_curve_coverage = if curves.is_empty() {
        0.0
    } else {
        compared_curve_count as f64 / curves.len() as f64
    };
    let source_branch_coverage = if source_curve_count == 0 {
        0.0
    } else {
        matched_source_curve_count as f64 / source_curve_count as f64
    };
    let total_curve_points = curves.iter().map(|curve| curve.point_count).sum::<usize>();
    let overlap_point_count = compared_curves
        .iter()
        .map(|curve| curve.source_comparison.overlap_point_count)
        .sum::<usize>();
    let overlap_point_coverage = if total_curve_points == 0 {
        0.0
    } else {
        overlap_point_count as f64 / total_curve_points as f64
    };
    let continuity_score = if compared_curves.is_empty() {
        0.0
    } else {
        compared_curves
            .iter()
            .map(|curve| curve.continuity_score.clamp(0.0, 1.0))
            .sum::<f64>()
            / compared_curves.len() as f64
    };
    let underresolved_branch_count = curves.iter().filter(|curve| curve.point_count < 2).count();
    let underresolved_branch_fraction = if curves.is_empty() {
        1.0
    } else {
        underresolved_branch_count as f64 / curves.len() as f64
    };
    let ordering_score = rule_figure8_kind_ordering_score(curves);
    let coverage_score = (0.45 * source_branch_coverage
        + 0.35 * generated_curve_coverage
        + 0.20 * overlap_point_coverage)
        .clamp(0.0, 1.0);
    let score = 0.55 * domain.rms_error
        + 0.15 * affine.rms_error
        + 0.05 * scale_only.rms_error
        + 0.85 * (1.0 - coverage_score)
        + 0.35 * (1.0 - continuity_score)
        + 0.45 * (1.0 - ordering_score)
        + 0.25 * underresolved_branch_fraction;

    Some(RuleFigure8FitObjective {
        status: "scored",
        score,
        domain_normalized_rms_beta_error: domain.rms_error,
        raw_rms_beta_error: raw.rms_error,
        affine_rms_beta_error: affine.rms_error,
        scale_only_rms_beta_error: scale_only.rms_error,
        source_branch_coverage,
        generated_curve_coverage,
        overlap_point_coverage,
        continuity_score,
        ordering_score,
        underresolved_branch_fraction,
        compared_curve_count,
        source_curve_count,
        matched_source_curve_count,
        overlap_point_count,
    })
}

fn rule_figure8_kind_ordering_score(curves: &[RuleFloquetBoundaryCurve]) -> f64 {
    let minus_mean = mean_curve_period_for_kind(curves, "minus_period_doubling");
    let plus_mean = mean_curve_period_for_kind(curves, "plus_one_to_one");
    match (minus_mean, plus_mean) {
        (Some(minus), Some(plus)) if minus < plus => 1.0,
        (Some(_), Some(_)) => 0.25,
        _ => 0.0,
    }
}

fn mean_curve_period_for_kind(curves: &[RuleFloquetBoundaryCurve], kind: &str) -> Option<f64> {
    let mut count = 0usize;
    let mut sum = 0.0;
    for point in curves
        .iter()
        .filter(|curve| curve.kind == kind)
        .flat_map(|curve| curve.points.iter())
    {
        sum += point.period_ms;
        count += 1;
    }
    (count > 0).then(|| sum / count as f64)
}
