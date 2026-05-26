use crate::models::{bressloff::presets::PaperPreset, rule::presets::RulePreset};

pub(crate) const PI: f64 = std::f64::consts::PI;
pub(crate) const DYNAMIC_CELL_MM: f64 = 0.7;
pub(crate) const RETINO_EPS: f64 = 0.051;
pub(crate) const RETINO_W0: f64 = 0.087;
pub(crate) const RETINO_ALPHA: f64 = 3.0 / PI;
pub(crate) const RETINO_BETA: f64 = 1.589 / 2.0;
pub(crate) const MODEL_FAMILY_BRESSLOFF: &str = "bressloff_orientation_hypercolumn";
pub(crate) const MODEL_FAMILY_RULE: &str = "rule_flicker_ei";
pub(crate) const MODEL_FAMILY_DRIVEN_ORTHOGONAL: &str = "spatial_forcing_orthogonal_response";
pub(crate) const MODEL_FAMILY_MACKAY: &str = "mackay_localized_input";
pub(crate) const MODEL_FAMILY_LOCALIZED_PERIODIC: &str = "localized_time_periodic_input";
pub(crate) const RULE_FIGURE8_SOURCE_BETA_PER_MODEL_CYCLE: f64 = 0.42868451880191133;

#[derive(Clone, Copy, Debug)]
pub(crate) struct FrameParams {
    pub(crate) paper_preset: PaperPreset,
    pub(crate) rule_preset: RulePreset,
    pub(crate) generator: Generator,
    pub(crate) pattern: PatternPreset,
    pub(crate) contour_mode: ContourMode,
    pub(crate) parity: Parity,
    pub(crate) n: usize,
    pub(crate) m: usize,
    pub(crate) t: f64,
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
    pub(crate) solver: Solver,
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
    pub(crate) rule_seed_pattern: RuleSeedPattern,
    pub(crate) rule_seed_strength: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Generator {
    Dynamics,
    Planform,
    RuleFlicker,
}

impl Generator {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Generator::Dynamics => "dynamics",
            Generator::Planform => "planform",
            Generator::RuleFlicker => "rule_flicker",
        }
    }

    pub(crate) fn model_family(self) -> &'static str {
        match self {
            Generator::Dynamics | Generator::Planform => MODEL_FAMILY_BRESSLOFF,
            Generator::RuleFlicker => MODEL_FAMILY_RULE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PatternPreset {
    Auto,
    Rings,
    Rays,
    Spiral,
    Cobweb,
    Honeycomb,
    Rhombic,
    HexPi,
    Triangle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuleSeedPattern {
    Random,
    Stripes,
    Hexagonal,
}

impl RuleSeedPattern {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            RuleSeedPattern::Random => "random",
            RuleSeedPattern::Stripes => "stripes",
            RuleSeedPattern::Hexagonal => "hexagonal",
        }
    }
}

impl PatternPreset {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PatternPreset::Auto => "auto",
            PatternPreset::Rings => "rings",
            PatternPreset::Rays => "rays",
            PatternPreset::Spiral => "spiral",
            PatternPreset::Cobweb => "cobweb",
            PatternPreset::Honeycomb => "honeycomb",
            PatternPreset::Rhombic => "rhombic",
            PatternPreset::HexPi => "hex_pi",
            PatternPreset::Triangle => "triangle",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContourMode {
    Contoured,
    Noncontoured,
}

impl ContourMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ContourMode::Contoured => "contoured",
            ContourMode::Noncontoured => "noncontoured",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Parity {
    Even,
    Odd,
}

impl Parity {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Parity::Even => "even",
            Parity::Odd => "odd",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Solver {
    Preview,
    Accurate,
}

impl Solver {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Solver::Preview => "preview",
            Solver::Accurate => "accurate",
        }
    }
}

impl Default for FrameParams {
    fn default() -> Self {
        Self {
            paper_preset: PaperPreset::Manual,
            rule_preset: RulePreset::Manual,
            generator: Generator::Dynamics,
            pattern: PatternPreset::Cobweb,
            contour_mode: ContourMode::Contoured,
            parity: Parity::Even,
            n: 128,
            m: 12,
            t: 60.0,
            frames: 120,
            seed: 20_260_522,
            alpha: 1.0,
            beta: 3.0,
            mu: 17.0,
            r0: 3.2 / 50.0,
            low_percentile: 1.0,
            high_percentile: 99.0,
            cmap: "twilight",
            trim_warmup: true,
            trim_threshold: 0.08,
            solver: Solver::Preview,
            preview_step: 0.5,
            wave_count: 12.0,
            drift: 0.35,
            pattern_angle: 45.0,
            sharpness: 1.0,
            eigen_beta: 0.35,
            hypercolumn_mm: 2.0,
            local_sigma_deg: 20.0,
            local_wide_sigma_deg: 60.0,
            local_inhibition: 1.0,
            lateral_sigma: 1.0,
            lateral_wide_sigma: 1.5,
            lateral_inhibition: 1.0,
            lateral_spread_deg: 0.0,
            stability_q_min: 0.05,
            stability_q_max: 3.5,
            stability_samples: 80,
            export_orientation_channels: false,
            rule_tau_e_ms: 10.0,
            rule_tau_i_ms: 20.0,
            rule_aee: 10.0,
            rule_aei: 12.0,
            rule_aie: 8.5,
            rule_aii: 3.0,
            rule_theta_e: 2.0,
            rule_theta_i: 3.5,
            rule_sigma_e: 2.0,
            rule_sigma_i: 5.0,
            rule_stim_amplitude: 0.8,
            rule_stim_period_ms: 55.0,
            rule_stim_threshold: 0.8,
            rule_stim_smoothing: 50.0,
            rule_stim_i_fraction: 0.0,
            rule_seed_pattern: RuleSeedPattern::Stripes,
            rule_seed_strength: 0.04,
        }
    }
}
