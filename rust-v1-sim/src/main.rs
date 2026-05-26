#![recursion_limit = "256"]

use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

mod cli;
mod export;
mod models;
mod numeric;
mod params;
mod payload;
mod server;

pub(crate) use export::export_command;
pub(crate) use params::*;
pub(crate) use payload::*;

use models::{
    bressloff::planform::{effective_pattern_from_params, pattern_family},
    bressloff::presets::{parse_paper_preset, PaperPreset},
    driven::{
        bolelli_time_periodic_report, driven_registry_report, mackay_localized_input_report,
        nicks_orthogonal_response_report, BolelliReportConfig, MackayReportConfig,
        NicksReportConfig,
    },
    rule::presets::{parse_rule_preset, RulePreset},
};

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
    acceptance_status: &'static str,
    calibration_claim_allowed: bool,
    score: f64,
    domain_normalized_rms_beta_error_max: f64,
    source_branch_coverage_min: f64,
    generated_curve_coverage_min: f64,
    overlap_point_coverage_min: f64,
    continuity_score_min: f64,
    ordering_score_min: f64,
    underresolved_branch_fraction_max: f64,
    failed_acceptance_checks: Vec<&'static str>,
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
    acceptance_policy: BressloffGeometryAcceptancePolicy,
    calibration_claim_allowed: bool,
    source_profile_dir: String,
    width: usize,
    height: usize,
    still_count: usize,
    compared_still_count: usize,
    threshold_accepted_still_count: usize,
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
    acceptance: BressloffGeometryAcceptanceResult,
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
    generated_dominant_angle_degrees: Option<f64>,
    source_lattice_angle_degrees: Option<f64>,
    radial_profile_error: Option<f64>,
    angular_profile_error: Option<f64>,
    edge_overlap: Option<f64>,
    active_fraction_error: Option<f64>,
    edge_density_error: Option<f64>,
    lattice_angle_error_degrees: Option<f64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct BressloffGeometryAcceptancePolicy {
    policy_id: &'static str,
    claim_level: &'static str,
    calibration_claim_allowed: bool,
    radial_profile_error_max: f64,
    angular_profile_error_max: f64,
    edge_overlap_min: f64,
    active_fraction_error_max: f64,
    edge_density_error_max: f64,
    lattice_angle_error_degrees_max: f64,
    required_compared_still_fraction: f64,
    note: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct BressloffGeometryAcceptanceResult {
    status: &'static str,
    passes_thresholds: bool,
    evaluated_metric_count: usize,
    passed_metric_count: usize,
    failed_metrics: Vec<&'static str>,
    note: &'static str,
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

    use base64::{engine::general_purpose, Engine as _};

    use super::*;
    use crate::models::{
        bressloff::presets::paper_preset_details, rule::presets::apply_rule_preset,
    };

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
        assert_eq!(report.format, "bolelli-time-periodic-input-report-v6");
        assert_eq!(report.model_family, MODEL_FAMILY_LOCALIZED_PERIODIC);
        assert_eq!(report.examples.len(), 1);
        assert_eq!(report.frequency_sweep.len(), 2);
        assert!(report.parameters.contraction_mu_l1.is_finite());
        assert_eq!(
            report.figure5_source_curves.format,
            "bolelli-figure5-source-equation-curves-v1"
        );
        assert_eq!(report.figure5_source_curves.parameter_curves.len(), 3);
        assert!(
            report.figure5_source_curves.max_pole_residual.unwrap()
                < report.figure5_source_curves.pole_residual_tolerance
        );
        assert!(!report.figure5_source_curves.calibration_claim_allowed);
        assert!(!report.figure5_source_curves.calibrated);
        for curve in &report.figure5_source_curves.parameter_curves {
            assert_eq!(curve.point_count, 10);
            assert_eq!(curve.root_resolved_count, curve.point_count);
            assert!(curve
                .max_asymptotic_relative_width_error
                .unwrap()
                .is_finite());
        }
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
            assert!(row.source_target.generated_width_fit_points > 0);
            assert_eq!(row.source_target.generated_width_fit_min_r_squared, 0.70);
            if row.source_target.generated_width_comparable {
                assert!(row
                    .source_target
                    .generated_width_pole_convention
                    .unwrap()
                    .is_finite());
                assert!(row.source_target.generated_width_decay_rate.unwrap() > 0.0);
                assert!(
                    row.source_target.generated_width_fit_r_squared.unwrap()
                        >= row.source_target.generated_width_fit_min_r_squared
                );
                assert!(row.source_target.absolute_width_error.unwrap().is_finite());
                assert!(row.source_target.relative_width_error.unwrap().is_finite());
            } else {
                assert!(row.source_target.absolute_width_error.is_none());
                assert!(row.source_target.relative_width_error.is_none());
            }
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
        assert_eq!(report.format, "nicks-orthogonal-response-report-v7");
        assert_eq!(report.model_family, MODEL_FAMILY_DRIVEN_ORTHOGONAL);
        assert!(!report.figure8_source_curves.curve_points.is_empty());
        assert_eq!(
            report.figure8_source_curves.curve_residual_tolerance_gamma,
            1.0e-8
        );
        assert_eq!(report.figure8_residual_field.rows.len(), 20);
        assert!(!report.figure8_residual_field.calibrated);
        assert_eq!(report.figure8_acceptance_policy.source_grid_rows, 20);
        assert_eq!(report.figure8_acceptance_policy.robust_region_rows, 15);
        assert_eq!(report.figure8_acceptance_policy.boundary_adjacent_rows, 5);
        assert!((report.figure8_acceptance_policy.source_gamma_min_spacing - 0.25).abs() < 1.0e-12);
        assert!(
            (report
                .figure8_acceptance_policy
                .region_margin_threshold_gamma
                - 0.125)
                .abs()
                < 1.0e-12
        );
        assert_eq!(
            report
                .figure8_acceptance_policy
                .curve_residual_tolerance_gamma,
            1.0e-8
        );
        assert!(!report.figure8_acceptance_policy.calibration_claim_allowed);
        assert!(!report.figure8_acceptance_policy.calibrated);
        assert!(report
            .figure8_residual_field
            .rows
            .iter()
            .all(|row| row.source_grid_point && row.residual_abs_gamma.is_finite()));
        assert!(report
            .figure8_residual_field
            .rows
            .iter()
            .any(|row| !row.robust_region_side));
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
