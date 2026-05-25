use std::{fs, path::PathBuf};

use super::presets::{parse_rule_preset, rule_preset_catalog, RulePreset};
use crate::*;
pub(crate) fn rule_report_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/rule-2011-regimes.json");
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
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

    raw.entry("n".to_string())
        .or_insert_with(|| "40".to_string());
    raw.entry("frames".to_string())
        .or_insert_with(|| "144".to_string());
    raw.entry("preview_step".to_string())
        .or_insert_with(|| "0.5".to_string());

    let state = ServerState::default();
    let mut runs = Vec::new();
    for preset in rule_preset_catalog()
        .into_iter()
        .map(|details| parse_rule_preset(Some(details.id)))
        .filter(|preset| *preset != RulePreset::Manual)
    {
        let mut preset_raw = raw.clone();
        preset_raw.insert("rule_preset".to_string(), preset.as_str().to_string());
        let params = coerce_params(&preset_raw);
        let payload = generate_payload(params, &state)?;
        let rule = payload
            .rule
            .as_ref()
            .ok_or("Rule regime report missing for Rule preset")?;
        let preset = rule
            .preset
            .ok_or("Rule preset metadata missing for Rule report")?;
        runs.push(RuleCalibrationRun {
            preset,
            status: rule.status,
            spatial_family: rule.spatial_family,
            response_mode: rule.response_mode,
            pattern_strength: rule.pattern_strength,
            dominant_cycles: rule.dominant_cycles,
            temporal_corr_t: rule.temporal_corr_t,
            temporal_corr_2t: rule.temporal_corr_2t,
            stimulus_frequency_hz: rule.stimulus_frequency_hz,
            checks: rule.checks.clone(),
        });
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let run_count = runs.len();
    let body = serde_json::json!({
        "format": "rule-2011-regime-report-v1",
        "model_family": MODEL_FAMILY_RULE,
        "runs": runs,
    });
    fs::write(&out, serde_json::to_vec_pretty(&body)?)?;
    println!("wrote {} rule_presets={run_count}", out.display());
    Ok(())
}

pub(crate) fn rule_details(
    preset: Option<RulePresetDetails>,
    frames: &[f32],
    times: &[f64],
    metrics: &Metrics,
    params: FrameParams,
) -> RuleDetails {
    let final_frame = representative_rule_frame(frames, params.n).unwrap_or(&[]);
    let pattern_strength = rule_pattern_strength(frames, params.n);
    let spatial = analyze_rule_spatial(final_frame, params.n);
    let temporal_corr_t =
        temporal_correlation_at_period(frames, times, params.n, params.rule_stim_period_ms);
    let temporal_corr_2t =
        temporal_correlation_at_period(frames, times, params.n, 2.0 * params.rule_stim_period_ms);
    let temporal_corr_3t =
        temporal_correlation_at_period(frames, times, params.n, 3.0 * params.rule_stim_period_ms);
    let temporal = analyze_rule_temporal(
        temporal_corr_t,
        temporal_corr_2t,
        temporal_corr_3t,
        pattern_strength,
    );
    let spatial_family = spatial.family;
    let dominant_cycles = spatial.dominant_cycles;
    let response_mode = temporal.response_mode;
    let mut checks = Vec::new();

    if let Some(preset) = preset {
        checks.push(CalibrationCheck {
            name: "model-family",
            expected: MODEL_FAMILY_RULE,
            actual: MODEL_FAMILY_RULE.to_string(),
            passed: true,
        });
        checks.push(CalibrationCheck {
            name: "spatial-family",
            expected: preset.expected_family,
            actual: spatial_family.to_string(),
            passed: spatial_family == preset.expected_family,
        });
        checks.push(CalibrationCheck {
            name: "response-mode",
            expected: preset.expected_response_mode,
            actual: response_mode.to_string(),
            passed: response_mode == preset.expected_response_mode,
        });
    }

    let status = if preset.is_some() && checks.iter().all(|check| check.passed) {
        "qualitative-pass"
    } else if preset.is_some() {
        "qualitative-review"
    } else {
        "manual"
    };

    RuleDetails {
        preset,
        model_family: MODEL_FAMILY_RULE,
        source_key: "rule-2011",
        equation: "Wilson-Cowan E/I field, Rule 2011 equations 1-2",
        status,
        spatial_family,
        response_mode,
        pattern_strength,
        dominant_cycles: if dominant_cycles > 0.0 {
            dominant_cycles
        } else {
            metrics.dominant_cycles
        },
        temporal_corr_t,
        temporal_corr_2t,
        stimulus_frequency_hz: 1000.0 / params.rule_stim_period_ms.max(1.0e-9),
        spatial,
        temporal,
        parameters: RuleParamDetails {
            tau_e_ms: params.rule_tau_e_ms,
            tau_i_ms: params.rule_tau_i_ms,
            aee: params.rule_aee,
            aei: params.rule_aei,
            aie: params.rule_aie,
            aii: params.rule_aii,
            theta_e: params.rule_theta_e,
            theta_i: params.rule_theta_i,
            sigma_e: params.rule_sigma_e,
            sigma_i: params.rule_sigma_i,
            stim_amplitude: params.rule_stim_amplitude,
            stim_period_ms: params.rule_stim_period_ms,
            stim_threshold: params.rule_stim_threshold,
            stim_smoothing: params.rule_stim_smoothing,
            stim_i_fraction: params.rule_stim_i_fraction,
            seed_pattern: params.rule_seed_pattern.as_str(),
            seed_strength: params.rule_seed_strength,
        },
        checks,
    }
}

fn rule_pattern_strength(frames: &[f32], n: usize) -> f32 {
    let frame_size = n * n;
    if frame_size == 0 || frames.len() < frame_size {
        return 0.0;
    }
    let frame_count = frames.len() / frame_size;
    let start = frame_count.saturating_mul(2) / 3;
    let tail = &frames[start * frame_size..];
    let count = tail.len() / frame_size;
    if count == 0 {
        return 0.0;
    }
    tail.chunks(frame_size).map(stddev).sum::<f32>() / count as f32
}

pub(crate) fn representative_rule_frame(frames: &[f32], n: usize) -> Option<&[f32]> {
    let frame_size = n * n;
    if frame_size == 0 || frames.len() < frame_size {
        return None;
    }
    let frame_count = frames.len() / frame_size;
    let start = frame_count.saturating_mul(2) / 3;
    frames[start * frame_size..]
        .chunks(frame_size)
        .max_by(|a, b| stddev(a).total_cmp(&stddev(b)))
}

fn analyze_rule_spatial(frame: &[f32], n: usize) -> RuleSpatialDiagnostics {
    let strength = stddev(frame);
    if frame.is_empty() || strength < 0.001 {
        return RuleSpatialDiagnostics {
            family: "homogeneous",
            dominant_cycles: 0.0,
            stripe_power: 0.0,
            square_power: 0.0,
            hex_power: 0.0,
            total_power: 0.0,
            mode_entropy: 0.0,
            confidence: 1.0,
            top_modes: Vec::new(),
        };
    }

    let angles = rule_mode_scan_angles();
    let mut top_modes = Vec::new();
    let mut total_power = 0.0_f64;
    let mut best_stripe = RuleModePower {
        cycles: 0.0,
        angle_degrees: 0.0,
        family: "stripe",
        power: 0.0,
    };
    let mut best_square = RuleModePower {
        cycles: 0.0,
        angle_degrees: 0.0,
        family: "square",
        power: 0.0,
    };
    let mut best_hex = RuleModePower {
        cycles: 0.0,
        angle_degrees: 0.0,
        family: "hexagonal",
        power: 0.0,
    };

    for cycles in 2..=10 {
        let cycles_f = cycles as f64;
        for angle in angles {
            let power = projection_power(frame, n, cycles_f, angle);
            total_power += power;
            let mode = RuleModePower {
                cycles: cycles_f,
                angle_degrees: angle.to_degrees(),
                family: "axis",
                power,
            };
            top_modes.push(mode);
            if power > best_stripe.power {
                best_stripe = RuleModePower {
                    family: "stripe",
                    ..mode
                };
            }
        }

        for angle in [0.0, PI / 8.0, PI / 4.0, 3.0 * PI / 8.0] {
            let square_power = projection_power(frame, n, cycles_f, angle)
                + projection_power(frame, n, cycles_f, angle + PI / 2.0);
            if square_power > best_square.power {
                best_square = RuleModePower {
                    cycles: cycles_f,
                    angle_degrees: angle.to_degrees(),
                    family: "square",
                    power: square_power,
                };
            }
        }

        for angle in [0.0, PI / 12.0, PI / 6.0, PI / 4.0] {
            let hex_power = projection_power(frame, n, cycles_f, angle)
                + projection_power(frame, n, cycles_f, angle + PI / 3.0)
                + projection_power(frame, n, cycles_f, angle - PI / 3.0);
            if hex_power > best_hex.power {
                best_hex = RuleModePower {
                    cycles: cycles_f,
                    angle_degrees: angle.to_degrees(),
                    family: "hexagonal",
                    power: hex_power,
                };
            }
        }
    }

    top_modes.sort_by(|a, b| b.power.total_cmp(&a.power));
    top_modes.truncate(8);
    let stripe_score = best_stripe.power;
    let square_score = best_square.power / 1.45;
    let hex_score = best_hex.power / 1.65;
    let mut scores = [
        ("stripe", best_stripe.cycles as f32, stripe_score),
        ("square", best_square.cycles as f32, square_score),
        ("hexagonal", best_hex.cycles as f32, hex_score),
    ];
    scores.sort_by(|a, b| b.2.total_cmp(&a.2));
    let family = if best_hex.power > 1.65 * best_stripe.power {
        "hexagonal"
    } else if best_square.power > 1.45 * best_stripe.power
        && best_square.power > 0.75 * best_hex.power
    {
        "square"
    } else {
        scores[0].0
    };
    let dominant_cycles = match family {
        "hexagonal" => best_hex.cycles as f32,
        "square" => best_square.cycles as f32,
        _ => best_stripe.cycles as f32,
    };
    let winner = scores[0].2.max(1.0e-12);
    let runner_up = scores[1].2.max(0.0);
    let confidence = ((winner - runner_up) / winner).clamp(0.0, 1.0);

    RuleSpatialDiagnostics {
        family,
        dominant_cycles,
        stripe_power: best_stripe.power,
        square_power: best_square.power,
        hex_power: best_hex.power,
        total_power,
        mode_entropy: rule_mode_entropy(&top_modes),
        confidence,
        top_modes,
    }
}

fn rule_mode_scan_angles() -> [f64; 12] {
    [
        0.0,
        PI / 12.0,
        PI / 6.0,
        PI / 4.0,
        PI / 3.0,
        5.0 * PI / 12.0,
        PI / 2.0,
        7.0 * PI / 12.0,
        2.0 * PI / 3.0,
        3.0 * PI / 4.0,
        5.0 * PI / 6.0,
        11.0 * PI / 12.0,
    ]
}

fn rule_mode_entropy(modes: &[RuleModePower]) -> f64 {
    if modes.len() < 2 {
        return 0.0;
    }
    let sum: f64 = modes.iter().map(|mode| mode.power.max(0.0)).sum();
    if sum <= 1.0e-18 {
        return 0.0;
    }
    let entropy = modes
        .iter()
        .map(|mode| mode.power.max(0.0) / sum)
        .filter(|p| *p > 1.0e-12)
        .map(|p| -p * p.ln())
        .sum::<f64>();
    (entropy / (modes.len() as f64).ln()).clamp(0.0, 1.0)
}

fn analyze_rule_temporal(
    corr_t: f32,
    corr_2t: f32,
    corr_3t: f32,
    pattern_strength: f32,
) -> RuleTemporalDiagnostics {
    let response_mode = classify_rule_response_mode(corr_t, corr_2t);
    let (estimated_period_cycles, mut confidence, note) = match response_mode {
        "period_doubled" => {
            let confidence = ((-corr_t).max(0.0) + corr_2t.max(0.0) + (-corr_3t).max(0.0)) / 3.0;
            (2.0, confidence, "two stimulus cycles per response repeat")
        }
        "one_to_one" => {
            let confidence = (corr_t.max(0.0) + corr_2t.max(0.0)) * 0.5;
            (1.0, confidence, "one stimulus cycle per response repeat")
        }
        _ => {
            let strongest = corr_t.abs().max(corr_2t.abs()).max(corr_3t.abs());
            (
                0.0,
                (1.0 - strongest).clamp(0.0, 1.0),
                "mixed or weak temporal repeat",
            )
        }
    };
    if pattern_strength < 0.001 {
        confidence *= 0.65;
    }
    RuleTemporalDiagnostics {
        corr_t,
        corr_2t,
        corr_3t,
        response_mode,
        estimated_period_cycles,
        confidence: confidence.clamp(0.0, 1.0),
        note,
    }
}

fn classify_rule_response_mode(corr_t: f32, corr_2t: f32) -> &'static str {
    if corr_t < -0.2 && corr_2t > 0.2 {
        "period_doubled"
    } else if corr_t > 0.2 {
        "one_to_one"
    } else {
        "mixed"
    }
}

fn projection_power(frame: &[f32], n: usize, cycles: f64, angle: f64) -> f64 {
    if frame.is_empty() {
        return 0.0;
    }
    let mean = frame.iter().sum::<f32>() as f64 / frame.len() as f64;
    let q = 2.0 * PI * cycles;
    let nx = angle.cos();
    let ny = angle.sin();
    let mut re = 0.0;
    let mut im = 0.0;
    for row in 0..n {
        let y = row as f64 / n as f64 - 0.5;
        for col in 0..n {
            let x = col as f64 / n as f64 - 0.5;
            let phase = q * (x * nx + y * ny);
            let value = frame[row * n + col] as f64 - mean;
            re += value * phase.cos();
            im += value * phase.sin();
        }
    }
    (re * re + im * im) / (frame.len() as f64 * frame.len() as f64)
}

fn temporal_correlation_at_period(frames: &[f32], times: &[f64], n: usize, period_ms: f64) -> f32 {
    let frame_size = n * n;
    if frame_size == 0 || frames.len() < 2 * frame_size || times.len() < 2 {
        return 0.0;
    }
    let final_index = times.len() - 1;
    let target_time = times[final_index] - period_ms;
    let Some(compare_index) = nearest_time_index(times, target_time) else {
        return 0.0;
    };
    if compare_index == final_index {
        return 0.0;
    }
    frame_correlation(
        &frames[compare_index * frame_size..(compare_index + 1) * frame_size],
        &frames[final_index * frame_size..(final_index + 1) * frame_size],
    )
}

fn nearest_time_index(times: &[f64], target: f64) -> Option<usize> {
    if target < *times.first()? {
        return None;
    }
    times
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (*a - target).abs().total_cmp(&(*b - target).abs()))
        .map(|(index, _)| index)
}

fn frame_correlation(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mean_a = a.iter().sum::<f32>() / a.len() as f32;
    let mean_b = b.iter().sum::<f32>() / b.len() as f32;
    let mut numerator = 0.0_f64;
    let mut denom_a = 0.0_f64;
    let mut denom_b = 0.0_f64;
    for (a, b) in a.iter().zip(b.iter()) {
        let da = (*a - mean_a) as f64;
        let db = (*b - mean_b) as f64;
        numerator += da * db;
        denom_a += da * da;
        denom_b += db * db;
    }
    if denom_a <= 1.0e-18 || denom_b <= 1.0e-18 {
        0.0
    } else {
        (numerator / (denom_a * denom_b).sqrt()).clamp(-1.0, 1.0) as f32
    }
}
