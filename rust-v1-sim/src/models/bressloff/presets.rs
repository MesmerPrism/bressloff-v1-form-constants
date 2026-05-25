use crate::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaperPreset {
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

impl PaperPreset {
    pub(crate) const fn as_str(self) -> &'static str {
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
pub(crate) struct PaperPresetRegistryEntry {
    pub(crate) preset: PaperPreset,
    pub(crate) details: PaperPresetDetails,
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

pub(crate) static PAPER_PRESET_REGISTRY: &[PaperPresetRegistryEntry] = &[
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

pub(crate) fn parse_paper_preset(value: Option<&str>) -> PaperPreset {
    value
        .and_then(|id| {
            PAPER_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == id)
                .map(|entry| entry.preset)
        })
        .unwrap_or(PaperPreset::Manual)
}

pub(crate) fn paper_preset_details(preset: PaperPreset) -> Option<PaperPresetDetails> {
    PAPER_PRESET_REGISTRY
        .iter()
        .find(|entry| entry.preset == preset)
        .map(|entry| entry.details)
}

pub(crate) fn paper_preset_catalog() -> Vec<PaperPresetDetails> {
    PAPER_PRESET_REGISTRY
        .iter()
        .map(|entry| entry.details)
        .collect()
}

pub(crate) fn apply_paper_preset(mut params: FrameParams, preset: PaperPreset) -> FrameParams {
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
