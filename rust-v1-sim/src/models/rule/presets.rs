use crate::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RulePreset {
    Manual,
    Fig4HighFreqStripes,
    Fig4LowFreqHexagons,
    Fig5PeriodDoubledStripes,
    Fig5OneToOneHexagons,
}

impl RulePreset {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            RulePreset::Manual => "manual",
            RulePreset::Fig4HighFreqStripes => "rule_fig4_high_freq_stripes",
            RulePreset::Fig4LowFreqHexagons => "rule_fig4_low_freq_hexagons",
            RulePreset::Fig5PeriodDoubledStripes => "rule_fig5_period_doubled_stripes",
            RulePreset::Fig5OneToOneHexagons => "rule_fig5_one_to_one_hexagons",
        }
    }
}

struct RulePresetRegistryEntry {
    preset: RulePreset,
    details: RulePresetDetails,
}

macro_rules! rule_preset_entry {
    (
        $preset:ident,
        label: $label:literal,
        source_page: $source_page:literal,
        paper_figure: $paper_figure:literal,
        render_domain: $render_domain:literal,
        expected_response_mode: $expected_response_mode:literal,
        expected_family: $expected_family:literal,
        expected_period_ms: $expected_period_ms:expr,
        calibration_status: $calibration_status:literal,
        source_note: $source_note:literal $(,)?
    ) => {
        RulePresetRegistryEntry {
            preset: RulePreset::$preset,
            details: RulePresetDetails {
                id: RulePreset::$preset.as_str(),
                label: $label,
                source_key: "rule-2011",
                model_family: MODEL_FAMILY_RULE,
                render_domain: $render_domain,
                source_page: $source_page,
                paper_figure: $paper_figure,
                expected_response_mode: $expected_response_mode,
                expected_family: $expected_family,
                expected_period_ms: $expected_period_ms,
                calibration_status: $calibration_status,
                source_note: $source_note,
            },
        }
    };
}

static RULE_PRESET_REGISTRY: &[RulePresetRegistryEntry] = &[
    rule_preset_entry!(
        Fig4HighFreqStripes,
        label: "Rule Fig 4 high-frequency period-doubled stripes",
        source_page: "5",
        paper_figure: "Figure 4",
        render_domain: "cortical",
        expected_response_mode: "period_doubled",
        expected_family: "stripe",
        expected_period_ms: 55.0,
        calibration_status: "qualitative-seeded",
        source_note: "Deterministic qualitative preset for the high-frequency stripe island; full Floquet and sweep calibration is deferred.",
    ),
    rule_preset_entry!(
        Fig4LowFreqHexagons,
        label: "Rule Fig 4 low-frequency one-to-one hexagons",
        source_page: "5",
        paper_figure: "Figure 4",
        render_domain: "cortical",
        expected_response_mode: "one_to_one",
        expected_family: "hexagonal",
        expected_period_ms: 120.0,
        calibration_status: "qualitative-seeded",
        source_note: "Deterministic qualitative preset for the low-frequency hexagonal island; full phase-diagram calibration is deferred.",
    ),
    rule_preset_entry!(
        Fig5PeriodDoubledStripes,
        label: "Rule Fig 5 period-doubled stripe frames",
        source_page: "6",
        paper_figure: "Figure 5A",
        render_domain: "cortical",
        expected_response_mode: "period_doubled",
        expected_family: "stripe",
        expected_period_ms: 55.0,
        calibration_status: "qualitative-seeded",
        source_note: "High-frequency example where the foreground/background swap after one stimulus period.",
    ),
    rule_preset_entry!(
        Fig5OneToOneHexagons,
        label: "Rule Fig 5 one-to-one hexagon frames",
        source_page: "6",
        paper_figure: "Figure 5B",
        render_domain: "cortical",
        expected_response_mode: "one_to_one",
        expected_family: "hexagonal",
        expected_period_ms: 120.0,
        calibration_status: "qualitative-seeded",
        source_note: "Low-frequency example where the spatial pattern repeats with the stimulus period.",
    ),
];

pub(crate) fn parse_rule_preset(value: Option<&str>) -> RulePreset {
    value
        .and_then(|id| {
            RULE_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == id)
                .map(|entry| entry.preset)
        })
        .unwrap_or(RulePreset::Manual)
}

pub(crate) fn rule_preset_details(preset: RulePreset) -> Option<RulePresetDetails> {
    RULE_PRESET_REGISTRY
        .iter()
        .find(|entry| entry.preset == preset)
        .map(|entry| entry.details)
}

pub(crate) fn rule_preset_catalog() -> Vec<RulePresetDetails> {
    RULE_PRESET_REGISTRY
        .iter()
        .map(|entry| entry.details)
        .collect()
}

pub(crate) fn apply_rule_preset(mut params: FrameParams, preset: RulePreset) -> FrameParams {
    params.rule_preset = preset;
    if preset != RulePreset::Manual {
        params.paper_preset = PaperPreset::Manual;
        params.generator = Generator::RuleFlicker;
        params.contour_mode = ContourMode::Noncontoured;
        params.n = 40;
        params.m = 4;
        params.frames = 144;
        params.low_percentile = 1.0;
        params.high_percentile = 99.0;
        params.trim_warmup = false;
        params.solver = Solver::Preview;
        params.preview_step = 0.5;
        params.rule_tau_e_ms = 10.0;
        params.rule_tau_i_ms = 20.0;
        params.rule_aee = 10.0;
        params.rule_aei = 12.0;
        params.rule_aie = 8.5;
        params.rule_aii = 3.0;
        params.rule_theta_e = 2.0;
        params.rule_theta_i = 3.5;
        params.rule_sigma_e = 2.0;
        params.rule_sigma_i = 5.0;
        params.rule_stim_amplitude = 0.8;
        params.rule_stim_threshold = 0.8;
        params.rule_stim_smoothing = 50.0;
        params.rule_stim_i_fraction = 0.0;
        params.rule_seed_strength = 0.2;
    }
    match preset {
        RulePreset::Manual => params,
        RulePreset::Fig4HighFreqStripes | RulePreset::Fig5PeriodDoubledStripes => {
            params.t = 440.0;
            params.rule_stim_period_ms = 55.0;
            params.rule_seed_pattern = RuleSeedPattern::Stripes;
            params
        }
        RulePreset::Fig4LowFreqHexagons | RulePreset::Fig5OneToOneHexagons => {
            params.t = 660.0;
            params.rule_stim_period_ms = 120.0;
            params.rule_stim_amplitude = 1.0;
            params.rule_seed_pattern = RuleSeedPattern::Hexagonal;
            params
        }
    }
}
