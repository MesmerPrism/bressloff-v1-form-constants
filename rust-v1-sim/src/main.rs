#![recursion_limit = "256"]

use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

const PI: f64 = std::f64::consts::PI;
const DYNAMIC_CELL_MM: f64 = 0.7;
const RETINO_EPS: f64 = 0.051;
const RETINO_W0: f64 = 0.087;
const RETINO_ALPHA: f64 = 3.0 / PI;
const RETINO_BETA: f64 = 1.589 / 2.0;
const MODEL_FAMILY_BRESSLOFF: &str = "bressloff_orientation_hypercolumn";
const MODEL_FAMILY_RULE: &str = "rule_flicker_ei";

#[derive(Clone, Copy, Debug)]
struct FrameParams {
    paper_preset: PaperPreset,
    rule_preset: RulePreset,
    generator: Generator,
    pattern: PatternPreset,
    contour_mode: ContourMode,
    parity: Parity,
    n: usize,
    m: usize,
    t: f64,
    frames: usize,
    seed: u64,
    alpha: f64,
    beta: f64,
    mu: f64,
    r0: f64,
    low_percentile: f64,
    high_percentile: f64,
    cmap: &'static str,
    trim_warmup: bool,
    trim_threshold: f64,
    solver: Solver,
    preview_step: f64,
    wave_count: f64,
    drift: f64,
    pattern_angle: f64,
    sharpness: f64,
    eigen_beta: f64,
    hypercolumn_mm: f64,
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
    stability_q_min: f64,
    stability_q_max: f64,
    stability_samples: usize,
    export_orientation_channels: bool,
    rule_tau_e_ms: f64,
    rule_tau_i_ms: f64,
    rule_aee: f64,
    rule_aei: f64,
    rule_aie: f64,
    rule_aii: f64,
    rule_theta_e: f64,
    rule_theta_i: f64,
    rule_sigma_e: f64,
    rule_sigma_i: f64,
    rule_stim_amplitude: f64,
    rule_stim_period_ms: f64,
    rule_stim_threshold: f64,
    rule_stim_smoothing: f64,
    rule_stim_i_fraction: f64,
    rule_seed_pattern: RuleSeedPattern,
    rule_seed_strength: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Generator {
    Dynamics,
    Planform,
    RuleFlicker,
}

impl Generator {
    fn as_str(self) -> &'static str {
        match self {
            Generator::Dynamics => "dynamics",
            Generator::Planform => "planform",
            Generator::RuleFlicker => "rule_flicker",
        }
    }

    fn model_family(self) -> &'static str {
        match self {
            Generator::Dynamics | Generator::Planform => MODEL_FAMILY_BRESSLOFF,
            Generator::RuleFlicker => MODEL_FAMILY_RULE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatternPreset {
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
enum RuleSeedPattern {
    Random,
    Stripes,
    Hexagonal,
}

impl RuleSeedPattern {
    fn as_str(self) -> &'static str {
        match self {
            RuleSeedPattern::Random => "random",
            RuleSeedPattern::Stripes => "stripes",
            RuleSeedPattern::Hexagonal => "hexagonal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaperPreset {
    Manual,
    Fig16Odd,
    Fig17Even,
    Fig29SquareNoncontoured,
    Fig29RollNoncontoured,
    Fig30RhombicNoncontoured,
    Fig30HexNoncontoured,
    Fig31SquareEven,
    Fig31SquareEvenRoll,
    Fig32SquareOdd,
    Fig32SquareOddRoll,
    Fig33RhombicEven,
    Fig33RhombicEvenRoll,
    Fig34RhombicOdd,
    Fig34RhombicOddRoll,
    Fig35HexEven,
    Fig35HexZeroEven,
    Fig36TriangleOdd,
    Fig36HexZeroOdd,
    Fig5RollCortical,
    Fig5HexCortical,
    Fig5HoneycombCortical,
    Fig5SquareCortical,
    Fig6VisualFieldPlanforms,
    Fig7LatticeTunnel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RulePreset {
    Manual,
    Fig4HighFreqStripes,
    Fig4LowFreqHexagons,
    Fig5PeriodDoubledStripes,
    Fig5OneToOneHexagons,
}

impl RulePreset {
    const fn as_str(self) -> &'static str {
        match self {
            RulePreset::Manual => "manual",
            RulePreset::Fig4HighFreqStripes => "rule_fig4_high_freq_stripes",
            RulePreset::Fig4LowFreqHexagons => "rule_fig4_low_freq_hexagons",
            RulePreset::Fig5PeriodDoubledStripes => "rule_fig5_period_doubled_stripes",
            RulePreset::Fig5OneToOneHexagons => "rule_fig5_one_to_one_hexagons",
        }
    }
}

impl PaperPreset {
    const fn as_str(self) -> &'static str {
        match self {
            PaperPreset::Manual => "manual",
            PaperPreset::Fig16Odd => "fig16_odd",
            PaperPreset::Fig17Even => "fig17_even",
            PaperPreset::Fig29SquareNoncontoured => "fig29_square_noncontoured",
            PaperPreset::Fig29RollNoncontoured => "fig29_roll_noncontoured",
            PaperPreset::Fig30RhombicNoncontoured => "fig30_rhombic_noncontoured",
            PaperPreset::Fig30HexNoncontoured => "fig30_hex_noncontoured",
            PaperPreset::Fig31SquareEven => "fig31_square_even",
            PaperPreset::Fig31SquareEvenRoll => "fig31_square_even_roll",
            PaperPreset::Fig32SquareOdd => "fig32_square_odd",
            PaperPreset::Fig32SquareOddRoll => "fig32_square_odd_roll",
            PaperPreset::Fig33RhombicEven => "fig33_rhombic_even",
            PaperPreset::Fig34RhombicOdd => "fig34_rhombic_odd",
            PaperPreset::Fig33RhombicEvenRoll => "fig33_rhombic_even_roll",
            PaperPreset::Fig34RhombicOddRoll => "fig34_rhombic_odd_roll",
            PaperPreset::Fig35HexEven => "fig35_hex_even",
            PaperPreset::Fig35HexZeroEven => "fig35_hex_zero_even",
            PaperPreset::Fig36TriangleOdd => "fig36_triangle_odd",
            PaperPreset::Fig36HexZeroOdd => "fig36_hex_zero_odd",
            PaperPreset::Fig5RollCortical => "fig5_roll_cortical",
            PaperPreset::Fig5HexCortical => "fig5_hex_cortical",
            PaperPreset::Fig5HoneycombCortical => "fig5_honeycomb_cortical",
            PaperPreset::Fig5SquareCortical => "fig5_square_cortical",
            PaperPreset::Fig6VisualFieldPlanforms => "fig6_visual_field_planforms",
            PaperPreset::Fig7LatticeTunnel => "fig7_lattice_tunnel",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PaperPresetRegistryEntry {
    preset: PaperPreset,
    details: PaperPresetDetails,
}

macro_rules! paper_preset_entry {
    (
        $preset:ident,
        label: $label:literal,
        source_key: $source_key:literal,
        source_page: $source_page:literal,
        paper_figure: $paper_figure:literal,
        source_table: $source_table:literal,
        source_view: $source_view:literal,
        expected_kind: $expected_kind:literal,
        expected_contour_mode: $expected_contour_mode:literal,
        expected_parity: $expected_parity:literal,
        expected_family: $expected_family:literal,
        expected_pattern: $expected_pattern:literal,
        calibration_status: $calibration_status:literal,
        visual_target: $visual_target:literal,
        source_note: $source_note:literal $(,)?
    ) => {
        PaperPresetRegistryEntry {
            preset: PaperPreset::$preset,
            details: PaperPresetDetails {
                id: PaperPreset::$preset.as_str(),
                label: $label,
                source_key: $source_key,
                model_family: MODEL_FAMILY_BRESSLOFF,
                render_domain: $source_view,
                source_page: $source_page,
                paper_figure: $paper_figure,
                source_table: $source_table,
                source_view: $source_view,
                expected_kind: $expected_kind,
                expected_contour_mode: $expected_contour_mode,
                expected_parity: $expected_parity,
                expected_family: $expected_family,
                expected_pattern: $expected_pattern,
                calibration_status: $calibration_status,
                visual_target: $visual_target,
                source_note: $source_note,
            },
        }
    };
}

static PAPER_PRESET_REGISTRY: &[PaperPresetRegistryEntry] = &[
    paper_preset_entry!(
        Fig16Odd,
        label: "Fig 16 odd marginal-stability scan",
        source_key: "bressloff-2001",
        source_page: "14",
        paper_figure: "Figure 16",
        source_table: "",
        source_view: "stability",
        expected_kind: "linear-stability",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "branch-selected",
        expected_pattern: "auto",
        calibration_status: "analytic-normalized",
        visual_target: "odd critical branch from the marginal-stability calculation",
        source_note: "Starting preset for the narrow lateral-connection example; use the report as a calibration check, not a digitized paper figure.",
    ),
    paper_preset_entry!(
        Fig17Even,
        label: "Fig 17 even widened-spread scan",
        source_key: "bressloff-2001",
        source_page: "14",
        paper_figure: "Figure 17",
        source_table: "",
        source_view: "stability",
        expected_kind: "linear-stability",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "branch-selected",
        expected_pattern: "auto",
        calibration_status: "analytic-normalized",
        visual_target: "even critical branch after the widened lateral angular spread",
        source_note: "Starting preset for the theta0-spread marginal-stability example.",
    ),
    paper_preset_entry!(
        Fig29SquareNoncontoured,
        label: "Fig 29 square non-contoured planform",
        source_key: "bressloff-2001",
        source_page: "21",
        paper_figure: "Figure 29",
        source_table: "Table 2",
        source_view: "visual_field",
        expected_kind: "single-map-noncontoured-planform",
        expected_contour_mode: "noncontoured",
        expected_parity: "even",
        expected_family: "square",
        expected_pattern: "cobweb",
        calibration_status: "rendered-target",
        visual_target: "single inverse retinocortical map of a non-contoured square activity planform",
        source_note: "Activity-threshold planform; no local orientation contours are sampled.",
    ),
    paper_preset_entry!(
        Fig29RollNoncontoured,
        label: "Fig 29 roll non-contoured planform",
        source_key: "bressloff-2001",
        source_page: "21",
        paper_figure: "Figure 29",
        source_table: "Table 2",
        source_view: "visual_field",
        expected_kind: "single-map-noncontoured-planform",
        expected_contour_mode: "noncontoured",
        expected_parity: "even",
        expected_family: "roll",
        expected_pattern: "rings",
        calibration_status: "rendered-target",
        visual_target: "single inverse retinocortical map of a non-contoured roll activity planform",
        source_note: "Rendered with the rings/tunnel roll orientation; rotate manually for ray or spiral variants.",
    ),
    paper_preset_entry!(
        Fig30RhombicNoncontoured,
        label: "Fig 30 rhombic non-contoured planform",
        source_key: "bressloff-2001",
        source_page: "21",
        paper_figure: "Figure 30",
        source_table: "Table 2",
        source_view: "visual_field",
        expected_kind: "single-map-noncontoured-planform",
        expected_contour_mode: "noncontoured",
        expected_parity: "even",
        expected_family: "rhombic",
        expected_pattern: "rhombic",
        calibration_status: "rendered-target",
        visual_target: "single inverse retinocortical map of a non-contoured rhombic activity planform",
        source_note: "Activity-threshold companion to the contoured rhombic Figure 33/34 family.",
    ),
    paper_preset_entry!(
        Fig30HexNoncontoured,
        label: "Fig 30 hexagonal non-contoured planform",
        source_key: "bressloff-2001",
        source_page: "21",
        paper_figure: "Figure 30",
        source_table: "Table 2",
        source_view: "visual_field",
        expected_kind: "single-map-noncontoured-planform",
        expected_contour_mode: "noncontoured",
        expected_parity: "even",
        expected_family: "hexagonal",
        expected_pattern: "honeycomb",
        calibration_status: "rendered-target",
        visual_target: "single inverse retinocortical map of a non-contoured hexagonal activity planform",
        source_note: "Uses the neutral three-wave hexagonal basis without interpreting local contour orientation.",
    ),
    paper_preset_entry!(
        Fig31SquareEven,
        label: "Fig 31 square/cobweb even planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 31",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "square",
        expected_pattern: "cobweb",
        calibration_status: "rendered-target",
        visual_target: "square lattice in cortex mapped to cobweb-like visual-field structure",
        source_note: "Analytic planform preset for checking the double-map geometry.",
    ),
    paper_preset_entry!(
        Fig31SquareEvenRoll,
        label: "Fig 31 even roll subpanel",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 31b",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-roll-subpanel",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "roll",
        expected_pattern: "rings",
        calibration_status: "rendered-target",
        visual_target: "even roll branch mapped through the double retinocortical map",
        source_note: "Roll companion subpanel for the even square/cobweb figure.",
    ),
    paper_preset_entry!(
        Fig32SquareOdd,
        label: "Fig 32 square/cobweb odd planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 32",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "square",
        expected_pattern: "cobweb",
        calibration_status: "rendered-target",
        visual_target: "odd square lattice variant mapped through the retino-cortical double map",
        source_note: "Analytic planform preset for checking parity-dependent contour structure.",
    ),
    paper_preset_entry!(
        Fig32SquareOddRoll,
        label: "Fig 32 odd roll subpanel",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 32b",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-roll-subpanel",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "roll",
        expected_pattern: "rings",
        calibration_status: "rendered-target",
        visual_target: "odd roll branch mapped through the double retinocortical map",
        source_note: "Roll companion subpanel for the odd square/cobweb figure.",
    ),
    paper_preset_entry!(
        Fig33RhombicEven,
        label: "Fig 33 rhombic even planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 33",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "rhombic",
        expected_pattern: "rhombic",
        calibration_status: "rendered-target",
        visual_target: "rhombic cortical lattice mapped into warped visual-field contours",
        source_note: "Analytic planform preset for the oblique-lattice family.",
    ),
    paper_preset_entry!(
        Fig34RhombicOdd,
        label: "Fig 34 rhombic odd planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 34",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "rhombic",
        expected_pattern: "rhombic",
        calibration_status: "rendered-target",
        visual_target: "odd rhombic cortical lattice mapped through the retino-cortical double map",
        source_note: "Analytic companion to the even rhombic Fig 33 preset; calibration still depends on the paper rhombic angle.",
    ),
    paper_preset_entry!(
        Fig33RhombicEvenRoll,
        label: "Fig 33 even rhombic-roll subpanel",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 33b",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-roll-subpanel",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "roll",
        expected_pattern: "spiral",
        calibration_status: "rendered-target",
        visual_target: "even roll branch in the rhombic family mapped through the double map",
        source_note: "Roll companion subpanel for the even rhombic figure.",
    ),
    paper_preset_entry!(
        Fig34RhombicOddRoll,
        label: "Fig 34 odd rhombic-roll subpanel",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 34b",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-roll-subpanel",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "roll",
        expected_pattern: "spiral",
        calibration_status: "rendered-target",
        visual_target: "odd roll branch in the rhombic family mapped through the double map",
        source_note: "Roll companion subpanel for the odd rhombic figure.",
    ),
    paper_preset_entry!(
        Fig35HexEven,
        label: "Fig 35 hexagonal even planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 35",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "hexagonal",
        expected_pattern: "hex_pi",
        calibration_status: "phase-selection-review",
        visual_target: "hexagonal branch with pi-phase sign structure",
        source_note: "Analytic planform preset for the three-wave hexagonal family.",
    ),
    paper_preset_entry!(
        Fig35HexZeroEven,
        label: "Fig 35 zero-hexagonal even planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 35",
        source_table: "Table 3",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "hexagonal",
        expected_pattern: "honeycomb",
        calibration_status: "rendered-target",
        visual_target: "0-hexagonal even phase partner of the Fig 35 hexagonal planform",
        source_note: "Analytic phase companion to the pi-hexagonal Fig 35 preset.",
    ),
    paper_preset_entry!(
        Fig36TriangleOdd,
        label: "Fig 36 triangular odd planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 36",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "hexagonal",
        expected_pattern: "triangle",
        calibration_status: "higher-order-review",
        visual_target: "odd triangular branch on a hexagonal lattice mapped through the double map",
        source_note: "Uses the odd hexagonal sine-combination branch; higher-order terms determine stability in the source discussion.",
    ),
    paper_preset_entry!(
        Fig36HexZeroOdd,
        label: "Fig 36 zero-hexagonal odd planform",
        source_key: "bressloff-2001",
        source_page: "22",
        paper_figure: "Figure 36",
        source_table: "Table 4",
        source_view: "visual_field",
        expected_kind: "double-map-planform",
        expected_contour_mode: "contoured",
        expected_parity: "odd",
        expected_family: "hexagonal",
        expected_pattern: "honeycomb",
        calibration_status: "rendered-target",
        visual_target: "odd 0-hexagonal branch on a hexagonal lattice mapped through the double map",
        source_note: "Odd-parity companion to the even 0-hexagonal phase target.",
    ),
    paper_preset_entry!(
        Fig5RollCortical,
        label: "2002 Fig 5 roll cortical planform",
        source_key: "bressloff-2002",
        source_page: "12",
        paper_figure: "Figure 5",
        source_table: "",
        source_view: "cortical",
        expected_kind: "cortical-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "roll",
        expected_pattern: "rings",
        calibration_status: "rendered-target",
        visual_target: "cortical roll planform before visual-field inverse mapping",
        source_note: "2002 convenience alias for the cortical roll operating-mode panel.",
    ),
    paper_preset_entry!(
        Fig5HexCortical,
        label: "2002 Fig 5 pi-hexagonal cortical planform",
        source_key: "bressloff-2002",
        source_page: "12",
        paper_figure: "Figure 5",
        source_table: "",
        source_view: "cortical",
        expected_kind: "cortical-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "hexagonal",
        expected_pattern: "hex_pi",
        calibration_status: "phase-selection-review",
        visual_target: "cortical pi-hexagonal planform before visual-field inverse mapping",
        source_note: "2002 convenience alias for the hexagonal operating-mode panel.",
    ),
    paper_preset_entry!(
        Fig5HoneycombCortical,
        label: "2002 Fig 5 honeycomb cortical planform",
        source_key: "bressloff-2002",
        source_page: "12",
        paper_figure: "Figure 5",
        source_table: "",
        source_view: "cortical",
        expected_kind: "cortical-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "hexagonal",
        expected_pattern: "honeycomb",
        calibration_status: "rendered-target",
        visual_target: "cortical honeycomb/coupled-ring planform before visual-field inverse mapping",
        source_note: "2002 convenience alias for the honeycomb operating-mode panel.",
    ),
    paper_preset_entry!(
        Fig5SquareCortical,
        label: "2002 Fig 5 square cortical planform",
        source_key: "bressloff-2002",
        source_page: "12",
        paper_figure: "Figure 5",
        source_table: "",
        source_view: "cortical",
        expected_kind: "cortical-planform",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "square",
        expected_pattern: "cobweb",
        calibration_status: "rendered-target",
        visual_target: "cortical square planform before visual-field inverse mapping",
        source_note: "2002 convenience alias for the square operating-mode panel.",
    ),
    paper_preset_entry!(
        Fig6VisualFieldPlanforms,
        label: "2002 Fig 6 visual-field planform representative",
        source_key: "bressloff-2002",
        source_page: "13",
        paper_figure: "Figure 6",
        source_table: "",
        source_view: "visual_field",
        expected_kind: "visual-field-planform-set",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "square",
        expected_pattern: "cobweb",
        calibration_status: "representative-panel",
        visual_target: "visual-field inverse-map representative for the Fig 6 planform set",
        source_note: "Fig 6 is a multi-panel summary; this preset uses the square/cobweb representative and the full set is covered by the other named aliases.",
    ),
    paper_preset_entry!(
        Fig7LatticeTunnel,
        label: "2002 Fig 7 lattice tunnel",
        source_key: "bressloff-2002",
        source_page: "15",
        paper_figure: "Figure 7",
        source_table: "",
        source_view: "visual_field",
        expected_kind: "double-map-roll-subpanel",
        expected_contour_mode: "contoured",
        expected_parity: "even",
        expected_family: "roll",
        expected_pattern: "rings",
        calibration_status: "source-wording-resolved",
        visual_target: "lattice tunnel representative using an even roll branch mapped through the visual field",
        source_note: "The caption mixes hexagonal-roll and square-lattice wording; this preset implements the visual tunnel as the even roll representative and records the ambiguity in the source note.",
    ),
];

#[derive(Clone, Copy, Debug)]
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

fn parse_paper_preset(value: Option<&str>) -> PaperPreset {
    value
        .and_then(|id| {
            PAPER_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == id)
                .map(|entry| entry.preset)
        })
        .unwrap_or(PaperPreset::Manual)
}

fn parse_rule_preset(value: Option<&str>) -> RulePreset {
    value
        .and_then(|id| {
            RULE_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == id)
                .map(|entry| entry.preset)
        })
        .unwrap_or(RulePreset::Manual)
}

fn paper_preset_details(preset: PaperPreset) -> Option<PaperPresetDetails> {
    PAPER_PRESET_REGISTRY
        .iter()
        .find(|entry| entry.preset == preset)
        .map(|entry| entry.details)
}

fn paper_preset_catalog() -> Vec<PaperPresetDetails> {
    PAPER_PRESET_REGISTRY
        .iter()
        .map(|entry| entry.details)
        .collect()
}

fn rule_preset_details(preset: RulePreset) -> Option<RulePresetDetails> {
    RULE_PRESET_REGISTRY
        .iter()
        .find(|entry| entry.preset == preset)
        .map(|entry| entry.details)
}

fn rule_preset_catalog() -> Vec<RulePresetDetails> {
    RULE_PRESET_REGISTRY
        .iter()
        .map(|entry| entry.details)
        .collect()
}

fn apply_paper_preset(mut params: FrameParams, preset: PaperPreset) -> FrameParams {
    params.paper_preset = preset;
    if preset != PaperPreset::Manual {
        params.rule_preset = RulePreset::Manual;
    }
    match preset {
        PaperPreset::Manual => params,
        PaperPreset::Fig16Odd => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Auto;
            params.parity = Parity::Even;
            params.eigen_beta = 0.4;
            params.lateral_sigma = 1.0;
            params.lateral_wide_sigma = 3.0;
            params.lateral_inhibition = 1.0;
            params.lateral_spread_deg = 0.0;
            params.stability_q_min = 0.05;
            params.stability_q_max = 3.5;
            params.stability_samples = 128;
            params.wave_count = 12.0;
            params.sharpness = 1.8;
            params
        }
        PaperPreset::Fig17Even => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Auto;
            params.parity = Parity::Even;
            params.eigen_beta = 0.4;
            params.lateral_sigma = 1.0;
            params.lateral_wide_sigma = 3.0;
            params.lateral_inhibition = 1.0;
            params.lateral_spread_deg = 60.0;
            params.stability_q_min = 0.05;
            params.stability_q_max = 3.5;
            params.stability_samples = 128;
            params.wave_count = 12.0;
            params.sharpness = 1.8;
            params
        }
        PaperPreset::Fig29SquareNoncontoured => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Cobweb;
            params.contour_mode = ContourMode::Noncontoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 90.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig29RollNoncontoured => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rings;
            params.contour_mode = ContourMode::Noncontoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 0.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig30RhombicNoncontoured => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rhombic;
            params.contour_mode = ContourMode::Noncontoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 45.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig30HexNoncontoured => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Honeycomb;
            params.contour_mode = ContourMode::Noncontoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig31SquareEven => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Cobweb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 90.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig31SquareEvenRoll => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rings;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 0.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig32SquareOdd => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Cobweb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 90.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig32SquareOddRoll => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rings;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 0.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig33RhombicEven => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rhombic;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 45.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig34RhombicOdd => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rhombic;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 45.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig33RhombicEvenRoll => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Spiral;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 45.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig34RhombicOddRoll => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Spiral;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 45.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig35HexEven => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::HexPi;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig35HexZeroEven => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Honeycomb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig36TriangleOdd => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Triangle;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig36HexZeroOdd => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Honeycomb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Odd;
            params.lateral_spread_deg = 0.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig5RollCortical => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rings;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 0.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig5HexCortical => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::HexPi;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig5HoneycombCortical => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Honeycomb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 60.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig5SquareCortical | PaperPreset::Fig6VisualFieldPlanforms => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Cobweb;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 12.0;
            params.pattern_angle = 90.0;
            params.sharpness = 2.1;
            params
        }
        PaperPreset::Fig7LatticeTunnel => {
            params.generator = Generator::Planform;
            params.pattern = PatternPreset::Rings;
            params.contour_mode = ContourMode::Contoured;
            params.parity = Parity::Even;
            params.lateral_spread_deg = 60.0;
            params.wave_count = 14.0;
            params.pattern_angle = 0.0;
            params.sharpness = 2.3;
            params
        }
    }
}

fn apply_rule_preset(mut params: FrameParams, preset: RulePreset) -> FrameParams {
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

impl PatternPreset {
    fn as_str(self) -> &'static str {
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
enum ContourMode {
    Contoured,
    Noncontoured,
}

impl ContourMode {
    fn as_str(self) -> &'static str {
        match self {
            ContourMode::Contoured => "contoured",
            ContourMode::Noncontoured => "noncontoured",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parity {
    Even,
    Odd,
}

impl Parity {
    fn as_str(self) -> &'static str {
        match self {
            Parity::Even => "even",
            Parity::Odd => "odd",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Solver {
    Preview,
    Accurate,
}

impl Solver {
    fn as_str(self) -> &'static str {
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
            n: 64,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StructureKey {
    n: usize,
    m: usize,
    r0_key: i64,
}

impl StructureKey {
    fn new(params: FrameParams) -> Self {
        Self {
            n: params.n,
            m: params.m,
            r0_key: (params.r0 * 100_000_000.0).round() as i64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Offset {
    dr: isize,
    dc: isize,
    weight: f64,
}

#[derive(Clone, Copy, Debug)]
struct SourceWeight {
    source_index: usize,
    weight: f64,
}

#[derive(Debug)]
struct SectorSources {
    per_cell: usize,
    entries: Vec<SourceWeight>,
}

#[derive(Debug)]
struct Structure {
    m: usize,
    angle_weights: Vec<f64>,
    sector_sources: Vec<SectorSources>,
}

#[derive(Default)]
struct ServerState {
    structures: Mutex<HashMap<StructureKey, Arc<Structure>>>,
    payloads: Mutex<HashMap<String, Arc<Payload>>>,
}

#[derive(Serialize)]
struct Payload {
    format: &'static str,
    model_family: &'static str,
    width: usize,
    height: usize,
    frame_count: usize,
    orientation_count: usize,
    times: Vec<f64>,
    scale_min: f64,
    scale_max: f64,
    raw_min: f32,
    raw_max: f32,
    cell_mm: f64,
    retino_bounds: RetinoBounds,
    retino_params: RetinoParams,
    palette: Vec<[u8; 3]>,
    paper_preset: Option<PaperPresetDetails>,
    rule_preset: Option<RulePresetDetails>,
    planform: Option<PlanformDetails>,
    rule: Option<RuleDetails>,
    calibration: Option<CalibrationReport>,
    orientation_channels: Option<OrientationChannelPayload>,
    params: PayloadParams,
    metrics: Metrics,
    warmup: Warmup,
    timing: Timing,
    data_base64: String,
}

#[derive(Serialize)]
struct PayloadParams {
    model_family: &'static str,
    paper_preset: &'static str,
    rule_preset: &'static str,
    generator: &'static str,
    pattern: &'static str,
    contour_mode: &'static str,
    parity: &'static str,
    n: usize,
    m: usize,
    t: f64,
    frames: usize,
    seed: u64,
    alpha: f64,
    beta: f64,
    mu: f64,
    r0: f64,
    low_percentile: f64,
    high_percentile: f64,
    cmap: &'static str,
    trim_warmup: bool,
    trim_threshold: f64,
    solver: &'static str,
    preview_step: f64,
    wave_count: f64,
    drift: f64,
    pattern_angle: f64,
    sharpness: f64,
    eigen_beta: f64,
    hypercolumn_mm: f64,
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
    stability_q_min: f64,
    stability_q_max: f64,
    stability_samples: usize,
    export_orientation_channels: bool,
    rule_tau_e_ms: f64,
    rule_tau_i_ms: f64,
    rule_aee: f64,
    rule_aei: f64,
    rule_aie: f64,
    rule_aii: f64,
    rule_theta_e: f64,
    rule_theta_i: f64,
    rule_sigma_e: f64,
    rule_sigma_i: f64,
    rule_stim_amplitude: f64,
    rule_stim_period_ms: f64,
    rule_stim_threshold: f64,
    rule_stim_smoothing: f64,
    rule_stim_i_fraction: f64,
    rule_seed_pattern: &'static str,
    rule_seed_strength: f64,
}

#[derive(Serialize)]
struct PlanformDetails {
    contour_mode: &'static str,
    parity: &'static str,
    q: f64,
    wave_number: f64,
    phase_base: f64,
    modes: Vec<PlanformModeDetails>,
    eigen: OrientationEigenDetails,
    stability: StabilityDetails,
    branch_selection: BranchSelectionDetails,
    kernel: KernelDetails,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct PaperPresetDetails {
    id: &'static str,
    label: &'static str,
    source_key: &'static str,
    model_family: &'static str,
    render_domain: &'static str,
    source_page: &'static str,
    paper_figure: &'static str,
    source_table: &'static str,
    source_view: &'static str,
    expected_kind: &'static str,
    expected_contour_mode: &'static str,
    expected_parity: &'static str,
    expected_family: &'static str,
    expected_pattern: &'static str,
    calibration_status: &'static str,
    visual_target: &'static str,
    source_note: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RulePresetDetails {
    id: &'static str,
    label: &'static str,
    source_key: &'static str,
    model_family: &'static str,
    render_domain: &'static str,
    source_page: &'static str,
    paper_figure: &'static str,
    expected_response_mode: &'static str,
    expected_family: &'static str,
    expected_period_ms: f64,
    calibration_status: &'static str,
    source_note: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct RuleDetails {
    preset: Option<RulePresetDetails>,
    model_family: &'static str,
    source_key: &'static str,
    equation: &'static str,
    status: &'static str,
    spatial_family: &'static str,
    response_mode: &'static str,
    pattern_strength: f32,
    dominant_cycles: f32,
    temporal_corr_t: f32,
    temporal_corr_2t: f32,
    stimulus_frequency_hz: f64,
    spatial: RuleSpatialDiagnostics,
    temporal: RuleTemporalDiagnostics,
    parameters: RuleParamDetails,
    checks: Vec<CalibrationCheck>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleParamDetails {
    tau_e_ms: f64,
    tau_i_ms: f64,
    aee: f64,
    aei: f64,
    aie: f64,
    aii: f64,
    theta_e: f64,
    theta_i: f64,
    sigma_e: f64,
    sigma_i: f64,
    stim_amplitude: f64,
    stim_period_ms: f64,
    stim_threshold: f64,
    stim_smoothing: f64,
    stim_i_fraction: f64,
    seed_pattern: &'static str,
    seed_strength: f64,
}

#[derive(Clone, Debug, Serialize)]
struct CalibrationReport {
    preset: PaperPresetDetails,
    status: &'static str,
    rendered_contour_mode: &'static str,
    rendered_parity: &'static str,
    rendered_pattern: &'static str,
    selected_family: &'static str,
    selected_pattern: &'static str,
    selected_scope: &'static str,
    global_selected_family: &'static str,
    global_selected_pattern: &'static str,
    target_lattice: &'static str,
    critical_q: f64,
    critical_branch: &'static str,
    dominant_cycles: f32,
    checks: Vec<CalibrationCheck>,
}

#[derive(Clone, Debug, Serialize)]
struct CalibrationCheck {
    name: &'static str,
    expected: &'static str,
    actual: String,
    passed: bool,
}

#[derive(Serialize)]
struct CalibrationRun {
    preset: PaperPresetDetails,
    status: &'static str,
    rendered_contour_mode: &'static str,
    rendered_parity: &'static str,
    rendered_pattern: &'static str,
    selected_family: &'static str,
    selected_pattern: &'static str,
    selected_scope: &'static str,
    global_selected_family: &'static str,
    global_selected_pattern: &'static str,
    target_lattice: &'static str,
    critical_q: f64,
    critical_branch: &'static str,
    dominant_cycles: f32,
    checks: Vec<CalibrationCheck>,
}

#[derive(Serialize)]
struct StabilityCalibrationRun {
    id: &'static str,
    label: &'static str,
    source_key: &'static str,
    source_page: &'static str,
    paper_figure: &'static str,
    target: &'static str,
    status: &'static str,
    rendered_parity: &'static str,
    critical_q: f64,
    critical_branch: &'static str,
    selected_family: &'static str,
    selected_pattern: &'static str,
    global_selected_family: &'static str,
    global_selected_pattern: &'static str,
    eta_hex: f64,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
    checks: Vec<CalibrationCheck>,
}

#[derive(Serialize)]
struct RuleCalibrationRun {
    preset: RulePresetDetails,
    status: &'static str,
    spatial_family: &'static str,
    response_mode: &'static str,
    pattern_strength: f32,
    dominant_cycles: f32,
    temporal_corr_t: f32,
    temporal_corr_2t: f32,
    stimulus_frequency_hz: f64,
    checks: Vec<CalibrationCheck>,
}

#[derive(Serialize)]
struct RuleSweepReport {
    format: &'static str,
    model_family: &'static str,
    source_key: &'static str,
    status: &'static str,
    note: &'static str,
    classification_version: &'static str,
    grid: RuleSweepGridDetails,
    periods_ms: Vec<f64>,
    amplitudes: Vec<f64>,
    stim_i_fractions: Vec<f64>,
    points: Vec<RuleSweepPoint>,
    floquet_reports: Vec<RuleFloquetReport>,
}

#[derive(Serialize)]
struct RuleSweepGridDetails {
    preset: &'static str,
    period_min_ms: f64,
    period_max_ms: f64,
    period_steps: usize,
    amplitude_min: f64,
    amplitude_max: f64,
    amplitude_steps: usize,
    stim_i_fraction_min: f64,
    stim_i_fraction_max: f64,
    stim_i_fraction_steps: usize,
    n: usize,
    frames: usize,
    preview_step: f64,
}

#[derive(Serialize)]
struct RuleSweepPoint {
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
    seed_pattern: &'static str,
    spatial_family: &'static str,
    response_mode: &'static str,
    pattern_strength: f32,
    dominant_cycles: f32,
    temporal_corr_t: f32,
    temporal_corr_2t: f32,
    stimulus_frequency_hz: f64,
    peak_activity: f32,
    status_level: &'static str,
    spatial: RuleSpatialDiagnostics,
    temporal: RuleTemporalDiagnostics,
    classification_note: &'static str,
    thumbnail: RuleThumbnail,
}

#[derive(Clone, Debug, Serialize)]
struct RuleSpatialDiagnostics {
    family: &'static str,
    dominant_cycles: f32,
    stripe_power: f64,
    square_power: f64,
    hex_power: f64,
    total_power: f64,
    mode_entropy: f64,
    confidence: f64,
    top_modes: Vec<RuleModePower>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleModePower {
    cycles: f64,
    angle_degrees: f64,
    family: &'static str,
    power: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleTemporalDiagnostics {
    corr_t: f32,
    corr_2t: f32,
    corr_3t: f32,
    response_mode: &'static str,
    estimated_period_cycles: f32,
    confidence: f32,
    note: &'static str,
}

#[derive(Serialize)]
struct RuleThumbnail {
    format: &'static str,
    encoding: &'static str,
    width: usize,
    height: usize,
    scale_min: f64,
    scale_max: f64,
    data_base64: String,
}

#[derive(Serialize)]
struct RuleFloquetReport {
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
    orbit: RuleOrbitSummary,
    modes: Vec<RuleFloquetMode>,
    strongest_mode: RuleFloquetMode,
    plus_crossing_modes: Vec<f64>,
    minus_crossing_modes: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleOrbitSummary {
    period_ms: f64,
    samples: usize,
    e_min: f64,
    e_max: f64,
    e_mean: f64,
    i_min: f64,
    i_max: f64,
    i_mean: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleFloquetMode {
    beta_cycles: f64,
    wave_number_radians: f64,
    multiplier_1_real: f64,
    multiplier_1_imag: f64,
    multiplier_2_real: f64,
    multiplier_2_imag: f64,
    max_abs_multiplier: f64,
    monodromy_trace: f64,
    monodromy_determinant: f64,
    plus_condition: f64,
    minus_condition: f64,
    determinant_condition: f64,
    crossing_hint: &'static str,
}

#[derive(Serialize)]
struct RuleFloquetCalibrationReport {
    format: &'static str,
    model_family: &'static str,
    source_key: &'static str,
    parameter_set: &'static str,
    status: &'static str,
    note: &'static str,
    source_axes: RuleFloquetSourceAxes,
    curve_refinement: RuleFloquetCurveRefinement,
    source_curve_comparison: RuleFloquetSourceCurveComparisonSummary,
    grid: RuleSweepGridDetails,
    mode_cycles: Vec<f64>,
    points: Vec<RuleFloquetGridPoint>,
    boundary_candidates: Vec<RuleFloquetBoundaryCandidate>,
    boundary_curves: Vec<RuleFloquetBoundaryCurve>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleFloquetSourceAxes {
    x_axis: &'static str,
    x_units: &'static str,
    x_secondary_axis: &'static str,
    x_secondary_units: &'static str,
    y_axis: &'static str,
    y_units: &'static str,
    y_secondary_axis: &'static str,
    y_secondary_units: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleFloquetCurveRefinement {
    method: &'static str,
    tolerance: f64,
    max_steps: usize,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetGridPoint {
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
    dominant_beta_cycles: f64,
    max_abs_multiplier: f64,
    crossing_hint: &'static str,
    plus_margin: f64,
    minus_margin: f64,
    complex_margin: f64,
    orbit: RuleOrbitSummary,
    modes: Vec<RuleFloquetMode>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleFloquetBoundaryCandidate {
    kind: &'static str,
    evidence: &'static str,
    beta_cycles: f64,
    axis: &'static str,
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
    from_period_ms: f64,
    from_amplitude: f64,
    from_beta_cycles: f64,
    to_period_ms: f64,
    to_amplitude: f64,
    to_beta_cycles: f64,
    margin_from: f64,
    margin_to: f64,
    confidence: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetBoundaryCurve {
    curve_id: String,
    kind: &'static str,
    branch_label: String,
    branch_periodicity: &'static str,
    axis: &'static str,
    source_axis: &'static str,
    amplitude: f64,
    stim_i_fraction: f64,
    point_count: usize,
    period_min_ms: f64,
    period_max_ms: f64,
    beta_min_cycles: f64,
    beta_max_cycles: f64,
    wave_number_min_radians: f64,
    wave_number_max_radians: f64,
    mean_residual_abs: f64,
    max_residual_abs: f64,
    mean_bracket_width_beta_cycles: f64,
    max_bracket_width_beta_cycles: f64,
    mean_period_gap_ms: f64,
    max_period_gap_ms: f64,
    continuity_score: f64,
    fit: RuleFloquetBoundaryCurveFit,
    source_comparison: RuleFloquetBoundarySourceComparison,
    points: Vec<RuleFloquetBoundaryCurvePoint>,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetSourceCurveComparisonSummary {
    status: &'static str,
    source_curve_file: Option<String>,
    source_curve_count: usize,
    compared_curve_count: usize,
    mean_rms_wave_number_error: Option<f64>,
    max_rms_wave_number_error: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetBoundarySourceComparison {
    status: &'static str,
    source_curve_id: Option<String>,
    source_branch_label: Option<String>,
    overlap_point_count: usize,
    period_overlap_min_ms: Option<f64>,
    period_overlap_max_ms: Option<f64>,
    mean_abs_wave_number_error: Option<f64>,
    rms_wave_number_error: Option<f64>,
    max_abs_wave_number_error: Option<f64>,
}

impl RuleFloquetBoundarySourceComparison {
    fn disabled() -> Self {
        Self {
            status: "source-curve-comparison-disabled",
            source_curve_id: None,
            source_branch_label: None,
            overlap_point_count: 0,
            period_overlap_min_ms: None,
            period_overlap_max_ms: None,
            mean_abs_wave_number_error: None,
            rms_wave_number_error: None,
            max_abs_wave_number_error: None,
        }
    }

    fn missing() -> Self {
        Self {
            status: "source-curve-file-missing",
            source_curve_id: None,
            source_branch_label: None,
            overlap_point_count: 0,
            period_overlap_min_ms: None,
            period_overlap_max_ms: None,
            mean_abs_wave_number_error: None,
            rms_wave_number_error: None,
            max_abs_wave_number_error: None,
        }
    }

    fn no_overlap() -> Self {
        Self {
            status: "no-overlapping-source-curve",
            source_curve_id: None,
            source_branch_label: None,
            overlap_point_count: 0,
            period_overlap_min_ms: None,
            period_overlap_max_ms: None,
            mean_abs_wave_number_error: None,
            rms_wave_number_error: None,
            max_abs_wave_number_error: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RuleFigure8SourceCurves {
    format: String,
    source_key: String,
    figure: String,
    curves: Vec<RuleFigure8SourceCurve>,
}

#[derive(Clone, Debug, Deserialize)]
struct RuleFigure8SourceCurve {
    curve_id: String,
    kind: String,
    branch_label: String,
    points: Vec<RuleFigure8SourcePoint>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct RuleFigure8SourcePoint {
    period_ms: f64,
    wave_number_beta: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetBoundaryCurveFit {
    model: &'static str,
    degree: usize,
    x_axis: &'static str,
    y_axis: &'static str,
    x_origin_ms: f64,
    x_scale_ms: f64,
    coefficients: Vec<f64>,
    rms_residual: f64,
    max_abs_residual: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct RuleFloquetBoundaryCurvePoint {
    kind: &'static str,
    branch_label: &'static str,
    branch_periodicity: &'static str,
    axis: &'static str,
    period_ms: f64,
    stimulus_frequency_hz: f64,
    amplitude: f64,
    stim_i_fraction: f64,
    beta_cycles: f64,
    wave_number_radians: f64,
    bracket_low_beta_cycles: f64,
    bracket_high_beta_cycles: f64,
    bracket_width_beta_cycles: f64,
    margin: f64,
    condition_value: f64,
    iterations: usize,
    residual_abs: f64,
}

#[derive(Clone, Debug)]
struct RuleSweepGridConfig {
    preset: &'static str,
    periods: Vec<f64>,
    amplitudes: Vec<f64>,
    stim_i_fractions: Vec<f64>,
    n: usize,
    frames: usize,
    preview_step: f64,
}

#[derive(Serialize)]
struct OrientationChannelPayload {
    format: &'static str,
    order: &'static str,
    width: usize,
    height: usize,
    frame_count: usize,
    orientation_count: usize,
    phi_radians: Vec<f64>,
    scale_min: f64,
    scale_max: f64,
    raw_min: f32,
    raw_max: f32,
    data_base64: String,
}

#[derive(Serialize)]
struct BressloffFigureGeometryReport {
    format: &'static str,
    model_family: &'static str,
    source_key: &'static str,
    status: &'static str,
    note: &'static str,
    source_profile_dir: String,
    width: usize,
    height: usize,
    still_count: usize,
    compared_still_count: usize,
    stills: Vec<BressloffFigureStill>,
}

#[derive(Serialize)]
struct BressloffFigureStill {
    preset: PaperPresetDetails,
    target_mask_status: &'static str,
    target_mask_id: Option<String>,
    width: usize,
    height: usize,
    frame_index: usize,
    rendered_contour_mode: &'static str,
    rendered_pattern: &'static str,
    selected_family: &'static str,
    image: BressloffStillImage,
    metrics: BressloffStillMetrics,
    source_comparison: BressloffSourceComparison,
}

#[derive(Serialize)]
struct BressloffStillImage {
    format: &'static str,
    encoding: &'static str,
    color_space: &'static str,
    data_base64: String,
}

#[derive(Serialize)]
struct BressloffStillMetrics {
    mean_luma: f64,
    std_luma: f64,
    active_fraction: f64,
    edge_density: f64,
    dominant_angle_degrees: f64,
    radial_profile: Vec<f64>,
    angular_profile: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct BressloffSourceComparison {
    status: &'static str,
    source_profile_id: Option<String>,
    source_mask_id: Option<String>,
    radial_profile_error: Option<f64>,
    angular_profile_error: Option<f64>,
    edge_overlap: Option<f64>,
    active_fraction_error: Option<f64>,
    edge_density_error: Option<f64>,
    lattice_angle_error_degrees: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct BressloffSourceProfile {
    preset_id: String,
    profile_id: Option<String>,
    mask_id: Option<String>,
    active_fraction: Option<f64>,
    edge_density: Option<f64>,
    lattice_angle_degrees: Option<f64>,
    radial_profile: Option<Vec<f64>>,
    angular_profile: Option<Vec<f64>>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct PlanformModeDetails {
    normal_angle: f64,
    phase_scale: f64,
    phase_offset: f64,
    amplitude: f64,
}

#[derive(Clone, Debug, Serialize)]
struct OrientationEigenDetails {
    parity: &'static str,
    beta: f64,
    cos_coefficients: Vec<[f64; 2]>,
    sin_coefficients: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize)]
struct KernelDetails {
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
}

#[derive(Clone, Debug, Serialize)]
struct StabilityDetails {
    q_min: f64,
    q_max: f64,
    samples: usize,
    critical_q: f64,
    critical_branch: &'static str,
    critical_growth: f64,
    selected_pattern: &'static str,
    points: Vec<StabilityPoint>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct StabilityPoint {
    q: f64,
    even_growth: f64,
    odd_growth: f64,
}

#[derive(Clone, Debug, Serialize)]
struct BranchSelectionDetails {
    model: &'static str,
    lambda: f64,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
    eta_hex: f64,
    target_lattice: &'static str,
    selected_scope: &'static str,
    selected_family: &'static str,
    selected_pattern: &'static str,
    selected_lattice_stable: bool,
    global_selected_family: &'static str,
    global_selected_pattern: &'static str,
    global_selected_stable: bool,
    candidates: Vec<BranchCandidate>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct BranchCandidate {
    family: &'static str,
    pattern: &'static str,
    mode_count: usize,
    theta_rad: f64,
    gamma_cross: f64,
    eta: f64,
    amplitude: f64,
    score: f64,
    stable: bool,
    note: &'static str,
}

#[derive(Serialize)]
struct Timing {
    matrix_build_sec: f64,
    solve_sec: f64,
    total_sec: f64,
    matrix_cache_hit: bool,
    backend: &'static str,
}

#[derive(Serialize)]
struct Metrics {
    final_mean: f32,
    final_std: f32,
    final_range: f32,
    dominant_cycles: f32,
    temporal_delta: f32,
}

#[derive(Serialize)]
struct Warmup {
    enabled: bool,
    dropped_frames: usize,
    start_time: f64,
    threshold_fraction: f64,
    threshold_std: f32,
    max_std: f32,
}

#[derive(Serialize)]
struct RetinoBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

#[derive(Serialize)]
struct RetinoParams {
    eps: f64,
    w0: f64,
    alpha: f64,
    beta: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("serve");
    match command {
        "calibrate" => calibrate_command(&args[2..])?,
        "bressloff-geometry" | "figure-stills" => bressloff_geometry_command(&args[2..])?,
        "export" => export_command(&args[2..])?,
        "rule-report" | "rule-calibrate" => rule_report_command(&args[2..])?,
        "rule-sweep" => rule_sweep_command(&args[2..])?,
        "rule-floquet" => rule_floquet_command(&args[2..])?,
        "serve" => serve_command(&args[2..])?,
        "--help" | "-h" => print_usage(),
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
        }
    }
    Ok(())
}

fn print_usage() {
    println!(
        "usage:\n  bressloff-v1 serve [--host 127.0.0.1] [--port 8892] [--root .]\n  bressloff-v1 export [--out viewer/frames.json] [--paper-preset fig31_square_even] [--rule-preset rule_fig4_high_freq_stripes] [--export-orientations] [model params]\n  bressloff-v1 calibrate [--out reports/paper-calibration.json] [model params]\n  bressloff-v1 bressloff-geometry [--out reports/figure-targets/bressloff-generated-stills.json] [--preset-set figures29-36|all] [model params]\n  bressloff-v1 rule-report [--out reports/rule-2011-regimes.json] [model params]\n  bressloff-v1 rule-sweep [--out reports/rule-2011-sweep.json] [--preset-grid quick|paper|dense] [--periods 140,120,85,65,55] [--period-min 40 --period-max 160 --period-steps 13] [--amplitudes 0.65,0.8,1.0] [model params]\n  bressloff-v1 rule-floquet [--out reports/rule-2011-floquet.json] [--preset-grid quick|paper|dense] [--modes 0.5,0.75,...,4.0] [--mode-min 0.5 --mode-max 4.0 --mode-steps 15] [model params]"
    );
}

fn export_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("viewer/frames.json");
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--export-orientations" | "--export-orientation-channels" => {
                raw.insert(
                    "export_orientation_channels".to_string(),
                    "true".to_string(),
                );
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

    let state = ServerState::default();
    let params = coerce_params(&raw);
    let payload = generate_payload(params, &state)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec(&payload)?)?;
    println!(
        "wrote {} grid={}x{} orientations={} frames={} backend={} build={:.3}s solve={:.3}s trim={}",
        out.display(),
        payload.width,
        payload.height,
        payload.orientation_count,
        payload.frame_count,
        payload.timing.backend,
        payload.timing.matrix_build_sec,
        payload.timing.solve_sec,
        payload.warmup.dropped_frames
    );
    if let Some(channels) = &payload.orientation_channels {
        println!(
            "orientation_channels={}x{}x{}x{} raw=[{:.4},{:.4}]",
            channels.frame_count,
            channels.width,
            channels.height,
            channels.orientation_count,
            channels.raw_min,
            channels.raw_max
        );
    }
    if let Some(calibration) = &payload.calibration {
        println!(
            "calibration={} status={} rendered={} selected={}",
            calibration.preset.id,
            calibration.status,
            calibration.rendered_pattern,
            calibration.selected_family
        );
    }
    if let Some(rule) = &payload.rule {
        println!(
            "rule={} status={} family={} response={} freq={:.2}Hz",
            rule.preset.map(|preset| preset.id).unwrap_or("manual"),
            rule.status,
            rule.spatial_family,
            rule.response_mode,
            rule.stimulus_frequency_hz
        );
    }
    Ok(())
}

fn calibrate_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/paper-calibration.json");
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--export-orientations" | "--export-orientation-channels" => {
                raw.insert(
                    "export_orientation_channels".to_string(),
                    "true".to_string(),
                );
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    raw.entry("generator".to_string())
        .or_insert_with(|| "planform".to_string());
    raw.entry("n".to_string())
        .or_insert_with(|| "96".to_string());
    raw.entry("m".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("frames".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("t".to_string())
        .or_insert_with(|| "18".to_string());

    let state = ServerState::default();
    let mut runs = Vec::new();
    for preset in paper_preset_catalog()
        .into_iter()
        .map(|details| parse_paper_preset(Some(details.id)))
        .filter(|preset| *preset != PaperPreset::Manual)
    {
        let mut preset_raw = raw.clone();
        preset_raw.insert("paper_preset".to_string(), preset.as_str().to_string());
        let params = coerce_params(&preset_raw);
        let payload = generate_payload(params, &state)?;
        let calibration = payload
            .calibration
            .as_ref()
            .ok_or("calibration report missing for paper preset")?;
        runs.push(CalibrationRun {
            preset: calibration.preset,
            status: calibration.status,
            rendered_contour_mode: calibration.rendered_contour_mode,
            rendered_parity: calibration.rendered_parity,
            rendered_pattern: calibration.rendered_pattern,
            selected_family: calibration.selected_family,
            selected_pattern: calibration.selected_pattern,
            selected_scope: calibration.selected_scope,
            global_selected_family: calibration.global_selected_family,
            global_selected_pattern: calibration.global_selected_pattern,
            target_lattice: calibration.target_lattice,
            critical_q: calibration.critical_q,
            critical_branch: calibration.critical_branch,
            dominant_cycles: calibration.dominant_cycles,
            checks: calibration.checks.clone(),
        });
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let run_count = runs.len();
    let body = serde_json::json!({
        "format": "bressloff-paper-calibration-v4",
        "model_family": MODEL_FAMILY_BRESSLOFF,
        "runs": runs,
        "stability_reports": bressloff_stability_reports(),
    });
    fs::write(&out, serde_json::to_vec_pretty(&body)?)?;
    println!("wrote {} presets={run_count}", out.display());
    Ok(())
}

fn bressloff_geometry_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/figure-targets/bressloff-generated-stills.json");
    let mut source_profile_dir = PathBuf::from("private/figure-targets/derived");
    let mut preset_set = "figures29-36".to_string();
    let mut preset_override: Option<Vec<PaperPreset>> = None;
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--source-profile-dir" => {
                source_profile_dir =
                    PathBuf::from(iter.next().ok_or("--source-profile-dir requires a value")?);
            }
            "--preset-set" => {
                preset_set = iter
                    .next()
                    .ok_or("--preset-set requires a value")?
                    .to_string();
            }
            "--presets" => {
                preset_override = Some(parse_paper_preset_csv(
                    iter.next().ok_or("--presets requires a value")?,
                )?);
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    raw.entry("generator".to_string())
        .or_insert_with(|| "planform".to_string());
    raw.entry("n".to_string())
        .or_insert_with(|| "96".to_string());
    raw.entry("m".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("frames".to_string())
        .or_insert_with(|| "8".to_string());
    raw.entry("t".to_string())
        .or_insert_with(|| "18".to_string());

    let state = ServerState::default();
    let presets = preset_override.unwrap_or_else(|| bressloff_geometry_preset_set(&preset_set));
    let mut stills = Vec::new();
    for preset in presets {
        let mut preset_raw = raw.clone();
        preset_raw.insert("paper_preset".to_string(), preset.as_str().to_string());
        let params = coerce_params(&preset_raw);
        let payload = generate_payload(params, &state)?;
        let calibration = payload
            .calibration
            .as_ref()
            .ok_or("geometry still missing Bressloff calibration metadata")?;
        let frame_index = payload.frame_count.saturating_sub(1);
        let frame = payload_frame_u8(&payload, frame_index)?;
        let metrics = bressloff_still_metrics(&frame, payload.width, payload.height);
        let source_profile =
            load_bressloff_source_profile(&source_profile_dir, calibration.preset.id)?;
        let source_comparison =
            bressloff_source_comparison(calibration.preset.id, &metrics, source_profile.as_ref());
        stills.push(BressloffFigureStill {
            preset: calibration.preset,
            target_mask_status: source_comparison.status,
            target_mask_id: source_comparison.source_mask_id.clone(),
            width: payload.width,
            height: payload.height,
            frame_index,
            rendered_contour_mode: calibration.rendered_contour_mode,
            rendered_pattern: calibration.rendered_pattern,
            selected_family: calibration.selected_family,
            image: BressloffStillImage {
                format: "u8-frame-v1",
                encoding: "base64",
                color_space: "normalized-luma",
                data_base64: general_purpose::STANDARD.encode(frame),
            },
            metrics,
            source_comparison,
        });
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let still_count = stills.len();
    let compared_still_count = stills
        .iter()
        .filter(|still| still.source_comparison.status == "compared")
        .count();
    let width = stills.first().map(|still| still.width).unwrap_or(0);
    let height = stills.first().map(|still| still.height).unwrap_or(0);
    let report = BressloffFigureGeometryReport {
        format: "bressloff-generated-figure-stills-v2",
        model_family: MODEL_FAMILY_BRESSLOFF,
        source_key: "bressloff-2001-2002",
        status: if compared_still_count > 0 {
            "generated-vs-source-derived-comparison"
        } else {
            "generated-targets-ready-for-private-mask-calibration"
        },
        note: "Bressloff figure stills and public-safe geometry metrics. Private source scans/crops stay out of the report; comparisons use only derived numeric masks/profiles when available.",
        source_profile_dir: source_profile_dir.display().to_string(),
        width,
        height,
        still_count,
        compared_still_count,
        stills,
    };
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!("wrote {} generated_stills={still_count}", out.display());
    Ok(())
}

fn parse_paper_preset_csv(value: &str) -> Result<Vec<PaperPreset>, Box<dyn std::error::Error>> {
    let presets = value
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .map(|part| {
            PAPER_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == part)
                .map(|entry| entry.preset)
                .ok_or_else(|| format!("unknown paper preset: {part}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if presets.is_empty() {
        Err("preset list must contain at least one paper preset".into())
    } else {
        Ok(presets)
    }
}

fn bressloff_geometry_preset_set(name: &str) -> Vec<PaperPreset> {
    match name {
        "all" => PAPER_PRESET_REGISTRY
            .iter()
            .map(|entry| entry.preset)
            .filter(|preset| *preset != PaperPreset::Manual)
            .collect(),
        _ => vec![
            PaperPreset::Fig29SquareNoncontoured,
            PaperPreset::Fig29RollNoncontoured,
            PaperPreset::Fig30RhombicNoncontoured,
            PaperPreset::Fig30HexNoncontoured,
            PaperPreset::Fig31SquareEven,
            PaperPreset::Fig31SquareEvenRoll,
            PaperPreset::Fig32SquareOdd,
            PaperPreset::Fig32SquareOddRoll,
            PaperPreset::Fig33RhombicEven,
            PaperPreset::Fig33RhombicEvenRoll,
            PaperPreset::Fig34RhombicOdd,
            PaperPreset::Fig34RhombicOddRoll,
            PaperPreset::Fig35HexEven,
            PaperPreset::Fig35HexZeroEven,
            PaperPreset::Fig36TriangleOdd,
            PaperPreset::Fig36HexZeroOdd,
        ],
    }
}

fn payload_frame_u8(
    payload: &Payload,
    frame_index: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let frame_size = payload.width * payload.height;
    let bytes = general_purpose::STANDARD.decode(&payload.data_base64)?;
    let start = frame_index
        .min(payload.frame_count.saturating_sub(1))
        .saturating_mul(frame_size);
    let end = start + frame_size;
    if end > bytes.len() {
        return Err("payload frame index outside encoded data".into());
    }
    Ok(bytes[start..end].to_vec())
}

fn bressloff_still_metrics(frame: &[u8], width: usize, height: usize) -> BressloffStillMetrics {
    let len = frame.len().max(1) as f64;
    let values = frame.iter().map(|value| *value as f64 / 255.0);
    let mean = values.clone().sum::<f64>() / len;
    let variance = values
        .clone()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / len;
    let active_fraction = frame.iter().filter(|value| **value >= 128).count() as f64 / len;

    let mut edge_sum = 0.0;
    let mut edge_count = 0usize;
    for y in 0..height {
        for x in 0..width {
            let here = frame[y * width + x] as f64;
            if x + 1 < width {
                edge_sum += (here - frame[y * width + x + 1] as f64).abs() / 255.0;
                edge_count += 1;
            }
            if y + 1 < height {
                edge_sum += (here - frame[(y + 1) * width + x] as f64).abs() / 255.0;
                edge_count += 1;
            }
        }
    }

    let radial_profile = normalized_radial_profile(frame, width, height, 16);
    let angular_profile = normalized_angular_profile(frame, width, height, 24);
    let dominant_angle_degrees = dominant_profile_angle_degrees(&angular_profile);

    BressloffStillMetrics {
        mean_luma: mean,
        std_luma: variance.sqrt(),
        active_fraction,
        edge_density: if edge_count == 0 {
            0.0
        } else {
            edge_sum / edge_count as f64
        },
        dominant_angle_degrees,
        radial_profile,
        angular_profile,
    }
}

fn normalized_radial_profile(frame: &[u8], width: usize, height: usize, bins: usize) -> Vec<f64> {
    let bins = bins.max(1);
    let mut sums = vec![0.0; bins];
    let mut counts = vec![0usize; bins];
    let cx = (width.saturating_sub(1)) as f64 * 0.5;
    let cy = (height.saturating_sub(1)) as f64 * 0.5;
    let max_radius = (cx * cx + cy * cy).sqrt().max(1.0e-9);
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let bin = (((dx * dx + dy * dy).sqrt() / max_radius) * bins as f64)
                .floor()
                .min((bins - 1) as f64) as usize;
            sums[bin] += frame[y * width + x] as f64 / 255.0;
            counts[bin] += 1;
        }
    }
    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| if count == 0 { 0.0 } else { sum / count as f64 })
        .collect()
}

fn normalized_angular_profile(frame: &[u8], width: usize, height: usize, bins: usize) -> Vec<f64> {
    let bins = bins.max(1);
    let mut sums = vec![0.0; bins];
    let mut counts = vec![0usize; bins];
    let cx = (width.saturating_sub(1)) as f64 * 0.5;
    let cy = (height.saturating_sub(1)) as f64 * 0.5;
    for y in 0..height {
        for x in 0..width {
            let angle = (y as f64 - cy).atan2(x as f64 - cx).rem_euclid(2.0 * PI);
            let bin = ((angle / (2.0 * PI)) * bins as f64)
                .floor()
                .min((bins - 1) as f64) as usize;
            sums[bin] += frame[y * width + x] as f64 / 255.0;
            counts[bin] += 1;
        }
    }
    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| if count == 0 { 0.0 } else { sum / count as f64 })
        .collect()
}

fn dominant_profile_angle_degrees(profile: &[f64]) -> f64 {
    if profile.is_empty() {
        return 0.0;
    }
    let (index, _) = profile
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap_or((0, &0.0));
    (index as f64 + 0.5) * 360.0 / profile.len() as f64
}

fn load_bressloff_source_profile(
    source_profile_dir: &Path,
    preset_id: &str,
) -> Result<Option<BressloffSourceProfile>, Box<dyn std::error::Error>> {
    let path = source_profile_dir.join(format!("{preset_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read_to_string(path)?;
    let profile = serde_json::from_str::<BressloffSourceProfile>(&body)?;
    if profile.preset_id != preset_id {
        return Err(format!(
            "source profile preset mismatch: expected {preset_id}, found {}",
            profile.preset_id
        )
        .into());
    }
    Ok(Some(profile))
}

fn bressloff_source_comparison(
    preset_id: &str,
    metrics: &BressloffStillMetrics,
    source_profile: Option<&BressloffSourceProfile>,
) -> BressloffSourceComparison {
    let Some(source) = source_profile else {
        return BressloffSourceComparison {
            status: "source-profile-missing",
            source_profile_id: None,
            source_mask_id: None,
            radial_profile_error: None,
            angular_profile_error: None,
            edge_overlap: None,
            active_fraction_error: None,
            edge_density_error: None,
            lattice_angle_error_degrees: None,
        };
    };

    let radial_profile_error = source
        .radial_profile
        .as_ref()
        .and_then(|profile| mean_absolute_profile_error(&metrics.radial_profile, profile));
    let angular_profile_error = source
        .angular_profile
        .as_ref()
        .and_then(|profile| mean_absolute_profile_error(&metrics.angular_profile, profile));
    let active_fraction_error = source
        .active_fraction
        .map(|value| (metrics.active_fraction - value).abs());
    let edge_density_error = source
        .edge_density
        .map(|value| (metrics.edge_density - value).abs());
    let edge_overlap = source
        .edge_density
        .map(|value| edge_overlap_from_densities(metrics.edge_density, value));
    let lattice_angle_error_degrees = source
        .lattice_angle_degrees
        .map(|value| angular_difference_degrees(metrics.dominant_angle_degrees, value, 180.0));

    BressloffSourceComparison {
        status: "compared",
        source_profile_id: source
            .profile_id
            .clone()
            .or_else(|| Some(format!("{preset_id}-source-profile"))),
        source_mask_id: source.mask_id.clone(),
        radial_profile_error,
        angular_profile_error,
        edge_overlap,
        active_fraction_error,
        edge_density_error,
        lattice_angle_error_degrees,
    }
}

fn mean_absolute_profile_error(generated: &[f64], source: &[f64]) -> Option<f64> {
    let len = generated.len().min(source.len());
    if len == 0 {
        return None;
    }
    Some(
        generated
            .iter()
            .take(len)
            .zip(source.iter().take(len))
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
            / len as f64,
    )
}

fn edge_overlap_from_densities(generated: f64, source: f64) -> f64 {
    let denom = generated.max(source).max(1.0e-9);
    1.0 - ((generated - source).abs() / denom).clamp(0.0, 1.0)
}

fn angular_difference_degrees(a: f64, b: f64, period: f64) -> f64 {
    let mut delta = (a - b).abs().rem_euclid(period);
    if delta > period * 0.5 {
        delta = period - delta;
    }
    delta
}

fn rule_report_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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

fn rule_sweep_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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

fn rule_floquet_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/rule-2011-floquet.json");
    let mut grid = rule_sweep_grid_defaults("dense");
    let mut parameter_set = "rule_fig8_source_like";
    let mut curve_refine_steps = 48usize;
    let mut curve_refine_tolerance = 1.0e-6;
    let mut source_curve_file: Option<PathBuf> = None;
    let mut source_curve_comparison_enabled = true;
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

    if mode_min.is_some() || mode_max.is_some() || mode_steps.is_some() {
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

    let mut points = Vec::new();
    for period in &grid.periods {
        for amplitude in &grid.amplitudes {
            for stim_i_fraction in &grid.stim_i_fractions {
                let params = rule_sweep_params(&raw, &grid, *period, *amplitude, *stim_i_fraction);
                points.push(rule_floquet_grid_point_for(params, &mode_cycles));
            }
        }
    }

    let boundary_candidates = rule_floquet_boundary_candidates(
        &points,
        &grid.periods,
        &grid.amplitudes,
        &grid.stim_i_fractions,
    );
    let mut boundary_curves = rule_floquet_beta_boundary_curves(
        &points,
        &raw,
        &grid,
        curve_refine_tolerance,
        curve_refine_steps,
    );
    let source_curve_file = if source_curve_comparison_enabled {
        Some(source_curve_file.unwrap_or_else(|| default_rule_figure8_source_curve_file(&out)))
    } else {
        None
    };
    let source_curves = match source_curve_file.as_ref() {
        Some(path) if path.exists() => Some(load_rule_figure8_source_curves(path)?),
        _ => None,
    };
    let source_curve_comparison = apply_rule_figure8_source_comparison(
        &mut boundary_curves,
        source_curves.as_ref(),
        source_curve_file.as_ref(),
    );
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let point_count = points.len();
    let boundary_count = boundary_candidates.len();
    let curve_count = boundary_curves.len();
    let report = RuleFloquetCalibrationReport {
        format: "rule-2011-floquet-calibration-v3",
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
        curve_refinement: RuleFloquetCurveRefinement {
            method: "bisection_on_beta_sign_change",
            tolerance: curve_refine_tolerance,
            max_steps: curve_refine_steps,
        },
        source_curve_comparison,
        grid: rule_sweep_grid_details(&grid),
        mode_cycles,
        points,
        boundary_candidates,
        boundary_curves,
    };
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} floquet_points={point_count} boundary_candidates={boundary_count} boundary_curves={curve_count}",
        out.display()
    );
    Ok(())
}

fn default_rule_figure8_source_curve_file(out: &Path) -> PathBuf {
    out.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("source-curves")
        .join("rule-2011-fig8-source-curves.json")
}

fn parse_rule_parameter_set(value: &str) -> Result<&'static str, Box<dyn std::error::Error>> {
    match value {
        "rule_fig8_source_like" | "source_like" | "figure8" => Ok("rule_fig8_source_like"),
        "current_defaults" | "default" => Ok("current_defaults"),
        _ => Err(format!(
            "unknown Rule parameter set: {value}; expected rule_fig8_source_like or current_defaults"
        )
        .into()),
    }
}

fn apply_rule_parameter_set(raw: &mut HashMap<String, String>, parameter_set: &str) {
    match parameter_set {
        "rule_fig8_source_like" => {
            raw.entry("generator".to_string())
                .or_insert_with(|| "rule_flicker".to_string());
            raw.entry("rule_tau_e_ms".to_string())
                .or_insert_with(|| "10".to_string());
            raw.entry("rule_tau_i_ms".to_string())
                .or_insert_with(|| "20".to_string());
            raw.entry("rule_aee".to_string())
                .or_insert_with(|| "10".to_string());
            raw.entry("rule_aei".to_string())
                .or_insert_with(|| "12".to_string());
            raw.entry("rule_aie".to_string())
                .or_insert_with(|| "8.5".to_string());
            raw.entry("rule_aii".to_string())
                .or_insert_with(|| "3".to_string());
            raw.entry("rule_theta_e".to_string())
                .or_insert_with(|| "2".to_string());
            raw.entry("rule_theta_i".to_string())
                .or_insert_with(|| "3.5".to_string());
            raw.entry("rule_sigma_e".to_string())
                .or_insert_with(|| "2".to_string());
            raw.entry("rule_sigma_i".to_string())
                .or_insert_with(|| "5".to_string());
            raw.entry("rule_stim_threshold".to_string())
                .or_insert_with(|| "0.8".to_string());
            raw.entry("rule_stim_smoothing".to_string())
                .or_insert_with(|| "50".to_string());
        }
        "current_defaults" => {}
        _ => {}
    }
}

fn load_rule_figure8_source_curves(
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

fn apply_rule_figure8_source_comparison(
    curves: &mut [RuleFloquetBoundaryCurve],
    source_curves: Option<&RuleFigure8SourceCurves>,
    source_curve_file: Option<&PathBuf>,
) -> RuleFloquetSourceCurveComparisonSummary {
    let Some(source) = source_curves else {
        for curve in curves {
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
        };
    };

    let mut rms_values = Vec::new();
    for curve in curves {
        curve.source_comparison = best_rule_source_curve_comparison(curve, &source.curves);
        if let Some(rms) = curve.source_comparison.rms_wave_number_error {
            rms_values.push(rms);
        }
    }
    let compared_curve_count = rms_values.len();
    let mean_rms_wave_number_error =
        (!rms_values.is_empty()).then(|| rms_values.iter().sum::<f64>() / rms_values.len() as f64);
    let max_rms_wave_number_error = rms_values.iter().copied().reduce(f64::max);

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
    }
}

fn best_rule_source_curve_comparison(
    curve: &RuleFloquetBoundaryCurve,
    source_curves: &[RuleFigure8SourceCurve],
) -> RuleFloquetBoundarySourceComparison {
    source_curves
        .iter()
        .filter(|source| source.kind == curve.kind)
        .filter_map(|source| compare_rule_source_curve(curve, source))
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
        errors.push(point.beta_cycles - source_wave);
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

fn parse_f64_csv(value: &str, min: f64, max: f64) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let values = value
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .map(|part| part.parse::<f64>().map(|value| value.clamp(min, max)))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        Err("comma-separated list must contain at least one number".into())
    } else {
        Ok(values)
    }
}

fn parse_clamped_f64(value: &str, min: f64, max: f64) -> Result<f64, Box<dyn std::error::Error>> {
    Ok(value.parse::<f64>()?.clamp(min, max))
}

fn parse_clamped_usize(
    value: &str,
    min: usize,
    max: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(value.parse::<usize>()?.clamp(min, max))
}

fn linspace_values(min: f64, max: f64, steps: usize) -> Vec<f64> {
    let steps = steps.max(1);
    if steps == 1 {
        return vec![min.clamp(0.0, f64::INFINITY)];
    }
    let lo = min.min(max);
    let hi = min.max(max);
    (0..steps)
        .map(|index| {
            let t = index as f64 / (steps - 1) as f64;
            lo + (hi - lo) * t
        })
        .collect()
}

fn rule_sweep_grid_defaults(name: &str) -> RuleSweepGridConfig {
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

fn rule_floquet_mode_defaults() -> Vec<f64> {
    linspace_values(0.5, 4.0, 15)
}

fn rule_sweep_grid_details(grid: &RuleSweepGridConfig) -> RuleSweepGridDetails {
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

fn value_min_max(values: &[f64]) -> (f64, f64) {
    values
        .iter()
        .copied()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        })
}

fn rule_sweep_params(
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

fn rule_sweep_duration_ms(period_ms: f64) -> f64 {
    if period_ms < 80.0 {
        (period_ms * 8.0).max(330.0)
    } else {
        (period_ms * 5.5).max(440.0)
    }
}

fn rule_seed_for_period(period_ms: f64) -> RuleSeedPattern {
    if period_ms >= 105.0 {
        RuleSeedPattern::Hexagonal
    } else if period_ms <= 70.0 {
        RuleSeedPattern::Stripes
    } else {
        RuleSeedPattern::Random
    }
}

fn rule_sweep_point_for(params: FrameParams) -> RuleSweepPoint {
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

fn rule_sweep_status_level(details: &RuleDetails) -> &'static str {
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

fn rule_classification_note(details: &RuleDetails) -> &'static str {
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

fn rule_thumbnail_from_frame(frame: &[f32], n: usize) -> RuleThumbnail {
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

fn bressloff_stability_reports() -> Vec<StabilityCalibrationRun> {
    vec![
        stability_report_for(
            "fig37_even_coefficients",
            "Fig 37 even eigen/coefficient sign target",
            "bressloff-2001",
            "24",
            "Figure 37",
            "even perturbative eigenfunction coefficients and even marginal branch",
            apply_paper_preset(FrameParams::default(), PaperPreset::Fig17Even),
            "even",
            "any",
            "any",
        ),
        stability_report_for(
            "fig38_even_hex_bifurcation",
            "Fig 38 even hexagonal bifurcation target",
            "bressloff-2001",
            "24",
            "Figure 38",
            "even hexagonal branch and roll exchange diagnostic",
            apply_paper_preset(FrameParams::default(), PaperPreset::Fig35HexZeroEven),
            "even",
            "hexagonal",
            "honeycomb",
        ),
        stability_report_for(
            "fig39_odd_coefficients",
            "Fig 39 odd eigen/coefficient sign target",
            "bressloff-2001",
            "25",
            "Figure 39",
            "odd perturbative eigenfunction coefficients and odd marginal branch",
            apply_paper_preset(FrameParams::default(), PaperPreset::Fig16Odd),
            "odd",
            "any",
            "any",
        ),
        stability_report_for(
            "fig40_odd_hex_bifurcation",
            "Fig 40 odd hexagonal bifurcation target",
            "bressloff-2001",
            "25",
            "Figure 40",
            "odd hexagonal/triangular higher-order selection target",
            apply_paper_preset(FrameParams::default(), PaperPreset::Fig36TriangleOdd),
            "odd",
            "hexagonal",
            "triangle",
        ),
        stability_report_for(
            "rhombic_stability_angle",
            "Rhombic stability angle target",
            "bressloff-2001",
            "23",
            "Rhombic stability discussion",
            "rhombic branch check at the current representative angle",
            apply_paper_preset(FrameParams::default(), PaperPreset::Fig33RhombicEven),
            "even",
            "rhombic",
            "rhombic",
        ),
    ]
}

fn stability_report_for(
    id: &'static str,
    label: &'static str,
    source_key: &'static str,
    source_page: &'static str,
    paper_figure: &'static str,
    target: &'static str,
    params: FrameParams,
    expected_branch: &'static str,
    expected_family: &'static str,
    expected_pattern: &'static str,
) -> StabilityCalibrationRun {
    let planform = planform_details(params, cell_mm_for(params));
    let branch = &planform.branch_selection;
    let mut checks = Vec::new();
    checks.push(CalibrationCheck {
        name: "critical-branch",
        expected: expected_branch,
        actual: planform.stability.critical_branch.to_string(),
        passed: planform.stability.critical_branch == expected_branch,
    });
    if expected_family != "any" {
        checks.push(CalibrationCheck {
            name: "selected-family",
            expected: expected_family,
            actual: branch.selected_family.to_string(),
            passed: branch.selected_family == expected_family,
        });
    }
    if expected_pattern != "any" {
        checks.push(CalibrationCheck {
            name: "selected-pattern",
            expected: expected_pattern,
            actual: branch.selected_pattern.to_string(),
            passed: branch.selected_pattern == expected_pattern,
        });
    }
    let status = if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "review"
    };
    StabilityCalibrationRun {
        id,
        label,
        source_key,
        source_page,
        paper_figure,
        target,
        status,
        rendered_parity: planform.parity,
        critical_q: planform.stability.critical_q,
        critical_branch: planform.stability.critical_branch,
        selected_family: branch.selected_family,
        selected_pattern: branch.selected_pattern,
        global_selected_family: branch.global_selected_family,
        global_selected_pattern: branch.global_selected_pattern,
        eta_hex: branch.eta_hex,
        gamma0: branch.gamma0,
        gamma_square: branch.gamma_square,
        gamma_rhombic: branch.gamma_rhombic,
        gamma_hex: branch.gamma_hex,
        checks,
    }
}

fn serve_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 8892_u16;
    let mut root = env::current_dir()?;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--host" => host = iter.next().ok_or("--host requires a value")?.clone(),
            "--port" => port = iter.next().ok_or("--port requires a value")?.parse()?,
            "--root" => root = PathBuf::from(iter.next().ok_or("--root requires a value")?),
            _ => {}
        }
    }

    let listener = TcpListener::bind((host.as_str(), port))?;
    let state = Arc::new(ServerState::default());
    println!("Serving Bressloff V1 viewer on http://{host}:{port}/viewer/index.html");
    for stream in listener.incoming() {
        let stream = stream?;
        let root = root.clone();
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            if let Err(error) = handle_connection(stream, &root, &state) {
                eprintln!("request error: {error}");
            }
        });
    }
    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    root: &Path,
    state: &ServerState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0_u8; 8192];
    let size = stream.read(&mut buffer)?;
    if size == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buffer[..size]);
    let Some(line) = request.lines().next() else {
        return Ok(());
    };
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 || parts[0] != "GET" {
        write_response(&mut stream, 405, "text/plain", b"method not allowed")?;
        return Ok(());
    }

    let (path, query) = split_query(parts[1]);
    match path.as_str() {
        "/api/defaults" => {
            let body = serde_json::json!({
                "defaults": default_params_json(),
                "limits": {
                    "n": [32, 96],
                    "m": [4, 24],
                    "t": [5.0, 800.0],
                    "frames": [8, 240],
                    "seed": [0, u32::MAX],
                    "alpha": [0.1, 4.0],
                    "beta": [0.1, 10.0],
                    "mu": [1.0, 40.0],
                    "r0": [0.02, 0.14],
                    "low_percentile": [0.0, 20.0],
                    "high_percentile": [80.0, 100.0],
                    "trim_threshold": [0.0, 0.5],
                    "preview_step": [0.02, 1.0],
                    "wave_count": [1.0, 40.0],
                    "drift": [-4.0, 4.0],
                    "pattern_angle": [0.0, 90.0],
                    "sharpness": [0.25, 8.0],
                    "eigen_beta": [0.0, 1.5],
                    "hypercolumn_mm": [0.1, 4.0],
                    "local_sigma_deg": [1.0, 80.0],
                    "local_wide_sigma_deg": [5.0, 120.0],
                    "local_inhibition": [0.0, 3.0],
                    "lateral_sigma": [0.1, 4.0],
                    "lateral_wide_sigma": [0.1, 6.0],
                    "lateral_inhibition": [0.0, 3.0],
                    "lateral_spread_deg": [0.0, 90.0],
                    "stability_q_min": [0.0, 2.0],
                    "stability_q_max": [0.2, 8.0],
                    "stability_samples": [16, 256],
                    "export_orientation_channels": [false, true],
                    "rule_tau_e_ms": [1.0, 80.0],
                    "rule_tau_i_ms": [1.0, 120.0],
                    "rule_aee": [0.0, 30.0],
                    "rule_aei": [0.0, 30.0],
                    "rule_aie": [0.0, 30.0],
                    "rule_aii": [0.0, 30.0],
                    "rule_theta_e": [0.0, 8.0],
                    "rule_theta_i": [0.0, 8.0],
                    "rule_sigma_e": [0.4, 10.0],
                    "rule_sigma_i": [0.4, 16.0],
                    "rule_stim_amplitude": [0.0, 1.5],
                    "rule_stim_period_ms": [20.0, 180.0],
                    "rule_stim_threshold": [-1.0, 1.0],
                    "rule_stim_smoothing": [0.0, 100.0],
                    "rule_stim_i_fraction": [0.0, 1.0],
                    "rule_seed_strength": [0.0, 0.2]
                },
                "paper_presets": paper_preset_catalog(),
                "rule_presets": rule_preset_catalog(),
                "generator_options": ["dynamics", "planform", "rule_flicker"],
                "pattern_options": ["auto", "rings", "rays", "spiral", "cobweb", "honeycomb", "rhombic", "hex_pi", "triangle"],
                "contour_mode_options": ["contoured", "noncontoured"],
                "parity_options": ["even", "odd"],
                "resolution_options": [32, 40, 48, 64, 80, 96],
                "orientation_options": [4, 8, 12, 16, 24],
                "rule_seed_pattern_options": ["random", "stripes", "hexagonal"],
                "solver_options": ["preview", "accurate"],
                "colormaps": ["twilight", "viridis", "magma", "inferno", "turbo", "gray"],
                "backend": "rust"
            });
            write_json(&mut stream, &body)?;
        }
        "/api/generate" => {
            let params = coerce_params(&query);
            let cache_key = payload_cache_key(params);
            let cached = state.payloads.lock().unwrap().get(&cache_key).cloned();
            let payload = if let Some(payload) = cached {
                payload
            } else {
                let payload = Arc::new(generate_payload(params, state)?);
                state
                    .payloads
                    .lock()
                    .unwrap()
                    .insert(cache_key, Arc::clone(&payload));
                payload
            };
            write_json(&mut stream, payload.as_ref())?;
        }
        _ => serve_static(&mut stream, root, &path)?,
    }
    Ok(())
}

fn split_query(target: &str) -> (String, HashMap<String, String>) {
    let mut pieces = target.splitn(2, '?');
    let path = pieces.next().unwrap_or("/").to_string();
    let query = pieces.next().map(parse_query).unwrap_or_else(HashMap::new);
    (path, query)
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut pieces = pair.splitn(2, '=');
            let key = pieces.next()?;
            let value = pieces.next().unwrap_or("");
            Some((decode_uri_component(key), decode_uri_component(value)))
        })
        .collect()
}

fn decode_uri_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                    if let Ok(parsed) = u8::from_str_radix(hex, 16) {
                        out.push(parsed);
                        i += 3;
                        continue;
                    }
                }
                out.push(bytes[i]);
                i += 1;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn serve_static(
    stream: &mut TcpStream,
    root: &Path,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let safe_path = path.trim_start_matches('/').replace('\\', "/");
    if safe_path.contains("..") {
        write_response(stream, 400, "text/plain", b"bad path")?;
        return Ok(());
    }
    let path = if safe_path.is_empty() {
        root.join("viewer/index.html")
    } else {
        root.join(safe_path)
    };
    match fs::read(&path) {
        Ok(body) => write_response(stream, 200, content_type(&path), &body)?,
        Err(_) => write_response(stream, 404, "text/plain", b"not found")?,
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

fn write_json<T: Serialize>(
    stream: &mut TcpStream,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec(value)?;
    write_response(stream, 200, "application/json; charset=utf-8", &body)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    Ok(())
}

fn default_params_json() -> serde_json::Value {
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

fn payload_cache_key(params: FrameParams) -> String {
    format!("{params:?}")
}

fn coerce_params(raw: &HashMap<String, String>) -> FrameParams {
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

fn get_usize(
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

fn get_u64(raw: &HashMap<String, String>, key: &str, default: u64) -> u64 {
    raw.get(key)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn get_f64(raw: &HashMap<String, String>, key: &str, default: f64, min: f64, max: f64) -> f64 {
    raw.get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

fn get_bool(raw: &HashMap<String, String>, key: &str, default: bool) -> bool {
    raw.get(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn generate_payload(
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

fn generate_planform_frames(params: FrameParams) -> (Vec<f32>, Vec<f64>, Option<Vec<f32>>) {
    let frame_count = params.frames.max(1);
    let frame_size = params.n * params.n;
    let mut frames = vec![0.0_f32; frame_count * frame_size];
    let mut orientation_frames = params
        .export_orientation_channels
        .then(|| vec![0.0_f32; frame_count * frame_size * params.m]);
    let cell_mm = cell_mm_for(params);
    let extent = params.n as f64 * cell_mm;
    let half = extent / 2.0;
    let stability = stability_scan(params);
    let planform_params = effective_planform_params(params, &stability);
    let wave_number = planform_wave_number(params, cell_mm, Some(&stability));
    let q = wave_number * params.hypercolumn_mm;
    let eigen = orientation_eigen_details(planform_params, q);
    let branch_selection = branch_selection(planform_params, &stability);
    let effective_pattern = effective_pattern(params, &branch_selection);
    let modes = planform_modes(planform_params, effective_pattern);
    let times: Vec<f64> = (0..frame_count)
        .map(|frame_index| {
            let progress = if frame_count <= 1 {
                0.0
            } else {
                frame_index as f64 / (frame_count - 1) as f64
            };
            params.t * progress
        })
        .collect();

    match orientation_frames.as_mut() {
        Some(channels) => frames
            .par_chunks_mut(frame_size)
            .zip(channels.par_chunks_mut(frame_size * params.m))
            .enumerate()
            .for_each(|(frame_index, (frame, channel_frame))| {
                let progress = if frame_count <= 1 {
                    0.0
                } else {
                    frame_index as f64 / (frame_count - 1) as f64
                };
                let phase = 2.0 * PI * params.drift * progress;

                for row in 0..params.n {
                    let y = (row as f64 + 0.5) * cell_mm - half;
                    for col in 0..params.n {
                        let x = (col as f64 + 0.5) * cell_mm - half;
                        let cell = row * params.n + col;
                        if planform_params.contour_mode == ContourMode::Noncontoured {
                            let value = planform_scalar_activity(x, y, wave_number, phase, &modes);
                            let output = (value * params.sharpness).tanh() as f32;
                            for k in 0..params.m {
                                channel_frame[cell * params.m + k] = output;
                            }
                            frame[cell] = output;
                            continue;
                        }

                        let mut best = 0.0_f64;
                        for k in 0..params.m {
                            let phi = PI * k as f64 / params.m as f64;
                            let value = orientation_planform_activity(
                                x,
                                y,
                                phi,
                                wave_number,
                                phase,
                                &modes,
                                &eigen,
                            );
                            channel_frame[cell * params.m + k] =
                                (value * params.sharpness).tanh() as f32;
                            if value.abs() > best.abs() {
                                best = value;
                            }
                        }
                        frame[cell] = (best * params.sharpness).tanh() as f32;
                    }
                }
            }),
        None => frames
            .par_chunks_mut(frame_size)
            .enumerate()
            .for_each(|(frame_index, frame)| {
                let progress = if frame_count <= 1 {
                    0.0
                } else {
                    frame_index as f64 / (frame_count - 1) as f64
                };
                let phase = 2.0 * PI * params.drift * progress;

                for row in 0..params.n {
                    let y = (row as f64 + 0.5) * cell_mm - half;
                    for col in 0..params.n {
                        let x = (col as f64 + 0.5) * cell_mm - half;
                        let value = planform_value(
                            planform_params,
                            x,
                            y,
                            wave_number,
                            phase,
                            &modes,
                            &eigen,
                        );
                        frame[row * params.n + col] = (value * params.sharpness).tanh() as f32;
                    }
                }
            }),
    }

    (frames, times, orientation_frames)
}

#[derive(Clone, Debug)]
struct RuleGaussianKernel {
    radius: usize,
    weights: Vec<f64>,
}

fn simulate_rule_flicker_frames(params: FrameParams) -> (Vec<f32>, Vec<f64>) {
    let frame_count = params.frames.max(1);
    let frame_size = params.n * params.n;
    let mut frames = Vec::with_capacity(frame_count * frame_size);
    let mut times = Vec::with_capacity(frame_count);
    let (mut ue, mut ui) = initialize_rule_state(params);
    let kernel_e = rule_gaussian_kernel(params.rule_sigma_e);
    let kernel_i = rule_gaussian_kernel(params.rule_sigma_i);
    let mut tmp_e = vec![0.0; frame_size];
    let mut tmp_i = vec![0.0; frame_size];
    let mut conv_e = vec![0.0; frame_size];
    let mut conv_i = vec![0.0; frame_size];
    let mut next_e = vec![0.0; frame_size];
    let mut next_i = vec![0.0; frame_size];
    let step = match params.solver {
        Solver::Preview => params.preview_step,
        Solver::Accurate => params.preview_step.min(0.1),
    }
    .clamp(0.02, 1.0);
    let mut current_t = 0.0;

    for frame_index in 0..frame_count {
        let target_t = if frame_count <= 1 {
            0.0
        } else {
            params.t * frame_index as f64 / (frame_count - 1) as f64
        };

        while current_t + 1.0e-12 < target_t {
            let dt = step.min(target_t - current_t);
            convolve_periodic_separable(&ue, params.n, &kernel_e, &mut tmp_e, &mut conv_e);
            convolve_periodic_separable(&ui, params.n, &kernel_i, &mut tmp_i, &mut conv_i);
            let stim = params.rule_stim_amplitude * rule_stimulus(params, current_t);
            for i in 0..frame_size {
                let input_e =
                    params.rule_aee * conv_e[i] - params.rule_aie * conv_i[i] - params.rule_theta_e
                        + stim;
                let input_i =
                    params.rule_aei * conv_e[i] - params.rule_aii * conv_i[i] - params.rule_theta_i
                        + params.rule_stim_i_fraction * stim;
                let target_e = rule_sigmoid(input_e);
                let target_i = rule_sigmoid(input_i);
                next_e[i] =
                    (ue[i] + (dt / params.rule_tau_e_ms) * (-ue[i] + target_e)).clamp(0.0, 1.0);
                next_i[i] =
                    (ui[i] + (dt / params.rule_tau_i_ms) * (-ui[i] + target_i)).clamp(0.0, 1.0);
            }
            std::mem::swap(&mut ue, &mut next_e);
            std::mem::swap(&mut ui, &mut next_i);
            current_t += dt;
        }

        times.push(target_t);
        frames.extend(ue.iter().map(|value| *value as f32));
    }

    (frames, times)
}

fn initialize_rule_state(params: FrameParams) -> (Vec<f64>, Vec<f64>) {
    let frame_size = params.n * params.n;
    let (base_e, base_i) = rule_rest_state(params);
    let mut rng = SplitMix64::new(params.seed);
    let mut ue = vec![base_e; frame_size];
    let mut ui = vec![base_i; frame_size];

    if params.rule_seed_strength <= 0.0 {
        return (ue, ui);
    }

    for row in 0..params.n {
        for col in 0..params.n {
            let i = row * params.n + col;
            let structured = rule_seed_value(params, row, col);
            let noise = (rng.next_f64() * 2.0 - 1.0) * 0.2;
            let perturbation = params.rule_seed_strength * (structured + noise);
            ue[i] = (base_e + perturbation).clamp(0.0, 1.0);
            ui[i] = (base_i + 0.35 * perturbation).clamp(0.0, 1.0);
        }
    }
    (ue, ui)
}

fn rule_rest_state(params: FrameParams) -> (f64, f64) {
    let mut ue = 0.1;
    let mut ui = 0.1;
    for _ in 0..2000 {
        let target_e =
            rule_sigmoid(params.rule_aee * ue - params.rule_aie * ui - params.rule_theta_e);
        let target_i =
            rule_sigmoid(params.rule_aei * ue - params.rule_aii * ui - params.rule_theta_i);
        ue += 0.05 * (target_e - ue);
        ui += 0.05 * (target_i - ui);
    }
    (ue, ui)
}

fn rule_seed_value(params: FrameParams, row: usize, col: usize) -> f64 {
    let x = col as f64 / params.n as f64 - 0.5;
    let y = row as f64 / params.n as f64 - 0.5;
    let cycles = if params.rule_stim_period_ms < 80.0 {
        4.0
    } else {
        5.0
    };
    let q = 2.0 * PI * cycles;
    match params.rule_seed_pattern {
        RuleSeedPattern::Random => 0.0,
        RuleSeedPattern::Stripes => (q * x).cos(),
        RuleSeedPattern::Hexagonal => {
            let a = (q * x).cos();
            let b = (q * (-0.5 * x + 0.866_025_403_784_438_6 * y)).cos();
            let c = (q * (-0.5 * x - 0.866_025_403_784_438_6 * y)).cos();
            (a + b + c) / 3.0
        }
    }
}

fn rule_gaussian_kernel(sigma: f64) -> RuleGaussianKernel {
    let sigma = sigma.max(0.1);
    let radius = (3.0 * sigma).ceil() as usize;
    let mut weights: Vec<f64> = (0..=2 * radius)
        .map(|i| {
            let offset = i as isize - radius as isize;
            (-(offset as f64).powi(2) / (sigma * sigma)).exp()
        })
        .collect();
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for weight in &mut weights {
            *weight /= sum;
        }
    }
    RuleGaussianKernel { radius, weights }
}

fn convolve_periodic_separable(
    input: &[f64],
    n: usize,
    kernel: &RuleGaussianKernel,
    tmp: &mut [f64],
    output: &mut [f64],
) {
    for row in 0..n {
        for col in 0..n {
            let mut sum = 0.0;
            for (k, weight) in kernel.weights.iter().enumerate() {
                let delta = k as isize - kernel.radius as isize;
                let source_col = wrap_index(col, delta, n);
                sum += weight * input[row * n + source_col];
            }
            tmp[row * n + col] = sum;
        }
    }

    for row in 0..n {
        for col in 0..n {
            let mut sum = 0.0;
            for (k, weight) in kernel.weights.iter().enumerate() {
                let delta = k as isize - kernel.radius as isize;
                let source_row = wrap_index(row, delta, n);
                sum += weight * tmp[source_row * n + col];
            }
            output[row * n + col] = sum;
        }
    }
}

fn rule_stimulus(params: FrameParams, time_ms: f64) -> f64 {
    let phase = (2.0 * PI * time_ms / params.rule_stim_period_ms.max(1.0e-9)).sin()
        - params.rule_stim_threshold;
    if params.rule_stim_smoothing <= 0.0 {
        if phase > 0.0 {
            1.0
        } else {
            0.0
        }
    } else {
        rule_sigmoid(params.rule_stim_smoothing * phase)
    }
}

fn rule_sigmoid(x: f64) -> f64 {
    if x <= -50.0 {
        0.0
    } else if x >= 50.0 {
        1.0
    } else {
        1.0 / (1.0 + (-x).exp())
    }
}

#[derive(Clone, Copy, Debug)]
struct RuleOrbitStep {
    dt: f64,
    time_ms: f64,
    ue: f64,
    ui: f64,
}

fn rule_floquet_report(params: FrameParams, mode_cycles: &[f64]) -> RuleFloquetReport {
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

fn rule_floquet_grid_point_for(params: FrameParams, mode_cycles: &[f64]) -> RuleFloquetGridPoint {
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

fn rule_floquet_boundary_candidates(
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

fn find_rule_floquet_point<'a>(
    points: &'a [RuleFloquetGridPoint],
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
) -> Option<&'a RuleFloquetGridPoint> {
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

fn rule_floquet_branch_label(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "-1 period-doubling",
        "plus_one_to_one" => "+1 one-to-one",
        "unstable_complex" => "complex unit-circle",
        _ => "unknown",
    }
}

fn rule_floquet_branch_periodicity(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "2T",
        "plus_one_to_one" => "T",
        "unstable_complex" => "complex",
        _ => "unknown",
    }
}

fn refine_scalar_sign_change<F>(
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

fn empty_rule_floquet_curve_fit() -> RuleFloquetBoundaryCurveFit {
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
        for j in col..n {
            matrix[col][j] /= pivot_value;
        }
        rhs[col] /= pivot_value;
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = matrix[row][col];
            for j in col..n {
                matrix[row][j] -= factor * matrix[col][j];
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

fn floquet_mode_margin(mode: &RuleFloquetMode, kind: &'static str) -> f64 {
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

fn floquet_mode_from_matrix(
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

fn rule_wave_number_for_cycles(beta_cycles: f64, n: usize) -> f64 {
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

fn cell_mm_for(params: FrameParams) -> f64 {
    match params.generator {
        Generator::Dynamics | Generator::RuleFlicker => DYNAMIC_CELL_MM,
        Generator::Planform => (2.0 * PI * RETINO_BETA / RETINO_EPS) / params.n as f64,
    }
}

fn orientation_count_for(params: FrameParams) -> usize {
    match params.generator {
        Generator::RuleFlicker => 1,
        Generator::Dynamics | Generator::Planform => params.m,
    }
}

fn planform_details(params: FrameParams, cell_mm: f64) -> PlanformDetails {
    let stability = stability_scan(params);
    let planform_params = effective_planform_params(params, &stability);
    let wave_number = planform_wave_number(params, cell_mm, Some(&stability));
    let q = wave_number * params.hypercolumn_mm;
    let branch_selection = branch_selection(planform_params, &stability);
    let effective_pattern = effective_pattern(params, &branch_selection);
    PlanformDetails {
        contour_mode: params.contour_mode.as_str(),
        parity: planform_params.parity.as_str(),
        q,
        wave_number,
        phase_base: 2.0 * PI * params.drift,
        modes: planform_modes(planform_params, effective_pattern),
        eigen: orientation_eigen_details(planform_params, q),
        stability,
        branch_selection,
        kernel: kernel_details(params),
    }
}

fn calibration_report(
    preset: PaperPresetDetails,
    planform: &PlanformDetails,
    metrics: &Metrics,
    params: FrameParams,
) -> CalibrationReport {
    let rendered_pattern = effective_pattern_from_params(params, planform);
    let rendered_family = pattern_family(rendered_pattern);
    let selected_family = planform.branch_selection.selected_family;
    let selected_pattern = planform.branch_selection.selected_pattern;
    let global_selected_family = planform.branch_selection.global_selected_family;
    let global_selected_pattern = planform.branch_selection.global_selected_pattern;
    let check_branch_selection = preset.expected_kind != "single-map-noncontoured-planform";
    let check_branch_pattern = check_branch_selection && preset.expected_family != "roll";
    let mut checks = Vec::new();

    checks.push(CalibrationCheck {
        name: "contour-mode",
        expected: preset.expected_contour_mode,
        actual: params.contour_mode.as_str().to_string(),
        passed: params.contour_mode.as_str() == preset.expected_contour_mode,
    });

    checks.push(CalibrationCheck {
        name: "parity",
        expected: preset.expected_parity,
        actual: planform.parity.to_string(),
        passed: planform.parity == preset.expected_parity,
    });

    if preset.expected_pattern != "auto" {
        checks.push(CalibrationCheck {
            name: "rendered-pattern",
            expected: preset.expected_pattern,
            actual: rendered_pattern.as_str().to_string(),
            passed: rendered_pattern.as_str() == preset.expected_pattern,
        });

        if check_branch_pattern {
            checks.push(CalibrationCheck {
                name: "same-lattice-branch-pattern",
                expected: preset.expected_pattern,
                actual: selected_pattern.to_string(),
                passed: selected_pattern == preset.expected_pattern,
            });
        }
    }

    if preset.expected_family != "branch-selected" {
        checks.push(CalibrationCheck {
            name: "rendered-family",
            expected: preset.expected_family,
            actual: rendered_family.to_string(),
            passed: rendered_family == preset.expected_family,
        });

        if check_branch_selection {
            checks.push(CalibrationCheck {
                name: "same-lattice-branch-family",
                expected: preset.expected_family,
                actual: selected_family.to_string(),
                passed: selected_family == preset.expected_family,
            });
        }
    }

    let status = if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "review"
    };

    CalibrationReport {
        preset,
        status,
        rendered_contour_mode: params.contour_mode.as_str(),
        rendered_parity: planform.parity,
        rendered_pattern: rendered_pattern.as_str(),
        selected_family,
        selected_pattern,
        selected_scope: planform.branch_selection.selected_scope,
        global_selected_family,
        global_selected_pattern,
        target_lattice: planform.branch_selection.target_lattice,
        critical_q: planform.stability.critical_q,
        critical_branch: planform.stability.critical_branch,
        dominant_cycles: metrics.dominant_cycles,
        checks,
    }
}

fn rule_details(
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

fn representative_rule_frame(frames: &[f32], n: usize) -> Option<&[f32]> {
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

fn effective_pattern_from_params(params: FrameParams, planform: &PlanformDetails) -> PatternPreset {
    match params.pattern {
        PatternPreset::Auto => match planform.branch_selection.selected_pattern {
            "honeycomb" => PatternPreset::Honeycomb,
            "hex_pi" => PatternPreset::HexPi,
            "triangle" => PatternPreset::Triangle,
            "rhombic" => PatternPreset::Rhombic,
            "spiral" => PatternPreset::Spiral,
            "rings" => PatternPreset::Rings,
            _ => PatternPreset::Cobweb,
        },
        other => other,
    }
}

fn pattern_family(pattern: PatternPreset) -> &'static str {
    match pattern {
        PatternPreset::Auto => "branch-selected",
        PatternPreset::Rings | PatternPreset::Rays | PatternPreset::Spiral => "roll",
        PatternPreset::Cobweb => "square",
        PatternPreset::Honeycomb | PatternPreset::HexPi | PatternPreset::Triangle => "hexagonal",
        PatternPreset::Rhombic => "rhombic",
    }
}

fn planform_wave_number(
    params: FrameParams,
    cell_mm: f64,
    stability: Option<&StabilityDetails>,
) -> f64 {
    if params.pattern == PatternPreset::Auto {
        let critical_q = stability
            .map(|details| details.critical_q)
            .unwrap_or_else(|| stability_scan(params).critical_q);
        return critical_q / params.hypercolumn_mm.max(1.0e-9);
    }
    let extent = params.n as f64 * cell_mm;
    2.0 * PI * params.wave_count / extent.max(1.0e-9)
}

fn effective_planform_params(mut params: FrameParams, stability: &StabilityDetails) -> FrameParams {
    if params.pattern == PatternPreset::Auto {
        params.parity = parity_from_branch(stability.critical_branch);
    }
    params
}

fn parity_from_branch(branch: &str) -> Parity {
    if branch == "odd" {
        Parity::Odd
    } else {
        Parity::Even
    }
}

fn effective_pattern(
    params: FrameParams,
    branch_selection: &BranchSelectionDetails,
) -> PatternPreset {
    if params.pattern != PatternPreset::Auto {
        return params.pattern;
    }
    match branch_selection.selected_pattern {
        "honeycomb" => PatternPreset::Honeycomb,
        "hex_pi" => PatternPreset::HexPi,
        "triangle" => PatternPreset::Triangle,
        "rhombic" => PatternPreset::Rhombic,
        "spiral" => PatternPreset::Spiral,
        "rings" => PatternPreset::Rings,
        _ => PatternPreset::Cobweb,
    }
}

fn kernel_details(params: FrameParams) -> KernelDetails {
    KernelDetails {
        local_sigma_deg: params.local_sigma_deg,
        local_wide_sigma_deg: params.local_wide_sigma_deg,
        local_inhibition: params.local_inhibition,
        lateral_sigma: params.lateral_sigma,
        lateral_wide_sigma: params.lateral_wide_sigma,
        lateral_inhibition: params.lateral_inhibition,
        lateral_spread_deg: params.lateral_spread_deg,
    }
}

fn stability_scan(params: FrameParams) -> StabilityDetails {
    let samples = params.stability_samples.max(2);
    let q_min = params.stability_q_min.min(params.stability_q_max);
    let q_max = params.stability_q_max.max(q_min + 1.0e-6);
    let mut points = Vec::with_capacity(samples);
    let mut critical_q = q_min;
    let mut critical_branch = "even";
    let mut critical_growth = f64::NEG_INFINITY;

    for i in 0..samples {
        let q = if samples <= 1 {
            q_min
        } else {
            q_min + (q_max - q_min) * i as f64 / (samples - 1) as f64
        };
        let even_growth = branch_growth(params, Parity::Even, q);
        let odd_growth = branch_growth(params, Parity::Odd, q);
        if even_growth >= critical_growth {
            critical_growth = even_growth;
            critical_q = q;
            critical_branch = "even";
        }
        if odd_growth >= critical_growth {
            critical_growth = odd_growth;
            critical_q = q;
            critical_branch = "odd";
        }
        points.push(StabilityPoint {
            q,
            even_growth,
            odd_growth,
        });
    }

    let mut branch_params = params;
    branch_params.parity = parity_from_branch(critical_branch);
    let selected_pattern =
        branch_selection_for(branch_params, critical_q, critical_growth).selected_pattern;
    StabilityDetails {
        q_min,
        q_max,
        samples,
        critical_q,
        critical_branch,
        critical_growth,
        selected_pattern,
        points,
    }
}

fn branch_growth(params: FrameParams, parity: Parity, q: f64) -> f64 {
    let beta = params.eigen_beta;
    local_weight_coeff(params, 1)
        + beta * signed_lateral_pair(params, parity, 0, 2, q)
        + beta * beta * branch_coupling_sum(params, parity, q, 10)
}

fn branch_coupling_sum(params: FrameParams, parity: Parity, q: f64, harmonics: usize) -> f64 {
    let w1 = local_weight_coeff(params, 1);
    (0..=harmonics)
        .filter(|&m| m != 1)
        .map(|m| {
            let left = if m == 0 {
                lateral_weight_coeff(params, 1, q)
            } else {
                lateral_weight_coeff(params, m - 1, q)
            };
            let right = lateral_weight_coeff(params, m + 1, q);
            let numerator = match parity {
                Parity::Even => left + right,
                Parity::Odd => left - right,
            };
            numerator * numerator / safe_denominator(w1 - local_weight_coeff(params, m))
        })
        .sum()
}

fn signed_lateral_pair(
    params: FrameParams,
    parity: Parity,
    left_harmonic: usize,
    right_harmonic: usize,
    q: f64,
) -> f64 {
    let left = lateral_weight_coeff(params, left_harmonic, q);
    let right = lateral_weight_coeff(params, right_harmonic, q);
    match parity {
        Parity::Even => left + right,
        Parity::Odd => left - right,
    }
}

fn branch_selection(params: FrameParams, stability: &StabilityDetails) -> BranchSelectionDetails {
    branch_selection_for(params, stability.critical_q, stability.critical_growth)
}

fn branch_selection_for(params: FrameParams, q: f64, growth: f64) -> BranchSelectionDetails {
    let lambda = growth.max(0.0);
    let eigen = orientation_eigen_details(params, q);
    let square_theta = PI / 2.0;
    let rhombic_theta = params
        .pattern_angle
        .to_radians()
        .clamp(PI / 12.0, 5.0 * PI / 12.0);
    let hex_theta = 2.0 * PI / 3.0;
    let gamma0 = amplitude_gamma3(0.0, &eigen);
    let gamma_square = amplitude_gamma3(square_theta, &eigen);
    let gamma_rhombic = amplitude_gamma3(rhombic_theta, &eigen);
    let gamma_hex = amplitude_gamma3(hex_theta, &eigen);
    let eta_hex = match params.parity {
        Parity::Even => amplitude_gamma2(&eigen),
        Parity::Odd => 0.0,
    };

    let roll_stable = gamma0 > 0.0
        && 2.0 * gamma_square > gamma0
        && 2.0 * gamma_rhombic > gamma0
        && 2.0 * gamma_hex > gamma0;
    let roll = branch_candidate(
        "roll",
        "spiral",
        1,
        0.0,
        gamma0,
        0.0,
        lambda,
        gamma0,
        0.0,
        roll_stable,
        "single active wavevector",
    );
    let square = branch_candidate(
        "square",
        "cobweb",
        2,
        square_theta,
        gamma0,
        gamma_square,
        lambda,
        gamma0 + 2.0 * gamma_square,
        0.0,
        gamma_square > 0.0 && 2.0 * gamma_square < gamma0,
        "two equal amplitudes on a square lattice",
    );
    let rhombic = branch_candidate(
        "rhombic",
        "rhombic",
        2,
        rhombic_theta,
        gamma0,
        gamma_rhombic,
        lambda,
        gamma0 + 2.0 * gamma_rhombic,
        0.0,
        gamma_rhombic > 0.0 && 2.0 * gamma_rhombic < gamma0,
        "two equal amplitudes on an oblique lattice",
    );
    let hex_pattern = if eta_hex < 0.0 { "hex_pi" } else { "honeycomb" };
    let hex_note = match params.parity {
        Parity::Even => "three-wave hexagonal branch with quadratic term",
        Parity::Odd => "odd hexagonal branch has zero quadratic term at cubic order",
    };
    let hex = branch_candidate(
        "hexagonal",
        hex_pattern,
        3,
        hex_theta,
        gamma0,
        gamma_hex,
        lambda,
        gamma0 + 4.0 * gamma_hex,
        eta_hex,
        gamma_hex > 0.0
            && (params.parity == Parity::Even || 2.0 * gamma_hex < gamma0)
            && gamma0 + 4.0 * gamma_hex > 0.0,
        hex_note,
    );
    let mut candidates = vec![roll, square, rhombic, hex];
    candidates.sort_by(|a, b| {
        b.stable
            .cmp(&a.stable)
            .then_with(|| b.score.total_cmp(&a.score))
    });
    let global_selected = candidates.first().copied().unwrap_or(roll);
    let target_lattice = branch_target_lattice(params.pattern);
    let (selected, selected_scope, selected_lattice_stable) = select_lattice_branch(
        &candidates,
        target_lattice,
        gamma0,
        gamma_square,
        gamma_rhombic,
        gamma_hex,
        global_selected,
    );

    BranchSelectionDetails {
        model: "cubic-amplitude-equation",
        lambda,
        gamma0,
        gamma_square,
        gamma_rhombic,
        gamma_hex,
        eta_hex,
        target_lattice,
        selected_scope,
        selected_family: selected.family,
        selected_pattern: selected.pattern,
        selected_lattice_stable,
        global_selected_family: global_selected.family,
        global_selected_pattern: global_selected.pattern,
        global_selected_stable: global_selected.stable,
        candidates,
    }
}

fn branch_target_lattice(pattern: PatternPreset) -> &'static str {
    match pattern {
        PatternPreset::Auto => "global",
        PatternPreset::Cobweb => "square",
        PatternPreset::Rhombic => "rhombic",
        PatternPreset::Honeycomb | PatternPreset::HexPi | PatternPreset::Triangle => "hexagonal",
        PatternPreset::Rings | PatternPreset::Rays | PatternPreset::Spiral => "roll",
    }
}

fn select_lattice_branch(
    candidates: &[BranchCandidate],
    target_lattice: &'static str,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
    global_selected: BranchCandidate,
) -> (BranchCandidate, &'static str, bool) {
    if target_lattice == "global" {
        return (global_selected, "global", global_selected.stable);
    }

    let mut scoped: Vec<(BranchCandidate, bool)> = candidates
        .iter()
        .copied()
        .filter(|candidate| candidate_in_lattice(*candidate, target_lattice))
        .map(|candidate| {
            let stable = lattice_local_stable(
                candidate,
                target_lattice,
                gamma0,
                gamma_square,
                gamma_rhombic,
                gamma_hex,
            );
            (candidate, stable)
        })
        .collect();

    scoped.sort_by(|(a, a_stable), (b, b_stable)| {
        b_stable
            .cmp(a_stable)
            .then_with(|| b.score.total_cmp(&a.score))
    });

    scoped
        .first()
        .copied()
        .map(|(candidate, stable)| (candidate, "lattice-local", stable))
        .unwrap_or((global_selected, "global-fallback", global_selected.stable))
}

fn candidate_in_lattice(candidate: BranchCandidate, target_lattice: &str) -> bool {
    match target_lattice {
        "roll" => candidate.family == "roll",
        "square" => candidate.family == "square" || candidate.family == "roll",
        "rhombic" => candidate.family == "rhombic" || candidate.family == "roll",
        "hexagonal" => candidate.family == "hexagonal" || candidate.family == "roll",
        _ => true,
    }
}

fn lattice_local_stable(
    candidate: BranchCandidate,
    target_lattice: &str,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
) -> bool {
    if candidate.family == "roll" {
        return match target_lattice {
            "square" => gamma0 > 0.0 && 2.0 * gamma_square > gamma0,
            "rhombic" => gamma0 > 0.0 && 2.0 * gamma_rhombic > gamma0,
            "hexagonal" => gamma0 > 0.0 && 2.0 * gamma_hex > gamma0,
            "roll" => gamma0 > 0.0,
            _ => candidate.stable,
        };
    }

    match candidate.family {
        "square" => gamma_square > 0.0 && 2.0 * gamma_square < gamma0,
        "rhombic" => gamma_rhombic > 0.0 && 2.0 * gamma_rhombic < gamma0,
        "hexagonal" => candidate.stable,
        _ => candidate.stable,
    }
}

fn branch_candidate(
    family: &'static str,
    pattern: &'static str,
    mode_count: usize,
    theta_rad: f64,
    gamma0: f64,
    gamma_cross: f64,
    lambda: f64,
    denominator: f64,
    eta: f64,
    stable: bool,
    note: &'static str,
) -> BranchCandidate {
    let denominator = denominator.max(1.0e-9);
    let lambda = lambda.max(0.0);
    let amplitude = if lambda <= 0.0 {
        0.0
    } else if eta.abs() > 1.0e-9 {
        ((eta.abs() + (eta * eta + 4.0 * denominator * lambda).sqrt()) / (2.0 * denominator))
            .max(0.0)
    } else {
        (lambda / denominator).sqrt()
    };
    let score = if lambda <= 0.0 {
        f64::NEG_INFINITY
    } else if mode_count == 1 {
        lambda * lambda / (4.0 * gamma0.max(1.0e-9))
    } else {
        mode_count as f64
            * (0.5 * lambda * amplitude * amplitude + eta.abs() * amplitude.powi(3) / 3.0
                - 0.25 * denominator * amplitude.powi(4))
    };
    BranchCandidate {
        family,
        pattern,
        mode_count,
        theta_rad,
        gamma_cross,
        eta,
        amplitude,
        score,
        stable: stable && lambda > 0.0 && amplitude.is_finite(),
        note,
    }
}

fn amplitude_gamma2(eigen: &OrientationEigenDetails) -> f64 {
    const SAMPLES: usize = 720;
    (0..SAMPLES)
        .map(|i| {
            let phi = PI * (i as f64 + 0.5) / SAMPLES as f64;
            orientation_eigen_value(phi, eigen)
                * orientation_eigen_value(phi - 2.0 * PI / 3.0, eigen)
                * orientation_eigen_value(phi + 2.0 * PI / 3.0, eigen)
        })
        .sum::<f64>()
        / SAMPLES as f64
}

fn amplitude_gamma3(theta: f64, eigen: &OrientationEigenDetails) -> f64 {
    const SAMPLES: usize = 720;
    (0..SAMPLES)
        .map(|i| {
            let phi = PI * (i as f64 + 0.5) / SAMPLES as f64;
            let shifted = orientation_eigen_value(phi - theta, eigen);
            let base = orientation_eigen_value(phi, eigen);
            shifted * shifted * base * base
        })
        .sum::<f64>()
        / SAMPLES as f64
}

fn planform_modes(params: FrameParams, pattern: PatternPreset) -> Vec<PlanformModeDetails> {
    let angle = params.pattern_angle.to_radians();
    match pattern {
        PatternPreset::Auto => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(PI / 2.0, 0.35, 1.0),
        ],
        PatternPreset::Rings => vec![planform_mode(0.0, 1.0, 1.0)],
        PatternPreset::Rays => vec![planform_mode(PI / 2.0, 1.0, 1.0)],
        PatternPreset::Spiral => vec![planform_mode(angle, 1.0, 1.0)],
        PatternPreset::Cobweb => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(PI / 2.0, 0.35, 1.0),
        ],
        PatternPreset::Honeycomb => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(2.0 * PI / 3.0, 1.0, 1.0),
            planform_mode(-2.0 * PI / 3.0, 1.0, 1.0),
        ],
        PatternPreset::Rhombic => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(angle, -0.25, 1.0),
        ],
        PatternPreset::HexPi => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(2.0 * PI / 3.0, -1.0, -1.0),
            planform_mode(-2.0 * PI / 3.0, 0.5, 1.0),
        ],
        PatternPreset::Triangle => vec![
            planform_mode_with_phase(0.0, 1.0, -PI / 2.0, 1.0),
            planform_mode_with_phase(2.0 * PI / 3.0, 1.0, -PI / 2.0, 1.0),
            planform_mode_with_phase(-2.0 * PI / 3.0, 1.0, -PI / 2.0, 1.0),
        ],
    }
}

fn planform_mode(normal_angle: f64, phase_scale: f64, amplitude: f64) -> PlanformModeDetails {
    planform_mode_with_phase(normal_angle, phase_scale, 0.0, amplitude)
}

fn planform_mode_with_phase(
    normal_angle: f64,
    phase_scale: f64,
    phase_offset: f64,
    amplitude: f64,
) -> PlanformModeDetails {
    PlanformModeDetails {
        normal_angle,
        phase_scale,
        phase_offset,
        amplitude,
    }
}

fn planform_value(
    params: FrameParams,
    x: f64,
    y: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
    eigen: &OrientationEigenDetails,
) -> f64 {
    if params.contour_mode == ContourMode::Noncontoured {
        return planform_scalar_activity(x, y, wave_number, phase, modes);
    }

    let samples = params.m.max(8);
    let mut best = 0.0_f64;
    for k in 0..samples {
        let phi = PI * k as f64 / samples as f64;
        let value = orientation_planform_activity(x, y, phi, wave_number, phase, modes, eigen);
        if value.abs() > best.abs() {
            best = value;
        }
    }
    best
}

fn planform_scalar_activity(
    x: f64,
    y: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
) -> f64 {
    modes
        .iter()
        .map(|mode| {
            let projection = x * mode.normal_angle.cos() + y * mode.normal_angle.sin();
            mode.amplitude
                * (wave_number * projection + phase * mode.phase_scale + mode.phase_offset).cos()
        })
        .sum()
}

fn orientation_planform_activity(
    x: f64,
    y: f64,
    phi: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
    eigen: &OrientationEigenDetails,
) -> f64 {
    modes
        .iter()
        .map(|mode| {
            let projection = x * mode.normal_angle.cos() + y * mode.normal_angle.sin();
            let spatial = mode.amplitude
                * (wave_number * projection + phase * mode.phase_scale + mode.phase_offset).cos();
            let tangent_center = mode.normal_angle + PI / 2.0;
            spatial * orientation_eigen_value(phi - tangent_center, eigen)
        })
        .sum()
}

fn orientation_eigen_value(delta: f64, eigen: &OrientationEigenDetails) -> f64 {
    let cos_part = eigen
        .cos_coefficients
        .iter()
        .map(|[harmonic, coefficient]| coefficient * (2.0 * harmonic * delta).cos())
        .sum::<f64>();
    let sin_part = eigen
        .sin_coefficients
        .iter()
        .map(|[harmonic, coefficient]| coefficient * (2.0 * harmonic * delta).sin())
        .sum::<f64>();
    cos_part + sin_part
}

fn orientation_eigen_details(params: FrameParams, q: f64) -> OrientationEigenDetails {
    let max_harmonic = 4;
    let mut cos_coefficients = Vec::new();
    let mut sin_coefficients = Vec::new();
    match params.parity {
        Parity::Even => {
            cos_coefficients.push([1.0, 1.0]);
            let u0 = lateral_weight_coeff(params, 1, q)
                / safe_denominator(local_weight_coeff(params, 1) - local_weight_coeff(params, 0));
            cos_coefficients.push([0.0, (params.eigen_beta * u0).clamp(-1.5, 1.5)]);
            for m in 2..=max_harmonic {
                let coeff = (lateral_weight_coeff(params, m - 1, q)
                    + lateral_weight_coeff(params, m + 1, q))
                    / safe_denominator(
                        local_weight_coeff(params, 1) - local_weight_coeff(params, m),
                    );
                cos_coefficients.push([m as f64, (params.eigen_beta * coeff).clamp(-1.5, 1.5)]);
            }
        }
        Parity::Odd => {
            sin_coefficients.push([1.0, 1.0]);
            for m in 2..=max_harmonic {
                let coeff = (lateral_weight_coeff(params, m - 1, q)
                    - lateral_weight_coeff(params, m + 1, q))
                    / safe_denominator(
                        local_weight_coeff(params, 1) - local_weight_coeff(params, m),
                    );
                sin_coefficients.push([m as f64, (params.eigen_beta * coeff).clamp(-1.5, 1.5)]);
            }
        }
    }

    OrientationEigenDetails {
        parity: params.parity.as_str(),
        beta: params.eigen_beta,
        cos_coefficients,
        sin_coefficients,
    }
}

fn safe_denominator(value: f64) -> f64 {
    if value.abs() < 1.0e-6 {
        if value.is_sign_negative() {
            -1.0e-6
        } else {
            1.0e-6
        }
    } else {
        value
    }
}

fn local_weight_coeff(params: FrameParams, n: usize) -> f64 {
    let xi = params.local_sigma_deg.to_radians();
    let xi_hat = params.local_wide_sigma_deg.to_radians();
    let inhibition = params.local_inhibition;
    (-2.0 * (n as f64).powi(2) * xi * xi).exp()
        - inhibition * (-2.0 * (n as f64).powi(2) * xi_hat * xi_hat).exp()
}

fn lateral_weight_coeff(params: FrameParams, n: usize, q: f64) -> f64 {
    let xi = params.lateral_sigma;
    let xi_hat = params.lateral_wide_sigma;
    let inhibition = params.lateral_inhibition;
    let narrow = 0.25 * xi * xi * q * q;
    let broad = 0.25 * xi_hat * xi_hat * q * q;
    let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
    lateral_spread_factor(params, n)
        * 0.5
        * sign
        * ((-narrow).exp() * modified_bessel_i(n, narrow)
            - inhibition * (-broad).exp() * modified_bessel_i(n, broad))
}

fn lateral_spread_factor(params: FrameParams, n: usize) -> f64 {
    let theta0 = params.lateral_spread_deg.to_radians();
    if n == 0 || theta0.abs() < 1.0e-9 {
        return 1.0;
    }
    let x = 2.0 * n as f64 * theta0;
    x.sin() / x
}

fn modified_bessel_i(n: usize, x: f64) -> f64 {
    if x.abs() < 1.0e-12 {
        return if n == 0 { 1.0 } else { 0.0 };
    }
    let half_x = 0.5 * x;
    let mut factorial = 1.0;
    for value in 1..=n {
        factorial *= value as f64;
    }
    let mut term = half_x.powi(n as i32) / factorial;
    let mut sum = term;
    for k in 1..80 {
        term *= half_x * half_x / (k as f64 * (k + n) as f64);
        sum += term;
        if term.abs() < sum.abs().max(1.0) * 1.0e-13 {
            break;
        }
    }
    sum
}

fn get_structure(params: FrameParams, state: &ServerState) -> (Arc<Structure>, bool) {
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
    fn new(n: usize, m: usize, r0: f64) -> Self {
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

fn simulate_frames(
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

fn append_scalar_frame(state: &[f64], params: FrameParams, frames: &mut Vec<f32>) {
    for cell in 0..params.n * params.n {
        let base = cell * params.m;
        let mut sum = 0.0;
        for k in 0..params.m {
            sum += state[base + k];
        }
        frames.push((sum / params.m as f64) as f32);
    }
}

fn step_preview(
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

fn step_rk4(state: &mut [f64], structure: &Structure, params: FrameParams, dt: f64) {
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

fn derivative(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut conn = connectivity(state, structure, params);
    conn.par_iter_mut()
        .zip(state.par_iter())
        .for_each(|(value, a)| *value -= params.alpha * a);
    conn
}

fn connectivity(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut sigmoid_state = vec![0.0; state.len()];
    let mut out = vec![0.0; state.len()];
    connectivity_into(state, structure, params, &mut sigmoid_state, &mut out);
    out
}

fn connectivity_into(
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
        for k in 0..m {
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
            chunk[k] = params.mu * (angular_sum + params.beta * lateral_sum);
        }
    });
}

fn wrap_index(value: usize, delta: isize, size: usize) -> usize {
    (value as isize + delta).rem_euclid(size as isize) as usize
}

fn index(row: usize, col: usize, k: usize, n: usize, m: usize) -> usize {
    m * n * row + m * col + k
}

fn trim_warmup(
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

fn percentile_range(frames: &[f32], low_percentile: f64, high_percentile: f64) -> (f64, f64) {
    let mut sorted = frames.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let low = percentile_sorted(&sorted, low_percentile);
    let mut high = percentile_sorted(&sorted, high_percentile);
    if high <= low {
        high = low + 1.0e-9;
    }
    (low as f64, high as f64)
}

fn percentile_sorted(sorted: &[f32], percentile: f64) -> f32 {
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

fn normalize_u8(frames: &[f32], low: f64, high: f64) -> Vec<u8> {
    let denom = (high - low).max(1.0e-9);
    frames
        .iter()
        .map(|value| (((*value as f64 - low) / denom) * 255.0).clamp(0.0, 255.0) as u8)
        .collect()
}

fn orientation_channel_payload(
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

fn raw_range(frames: &[f32]) -> (f32, f32) {
    frames
        .iter()
        .copied()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        })
}

fn frame_metrics(frames: &[f32], n: usize) -> Metrics {
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

fn stddev(values: &[f32]) -> f32 {
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

fn projected_dominant_cycles(frame: &[f32], n: usize) -> f32 {
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

fn dominant_1d_frequency(values: &[f32]) -> usize {
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

fn retino_bounds(n: usize, cell_mm: f64) -> RetinoBounds {
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

fn inverse_retino_cortical_map(x: f64, y: f64) -> (f64, f64) {
    let r = RETINO_W0 / RETINO_EPS * (RETINO_EPS * x / RETINO_ALPHA).exp();
    let theta = RETINO_EPS * y / RETINO_BETA;
    (r * theta.cos(), r * theta.sin())
}

fn palette(name: &str) -> Vec<[u8; 3]> {
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

fn colormap_name(name: &str) -> &'static str {
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

fn interpolate_stops(t: f64, stops: &[Stop]) -> [u8; 3] {
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

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn turbo(t: f64) -> [u8; 3] {
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

fn weight_func(x: f64, sigma1: f64, sigma2: f64) -> f64 {
    (-(x * x) / (2.0 * sigma1 * sigma1)).exp() / sigma1
        - (-(x * x) / (2.0 * sigma2 * sigma2)).exp() / sigma2
}

fn sigmoid(x: f64) -> f64 {
    if x < -4.0 {
        0.0
    } else if x > 4.0 {
        1.0
    } else {
        1.0 / (1.0 + (-2.0 * x).exp())
    }
}

fn angle_dist(angle1: f64, angle2: f64) -> f64 {
    PI / 2.0 - (PI / 2.0 - (angle1 - angle2).abs().rem_euclid(PI)).abs()
}

fn get_lateral_sigmas(r0: f64) -> (f64, f64) {
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

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
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

    #[test]
    fn lateral_spread_can_flip_even_odd_gap() {
        let base = FrameParams::default();
        assert!((lateral_spread_factor(base, 2) - 1.0).abs() < 1.0e-12);

        let spread = FrameParams {
            lateral_spread_deg: 60.0,
            ..base
        };
        assert!(lateral_spread_factor(spread, 2) < 0.0);
    }

    #[test]
    fn auto_planform_metadata_uses_critical_parity() {
        let state = ServerState::default();
        let params = FrameParams {
            generator: Generator::Planform,
            pattern: PatternPreset::Auto,
            n: 32,
            m: 8,
            frames: 2,
            t: 1.0,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        let planform = payload.planform.as_ref().unwrap();
        assert_eq!(planform.parity, planform.stability.critical_branch);
        assert_eq!(planform.branch_selection.model, "cubic-amplitude-equation");
        assert!(!planform.branch_selection.candidates.is_empty());
        assert!(planform.branch_selection.gamma0.is_finite());
    }

    #[test]
    fn paper_preset_generates_calibration_report() {
        let state = ServerState::default();
        let mut raw = HashMap::new();
        raw.insert("paper_preset".to_string(), "fig31_square_even".to_string());
        raw.insert("n".to_string(), "32".to_string());
        raw.insert("m".to_string(), "4".to_string());
        raw.insert("frames".to_string(), "2".to_string());
        raw.insert("t".to_string(), "1".to_string());
        let payload = generate_payload(coerce_params(&raw), &state).unwrap();
        assert_eq!(payload.paper_preset.unwrap().id, "fig31_square_even");
        let calibration = payload.calibration.as_ref().unwrap();
        assert_eq!(calibration.rendered_pattern, "cobweb");
        assert!(!calibration.checks.is_empty());
    }

    #[test]
    fn paper_preset_registry_roundtrips_lookup_and_parse() {
        let mut ids = Vec::new();
        for entry in PAPER_PRESET_REGISTRY {
            assert_eq!(entry.details.id, entry.preset.as_str());
            assert_eq!(parse_paper_preset(Some(entry.details.id)), entry.preset);
            assert_eq!(
                paper_preset_details(entry.preset).unwrap().id,
                entry.details.id
            );
            ids.push(entry.details.id);
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PAPER_PRESET_REGISTRY.len());
    }

    #[test]
    fn new_contoured_paper_presets_are_cataloged() {
        let ids: Vec<&str> = paper_preset_catalog()
            .into_iter()
            .map(|preset| preset.id)
            .collect();
        for id in [
            "fig34_rhombic_odd",
            "fig35_hex_zero_even",
            "fig36_triangle_odd",
            "fig36_hex_zero_odd",
        ] {
            assert!(ids.contains(&id));
        }
    }

    #[test]
    fn noncontoured_paper_presets_are_cataloged() {
        let ids: Vec<&str> = paper_preset_catalog()
            .into_iter()
            .map(|preset| preset.id)
            .collect();
        for id in [
            "fig29_square_noncontoured",
            "fig29_roll_noncontoured",
            "fig30_rhombic_noncontoured",
            "fig30_hex_noncontoured",
        ] {
            assert!(ids.contains(&id));
        }
    }

    #[test]
    fn completed_bressloff_catalog_contains_roll_subpanels_and_2002_aliases() {
        let ids: Vec<&str> = paper_preset_catalog()
            .into_iter()
            .map(|preset| preset.id)
            .collect();
        assert_eq!(ids.len(), 24);
        for id in [
            "fig31_square_even_roll",
            "fig32_square_odd_roll",
            "fig33_rhombic_even_roll",
            "fig34_rhombic_odd_roll",
            "fig5_roll_cortical",
            "fig5_hex_cortical",
            "fig5_honeycomb_cortical",
            "fig5_square_cortical",
            "fig6_visual_field_planforms",
            "fig7_lattice_tunnel",
        ] {
            assert!(ids.contains(&id));
        }
    }

    #[test]
    fn bressloff_stability_reports_cover_source_targets() {
        let reports = bressloff_stability_reports();
        let ids: Vec<&str> = reports.iter().map(|report| report.id).collect();
        assert_eq!(reports.len(), 5);
        for id in [
            "fig37_even_coefficients",
            "fig38_even_hex_bifurcation",
            "fig39_odd_coefficients",
            "fig40_odd_hex_bifurcation",
            "rhombic_stability_angle",
        ] {
            assert!(ids.contains(&id));
        }
    }

    #[test]
    fn noncontoured_preset_reports_scalar_mode() {
        let state = ServerState::default();
        let mut raw = HashMap::new();
        raw.insert(
            "paper_preset".to_string(),
            "fig29_square_noncontoured".to_string(),
        );
        raw.insert("n".to_string(), "32".to_string());
        raw.insert("m".to_string(), "4".to_string());
        raw.insert("frames".to_string(), "2".to_string());
        raw.insert("t".to_string(), "1".to_string());
        let payload = generate_payload(coerce_params(&raw), &state).unwrap();
        assert_eq!(payload.params.contour_mode, "noncontoured");
        assert_eq!(
            payload.planform.as_ref().unwrap().contour_mode,
            "noncontoured"
        );
        assert_eq!(
            payload.calibration.as_ref().unwrap().rendered_contour_mode,
            "noncontoured"
        );
    }

    #[test]
    fn triangle_pattern_uses_hexagonal_sine_modes() {
        let modes = planform_modes(FrameParams::default(), PatternPreset::Triangle);
        assert_eq!(modes.len(), 3);
        assert!(modes
            .iter()
            .all(|mode| (mode.phase_offset + PI / 2.0).abs() < 1.0e-12));
        assert_eq!(pattern_family(PatternPreset::Triangle), "hexagonal");
    }

    #[test]
    fn orientation_channel_export_has_expected_shape() {
        let state = ServerState::default();
        let params = FrameParams {
            generator: Generator::Planform,
            n: 32,
            m: 4,
            frames: 3,
            t: 1.0,
            export_orientation_channels: true,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        let channels = payload.orientation_channels.as_ref().unwrap();
        assert_eq!(channels.frame_count, payload.frame_count);
        assert_eq!(channels.width, 32);
        assert_eq!(channels.height, 32);
        assert_eq!(channels.orientation_count, 4);
        assert_eq!(channels.phi_radians.len(), 4);
        assert!(!channels.data_base64.is_empty());
    }

    #[test]
    fn rule_preset_registry_is_separate_from_bressloff_papers() {
        let rule_ids: Vec<&str> = rule_preset_catalog()
            .into_iter()
            .map(|preset| preset.id)
            .collect();
        assert_eq!(rule_ids.len(), 4);
        assert!(rule_ids.contains(&"rule_fig4_high_freq_stripes"));
        assert!(rule_ids.contains(&"rule_fig4_low_freq_hexagons"));
        assert!(paper_preset_catalog()
            .into_iter()
            .all(|preset| preset.model_family == MODEL_FAMILY_BRESSLOFF));
        assert!(rule_preset_catalog()
            .into_iter()
            .all(|preset| preset.model_family == MODEL_FAMILY_RULE));
    }

    #[test]
    fn rule_stimulus_uses_period_in_milliseconds() {
        let params = FrameParams {
            generator: Generator::RuleFlicker,
            rule_stim_period_ms: 40.0,
            rule_stim_threshold: 0.8,
            rule_stim_smoothing: 0.0,
            ..FrameParams::default()
        };
        assert_eq!(rule_stimulus(params, 0.0), 0.0);
        assert_eq!(rule_stimulus(params, 10.0), 1.0);
        assert_eq!(rule_stimulus(params, 20.0), 0.0);
        assert_eq!(rule_stimulus(params, 50.0), 1.0);
    }

    #[test]
    fn rule_gaussian_kernel_is_normalized() {
        let kernel = rule_gaussian_kernel(2.0);
        let sum: f64 = kernel.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn rule_uniform_initial_state_stays_spatially_uniform() {
        let state = ServerState::default();
        let params = FrameParams {
            generator: Generator::RuleFlicker,
            n: 32,
            frames: 24,
            t: 80.0,
            rule_seed_strength: 0.0,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        assert_eq!(payload.model_family, MODEL_FAMILY_RULE);
        assert!(payload.metrics.final_range < 1.0e-7);
    }

    #[test]
    fn rule_qualitative_presets_report_expected_regimes() {
        let state = ServerState::default();
        let high = generate_payload(
            FrameParams {
                n: 32,
                frames: 72,
                t: 330.0,
                ..apply_rule_preset(FrameParams::default(), RulePreset::Fig4HighFreqStripes)
            },
            &state,
        )
        .unwrap();
        let high_rule = high.rule.as_ref().unwrap();
        assert_eq!(high_rule.status, "qualitative-pass");
        assert_eq!(high_rule.spatial_family, "stripe");
        assert_eq!(high_rule.response_mode, "period_doubled");

        let low = generate_payload(
            FrameParams {
                n: 40,
                frames: 96,
                t: 660.0,
                ..apply_rule_preset(FrameParams::default(), RulePreset::Fig4LowFreqHexagons)
            },
            &state,
        )
        .unwrap();
        let low_rule = low.rule.as_ref().unwrap();
        assert_eq!(low_rule.status, "qualitative-pass");
        assert_eq!(low_rule.spatial_family, "hexagonal");
        assert_eq!(low_rule.response_mode, "one_to_one");
    }

    #[test]
    fn rule_sweep_point_exports_thumbnail_and_rule_family_metrics() {
        let grid = rule_sweep_grid_defaults("quick");
        let params = rule_sweep_params(&HashMap::new(), &grid, 55.0, 0.8, 0.0);
        let point = rule_sweep_point_for(FrameParams {
            n: 32,
            frames: 48,
            t: 220.0,
            ..params
        });
        assert_eq!(point.period_ms, 55.0);
        assert_eq!(point.amplitude, 0.8);
        assert_eq!(point.seed_pattern, "stripes");
        assert!(point.pattern_strength.is_finite());
        assert_eq!(point.thumbnail.width, 32);
        assert!(!point.thumbnail.data_base64.is_empty());
        assert!(!point.spatial.top_modes.is_empty());
        assert!(point.temporal.confidence.is_finite());
    }

    #[test]
    fn rule_floquet_report_exports_mode_rows() {
        let grid = rule_sweep_grid_defaults("quick");
        let params = rule_sweep_params(&HashMap::new(), &grid, 55.0, 0.8, 0.0);
        let report = rule_floquet_report(
            FrameParams {
                n: 32,
                preview_step: 0.1,
                ..params
            },
            &[3.0, 4.0],
        );
        assert_eq!(report.period_ms, 55.0);
        assert_eq!(report.amplitude, 0.8);
        assert_eq!(report.modes.len(), 2);
        assert!(report.orbit.samples > 0);
        assert!(report.strongest_mode.max_abs_multiplier.is_finite());
        assert!(report.strongest_mode.monodromy_trace.is_finite());
        assert!(report.strongest_mode.plus_condition.is_finite());
    }

    #[test]
    fn rule_floquet_grid_point_exports_boundary_margins() {
        let grid = rule_sweep_grid_defaults("quick");
        let params = rule_sweep_params(&HashMap::new(), &grid, 55.0, 0.8, 0.0);
        let point = rule_floquet_grid_point_for(
            FrameParams {
                n: 32,
                preview_step: 0.1,
                ..params
            },
            &[3.0, 4.0],
        );
        assert_eq!(point.period_ms, 55.0);
        assert_eq!(point.amplitude, 0.8);
        assert_eq!(point.modes.len(), 2);
        assert!(point.plus_margin.is_finite());
        assert!(point.minus_margin.is_finite());
        assert!(point.max_abs_multiplier.is_finite());
    }

    #[test]
    fn rule_floquet_boundary_candidates_detect_sign_changes() {
        let stable = test_floquet_grid_point(40.0, 0.8, 0.98);
        let unstable = test_floquet_grid_point(50.0, 0.8, 1.04);
        let candidates =
            rule_floquet_boundary_candidates(&[stable, unstable], &[40.0, 50.0], &[0.8], &[0.0]);
        let plus = candidates
            .iter()
            .find(|candidate| {
                candidate.kind == "plus_one_to_one" && candidate.evidence == "sign_change"
            })
            .expect("expected +1 boundary candidate");
        assert_eq!(plus.axis, "period");
        assert_eq!(plus.beta_cycles, 4.0);
        assert!(plus.period_ms > 40.0 && plus.period_ms < 50.0);
        assert!(plus.confidence > 0.0);
    }

    #[test]
    fn rule_floquet_boundary_candidates_detect_beta_sign_changes() {
        let point = RuleFloquetGridPoint {
            modes: vec![test_floquet_mode(1.0, 0.98), test_floquet_mode(1.5, 1.04)],
            ..test_floquet_grid_point(60.0, 0.8, 0.9)
        };
        let candidates = rule_floquet_boundary_candidates(&[point], &[60.0], &[0.8], &[0.0]);
        let plus = candidates
            .iter()
            .find(|candidate| {
                candidate.kind == "plus_one_to_one"
                    && candidate.evidence == "sign_change"
                    && candidate.axis == "beta"
            })
            .expect("expected beta-axis +1 boundary candidate");
        assert!(plus.beta_cycles > 1.0 && plus.beta_cycles < 1.5);
        assert_eq!(plus.from_beta_cycles, 1.0);
        assert_eq!(plus.to_beta_cycles, 1.5);
    }

    #[test]
    fn rule_floquet_source_curve_comparison_attaches_best_overlap() {
        let mut curves = vec![RuleFloquetBoundaryCurve {
            curve_id: "test-plus-branch".to_string(),
            kind: "plus_one_to_one",
            branch_label: "+1 one-to-one".to_string(),
            branch_periodicity: "one_to_one",
            axis: "beta",
            source_axis: "forcing_period_ms_vs_wave_number",
            amplitude: 0.8,
            stim_i_fraction: 0.0,
            point_count: 2,
            period_min_ms: 100.0,
            period_max_ms: 110.0,
            beta_min_cycles: 0.5,
            beta_max_cycles: 0.6,
            wave_number_min_radians: 0.1,
            wave_number_max_radians: 0.2,
            mean_residual_abs: 0.0,
            max_residual_abs: 0.0,
            mean_bracket_width_beta_cycles: 0.01,
            max_bracket_width_beta_cycles: 0.01,
            mean_period_gap_ms: 10.0,
            max_period_gap_ms: 10.0,
            continuity_score: 1.0,
            fit: empty_rule_floquet_curve_fit(),
            source_comparison: RuleFloquetBoundarySourceComparison::missing(),
            points: vec![
                test_boundary_curve_point("plus_one_to_one", 100.0, 0.5),
                test_boundary_curve_point("plus_one_to_one", 110.0, 0.6),
            ],
        }];
        let source = RuleFigure8SourceCurves {
            format: "rule-2011-figure8-source-curves-v1".to_string(),
            source_key: "rule-2011".to_string(),
            figure: "Figure 8C".to_string(),
            curves: vec![RuleFigure8SourceCurve {
                curve_id: "source-plus".to_string(),
                kind: "plus_one_to_one".to_string(),
                branch_label: "+1 source branch".to_string(),
                points: vec![
                    RuleFigure8SourcePoint {
                        period_ms: 100.0,
                        wave_number_beta: 0.5,
                    },
                    RuleFigure8SourcePoint {
                        period_ms: 110.0,
                        wave_number_beta: 0.6,
                    },
                ],
            }],
        };

        let summary = apply_rule_figure8_source_comparison(
            &mut curves,
            Some(&source),
            Some(&PathBuf::from("reports/source-curves/test.json")),
        );

        assert_eq!(summary.status, "compared");
        assert_eq!(summary.compared_curve_count, 1);
        assert_eq!(
            curves[0].source_comparison.source_curve_id.as_deref(),
            Some("source-plus")
        );
        assert_eq!(curves[0].source_comparison.overlap_point_count, 2);
        assert!(curves[0].source_comparison.rms_wave_number_error.unwrap() < 1.0e-9);
    }

    #[test]
    fn scalar_sign_change_refinement_finds_root() {
        let (root, residual, iterations) =
            refine_scalar_sign_change(1.0, 3.0, 1.0e-8, 64, |x| x - 2.0)
                .expect("expected scalar sign-change root");
        assert!((root - 2.0).abs() < 1.0e-6);
        assert!(residual.abs() < 1.0e-6);
        assert!(iterations > 0);
    }

    #[test]
    fn bressloff_still_metrics_exports_profiles() {
        let frame = vec![
            0, 64, 128, 255, 255, 128, 64, 0, 0, 64, 128, 255, 255, 128, 64, 0,
        ];
        let metrics = bressloff_still_metrics(&frame, 4, 4);
        assert_eq!(metrics.radial_profile.len(), 16);
        assert_eq!(metrics.angular_profile.len(), 24);
        assert!(metrics.mean_luma > 0.0);
        assert!(metrics.std_luma > 0.0);
        assert!(metrics.edge_density > 0.0);
    }

    #[test]
    fn rule_sweep_dense_grid_has_expected_dimensions() {
        let grid = rule_sweep_grid_defaults("dense");
        assert_eq!(grid.preset, "dense");
        assert_eq!(grid.periods.len(), 13);
        assert_eq!(grid.amplitudes.len(), 5);
        assert_eq!(grid.stim_i_fractions.len(), 1);
        let details = rule_sweep_grid_details(&grid);
        assert_eq!(details.period_steps, 13);
        assert_eq!(details.amplitude_steps, 5);
        assert_eq!(details.n, 32);
    }

    fn test_boundary_curve_point(
        kind: &'static str,
        period_ms: f64,
        beta_cycles: f64,
    ) -> RuleFloquetBoundaryCurvePoint {
        RuleFloquetBoundaryCurvePoint {
            kind,
            branch_label: rule_floquet_branch_label(kind),
            branch_periodicity: rule_floquet_branch_periodicity(kind),
            axis: "beta",
            period_ms,
            stimulus_frequency_hz: 1000.0 / period_ms,
            amplitude: 0.8,
            stim_i_fraction: 0.0,
            beta_cycles,
            wave_number_radians: rule_wave_number_for_cycles(beta_cycles, 32),
            bracket_low_beta_cycles: beta_cycles - 0.01,
            bracket_high_beta_cycles: beta_cycles + 0.01,
            bracket_width_beta_cycles: 0.02,
            margin: 0.0,
            condition_value: 0.0,
            iterations: 1,
            residual_abs: 0.0,
        }
    }

    fn test_floquet_grid_point(
        period_ms: f64,
        amplitude: f64,
        plus_multiplier: f64,
    ) -> RuleFloquetGridPoint {
        let mode = test_floquet_mode(4.0, plus_multiplier);
        RuleFloquetGridPoint {
            period_ms,
            amplitude,
            stim_i_fraction: 0.0,
            dominant_beta_cycles: mode.beta_cycles,
            max_abs_multiplier: mode.max_abs_multiplier,
            crossing_hint: mode.crossing_hint,
            plus_margin: floquet_mode_margin(&mode, "plus_one_to_one"),
            minus_margin: floquet_mode_margin(&mode, "minus_period_doubling"),
            complex_margin: floquet_mode_margin(&mode, "unstable_complex"),
            orbit: RuleOrbitSummary {
                period_ms,
                samples: 1,
                e_min: 0.0,
                e_max: 0.0,
                e_mean: 0.0,
                i_min: 0.0,
                i_max: 0.0,
                i_mean: 0.0,
            },
            modes: vec![mode],
        }
    }

    fn test_floquet_mode(beta_cycles: f64, plus_multiplier: f64) -> RuleFloquetMode {
        floquet_mode_from_matrix(
            beta_cycles,
            rule_wave_number_for_cycles(beta_cycles, 32),
            plus_multiplier,
            0.0,
            0.0,
            0.2,
        )
    }
}
