use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;
use serde::Serialize;

use crate::models::{
    bressloff::planform::{
        cell_mm_for, generate_planform_frames, orientation_count_for, planform_details,
    },
    bressloff::presets::{apply_paper_preset, paper_preset_details, parse_paper_preset},
    rule::presets::{apply_rule_preset, rule_preset_details},
    rule::reports::rule_details,
    rule::simulate_rule_flicker_frames,
};
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct StructureKey {
    n: usize,
    m: usize,
    pub(crate) r0_key: i64,
}

impl StructureKey {
    pub(crate) fn new(params: FrameParams) -> Self {
        Self {
            n: params.n,
            m: params.m,
            r0_key: (params.r0 * 100_000_000.0).round() as i64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Offset {
    pub(crate) dr: isize,
    pub(crate) dc: isize,
    pub(crate) weight: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SourceWeight {
    pub(crate) source_index: usize,
    pub(crate) weight: f64,
}

#[derive(Debug)]
pub(crate) struct SectorSources {
    pub(crate) per_cell: usize,
    pub(crate) entries: Vec<SourceWeight>,
}

#[derive(Debug)]
pub(crate) struct Structure {
    m: usize,
    pub(crate) angle_weights: Vec<f64>,
    pub(crate) sector_sources: Vec<SectorSources>,
}

#[derive(Default)]
pub(crate) struct ServerState {
    pub(crate) structures: Mutex<HashMap<StructureKey, Arc<Structure>>>,
    pub(crate) payloads: Mutex<HashMap<String, Arc<Payload>>>,
}

#[derive(Serialize)]
pub(crate) struct Payload {
    pub(crate) format: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) frame_count: usize,
    pub(crate) orientation_count: usize,
    pub(crate) times: Vec<f64>,
    pub(crate) scale_min: f64,
    pub(crate) scale_max: f64,
    pub(crate) raw_min: f32,
    pub(crate) raw_max: f32,
    pub(crate) cell_mm: f64,
    pub(crate) retino_bounds: RetinoBounds,
    pub(crate) retino_params: RetinoParams,
    pub(crate) palette: Vec<[u8; 3]>,
    pub(crate) paper_preset: Option<PaperPresetDetails>,
    pub(crate) rule_preset: Option<RulePresetDetails>,
    pub(crate) planform: Option<PlanformDetails>,
    pub(crate) rule: Option<RuleDetails>,
    pub(crate) calibration: Option<CalibrationReport>,
    pub(crate) orientation_channels: Option<OrientationChannelPayload>,
    pub(crate) params: PayloadParams,
    pub(crate) metrics: Metrics,
    pub(crate) warmup: Warmup,
    pub(crate) timing: Timing,
    pub(crate) data_base64: String,
}

#[derive(Serialize)]
pub(crate) struct PayloadParams {
    pub(crate) model_family: &'static str,
    pub(crate) paper_preset: &'static str,
    pub(crate) rule_preset: &'static str,
    pub(crate) generator: &'static str,
    pub(crate) pattern: &'static str,
    pub(crate) contour_mode: &'static str,
    pub(crate) parity: &'static str,
    n: usize,
    m: usize,
    t: f64,
    pub(crate) frames: usize,
    pub(crate) seed: u64,
    pub(crate) alpha: f64,
    pub(crate) beta: f64,
    pub(crate) mu: f64,
    pub(crate) r0: f64,
    pub(crate) low_percentile: f64,
    pub(crate) high_percentile: f64,
    pub(crate) cmap: &'static str,
    pub(crate) trim_warmup: bool,
    pub(crate) trim_threshold: f64,
    pub(crate) solver: &'static str,
    pub(crate) preview_step: f64,
    pub(crate) wave_count: f64,
    pub(crate) drift: f64,
    pub(crate) pattern_angle: f64,
    pub(crate) sharpness: f64,
    pub(crate) eigen_beta: f64,
    pub(crate) hypercolumn_mm: f64,
    pub(crate) local_sigma_deg: f64,
    pub(crate) local_wide_sigma_deg: f64,
    pub(crate) local_inhibition: f64,
    pub(crate) lateral_sigma: f64,
    pub(crate) lateral_wide_sigma: f64,
    pub(crate) lateral_inhibition: f64,
    pub(crate) lateral_spread_deg: f64,
    pub(crate) stability_q_min: f64,
    pub(crate) stability_q_max: f64,
    pub(crate) stability_samples: usize,
    pub(crate) export_orientation_channels: bool,
    pub(crate) rule_tau_e_ms: f64,
    pub(crate) rule_tau_i_ms: f64,
    pub(crate) rule_aee: f64,
    pub(crate) rule_aei: f64,
    pub(crate) rule_aie: f64,
    pub(crate) rule_aii: f64,
    pub(crate) rule_theta_e: f64,
    pub(crate) rule_theta_i: f64,
    pub(crate) rule_sigma_e: f64,
    pub(crate) rule_sigma_i: f64,
    pub(crate) rule_stim_amplitude: f64,
    pub(crate) rule_stim_period_ms: f64,
    pub(crate) rule_stim_threshold: f64,
    pub(crate) rule_stim_smoothing: f64,
    pub(crate) rule_stim_i_fraction: f64,
    pub(crate) rule_seed_pattern: &'static str,
    pub(crate) rule_seed_strength: f64,
}

pub(crate) fn default_params_json() -> serde_json::Value {
    let defaults = FrameParams::default();
    serde_json::json!({
        "model_family": defaults.generator.model_family(),
        "paper_preset": defaults.paper_preset.as_str(),
        "rule_preset": defaults.rule_preset.as_str(),
        "generator": defaults.generator.as_str(),
        "pattern": defaults.pattern.as_str(),
        "contour_mode": defaults.contour_mode.as_str(),
        "parity": defaults.parity.as_str(),
        "n": defaults.n,
        "m": defaults.m,
        "t": defaults.t,
        "frames": defaults.frames,
        "seed": defaults.seed,
        "alpha": defaults.alpha,
        "beta": defaults.beta,
        "mu": defaults.mu,
        "r0": defaults.r0,
        "low_percentile": defaults.low_percentile,
        "high_percentile": defaults.high_percentile,
        "cmap": defaults.cmap,
        "trim_warmup": defaults.trim_warmup,
        "trim_threshold": defaults.trim_threshold,
        "solver": defaults.solver.as_str(),
        "preview_step": defaults.preview_step,
        "wave_count": defaults.wave_count,
        "drift": defaults.drift,
        "pattern_angle": defaults.pattern_angle,
        "sharpness": defaults.sharpness,
        "eigen_beta": defaults.eigen_beta,
        "hypercolumn_mm": defaults.hypercolumn_mm,
        "local_sigma_deg": defaults.local_sigma_deg,
        "local_wide_sigma_deg": defaults.local_wide_sigma_deg,
        "local_inhibition": defaults.local_inhibition,
        "lateral_sigma": defaults.lateral_sigma,
        "lateral_wide_sigma": defaults.lateral_wide_sigma,
        "lateral_inhibition": defaults.lateral_inhibition,
        "lateral_spread_deg": defaults.lateral_spread_deg,
        "stability_q_min": defaults.stability_q_min,
        "stability_q_max": defaults.stability_q_max,
        "stability_samples": defaults.stability_samples,
        "export_orientation_channels": defaults.export_orientation_channels,
        "rule_tau_e_ms": defaults.rule_tau_e_ms,
        "rule_tau_i_ms": defaults.rule_tau_i_ms,
        "rule_aee": defaults.rule_aee,
        "rule_aei": defaults.rule_aei,
        "rule_aie": defaults.rule_aie,
        "rule_aii": defaults.rule_aii,
        "rule_theta_e": defaults.rule_theta_e,
        "rule_theta_i": defaults.rule_theta_i,
        "rule_sigma_e": defaults.rule_sigma_e,
        "rule_sigma_i": defaults.rule_sigma_i,
        "rule_stim_amplitude": defaults.rule_stim_amplitude,
        "rule_stim_period_ms": defaults.rule_stim_period_ms,
        "rule_stim_threshold": defaults.rule_stim_threshold,
        "rule_stim_smoothing": defaults.rule_stim_smoothing,
        "rule_stim_i_fraction": defaults.rule_stim_i_fraction,
        "rule_seed_pattern": defaults.rule_seed_pattern.as_str(),
        "rule_seed_strength": defaults.rule_seed_strength
    })
}

pub(crate) fn payload_cache_key(params: FrameParams) -> String {
    format!("{params:?}")
}

pub(crate) fn coerce_params(raw: &HashMap<String, String>) -> FrameParams {
    let preset = parse_paper_preset(
        raw.get("paper_preset")
            .or_else(|| raw.get("preset"))
            .map(String::as_str),
    );
    let rule_preset = parse_rule_preset(
        raw.get("rule_preset")
            .or_else(|| raw.get("rule"))
            .map(String::as_str),
    );
    let defaults = if rule_preset != RulePreset::Manual {
        apply_rule_preset(FrameParams::default(), rule_preset)
    } else {
        apply_paper_preset(FrameParams::default(), preset)
    };
    let mut low = get_f64(raw, "low_percentile", defaults.low_percentile, 0.0, 20.0);
    let mut high = get_f64(
        raw,
        "high_percentile",
        defaults.high_percentile,
        80.0,
        100.0,
    );
    if high <= low {
        low = defaults.low_percentile;
        high = defaults.high_percentile;
    }

    FrameParams {
        paper_preset: defaults.paper_preset,
        rule_preset: defaults.rule_preset,
        generator: match raw.get("generator").map(String::as_str) {
            Some("planform") => Generator::Planform,
            Some("dynamics") => Generator::Dynamics,
            Some("rule_flicker") => Generator::RuleFlicker,
            _ => defaults.generator,
        },
        pattern: match raw.get("pattern").map(String::as_str) {
            Some("auto") => PatternPreset::Auto,
            Some("rings") => PatternPreset::Rings,
            Some("rays") => PatternPreset::Rays,
            Some("spiral") => PatternPreset::Spiral,
            Some("honeycomb") => PatternPreset::Honeycomb,
            Some("rhombic") => PatternPreset::Rhombic,
            Some("hex_pi") => PatternPreset::HexPi,
            Some("triangle") => PatternPreset::Triangle,
            _ => defaults.pattern,
        },
        contour_mode: match raw.get("contour_mode").map(String::as_str) {
            Some("noncontoured") => ContourMode::Noncontoured,
            Some("contoured") => ContourMode::Contoured,
            _ => defaults.contour_mode,
        },
        parity: match raw.get("parity").map(String::as_str) {
            Some("odd") => Parity::Odd,
            Some("even") => Parity::Even,
            _ => defaults.parity,
        },
        n: get_usize(raw, "n", defaults.n, 32, 96),
        m: get_usize(raw, "m", defaults.m, 4, 24),
        t: get_f64(raw, "t", defaults.t, 5.0, 800.0),
        frames: get_usize(raw, "frames", defaults.frames, 8, 240),
        seed: get_u64(raw, "seed", defaults.seed),
        alpha: get_f64(raw, "alpha", defaults.alpha, 0.1, 4.0),
        beta: get_f64(raw, "beta", defaults.beta, 0.1, 10.0),
        mu: get_f64(raw, "mu", defaults.mu, 1.0, 40.0),
        r0: get_f64(raw, "r0", defaults.r0, 0.02, 0.14),
        low_percentile: low,
        high_percentile: high,
        cmap: colormap_name(raw.get("cmap").map(String::as_str).unwrap_or(defaults.cmap)),
        trim_warmup: get_bool(raw, "trim_warmup", defaults.trim_warmup),
        trim_threshold: get_f64(raw, "trim_threshold", defaults.trim_threshold, 0.0, 0.5),
        solver: match raw.get("solver").map(String::as_str) {
            Some("accurate") => Solver::Accurate,
            _ => Solver::Preview,
        },
        preview_step: get_f64(raw, "preview_step", defaults.preview_step, 0.02, 1.0),
        wave_count: get_f64(raw, "wave_count", defaults.wave_count, 1.0, 40.0),
        drift: get_f64(raw, "drift", defaults.drift, -4.0, 4.0),
        pattern_angle: get_f64(raw, "pattern_angle", defaults.pattern_angle, 0.0, 90.0),
        sharpness: get_f64(raw, "sharpness", defaults.sharpness, 0.25, 8.0),
        eigen_beta: get_f64(raw, "eigen_beta", defaults.eigen_beta, 0.0, 1.5),
        hypercolumn_mm: get_f64(raw, "hypercolumn_mm", defaults.hypercolumn_mm, 0.1, 4.0),
        local_sigma_deg: get_f64(raw, "local_sigma_deg", defaults.local_sigma_deg, 1.0, 80.0),
        local_wide_sigma_deg: get_f64(
            raw,
            "local_wide_sigma_deg",
            defaults.local_wide_sigma_deg,
            5.0,
            120.0,
        ),
        local_inhibition: get_f64(raw, "local_inhibition", defaults.local_inhibition, 0.0, 3.0),
        lateral_sigma: get_f64(raw, "lateral_sigma", defaults.lateral_sigma, 0.1, 4.0),
        lateral_wide_sigma: get_f64(
            raw,
            "lateral_wide_sigma",
            defaults.lateral_wide_sigma,
            0.1,
            6.0,
        ),
        lateral_inhibition: get_f64(
            raw,
            "lateral_inhibition",
            defaults.lateral_inhibition,
            0.0,
            3.0,
        ),
        lateral_spread_deg: get_f64(
            raw,
            "lateral_spread_deg",
            defaults.lateral_spread_deg,
            0.0,
            90.0,
        ),
        stability_q_min: get_f64(raw, "stability_q_min", defaults.stability_q_min, 0.0, 2.0),
        stability_q_max: get_f64(raw, "stability_q_max", defaults.stability_q_max, 0.2, 8.0),
        stability_samples: get_usize(
            raw,
            "stability_samples",
            defaults.stability_samples,
            16,
            256,
        ),
        export_orientation_channels: get_bool(
            raw,
            "export_orientation_channels",
            defaults.export_orientation_channels,
        ),
        rule_tau_e_ms: get_f64(raw, "rule_tau_e_ms", defaults.rule_tau_e_ms, 1.0, 80.0),
        rule_tau_i_ms: get_f64(raw, "rule_tau_i_ms", defaults.rule_tau_i_ms, 1.0, 120.0),
        rule_aee: get_f64(raw, "rule_aee", defaults.rule_aee, 0.0, 30.0),
        rule_aei: get_f64(raw, "rule_aei", defaults.rule_aei, 0.0, 30.0),
        rule_aie: get_f64(raw, "rule_aie", defaults.rule_aie, 0.0, 30.0),
        rule_aii: get_f64(raw, "rule_aii", defaults.rule_aii, 0.0, 30.0),
        rule_theta_e: get_f64(raw, "rule_theta_e", defaults.rule_theta_e, 0.0, 8.0),
        rule_theta_i: get_f64(raw, "rule_theta_i", defaults.rule_theta_i, 0.0, 8.0),
        rule_sigma_e: get_f64(raw, "rule_sigma_e", defaults.rule_sigma_e, 0.4, 10.0),
        rule_sigma_i: get_f64(raw, "rule_sigma_i", defaults.rule_sigma_i, 0.4, 16.0),
        rule_stim_amplitude: get_f64(
            raw,
            "rule_stim_amplitude",
            defaults.rule_stim_amplitude,
            0.0,
            1.5,
        ),
        rule_stim_period_ms: get_f64(
            raw,
            "rule_stim_period_ms",
            defaults.rule_stim_period_ms,
            20.0,
            180.0,
        ),
        rule_stim_threshold: get_f64(
            raw,
            "rule_stim_threshold",
            defaults.rule_stim_threshold,
            -1.0,
            1.0,
        ),
        rule_stim_smoothing: get_f64(
            raw,
            "rule_stim_smoothing",
            defaults.rule_stim_smoothing,
            0.0,
            100.0,
        ),
        rule_stim_i_fraction: get_f64(
            raw,
            "rule_stim_i_fraction",
            defaults.rule_stim_i_fraction,
            0.0,
            1.0,
        ),
        rule_seed_pattern: match raw.get("rule_seed_pattern").map(String::as_str) {
            Some("random") => RuleSeedPattern::Random,
            Some("hexagonal") => RuleSeedPattern::Hexagonal,
            Some("stripes") => RuleSeedPattern::Stripes,
            _ => defaults.rule_seed_pattern,
        },
        rule_seed_strength: get_f64(
            raw,
            "rule_seed_strength",
            defaults.rule_seed_strength,
            0.0,
            0.2,
        ),
    }
}

pub(crate) fn get_usize(
    raw: &HashMap<String, String>,
    key: &str,
    default: usize,
    min: usize,
    max: usize,
) -> usize {
    raw.get(key)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

pub(crate) fn get_u64(raw: &HashMap<String, String>, key: &str, default: u64) -> u64 {
    raw.get(key)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

pub(crate) fn get_f64(
    raw: &HashMap<String, String>,
    key: &str,
    default: f64,
    min: f64,
    max: f64,
) -> f64 {
    raw.get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

pub(crate) fn get_bool(raw: &HashMap<String, String>, key: &str, default: bool) -> bool {
    raw.get(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

pub(crate) fn generate_payload(
    params: FrameParams,
    state: &ServerState,
) -> Result<Payload, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let (
        mut frames,
        mut times,
        mut orientation_frames,
        matrix_cache_hit,
        matrix_build_sec,
        solve_sec,
    ) = match params.generator {
        Generator::Dynamics => {
            let (structure, cache_hit) = get_structure(params, state);
            let built = Instant::now();
            let (frames, times, orientation_frames) = simulate_frames(params, &structure);
            let solved = Instant::now();
            (
                frames,
                times,
                orientation_frames,
                cache_hit,
                built.duration_since(started).as_secs_f64(),
                solved.duration_since(built).as_secs_f64(),
            )
        }
        Generator::Planform => {
            let built = Instant::now();
            let (frames, times, orientation_frames) = generate_planform_frames(params);
            let solved = Instant::now();
            (
                frames,
                times,
                orientation_frames,
                false,
                built.duration_since(started).as_secs_f64(),
                solved.duration_since(built).as_secs_f64(),
            )
        }
        Generator::RuleFlicker => {
            let built = Instant::now();
            let (frames, times) = simulate_rule_flicker_frames(params);
            let solved = Instant::now();
            (
                frames,
                times,
                None,
                false,
                built.duration_since(started).as_secs_f64(),
                solved.duration_since(built).as_secs_f64(),
            )
        }
    };
    let warmup = match params.generator {
        Generator::Dynamics | Generator::RuleFlicker => {
            trim_warmup(&mut frames, &mut times, orientation_frames.as_mut(), params)
        }
        Generator::Planform => Warmup {
            enabled: false,
            dropped_frames: 0,
            start_time: times.first().copied().unwrap_or(0.0),
            threshold_fraction: params.trim_threshold,
            threshold_std: 0.0,
            max_std: 0.0,
        },
    };
    let (scale_min, scale_max) =
        percentile_range(&frames, params.low_percentile, params.high_percentile);
    let (raw_min, raw_max) = raw_range(&frames);
    let normalized = normalize_u8(&frames, scale_min, scale_max);
    let metrics = frame_metrics(&frames, params.n);
    let cell_mm = cell_mm_for(params);
    let planform = match params.generator {
        Generator::Planform => Some(planform_details(params, cell_mm)),
        Generator::Dynamics | Generator::RuleFlicker => None,
    };
    let rule_preset = rule_preset_details(params.rule_preset);
    let rule = match params.generator {
        Generator::RuleFlicker => {
            Some(rule_details(rule_preset, &frames, &times, &metrics, params))
        }
        Generator::Dynamics | Generator::Planform => None,
    };
    let paper_preset = paper_preset_details(params.paper_preset);
    let calibration = match (&paper_preset, &planform) {
        (Some(preset), Some(planform)) => {
            Some(calibration_report(*preset, planform, &metrics, params))
        }
        _ => None,
    };
    let orientation_channels = if params.generator == Generator::RuleFlicker {
        None
    } else {
        orientation_frames
            .as_ref()
            .map(|channels| orientation_channel_payload(channels, params, times.len()))
    };

    Ok(Payload {
        format: "bressloff-v1-u8-frames",
        model_family: params.generator.model_family(),
        width: params.n,
        height: params.n,
        frame_count: times.len(),
        orientation_count: orientation_count_for(params),
        times,
        scale_min,
        scale_max,
        raw_min,
        raw_max,
        cell_mm,
        retino_bounds: retino_bounds(params.n, cell_mm),
        retino_params: RetinoParams {
            eps: RETINO_EPS,
            w0: RETINO_W0,
            alpha: RETINO_ALPHA,
            beta: RETINO_BETA,
        },
        palette: palette(params.cmap),
        paper_preset,
        rule_preset,
        planform,
        rule,
        calibration,
        orientation_channels,
        params: PayloadParams {
            model_family: params.generator.model_family(),
            paper_preset: params.paper_preset.as_str(),
            rule_preset: params.rule_preset.as_str(),
            generator: params.generator.as_str(),
            pattern: params.pattern.as_str(),
            contour_mode: params.contour_mode.as_str(),
            parity: params.parity.as_str(),
            n: params.n,
            m: params.m,
            t: params.t,
            frames: params.frames,
            seed: params.seed,
            alpha: params.alpha,
            beta: params.beta,
            mu: params.mu,
            r0: params.r0,
            low_percentile: params.low_percentile,
            high_percentile: params.high_percentile,
            cmap: params.cmap,
            trim_warmup: params.trim_warmup,
            trim_threshold: params.trim_threshold,
            solver: params.solver.as_str(),
            preview_step: params.preview_step,
            wave_count: params.wave_count,
            drift: params.drift,
            pattern_angle: params.pattern_angle,
            sharpness: params.sharpness,
            eigen_beta: params.eigen_beta,
            hypercolumn_mm: params.hypercolumn_mm,
            local_sigma_deg: params.local_sigma_deg,
            local_wide_sigma_deg: params.local_wide_sigma_deg,
            local_inhibition: params.local_inhibition,
            lateral_sigma: params.lateral_sigma,
            lateral_wide_sigma: params.lateral_wide_sigma,
            lateral_inhibition: params.lateral_inhibition,
            lateral_spread_deg: params.lateral_spread_deg,
            stability_q_min: params.stability_q_min,
            stability_q_max: params.stability_q_max,
            stability_samples: params.stability_samples,
            export_orientation_channels: params.export_orientation_channels,
            rule_tau_e_ms: params.rule_tau_e_ms,
            rule_tau_i_ms: params.rule_tau_i_ms,
            rule_aee: params.rule_aee,
            rule_aei: params.rule_aei,
            rule_aie: params.rule_aie,
            rule_aii: params.rule_aii,
            rule_theta_e: params.rule_theta_e,
            rule_theta_i: params.rule_theta_i,
            rule_sigma_e: params.rule_sigma_e,
            rule_sigma_i: params.rule_sigma_i,
            rule_stim_amplitude: params.rule_stim_amplitude,
            rule_stim_period_ms: params.rule_stim_period_ms,
            rule_stim_threshold: params.rule_stim_threshold,
            rule_stim_smoothing: params.rule_stim_smoothing,
            rule_stim_i_fraction: params.rule_stim_i_fraction,
            rule_seed_pattern: params.rule_seed_pattern.as_str(),
            rule_seed_strength: params.rule_seed_strength,
        },
        metrics,
        warmup,
        timing: Timing {
            matrix_build_sec,
            solve_sec,
            total_sec: started.elapsed().as_secs_f64(),
            matrix_cache_hit,
            backend: "rust",
        },
        data_base64: general_purpose::STANDARD.encode(normalized),
    })
}

pub(crate) fn get_structure(params: FrameParams, state: &ServerState) -> (Arc<Structure>, bool) {
    let key = StructureKey::new(params);
    if let Some(structure) = state.structures.lock().unwrap().get(&key).cloned() {
        return (structure, true);
    }
    let structure = Arc::new(Structure::new(params.n, params.m, params.r0));
    state
        .structures
        .lock()
        .unwrap()
        .insert(key, Arc::clone(&structure));
    (structure, false)
}

impl Structure {
    pub(crate) fn new(n: usize, m: usize, r0: f64) -> Self {
        let (sigma1, sigma2) = get_lateral_sigmas(r0);
        let step_size = 1.0 / n as f64;
        let cell_area = step_size * step_size;
        let delta_phi = PI / m as f64;
        let cutoff_radius = 3.5 * sigma2;
        let kernel_half_width = (cutoff_radius / step_size).floor() as isize;
        let mut offsets_by_sector = vec![Vec::new(); m];

        for dr in -kernel_half_width..=kernel_half_width {
            for dc in -kernel_half_width..=kernel_half_width {
                let x = step_size * dr as f64;
                let y = step_size * dc as f64;
                let dist = (x * x + y * y).sqrt();
                if dist > cutoff_radius || dist <= step_size / 2.0 {
                    continue;
                }
                let kernel_weight = weight_func(dist, sigma1, sigma2) / dist;
                let angle = y.atan2(x).rem_euclid(2.0 * PI);
                let sector =
                    (((angle + delta_phi / 2.0).rem_euclid(PI)) / delta_phi).floor() as usize;
                offsets_by_sector[sector.min(m - 1)].push(Offset {
                    dr,
                    dc,
                    weight: kernel_weight * cell_area / delta_phi,
                });
            }
        }

        let mut angle_weights = vec![0.0; m * m];
        for k in 0..m {
            for l in 0..m {
                let angle_k = PI * k as f64 / m as f64;
                let angle_l = PI * l as f64 / m as f64;
                angle_weights[k * m + l] = weight_func(
                    angle_dist(angle_k, angle_l),
                    0.6060482974023431,
                    1.538382226567759,
                );
            }
        }

        let cell_count = n * n;
        let mut sector_sources = Vec::with_capacity(m);
        for (sector, offsets) in offsets_by_sector.iter().enumerate() {
            let mut entries = Vec::with_capacity(cell_count * offsets.len());
            for row in 0..n {
                for col in 0..n {
                    for offset in offsets {
                        let source_row = wrap_index(row, offset.dr, n);
                        let source_col = wrap_index(col, offset.dc, n);
                        entries.push(SourceWeight {
                            source_index: index(source_row, source_col, sector, n, m),
                            weight: offset.weight,
                        });
                    }
                }
            }
            sector_sources.push(SectorSources {
                per_cell: offsets.len(),
                entries,
            });
        }

        Self {
            m,
            angle_weights,
            sector_sources,
        }
    }
}

pub(crate) fn simulate_frames(
    params: FrameParams,
    structure: &Structure,
) -> (Vec<f32>, Vec<f64>, Option<Vec<f32>>) {
    let total_dim = params.n * params.n * params.m;
    let mut rng = SplitMix64::new(params.seed);
    let mut state: Vec<f64> = (0..total_dim)
        .map(|_| (rng.next_f64() * 2.0 - 1.0) * 1.0e-12)
        .collect();
    let mut times = Vec::with_capacity(params.frames);
    let mut frames = Vec::with_capacity(params.frames * params.n * params.n);
    let mut orientation_frames = params
        .export_orientation_channels
        .then(|| Vec::with_capacity(params.frames * total_dim));
    let mut sigmoid_buffer = vec![0.0; total_dim];
    let mut coupling_buffer = vec![0.0; total_dim];
    let step = match params.solver {
        Solver::Preview => params.preview_step,
        Solver::Accurate => params.preview_step.min(0.08),
    };
    let mut current_t = 0.0;

    for frame_index in 0..params.frames {
        let target_t = if params.frames <= 1 {
            0.0
        } else {
            params.t * frame_index as f64 / (params.frames - 1) as f64
        };

        while current_t + 1.0e-12 < target_t {
            let dt = step.min(target_t - current_t);
            match params.solver {
                Solver::Preview => step_preview(
                    &mut state,
                    structure,
                    params,
                    dt,
                    &mut sigmoid_buffer,
                    &mut coupling_buffer,
                ),
                Solver::Accurate => step_rk4(&mut state, structure, params, dt),
            }
            current_t += dt;
        }

        times.push(target_t);
        append_scalar_frame(&state, params, &mut frames);
        if let Some(channels) = orientation_frames.as_mut() {
            channels.extend(state.iter().map(|value| *value as f32));
        }
    }

    (frames, times, orientation_frames)
}

pub(crate) fn append_scalar_frame(state: &[f64], params: FrameParams, frames: &mut Vec<f32>) {
    for cell in 0..params.n * params.n {
        let base = cell * params.m;
        let mut sum = 0.0;
        for k in 0..params.m {
            sum += state[base + k];
        }
        frames.push((sum / params.m as f64) as f32);
    }
}

pub(crate) fn step_preview(
    state: &mut [f64],
    structure: &Structure,
    params: FrameParams,
    dt: f64,
    sigmoid_buffer: &mut [f64],
    coupling_buffer: &mut [f64],
) {
    connectivity_into(state, structure, params, sigmoid_buffer, coupling_buffer);
    let decay = 1.0 + params.alpha * dt;
    state
        .par_iter_mut()
        .zip(coupling_buffer.par_iter())
        .for_each(|(value, coupling)| {
            *value = (*value + dt * coupling) / decay;
        });
}

pub(crate) fn step_rk4(state: &mut [f64], structure: &Structure, params: FrameParams, dt: f64) {
    let k1 = derivative(state, structure, params);
    let tmp2: Vec<f64> = state
        .iter()
        .zip(k1.iter())
        .map(|(a, k)| a + 0.5 * dt * k)
        .collect();
    let k2 = derivative(&tmp2, structure, params);
    let tmp3: Vec<f64> = state
        .iter()
        .zip(k2.iter())
        .map(|(a, k)| a + 0.5 * dt * k)
        .collect();
    let k3 = derivative(&tmp3, structure, params);
    let tmp4: Vec<f64> = state
        .iter()
        .zip(k3.iter())
        .map(|(a, k)| a + dt * k)
        .collect();
    let k4 = derivative(&tmp4, structure, params);
    state.par_iter_mut().enumerate().for_each(|(i, value)| {
        *value += dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;
    });
}

pub(crate) fn derivative(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut conn = connectivity(state, structure, params);
    conn.par_iter_mut()
        .zip(state.par_iter())
        .for_each(|(value, a)| *value -= params.alpha * a);
    conn
}

pub(crate) fn connectivity(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut sigmoid_state = vec![0.0; state.len()];
    let mut out = vec![0.0; state.len()];
    connectivity_into(state, structure, params, &mut sigmoid_state, &mut out);
    out
}

pub(crate) fn connectivity_into(
    state: &[f64],
    structure: &Structure,
    params: FrameParams,
    sigmoid_state: &mut [f64],
    out: &mut [f64],
) {
    let m = structure.m;
    sigmoid_state
        .par_iter_mut()
        .zip(state.par_iter())
        .for_each(|(target, value)| *target = sigmoid(*value));

    out.par_chunks_mut(m).enumerate().for_each(|(cell, chunk)| {
        let base = cell * m;
        for (k, target) in chunk.iter_mut().enumerate().take(m) {
            let mut angular_sum = 0.0;
            for l in 0..m {
                if l != k {
                    angular_sum +=
                        structure.angle_weights[k * m + l] * sigmoid_state[base + l] / m as f64;
                }
            }

            let mut lateral_sum = 0.0;
            let sector = &structure.sector_sources[k];
            let start = cell * sector.per_cell;
            let end = start + sector.per_cell;
            for source in &sector.entries[start..end] {
                lateral_sum += source.weight * sigmoid_state[source.source_index];
            }
            *target = params.mu * (angular_sum + params.beta * lateral_sum);
        }
    });
}

pub(crate) fn wrap_index(value: usize, delta: isize, size: usize) -> usize {
    (value as isize + delta).rem_euclid(size as isize) as usize
}

pub(crate) fn index(row: usize, col: usize, k: usize, n: usize, m: usize) -> usize {
    m * n * row + m * col + k
}

pub(crate) fn trim_warmup(
    frames: &mut Vec<f32>,
    times: &mut Vec<f64>,
    orientation_frames: Option<&mut Vec<f32>>,
    params: FrameParams,
) -> Warmup {
    if !params.trim_warmup || times.len() <= 3 {
        return Warmup {
            enabled: params.trim_warmup,
            dropped_frames: 0,
            start_time: times.first().copied().unwrap_or(0.0),
            threshold_fraction: params.trim_threshold,
            threshold_std: 0.0,
            max_std: 0.0,
        };
    }

    let frame_size = params.n * params.n;
    let contrast: Vec<f32> = frames.chunks(frame_size).map(stddev).collect();
    let max_std = contrast.iter().copied().fold(0.0_f32, f32::max);
    let threshold_std = max_std * params.trim_threshold as f32;
    let mut start = contrast
        .iter()
        .position(|value| *value >= threshold_std)
        .unwrap_or(0)
        .saturating_sub(2);
    let min_remaining = times.len().min(16.max(times.len() / 3));
    start = start.min(times.len().saturating_sub(min_remaining));

    if start > 0 {
        frames.drain(0..start * frame_size);
        if let Some(channels) = orientation_frames {
            channels.drain(0..start * frame_size * params.m);
        }
        times.drain(0..start);
    }

    Warmup {
        enabled: true,
        dropped_frames: start,
        start_time: times.first().copied().unwrap_or(0.0),
        threshold_fraction: params.trim_threshold,
        threshold_std,
        max_std,
    }
}

pub(crate) fn percentile_range(
    frames: &[f32],
    low_percentile: f64,
    high_percentile: f64,
) -> (f64, f64) {
    let mut sorted = frames.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let low = percentile_sorted(&sorted, low_percentile);
    let mut high = percentile_sorted(&sorted, high_percentile);
    if high <= low {
        high = low + 1.0e-9;
    }
    (low as f64, high as f64)
}

pub(crate) fn percentile_sorted(sorted: &[f32], percentile: f64) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (percentile.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let t = (rank - lo as f64) as f32;
        sorted[lo] * (1.0 - t) + sorted[hi] * t
    }
}

pub(crate) fn normalize_u8(frames: &[f32], low: f64, high: f64) -> Vec<u8> {
    let denom = (high - low).max(1.0e-9);
    frames
        .iter()
        .map(|value| (((*value as f64 - low) / denom) * 255.0).clamp(0.0, 255.0) as u8)
        .collect()
}

pub(crate) fn orientation_channel_payload(
    channels: &[f32],
    params: FrameParams,
    frame_count: usize,
) -> OrientationChannelPayload {
    let (scale_min, scale_max) = percentile_range(channels, 0.5, 99.5);
    let (raw_min, raw_max) = raw_range(channels);
    OrientationChannelPayload {
        format: "bressloff-v1-u8-orientation-channels",
        order: "frame,row,col,orientation",
        width: params.n,
        height: params.n,
        frame_count,
        orientation_count: params.m,
        phi_radians: (0..params.m)
            .map(|k| PI * k as f64 / params.m as f64)
            .collect(),
        scale_min,
        scale_max,
        raw_min,
        raw_max,
        data_base64: general_purpose::STANDARD.encode(normalize_u8(channels, scale_min, scale_max)),
    }
}

pub(crate) fn raw_range(frames: &[f32]) -> (f32, f32) {
    frames
        .iter()
        .copied()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        })
}

pub(crate) fn frame_metrics(frames: &[f32], n: usize) -> Metrics {
    let frame_size = n * n;
    let Some(final_frame) = frames.chunks(frame_size).last() else {
        return Metrics {
            final_mean: 0.0,
            final_std: 0.0,
            final_range: 0.0,
            dominant_cycles: 0.0,
            temporal_delta: 0.0,
        };
    };
    let mean = final_frame.iter().sum::<f32>() / final_frame.len() as f32;
    let std = stddev(final_frame);
    let (lo, hi) = raw_range(final_frame);
    let dominant_cycles = projected_dominant_cycles(final_frame, n);
    let temporal_delta = if frames.len() > frame_size {
        frames
            .windows(frame_size * 2)
            .step_by(frame_size)
            .map(|pair| {
                pair[..frame_size]
                    .iter()
                    .zip(pair[frame_size..].iter())
                    .map(|(a, b)| (b - a).abs())
                    .sum::<f32>()
                    / frame_size as f32
            })
            .sum::<f32>()
            / (frames.len() / frame_size - 1) as f32
    } else {
        0.0
    };

    Metrics {
        final_mean: mean,
        final_std: std,
        final_range: hi - lo,
        dominant_cycles,
        temporal_delta,
    }
}

pub(crate) fn stddev(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / values.len() as f32;
    variance.sqrt()
}

pub(crate) fn projected_dominant_cycles(frame: &[f32], n: usize) -> f32 {
    let mut row_profile = vec![0.0_f32; n];
    let mut col_profile = vec![0.0_f32; n];
    for row in 0..n {
        for col in 0..n {
            let value = frame[row * n + col];
            row_profile[row] += value;
            col_profile[col] += value;
        }
    }
    let row_k = dominant_1d_frequency(&row_profile);
    let col_k = dominant_1d_frequency(&col_profile);
    ((row_k * row_k + col_k * col_k) as f32).sqrt()
}

pub(crate) fn dominant_1d_frequency(values: &[f32]) -> usize {
    let n = values.len();
    let mean = values.iter().sum::<f32>() / n as f32;
    let mut best_k = 0;
    let mut best_power = 0.0_f64;
    for k in 1..n / 2 {
        let mut re = 0.0;
        let mut im = 0.0;
        for (i, value) in values.iter().enumerate() {
            let angle = -2.0 * PI * k as f64 * i as f64 / n as f64;
            let centered = (*value - mean) as f64;
            re += centered * angle.cos();
            im += centered * angle.sin();
        }
        let power = re * re + im * im;
        if power > best_power {
            best_power = power;
            best_k = k;
        }
    }
    best_k
}

pub(crate) fn retino_bounds(n: usize, cell_mm: f64) -> RetinoBounds {
    let mut bounds = RetinoBounds {
        min_x: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        min_y: f64::INFINITY,
        max_y: f64::NEG_INFINITY,
    };
    for row in 0..=n {
        for col in 0..=n {
            let x = cell_mm * col as f64 - n as f64 * cell_mm / 2.0;
            let y = cell_mm * row as f64 - n as f64 * cell_mm / 2.0;
            let (rx, ry) = inverse_retino_cortical_map(x, y);
            bounds.min_x = bounds.min_x.min(rx);
            bounds.max_x = bounds.max_x.max(rx);
            bounds.min_y = bounds.min_y.min(ry);
            bounds.max_y = bounds.max_y.max(ry);
        }
    }
    bounds
}

pub(crate) fn inverse_retino_cortical_map(x: f64, y: f64) -> (f64, f64) {
    let r = RETINO_W0 / RETINO_EPS * (RETINO_EPS * x / RETINO_ALPHA).exp();
    let theta = RETINO_EPS * y / RETINO_BETA;
    (r * theta.cos(), r * theta.sin())
}

pub(crate) fn palette(name: &str) -> Vec<[u8; 3]> {
    (0..256)
        .map(|i| {
            let t = i as f64 / 255.0;
            match name {
                "gray" => {
                    let v = (255.0 * t).round() as u8;
                    [v, v, v]
                }
                "viridis" => interpolate_stops(t, VIRIDIS),
                "magma" => interpolate_stops(t, MAGMA),
                "inferno" => interpolate_stops(t, INFERNO),
                "turbo" => turbo(t),
                _ => interpolate_stops(t, TWILIGHT),
            }
        })
        .collect()
}

pub(crate) fn colormap_name(name: &str) -> &'static str {
    match name {
        "viridis" => "viridis",
        "magma" => "magma",
        "inferno" => "inferno",
        "turbo" => "turbo",
        "gray" => "gray",
        _ => "twilight",
    }
}

type Stop = (f64, u8, u8, u8);

const TWILIGHT: &[Stop] = &[
    (0.0, 34, 25, 74),
    (0.18, 68, 56, 130),
    (0.36, 64, 125, 177),
    (0.50, 222, 219, 221),
    (0.66, 190, 91, 81),
    (0.84, 93, 35, 95),
    (1.0, 34, 25, 74),
];
const VIRIDIS: &[Stop] = &[
    (0.0, 68, 1, 84),
    (0.25, 59, 82, 139),
    (0.5, 33, 145, 140),
    (0.75, 94, 201, 98),
    (1.0, 253, 231, 37),
];
const MAGMA: &[Stop] = &[
    (0.0, 0, 0, 4),
    (0.25, 80, 18, 123),
    (0.5, 182, 54, 121),
    (0.75, 251, 136, 97),
    (1.0, 252, 253, 191),
];
const INFERNO: &[Stop] = &[
    (0.0, 0, 0, 4),
    (0.25, 87, 15, 109),
    (0.5, 188, 55, 84),
    (0.75, 249, 142, 8),
    (1.0, 252, 255, 164),
];

pub(crate) fn interpolate_stops(t: f64, stops: &[Stop]) -> [u8; 3] {
    for window in stops.windows(2) {
        let (a_t, ar, ag, ab) = window[0];
        let (b_t, br, bg, bb) = window[1];
        if t <= b_t {
            let local = ((t - a_t) / (b_t - a_t)).clamp(0.0, 1.0);
            return [
                lerp_u8(ar, br, local),
                lerp_u8(ag, bg, local),
                lerp_u8(ab, bb, local),
            ];
        }
    }
    let (_, r, g, b) = stops[stops.len() - 1];
    [r, g, b]
}

pub(crate) fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

pub(crate) fn turbo(t: f64) -> [u8; 3] {
    interpolate_stops(
        t,
        &[
            (0.0, 48, 18, 59),
            (0.18, 34, 113, 186),
            (0.36, 29, 185, 206),
            (0.54, 112, 218, 87),
            (0.72, 249, 206, 57),
            (0.88, 238, 98, 38),
            (1.0, 122, 4, 3),
        ],
    )
}

pub(crate) fn weight_func(x: f64, sigma1: f64, sigma2: f64) -> f64 {
    (-(x * x) / (2.0 * sigma1 * sigma1)).exp() / sigma1
        - (-(x * x) / (2.0 * sigma2 * sigma2)).exp() / sigma2
}

pub(crate) fn sigmoid(x: f64) -> f64 {
    if x < -4.0 {
        0.0
    } else if x > 4.0 {
        1.0
    } else {
        1.0 / (1.0 + (-2.0 * x).exp())
    }
}

pub(crate) fn angle_dist(angle1: f64, angle2: f64) -> f64 {
    PI / 2.0 - (PI / 2.0 - (angle1 - angle2).abs().rem_euclid(PI)).abs()
}

pub(crate) fn get_lateral_sigmas(r0: f64) -> (f64, f64) {
    let mut sigma2 = r0;
    for _ in 0..100 {
        let diff = r0 - ((2.0 * sigma2 * (sigma2 + 1.0).ln()) / (sigma2 + 2.0)).sqrt();
        if diff.abs() < 1.0e-7 {
            return (sigma2 / (1.0 + sigma2), sigma2);
        }
        sigma2 += diff;
    }
    (sigma2 / (1.0 + sigma2), sigma2)
}

pub(crate) struct SplitMix64 {
    pub(crate) state: u64,
}

impl SplitMix64 {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    pub(crate) fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_generates_payload() {
        let state = ServerState::default();
        let params = FrameParams {
            n: 32,
            m: 4,
            frames: 16,
            t: 10.0,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        assert_eq!(payload.width, 32);
        assert_eq!(payload.orientation_count, 4);
        assert!(!payload.data_base64.is_empty());
    }

    #[test]
    fn cache_marks_second_structure_hit() {
        let state = ServerState::default();
        let params = FrameParams {
            n: 32,
            m: 4,
            frames: 8,
            t: 5.0,
            ..FrameParams::default()
        };
        let first = generate_payload(params, &state).unwrap();
        let second = generate_payload(
            FrameParams {
                alpha: 1.2,
                ..params
            },
            &state,
        )
        .unwrap();
        assert!(!first.timing.matrix_cache_hit);
        assert!(second.timing.matrix_cache_hit);
    }
}
