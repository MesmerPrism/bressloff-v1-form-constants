use std::{collections::HashMap, fs, path::PathBuf};

use base64::{engine::general_purpose, Engine as _};

use super::floquet::rule_floquet_report;
use super::reports::{representative_rule_frame, rule_details};
use super::simulate_rule_flicker_frames;
use crate::*;
pub(crate) fn rule_sweep_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/rule-2011-sweep.json");
    let mut grid = rule_sweep_grid_defaults("quick");
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
    let mut floquet_periods = vec![120.0, 85.0, 55.0];
    let mut floquet_amplitude = 0.8;
    let mut floquet_stim_i_fraction = 0.0;
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--preset-grid" | "--grid" => {
                grid =
                    rule_sweep_grid_defaults(iter.next().ok_or("--preset-grid requires a value")?);
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
            "--floquet-periods" => {
                floquet_periods = parse_f64_csv(
                    iter.next().ok_or("--floquet-periods requires a value")?,
                    20.0,
                    180.0,
                )?;
            }
            "--floquet-amplitude" => {
                floquet_amplitude = iter
                    .next()
                    .ok_or("--floquet-amplitude requires a value")?
                    .parse::<f64>()?
                    .clamp(0.0, 1.5);
            }
            "--floquet-stim-i-fraction" => {
                floquet_stim_i_fraction = iter
                    .next()
                    .ok_or("--floquet-stim-i-fraction requires a value")?
                    .parse::<f64>()?
                    .clamp(0.0, 1.0);
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

    let mut points = Vec::new();
    for period in &grid.periods {
        for amplitude in &grid.amplitudes {
            for stim_i_fraction in &grid.stim_i_fractions {
                let params = rule_sweep_params(&raw, &grid, *period, *amplitude, *stim_i_fraction);
                points.push(rule_sweep_point_for(params));
            }
        }
    }

    let mode_cycles = [2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0];
    let floquet_reports = floquet_periods
        .iter()
        .map(|period| {
            let params = rule_sweep_params(
                &raw,
                &grid,
                *period,
                floquet_amplitude,
                floquet_stim_i_fraction,
            );
            rule_floquet_report(params, &mode_cycles)
        })
        .collect::<Vec<_>>();

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let point_count = points.len();
    let report = RuleSweepReport {
        format: "rule-2011-sweep-report-v1",
        model_family: MODEL_FAMILY_RULE,
        source_key: "rule-2011",
        status: "first-pass-simulator-backed",
        note: "Frequency/amplitude grid and homogeneous-orbit monodromy diagnostics; not yet a figure-level Rule 2011 calibration.",
        classification_version: "rule-spatial-temporal-diagnostics-v2",
        grid: rule_sweep_grid_details(&grid),
        periods_ms: grid.periods,
        amplitudes: grid.amplitudes,
        stim_i_fractions: grid.stim_i_fractions,
        points,
        floquet_reports,
    };
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!("wrote {} sweep_points={point_count}", out.display());
    Ok(())
}

pub(crate) fn rule_sweep_grid_defaults(name: &str) -> RuleSweepGridConfig {
    match name {
        "dense" => RuleSweepGridConfig {
            preset: "dense",
            periods: linspace_values(40.0, 160.0, 13),
            amplitudes: linspace_values(0.4, 1.2, 5),
            stim_i_fractions: vec![0.0],
            n: 32,
            frames: 72,
            preview_step: 0.5,
        },
        "paper" => RuleSweepGridConfig {
            preset: "paper",
            periods: vec![140.0, 130.0, 120.0, 110.0, 100.0, 85.0, 75.0, 65.0, 55.0],
            amplitudes: vec![0.4, 0.65, 0.8, 1.0, 1.2],
            stim_i_fractions: vec![0.0, 0.25, 0.5],
            n: 32,
            frames: 72,
            preview_step: 0.5,
        },
        _ => RuleSweepGridConfig {
            preset: "quick",
            periods: vec![140.0, 120.0, 85.0, 65.0, 55.0],
            amplitudes: vec![0.65, 0.8, 1.0],
            stim_i_fractions: vec![0.0],
            n: 40,
            frames: 120,
            preview_step: 0.5,
        },
    }
}

pub(crate) fn rule_sweep_grid_details(grid: &RuleSweepGridConfig) -> RuleSweepGridDetails {
    let (period_min, period_max) = value_min_max(&grid.periods);
    let (amplitude_min, amplitude_max) = value_min_max(&grid.amplitudes);
    let (stim_min, stim_max) = value_min_max(&grid.stim_i_fractions);
    RuleSweepGridDetails {
        preset: grid.preset,
        period_min_ms: period_min,
        period_max_ms: period_max,
        period_steps: grid.periods.len(),
        amplitude_min,
        amplitude_max,
        amplitude_steps: grid.amplitudes.len(),
        stim_i_fraction_min: stim_min,
        stim_i_fraction_max: stim_max,
        stim_i_fraction_steps: grid.stim_i_fractions.len(),
        n: grid.n,
        frames: grid.frames,
        preview_step: grid.preview_step,
    }
}

pub(crate) fn value_min_max(values: &[f64]) -> (f64, f64) {
    values
        .iter()
        .copied()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        })
}

pub(crate) fn rule_sweep_params(
    raw: &HashMap<String, String>,
    grid: &RuleSweepGridConfig,
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
) -> FrameParams {
    let mut sweep_raw = raw.clone();
    sweep_raw
        .entry("generator".to_string())
        .or_insert_with(|| "rule_flicker".to_string());
    sweep_raw
        .entry("n".to_string())
        .or_insert_with(|| grid.n.to_string());
    sweep_raw
        .entry("m".to_string())
        .or_insert_with(|| "4".to_string());
    sweep_raw
        .entry("frames".to_string())
        .or_insert_with(|| grid.frames.to_string());
    sweep_raw
        .entry("preview_step".to_string())
        .or_insert_with(|| format!("{:.6}", grid.preview_step));
    sweep_raw
        .entry("trim_warmup".to_string())
        .or_insert_with(|| "false".to_string());
    sweep_raw
        .entry("rule_seed_strength".to_string())
        .or_insert_with(|| "0.2".to_string());
    sweep_raw
        .entry("rule_seed_pattern".to_string())
        .or_insert_with(|| rule_seed_for_period(period_ms).as_str().to_string());
    sweep_raw
        .entry("t".to_string())
        .or_insert_with(|| format!("{:.6}", rule_sweep_duration_ms(period_ms)));
    sweep_raw.insert("rule_stim_period_ms".to_string(), format!("{period_ms:.6}"));
    sweep_raw.insert("rule_stim_amplitude".to_string(), format!("{amplitude:.6}"));
    sweep_raw.insert(
        "rule_stim_i_fraction".to_string(),
        format!("{stim_i_fraction:.6}"),
    );
    coerce_params(&sweep_raw)
}

pub(crate) fn rule_sweep_duration_ms(period_ms: f64) -> f64 {
    if period_ms < 80.0 {
        (period_ms * 8.0).max(330.0)
    } else {
        (period_ms * 5.5).max(440.0)
    }
}

pub(crate) fn rule_seed_for_period(period_ms: f64) -> RuleSeedPattern {
    if period_ms >= 105.0 {
        RuleSeedPattern::Hexagonal
    } else if period_ms <= 70.0 {
        RuleSeedPattern::Stripes
    } else {
        RuleSeedPattern::Random
    }
}

pub(crate) fn rule_sweep_point_for(params: FrameParams) -> RuleSweepPoint {
    let (frames, times) = simulate_rule_flicker_frames(params);
    let metrics = frame_metrics(&frames, params.n);
    let details = rule_details(None, &frames, &times, &metrics, params);
    let final_frame = representative_rule_frame(&frames, params.n).unwrap_or(&[]);
    let (_, peak_activity) = raw_range(final_frame);
    RuleSweepPoint {
        period_ms: params.rule_stim_period_ms,
        amplitude: params.rule_stim_amplitude,
        stim_i_fraction: params.rule_stim_i_fraction,
        seed_pattern: params.rule_seed_pattern.as_str(),
        spatial_family: details.spatial_family,
        response_mode: details.response_mode,
        pattern_strength: details.pattern_strength,
        dominant_cycles: details.dominant_cycles,
        temporal_corr_t: details.temporal_corr_t,
        temporal_corr_2t: details.temporal_corr_2t,
        stimulus_frequency_hz: details.stimulus_frequency_hz,
        peak_activity,
        status_level: rule_sweep_status_level(&details),
        spatial: details.spatial.clone(),
        temporal: details.temporal,
        classification_note: rule_classification_note(&details),
        thumbnail: rule_thumbnail_from_frame(final_frame, params.n),
    }
}

pub(crate) fn rule_sweep_status_level(details: &RuleDetails) -> &'static str {
    if details.spatial_family == "homogeneous" && details.pattern_strength < 0.001 {
        "suppressed"
    } else if details.response_mode == "period_doubled" {
        "period-doubled"
    } else if details.response_mode == "one_to_one" {
        "one-to-one"
    } else {
        "transition"
    }
}

pub(crate) fn rule_classification_note(details: &RuleDetails) -> &'static str {
    if details.status == "manual" && details.pattern_strength < 0.001 {
        "weak spatial contrast; temporal classification is more reliable than visible pattern family"
    } else if details.spatial.confidence < 0.12 {
        "mixed spatial spectrum; family label should be read qualitatively"
    } else if details.temporal.confidence < 0.35 {
        "weak temporal repeat confidence"
    } else {
        "qualitative classifier"
    }
}

pub(crate) fn rule_thumbnail_from_frame(frame: &[f32], n: usize) -> RuleThumbnail {
    let (scale_min, scale_max) = percentile_range(frame, 1.0, 99.0);
    let normalized = normalize_u8(frame, scale_min, scale_max);
    RuleThumbnail {
        format: "rule-2011-u8-thumbnail-v1",
        encoding: "base64/u8-row-major",
        width: n,
        height: n,
        scale_min,
        scale_max,
        data_base64: general_purpose::STANDARD.encode(normalized),
    }
}
