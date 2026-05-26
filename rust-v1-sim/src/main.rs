#![recursion_limit = "256"]

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

mod cli;
mod models;
mod numeric;
mod server;

use models::{
    bressloff::planform::{
        cell_mm_for, effective_pattern_from_params, generate_planform_frames,
        orientation_count_for, pattern_family, planform_details,
    },
    bressloff::presets::{
        apply_paper_preset, paper_preset_details, parse_paper_preset, PaperPreset,
    },
    driven::{
        bolelli_time_periodic_report, driven_registry_report, mackay_localized_input_report,
        nicks_orthogonal_response_report, BolelliReportConfig, MackayReportConfig,
        NicksReportConfig,
    },
    rule::presets::{apply_rule_preset, parse_rule_preset, rule_preset_details, RulePreset},
    rule::reports::rule_details,
    rule::simulate_rule_flicker_frames,
};

const PI: f64 = std::f64::consts::PI;
const DYNAMIC_CELL_MM: f64 = 0.7;
const RETINO_EPS: f64 = 0.051;
const RETINO_W0: f64 = 0.087;
const RETINO_ALPHA: f64 = 3.0 / PI;
const RETINO_BETA: f64 = 1.589 / 2.0;
const MODEL_FAMILY_BRESSLOFF: &str = "bressloff_orientation_hypercolumn";
const MODEL_FAMILY_RULE: &str = "rule_flicker_ei";
const MODEL_FAMILY_DRIVEN_ORTHOGONAL: &str = "spatial_forcing_orthogonal_response";
const MODEL_FAMILY_MACKAY: &str = "mackay_localized_input";
const MODEL_FAMILY_LOCALIZED_PERIODIC: &str = "localized_time_periodic_input";
const RULE_FIGURE8_SOURCE_BETA_PER_MODEL_CYCLE: f64 = 0.42868451880191133;

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

struct StabilityReportSpec {
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
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    curve_refinement: RuleFloquetCurveRefinement,
    source_curve_comparison: RuleFloquetSourceCurveComparisonSummary,
    grid: RuleSweepGridDetails,
    mode_cycles: Vec<f64>,
    mode_source_betas: Vec<f64>,
    points: Vec<RuleFloquetGridPoint>,
    boundary_candidates: Vec<RuleFloquetBoundaryCandidate>,
    boundary_curves: Vec<RuleFloquetBoundaryCurve>,
}

struct RuleFloquetEvaluation {
    points: Vec<RuleFloquetGridPoint>,
    boundary_candidates: Vec<RuleFloquetBoundaryCandidate>,
    boundary_curves: Vec<RuleFloquetBoundaryCurve>,
    source_curve_comparison: RuleFloquetSourceCurveComparisonSummary,
}

struct RuleFloquetEvaluationConfig<'a> {
    raw: &'a HashMap<String, String>,
    grid: &'a RuleSweepGridConfig,
    mode_cycles: &'a [f64],
    curve_refinement: RuleFloquetCurveRefinement,
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    source_curves: Option<&'a RuleFigure8SourceCurves>,
    source_curve_file: Option<&'a PathBuf>,
}

#[derive(Serialize)]
struct RuleFigure8FitSearchReport {
    format: &'static str,
    model_family: &'static str,
    source_key: &'static str,
    status: &'static str,
    note: &'static str,
    baseline_parameter_set: &'static str,
    source_curve_file: Option<String>,
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    grid: RuleSweepGridDetails,
    mode_cycles: Vec<f64>,
    mode_source_betas: Vec<f64>,
    curve_refinement: RuleFloquetCurveRefinement,
    trial_count: usize,
    best_trial_id: Option<String>,
    best_score: Option<f64>,
    trials: Vec<RuleFigure8FitTrial>,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFigure8FitTrial {
    trial_id: String,
    status: &'static str,
    parameter_changes: BTreeMap<String, String>,
    parameter_values: BTreeMap<String, String>,
    floquet_point_count: usize,
    boundary_candidate_count: usize,
    boundary_curve_count: usize,
    boundary_curve_point_count: usize,
    source_curve_comparison: RuleFloquetSourceCurveComparisonSummary,
}

struct RuleFigure8FitTrialConfig<'a> {
    spec: &'a RuleFigure8FitTrialSpec,
    baseline_raw: &'a HashMap<String, String>,
    grid: &'a RuleSweepGridConfig,
    mode_cycles: &'a [f64],
    curve_refinement: RuleFloquetCurveRefinement,
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    source_curves: Option<&'a RuleFigure8SourceCurves>,
    source_curve_file: Option<&'a PathBuf>,
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
struct RuleFigure8WaveNumberNormalization {
    model: &'static str,
    decision: &'static str,
    internal_wave_number_formula: &'static str,
    source_beta_per_model_cycle: f64,
    model_cycles_per_source_beta: f64,
    source_beta_offset: f64,
    model_domain_points: usize,
    note: &'static str,
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
    domain_beta_mapping: Option<RuleFloquetBetaAxisMapping>,
    raw_beta_mapping: Option<RuleFloquetBetaAxisMapping>,
    scale_only_beta_mapping: Option<RuleFloquetBetaAxisMapping>,
    affine_beta_mapping: Option<RuleFloquetBetaAxisMapping>,
    fit_objective: Option<RuleFigure8FitObjective>,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFloquetBetaAxisMapping {
    model: &'static str,
    generated_axis: &'static str,
    source_axis: &'static str,
    scale: f64,
    offset: f64,
    sample_count: usize,
    mean_abs_error: f64,
    rms_error: f64,
    max_abs_error: f64,
}

#[derive(Clone, Debug, Serialize)]
struct RuleFigure8FitObjective {
    status: &'static str,
    score: f64,
    domain_normalized_rms_beta_error: f64,
    raw_rms_beta_error: f64,
    affine_rms_beta_error: f64,
    scale_only_rms_beta_error: f64,
    source_branch_coverage: f64,
    generated_curve_coverage: f64,
    overlap_point_coverage: f64,
    continuity_score: f64,
    ordering_score: f64,
    underresolved_branch_fraction: f64,
    compared_curve_count: usize,
    source_curve_count: usize,
    matched_source_curve_count: usize,
    overlap_point_count: usize,
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

#[derive(Clone, Debug)]
struct RuleFigure8ComparisonSample {
    generated_beta_cycles: f64,
    source_beta: f64,
}

#[derive(Clone, Debug)]
struct RuleFigure8FitTrialSpec {
    trial_id: String,
    parameter_changes: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug)]
struct RuleFigure8FitParameterScan {
    key: &'static str,
    factors: [f64; 2],
    min: f64,
    max: f64,
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

struct BranchCandidateSpec {
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
    cli::run(&args)
}

fn driven_registry_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/driven-neural-fields-registry.json");
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--out" {
            out = PathBuf::from(iter.next().ok_or("--out requires a value")?);
        }
    }

    let report = driven_registry_report();
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let example_count = report.examples.len();
    let implemented_count = report.implemented_count;
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} driven_examples={} implemented={}",
        out.display(),
        example_count,
        implemented_count
    );
    Ok(())
}

fn mackay_report_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/mackay-localized-input.json");
    let mut config = MackayReportConfig::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--n" => {
                config.n =
                    parse_clamped_usize(iter.next().ok_or("--n requires a value")?, 16, 256)?;
            }
            "--iterations" => {
                config.iterations = parse_clamped_usize(
                    iter.next().ok_or("--iterations requires a value")?,
                    1,
                    1000,
                )?;
            }
            "--tolerance" => {
                config.tolerance = parse_clamped_f64(
                    iter.next().ok_or("--tolerance requires a value")?,
                    1.0e-12,
                    1.0e-2,
                )?;
            }
            "--mu" => {
                config.mu =
                    parse_clamped_f64(iter.next().ok_or("--mu requires a value")?, 0.0, 4.0)?;
            }
            "--epsilon" => {
                config.epsilon =
                    parse_clamped_f64(iter.next().ok_or("--epsilon requires a value")?, 0.0, 1.0)?;
            }
            "--kappa" => {
                config.kappa =
                    parse_clamped_f64(iter.next().ok_or("--kappa requires a value")?, 0.0, 4.0)?;
            }
            "--domain-min" => {
                config.domain_min = iter
                    .next()
                    .ok_or("--domain-min requires a value")?
                    .parse::<f64>()?;
            }
            "--domain-max" => {
                config.domain_max = iter
                    .next()
                    .ok_or("--domain-max requires a value")?
                    .parse::<f64>()?;
            }
            _ => {}
        }
    }

    let report = mackay_localized_input_report(config)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let example_count = report.examples.len();
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} mackay_examples={} grid={}x{} iterations={}",
        out.display(),
        example_count,
        report.parameters.n,
        report.parameters.n,
        report.parameters.iterations
    );
    Ok(())
}

fn bolelli_report_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/bolelli-time-periodic-input.json");
    let mut config = BolelliReportConfig::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--n" => {
                config.n =
                    parse_clamped_usize(iter.next().ok_or("--n requires a value")?, 16, 512)?;
            }
            "--samples-per-period" => {
                config.samples_per_period = parse_clamped_usize(
                    iter.next().ok_or("--samples-per-period requires a value")?,
                    16,
                    1024,
                )?;
            }
            "--warmup-periods" => {
                config.warmup_periods = parse_clamped_usize(
                    iter.next().ok_or("--warmup-periods requires a value")?,
                    0,
                    200,
                )?;
            }
            "--periodic-tolerance" => {
                config.periodic_tolerance = parse_clamped_f64(
                    iter.next().ok_or("--periodic-tolerance requires a value")?,
                    1.0e-12,
                    1.0,
                )?;
            }
            "--mu" => {
                config.mu =
                    parse_clamped_f64(iter.next().ok_or("--mu requires a value")?, 0.0, 4.0)?;
            }
            "--drive-amplitude" => {
                config.drive_amplitude = parse_clamped_f64(
                    iter.next().ok_or("--drive-amplitude requires a value")?,
                    0.0,
                    4.0,
                )?;
            }
            "--static-bias" => {
                config.static_bias = parse_clamped_f64(
                    iter.next().ok_or("--static-bias requires a value")?,
                    0.0,
                    4.0,
                )?;
            }
            "--spatial-frequency" => {
                config.spatial_frequency = parse_clamped_f64(
                    iter.next().ok_or("--spatial-frequency requires a value")?,
                    0.0,
                    32.0,
                )?;
            }
            "--sigma-exc" => {
                config.sigma_exc = parse_clamped_f64(
                    iter.next().ok_or("--sigma-exc requires a value")?,
                    1.0e-6,
                    10.0,
                )?;
            }
            "--sigma-inh" => {
                config.sigma_inh = parse_clamped_f64(
                    iter.next().ok_or("--sigma-inh requires a value")?,
                    1.0e-6,
                    10.0,
                )?;
            }
            "--inhibition" => {
                config.inhibition = parse_clamped_f64(
                    iter.next().ok_or("--inhibition requires a value")?,
                    0.0,
                    4.0,
                )?;
            }
            "--frequencies" | "--lambdas" => {
                config.frequencies = parse_f64_csv(
                    iter.next().ok_or("--frequencies requires a value")?,
                    1.0e-6,
                    1000.0,
                )?;
            }
            "--domain-min" => {
                config.domain_min = iter
                    .next()
                    .ok_or("--domain-min requires a value")?
                    .parse::<f64>()?;
            }
            "--domain-max" => {
                config.domain_max = iter
                    .next()
                    .ok_or("--domain-max requires a value")?
                    .parse::<f64>()?;
            }
            _ => {}
        }
    }

    let report = bolelli_time_periodic_report(config)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let example_count = report.examples.len();
    let sweep_count = report.frequency_sweep.len();
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} bolelli_examples={} frequency_rows={} grid={} samples_per_period={}",
        out.display(),
        example_count,
        sweep_count,
        report.parameters.n,
        report.parameters.samples_per_period
    );
    Ok(())
}

fn nicks_report_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/nicks-orthogonal-response.json");
    let mut config = NicksReportConfig::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--n" => {
                config.n =
                    parse_clamped_usize(iter.next().ok_or("--n requires a value")?, 16, 256)?;
            }
            "--turing-wavenumber" | "--k0" => {
                config.turing_wavenumber = parse_clamped_f64(
                    iter.next().ok_or("--turing-wavenumber requires a value")?,
                    1.0e-6,
                    32.0,
                )?;
            }
            "--epsilon2-delta" => {
                config.epsilon2_delta = parse_clamped_f64(
                    iter.next().ok_or("--epsilon2-delta requires a value")?,
                    0.0,
                    32.0,
                )?;
            }
            "--forcing-strengths" | "--gammas" => {
                config.forcing_strengths = parse_f64_csv(
                    iter.next().ok_or("--forcing-strengths requires a value")?,
                    0.0,
                    32.0,
                )?;
            }
            "--detuning-fractions" | "--detunings" => {
                config.detuning_fractions = parse_f64_csv(
                    iter.next().ok_or("--detuning-fractions requires a value")?,
                    0.0,
                    1.0,
                )?;
            }
            "--self-interaction" => {
                return Err(
                    "--self-interaction was removed; Nicks Phi coefficients are source-derived"
                        .into(),
                );
            }
            "--cross-interaction" => {
                return Err(
                    "--cross-interaction was removed; Nicks Phi coefficients are source-derived"
                        .into(),
                );
            }
            "--h" | "--threshold" => {
                config.h = iter.next().ok_or("--h requires a value")?.parse::<f64>()?;
            }
            "--sigma" => {
                config.sigma = parse_clamped_f64(
                    iter.next().ok_or("--sigma requires a value")?,
                    1.0e-6,
                    32.0,
                )?;
            }
            "--domain-min" => {
                config.domain_min = iter
                    .next()
                    .ok_or("--domain-min requires a value")?
                    .parse::<f64>()?;
            }
            "--domain-max" => {
                config.domain_max = iter
                    .next()
                    .ok_or("--domain-max requires a value")?
                    .parse::<f64>()?;
            }
            _ => {}
        }
    }

    let report = nicks_orthogonal_response_report(config)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let example_count = report.examples.len();
    let sweep_count = report.parameter_sweep.len();
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "wrote {} nicks_examples={} sweep_rows={} grid={}",
        out.display(),
        example_count,
        sweep_count,
        report.parameters.n
    );
    Ok(())
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

fn raw_usize(raw: &HashMap<String, String>, key: &str) -> Option<usize> {
    raw.get(key)?.parse::<usize>().ok()
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
    use crate::models::{
        bressloff::{
            planform::{lateral_spread_factor, pattern_family, planform_modes},
            presets::{paper_preset_catalog, PAPER_PRESET_REGISTRY},
            reports::{bressloff_stability_reports, bressloff_still_metrics},
        },
        rule::{
            floquet::{
                apply_rule_figure8_source_comparison, empty_rule_floquet_curve_fit,
                floquet_mode_from_matrix, floquet_mode_margin, refine_scalar_sign_change,
                rule_figure8_wave_number_normalization, rule_floquet_boundary_candidates,
                rule_floquet_branch_label, rule_floquet_branch_periodicity,
                rule_floquet_grid_point_for, rule_floquet_report, rule_wave_number_for_cycles,
            },
            presets::rule_preset_catalog,
            rule_gaussian_kernel, rule_stimulus,
            sweep::{
                rule_sweep_grid_defaults, rule_sweep_grid_details, rule_sweep_params,
                rule_sweep_point_for,
            },
        },
    };

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
    fn driven_registry_exports_high_priority_sources() {
        let catalog = crate::models::driven::driven_example_catalog();
        let ids: Vec<&str> = catalog.iter().map(|example| example.id).collect();
        assert!(ids.contains(&"mackay_rays_linear_stationary"));
        assert!(ids.contains(&"mackay_target_linear_stationary"));
        assert!(ids.contains(&"nicks_1d_resonance_tongues"));
        assert!(ids.contains(&"bolelli_periodic_attractor"));
        assert!(catalog
            .iter()
            .all(|example| example.rights_status.contains("private")
                || example.rights_status.contains("generated")));
        assert!(catalog
            .iter()
            .all(|example| example.model_family != MODEL_FAMILY_BRESSLOFF
                && example.model_family != MODEL_FAMILY_RULE));
        assert!(
            catalog
                .iter()
                .filter(|example| example.implementation_status == "implemented")
                .count()
                >= 3
        );
    }

    #[test]
    fn mackay_report_exports_generated_examples() {
        let report = mackay_localized_input_report(MackayReportConfig {
            n: 24,
            iterations: 4,
            ..MackayReportConfig::default()
        })
        .unwrap();
        assert_eq!(report.format, "mackay-localized-input-report-v2");
        assert_eq!(report.model_family, MODEL_FAMILY_MACKAY);
        assert_eq!(report.examples.len(), 3);
        for example in &report.examples {
            assert_eq!(example.model_family, MODEL_FAMILY_MACKAY);
            assert_eq!(example.status, "generated");
            assert!(example.fixed_point.residual_linf.is_finite());
            assert!(example.metrics.output_std.is_finite());
            assert!(example.metrics.rendered_target_coverage);
            assert!(example.metrics.diagnostic_metric_available);
            assert!(!example.metrics.source_target_comparison);
            assert!(!example.metrics.calibrated);
            assert_eq!(example.input_thumbnail.width, 24);
            let bytes = general_purpose::STANDARD
                .decode(&example.output_thumbnail.data_base64)
                .unwrap();
            assert_eq!(bytes.len(), 24 * 24);
        }
    }

    #[test]
    fn bolelli_report_exports_periodic_diagnostics() {
        let report = crate::models::driven::bolelli_time_periodic_report(BolelliReportConfig {
            n: 48,
            samples_per_period: 24,
            warmup_periods: 3,
            frequencies: vec![2.0, 20.0],
            ..BolelliReportConfig::default()
        })
        .unwrap();
        assert_eq!(report.format, "bolelli-time-periodic-input-report-v4");
        assert_eq!(report.model_family, MODEL_FAMILY_LOCALIZED_PERIODIC);
        assert_eq!(report.examples.len(), 1);
        assert_eq!(report.frequency_sweep.len(), 2);
        assert!(report.parameters.contraction_mu_l1.is_finite());
        for row in &report.frequency_sweep {
            assert!(row.metrics.periodic_residual_linf.is_finite());
            assert!(row.metrics.period_correlation.is_finite());
            assert!(row.metrics.stripe_width_half_max >= 0.0);
            assert!(row.metrics.rendered_target_coverage);
            assert!(row.metrics.diagnostic_metric_available);
            assert!(row.source_target.source_target_comparison);
            assert!(!row.source_target.calibrated);
            assert!(row.source_target.source_parameter_match);
            assert!(row.source_target.lambda_in_source_range);
            assert!(row.source_target.pole_residual_pass);
            assert!(row.source_target.source_width_convention_accepted);
            assert!(row.source_target.accepted_width_residual.unwrap() < 1.0e-8);
            assert!(!row.source_target.calibration_claim_allowed);
            assert!(!row.source_target.generated_width_comparable);
            assert!(row.source_target.absolute_width_error.is_none());
            assert!(row.source_target.pole_real.unwrap() > 0.0);
            assert!(row.source_target.pole_imaginary.unwrap() >= 0.0);
            assert!(row
                .source_target
                .target_width_principal_pole
                .unwrap()
                .is_finite());
            assert!(row
                .source_target
                .asymptotic_width_principal_pole
                .unwrap()
                .is_finite());
        }
        let example = &report.examples[0];
        assert_eq!(example.registry_id, "bolelli_heaviside_flicker_stripes");
        assert!(example.source_target.source_target_comparison);
        let bytes = general_purpose::STANDARD
            .decode(&example.amplitude_thumbnail.data_base64)
            .unwrap();
        assert_eq!(bytes.len(), 48);
    }

    #[test]
    fn nicks_report_exports_orthogonal_response_diagnostics() {
        let report = crate::models::driven::nicks_orthogonal_response_report(NicksReportConfig {
            n: 32,
            forcing_strengths: vec![0.1, 0.65],
            detuning_fractions: vec![0.0, 0.25, 1.0],
            ..NicksReportConfig::default()
        })
        .unwrap();
        assert_eq!(report.format, "nicks-orthogonal-response-report-v5");
        assert_eq!(report.model_family, MODEL_FAMILY_DRIVEN_ORTHOGONAL);
        assert!(!report.figure8_source_curves.curve_points.is_empty());
        assert_eq!(
            report.figure8_source_curves.curve_residual_tolerance_gamma,
            1.0e-8
        );
        assert_eq!(report.examples.len(), 3);
        assert_eq!(report.parameter_sweep.len(), 6);
        assert!(report
            .parameter_sweep
            .iter()
            .any(|row| row.metrics.classification.contains("orthogonal")));
        for row in &report.parameter_sweep {
            assert!(row.wavevectors.orthogonality_error_degrees.is_finite());
            assert!(row.amplitude_solution.residual_linf < 1.0e-9);
            assert!(row.metrics.diagnostic_metric_available);
            assert!(row.metrics.source_target_comparison);
            assert!(row.source_target.source_target_comparison);
            assert!(row.source_target.classification_matches);
            assert!(row.source_target.angle_error_degrees < 1.0e-9);
            assert!(row.source_target.amplitude_coefficients.beta_c.is_finite());
            assert!(row.source_target.amplitude_coefficients.beta_c > 0.0);
            assert!(
                row.source_target
                    .amplitude_coefficients
                    .source_turing_wavenumber
                    > 0.0
            );
            assert!(row.source_target.amplitude_coefficients.phi1 > 0.0);
            assert!(row.source_target.amplitude_coefficients.phi4 > 0.0);
            assert!(row.source_target.amplitude_coefficients.gamma_p.is_finite());
            assert!(!row.source_target.amplitude_coefficients.calibrated);
            assert!(row.source_target.figure8_region.source_parameter_match);
            assert!(row.source_target.figure8_region.gamma_on_source_grid);
            assert!(row.source_target.figure8_region.detuning_on_source_grid);
            assert!(row.source_target.figure8_region.region_label_matches);
            assert!(
                row.source_target
                    .figure8_region
                    .boundary_comparison_available
            );
            assert!(row.source_target.figure8_region.curve_residual_pass);
            assert!(row.source_target.figure8_region.curve_residual_abs_gamma <= 1.0e-8);
            assert!(
                row.source_target
                    .figure8_region
                    .region_margin_threshold_gamma
                    > 0.0
            );
            assert!(!row.source_target.figure8_region.calibrated);
            assert!(!row.metrics.calibrated);
        }
        let example = &report.examples[2];
        assert_eq!(example.registry_id, "nicks_billock_tsou_generated_map");
        assert!(example.source_target.source_target_comparison);
        let bytes = general_purpose::STANDARD
            .decode(&example.retinal_response_thumbnail.data_base64)
            .unwrap();
        assert_eq!(bytes.len(), 32 * 32);
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
            rule_figure8_wave_number_normalization(1.0, 32),
            Some(&source),
            Some(&PathBuf::from("reports/source-curves/test.json")),
        );

        assert_eq!(summary.status, "compared");
        assert_eq!(summary.compared_curve_count, 1);
        assert!(summary.fit_objective.as_ref().unwrap().score.is_finite());
        assert!(summary.affine_beta_mapping.as_ref().unwrap().rms_error < 1.0e-9);
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
