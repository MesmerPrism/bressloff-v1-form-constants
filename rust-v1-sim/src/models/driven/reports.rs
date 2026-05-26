use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct DrivenExampleDetails {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) paper: &'static str,
    pub(crate) figure_or_example: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) mathematical_object: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) symmetry_assumption: &'static str,
    pub(crate) kernel_family: &'static str,
    pub(crate) input_type: &'static str,
    pub(crate) parameter_summary: &'static str,
    pub(crate) method: &'static str,
    pub(crate) report_target: &'static str,
    pub(crate) expected_behavior: &'static str,
    pub(crate) implementation_status: &'static str,
    pub(crate) public_claim_level: &'static str,
    pub(crate) rights_status: &'static str,
    pub(crate) difficulty: &'static str,
    pub(crate) missing_evidence: &'static str,
}

#[derive(Serialize)]
pub(crate) struct DrivenRegistryReport {
    pub(crate) format: &'static str,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
    pub(crate) model_families: Vec<&'static str>,
    pub(crate) implemented_count: usize,
    pub(crate) future_count: usize,
    pub(crate) examples: Vec<DrivenExampleDetails>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MackayReportConfig {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) iterations: usize,
    pub(crate) tolerance: f64,
    pub(crate) mu: f64,
    pub(crate) epsilon: f64,
    pub(crate) kappa: f64,
}

impl Default for MackayReportConfig {
    fn default() -> Self {
        Self {
            n: 64,
            domain_min: -10.0,
            domain_max: 10.0,
            iterations: 60,
            tolerance: 1.0e-7,
            mu: 1.0,
            epsilon: 0.025,
            kappa: 1.0,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct MackayLocalizedInputReport {
    pub(crate) format: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
    pub(crate) rights_status: &'static str,
    pub(crate) solver: MackayReportSolver,
    pub(crate) parameters: MackayReportParameters,
    pub(crate) examples: Vec<MackayGeneratedExample>,
}

#[derive(Serialize)]
pub(crate) struct MackayReportSolver {
    pub(crate) method: &'static str,
    pub(crate) boundary_model: &'static str,
    pub(crate) transfer_function: &'static str,
    pub(crate) claim_level: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct MackayReportParameters {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) dx: f64,
    pub(crate) iterations: usize,
    pub(crate) tolerance: f64,
    pub(crate) mu: f64,
    pub(crate) epsilon: f64,
    pub(crate) kappa: f64,
    pub(crate) sigma1: f64,
    pub(crate) sigma2: f64,
}

#[derive(Serialize)]
pub(crate) struct MackayGeneratedExample {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) registry_id: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) mathematical_object: &'static str,
    pub(crate) domain: String,
    pub(crate) input_formula: &'static str,
    pub(crate) kernel: MackayExampleKernel,
    pub(crate) parameters: MackayExampleParameters,
    pub(crate) fixed_point: MackayFixedPointDiagnostics,
    pub(crate) metrics: MackayFieldMetrics,
    pub(crate) input_thumbnail: MackayFieldThumbnail,
    pub(crate) output_thumbnail: MackayFieldThumbnail,
    pub(crate) status: &'static str,
    pub(crate) expected_behavior: &'static str,
    pub(crate) public_claim_level: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Serialize)]
pub(crate) struct MackayExampleKernel {
    pub(crate) family: &'static str,
    pub(crate) formula: &'static str,
    pub(crate) sigma1: f64,
    pub(crate) sigma2: Option<f64>,
    pub(crate) kappa: Option<f64>,
}

#[derive(Serialize)]
pub(crate) struct MackayExampleParameters {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) dx: f64,
    pub(crate) mu: f64,
    pub(crate) epsilon: f64,
    pub(crate) transfer_function: &'static str,
    pub(crate) boundary_model: &'static str,
}

#[derive(Serialize)]
pub(crate) struct MackayFixedPointDiagnostics {
    pub(crate) iterations: usize,
    pub(crate) residual_linf: f64,
    pub(crate) converged: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct MackayFieldMetrics {
    pub(crate) input_mean: f64,
    pub(crate) input_std: f64,
    pub(crate) input_min: f64,
    pub(crate) input_max: f64,
    pub(crate) output_mean: f64,
    pub(crate) output_std: f64,
    pub(crate) output_min: f64,
    pub(crate) output_max: f64,
    pub(crate) output_active_fraction: f64,
    pub(crate) zero_crossings_along_x_mean: f64,
    pub(crate) zero_crossings_along_y_mean: f64,
    pub(crate) input_output_correlation: f64,
    pub(crate) output_input_delta_std: f64,
    pub(crate) rendered_target_coverage: bool,
    pub(crate) diagnostic_metric_available: bool,
    pub(crate) source_target_comparison: bool,
    pub(crate) calibrated: bool,
}

#[derive(Serialize)]
pub(crate) struct MackayFieldThumbnail {
    pub(crate) format: &'static str,
    pub(crate) encoding: &'static str,
    pub(crate) color_space: &'static str,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) scale_min: f64,
    pub(crate) scale_max: f64,
    pub(crate) data_base64: String,
}

#[derive(Clone, Debug)]
pub(crate) struct BolelliReportConfig {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) samples_per_period: usize,
    pub(crate) warmup_periods: usize,
    pub(crate) periodic_tolerance: f64,
    pub(crate) mu: f64,
    pub(crate) drive_amplitude: f64,
    pub(crate) static_bias: f64,
    pub(crate) spatial_frequency: f64,
    pub(crate) sigma_exc: f64,
    pub(crate) sigma_inh: f64,
    pub(crate) inhibition: f64,
    pub(crate) frequencies: Vec<f64>,
}

impl Default for BolelliReportConfig {
    fn default() -> Self {
        Self {
            n: 192,
            domain_min: -10.0,
            domain_max: 10.0,
            samples_per_period: 96,
            warmup_periods: 10,
            periodic_tolerance: 5.0e-3,
            mu: 0.35,
            drive_amplitude: 0.35,
            static_bias: 0.08,
            spatial_frequency: 1.0,
            sigma_exc: 0.4 / (2.0 * std::f64::consts::PI).sqrt(),
            sigma_inh: 0.8 / (2.0 * std::f64::consts::PI).sqrt(),
            inhibition: 1.0,
            frequencies: vec![2.0, 8.0, 20.0, 60.0, 100.0],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct BolelliTimePeriodicInputReport {
    pub(crate) format: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
    pub(crate) rights_status: &'static str,
    pub(crate) solver: BolelliReportSolver,
    pub(crate) parameters: BolelliReportParameters,
    pub(crate) examples: Vec<BolelliGeneratedExample>,
    pub(crate) frequency_sweep: Vec<BolelliFrequencySweepRow>,
}

#[derive(Serialize)]
pub(crate) struct BolelliReportSolver {
    pub(crate) method: &'static str,
    pub(crate) boundary_model: &'static str,
    pub(crate) transfer_function: &'static str,
    pub(crate) claim_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct BolelliReportParameters {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) dx: f64,
    pub(crate) samples_per_period: usize,
    pub(crate) warmup_periods: usize,
    pub(crate) periodic_tolerance: f64,
    pub(crate) mu: f64,
    pub(crate) drive_amplitude: f64,
    pub(crate) static_bias: f64,
    pub(crate) spatial_frequency: f64,
    pub(crate) sigma_exc: f64,
    pub(crate) sigma_inh: f64,
    pub(crate) inhibition: f64,
    pub(crate) approximate_kernel_l1: f64,
    pub(crate) contraction_mu_l1: f64,
}

#[derive(Serialize)]
pub(crate) struct BolelliGeneratedExample {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) registry_id: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) mathematical_object: &'static str,
    pub(crate) domain: String,
    pub(crate) input_formula: &'static str,
    pub(crate) kernel: BolelliExampleKernel,
    pub(crate) frequency_lambda: f64,
    pub(crate) period: f64,
    pub(crate) metrics: BolelliPeriodMetrics,
    pub(crate) source_target: BolelliPoleWidthComparison,
    pub(crate) input_thumbnail: BolelliProfileThumbnail,
    pub(crate) mean_response_thumbnail: BolelliProfileThumbnail,
    pub(crate) amplitude_thumbnail: BolelliProfileThumbnail,
    pub(crate) status: &'static str,
    pub(crate) expected_behavior: &'static str,
    pub(crate) public_claim_level: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BolelliExampleKernel {
    pub(crate) family: &'static str,
    pub(crate) formula: &'static str,
    pub(crate) sigma_exc: f64,
    pub(crate) sigma_inh: f64,
    pub(crate) inhibition: f64,
    pub(crate) approximate_l1: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BolelliPeriodMetrics {
    pub(crate) time_step: f64,
    pub(crate) total_steps: usize,
    pub(crate) warmup_periods: usize,
    pub(crate) periodic_residual_linf: f64,
    pub(crate) period_correlation: f64,
    pub(crate) response_phase_radians: f64,
    pub(crate) response_amplitude: f64,
    pub(crate) stripe_width_half_max: f64,
    pub(crate) active_fraction_half_max: f64,
    pub(crate) max_abs_response: f64,
    pub(crate) mean_zero_crossings: f64,
    pub(crate) locked: bool,
    pub(crate) rendered_target_coverage: bool,
    pub(crate) diagnostic_metric_available: bool,
    pub(crate) source_target_comparison: bool,
    pub(crate) calibrated: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BolelliPoleWidthComparison {
    pub(crate) source_target_kind: &'static str,
    pub(crate) source_target_reference: &'static str,
    pub(crate) pole_equation: &'static str,
    pub(crate) fourier_convention: &'static str,
    pub(crate) source_parameter_set: &'static str,
    pub(crate) source_parameter_match: bool,
    pub(crate) source_lambda_range: [f64; 2],
    pub(crate) lambda_in_source_range: bool,
    pub(crate) pole_residual_tolerance: f64,
    pub(crate) pole_residual_pass: bool,
    pub(crate) pole_real: Option<f64>,
    pub(crate) pole_imaginary: Option<f64>,
    pub(crate) pole_residual: Option<f64>,
    pub(crate) target_width_principal_pole: Option<f64>,
    pub(crate) asymptotic_width_principal_pole: Option<f64>,
    pub(crate) asymptotic_relative_width_error: Option<f64>,
    pub(crate) generated_width_half_max: f64,
    pub(crate) generated_width_comparison_convention: &'static str,
    pub(crate) generated_width_comparable: bool,
    pub(crate) absolute_width_error: Option<f64>,
    pub(crate) relative_width_error: Option<f64>,
    pub(crate) source_target_comparison: bool,
    pub(crate) calibrated: bool,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Serialize)]
pub(crate) struct BolelliFrequencySweepRow {
    pub(crate) registry_id: &'static str,
    pub(crate) frequency_lambda: f64,
    pub(crate) period: f64,
    pub(crate) metrics: BolelliPeriodMetrics,
    pub(crate) source_target: BolelliPoleWidthComparison,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Serialize)]
pub(crate) struct BolelliProfileThumbnail {
    pub(crate) format: &'static str,
    pub(crate) encoding: &'static str,
    pub(crate) color_space: &'static str,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) scale_min: f64,
    pub(crate) scale_max: f64,
    pub(crate) data_base64: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NicksReportConfig {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) turing_wavenumber: f64,
    pub(crate) epsilon2_delta: f64,
    pub(crate) forcing_strengths: Vec<f64>,
    pub(crate) detuning_fractions: Vec<f64>,
    pub(crate) h: f64,
    pub(crate) sigma: f64,
}

impl Default for NicksReportConfig {
    fn default() -> Self {
        Self {
            n: 96,
            domain_min: 0.0,
            domain_max: 10.0 * std::f64::consts::PI,
            turing_wavenumber: 1.0,
            epsilon2_delta: 0.3,
            forcing_strengths: vec![0.1, 0.4, 0.65, 1.1],
            detuning_fractions: vec![0.0, 0.05, 0.25, 0.75, 1.0],
            h: 0.0,
            sigma: 0.5,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct NicksOrthogonalResponseReport {
    pub(crate) format: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
    pub(crate) rights_status: &'static str,
    pub(crate) solver: NicksReportSolver,
    pub(crate) parameters: NicksReportParameters,
    pub(crate) examples: Vec<NicksGeneratedExample>,
    pub(crate) parameter_sweep: Vec<NicksSweepRow>,
}

#[derive(Serialize)]
pub(crate) struct NicksReportSolver {
    pub(crate) method: &'static str,
    pub(crate) boundary_model: &'static str,
    pub(crate) transfer_function: &'static str,
    pub(crate) claim_level: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NicksReportParameters {
    pub(crate) n: usize,
    pub(crate) domain_min: f64,
    pub(crate) domain_max: f64,
    pub(crate) dx: f64,
    pub(crate) turing_wavenumber: f64,
    pub(crate) epsilon2_delta: f64,
    pub(crate) forcing_strengths: Vec<f64>,
    pub(crate) detuning_fractions: Vec<f64>,
    pub(crate) coefficient_mode: &'static str,
    pub(crate) source_kernel: &'static str,
    pub(crate) h: f64,
    pub(crate) sigma: f64,
}

#[derive(Serialize)]
pub(crate) struct NicksGeneratedExample {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) registry_id: &'static str,
    pub(crate) source_key: &'static str,
    pub(crate) model_family: &'static str,
    pub(crate) mathematical_object: &'static str,
    pub(crate) domain: String,
    pub(crate) input_formula: &'static str,
    pub(crate) amplitude_equation: &'static str,
    pub(crate) wavevectors: NicksWavevectorDiagnostics,
    pub(crate) amplitude_solution: NicksAmplitudeSolution,
    pub(crate) metrics: NicksOrthogonalMetrics,
    pub(crate) source_target: NicksSourceTargetComparison,
    pub(crate) forcing_thumbnail: NicksFieldThumbnail,
    pub(crate) cortical_response_thumbnail: NicksFieldThumbnail,
    pub(crate) retinal_response_thumbnail: NicksFieldThumbnail,
    pub(crate) status: &'static str,
    pub(crate) expected_behavior: &'static str,
    pub(crate) public_claim_level: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksWavevectorDiagnostics {
    pub(crate) forcing_kx: f64,
    pub(crate) forcing_ky: f64,
    pub(crate) response_kx: f64,
    pub(crate) response_ky: f64,
    pub(crate) response_magnitude: f64,
    pub(crate) detuning_v2: f64,
    pub(crate) detuning_fraction: f64,
    pub(crate) forcing_to_response_x_ratio: f64,
    pub(crate) forcing_to_turing_ratio: f64,
    pub(crate) response_angle_degrees: f64,
    pub(crate) orthogonality_error_degrees: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksAmplitudeSolution {
    pub(crate) rho_a: f64,
    pub(crate) rho_b: f64,
    pub(crate) phase_a: f64,
    pub(crate) phase_b: f64,
    pub(crate) forcing_strength_gamma: f64,
    pub(crate) growth_term: f64,
    pub(crate) coupling_term: f64,
    pub(crate) residual_linf: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksAmplitudeCoefficientTable {
    pub(crate) source_target_reference: &'static str,
    pub(crate) source_branch: &'static str,
    pub(crate) source_parameter_set: &'static str,
    pub(crate) coefficient_normalization: &'static str,
    pub(crate) kernel_coefficient_status: &'static str,
    pub(crate) beta_c: f64,
    pub(crate) source_turing_wavenumber: f64,
    pub(crate) source_response_kx: f64,
    pub(crate) source_response_ky: f64,
    pub(crate) sigmoid_mu: f64,
    pub(crate) sigmoid_f0: f64,
    pub(crate) beta2: f64,
    pub(crate) beta3: f64,
    pub(crate) zeta1: f64,
    pub(crate) zeta4: f64,
    pub(crate) zeta6: f64,
    pub(crate) phi1: f64,
    pub(crate) phi4: f64,
    pub(crate) epsilon2_delta: f64,
    pub(crate) gamma: f64,
    pub(crate) gamma_c: f64,
    pub(crate) gamma_p: f64,
    pub(crate) boundary_distance_gamma: f64,
    pub(crate) symmetric_rectangle_rho: f64,
    pub(crate) rectangle_existence_margin: f64,
    pub(crate) rectangle_stability_margin: f64,
    pub(crate) rectangle_branch_available: bool,
    pub(crate) oblique_branch_available: bool,
    pub(crate) source_formula: &'static str,
    pub(crate) calibrated: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksFigure8RegionComparison {
    pub(crate) source_target_kind: &'static str,
    pub(crate) source_target_reference: &'static str,
    pub(crate) source_parameter_set: &'static str,
    pub(crate) source_sigma: f64,
    pub(crate) source_h: f64,
    pub(crate) source_epsilon2_delta: f64,
    pub(crate) source_gamma_values: [f64; 4],
    pub(crate) source_detuning_fractions: [f64; 5],
    pub(crate) source_parameter_match: bool,
    pub(crate) nearest_source_gamma: f64,
    pub(crate) gamma_grid_error: f64,
    pub(crate) gamma_on_source_grid: bool,
    pub(crate) nearest_source_detuning_fraction: f64,
    pub(crate) detuning_grid_error: f64,
    pub(crate) detuning_on_source_grid: bool,
    pub(crate) rectangle_oblique_boundary_gamma: f64,
    pub(crate) boundary_distance_gamma: f64,
    pub(crate) boundary_side: &'static str,
    pub(crate) target_region_label: &'static str,
    pub(crate) generated_region_label: &'static str,
    pub(crate) region_label_matches: bool,
    pub(crate) boundary_comparison_available: bool,
    pub(crate) calibrated: bool,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksOrthogonalMetrics {
    pub(crate) classification: &'static str,
    pub(crate) amplitude_norm: f64,
    pub(crate) forcing_response_correlation: f64,
    pub(crate) response_zero_crossings_x_mean: f64,
    pub(crate) response_zero_crossings_y_mean: f64,
    pub(crate) gradient_energy_x: f64,
    pub(crate) gradient_energy_y: f64,
    pub(crate) orthogonal_energy_fraction: f64,
    pub(crate) rendered_target_coverage: bool,
    pub(crate) diagnostic_metric_available: bool,
    pub(crate) source_target_comparison: bool,
    pub(crate) calibrated: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct NicksSourceTargetComparison {
    pub(crate) source_target_kind: &'static str,
    pub(crate) source_target_reference: &'static str,
    pub(crate) target_relationship: &'static str,
    pub(crate) amplitude_coefficients: NicksAmplitudeCoefficientTable,
    pub(crate) figure8_region: NicksFigure8RegionComparison,
    pub(crate) target_classification: &'static str,
    pub(crate) generated_classification: &'static str,
    pub(crate) classification_matches: bool,
    pub(crate) target_forcing_to_response_x_ratio: Option<f64>,
    pub(crate) generated_forcing_to_response_x_ratio: Option<f64>,
    pub(crate) ratio_error: Option<f64>,
    pub(crate) target_response_angle_degrees: f64,
    pub(crate) generated_response_angle_degrees: f64,
    pub(crate) angle_error_degrees: f64,
    pub(crate) source_target_comparison: bool,
    pub(crate) calibrated: bool,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Serialize)]
pub(crate) struct NicksSweepRow {
    pub(crate) registry_id: &'static str,
    pub(crate) forcing_strength_gamma: f64,
    pub(crate) detuning_fraction: f64,
    pub(crate) wavevectors: NicksWavevectorDiagnostics,
    pub(crate) amplitude_solution: NicksAmplitudeSolution,
    pub(crate) metrics: NicksOrthogonalMetrics,
    pub(crate) source_target: NicksSourceTargetComparison,
    pub(crate) status: &'static str,
    pub(crate) note: &'static str,
}

#[derive(Serialize)]
pub(crate) struct NicksFieldThumbnail {
    pub(crate) format: &'static str,
    pub(crate) encoding: &'static str,
    pub(crate) color_space: &'static str,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) scale_min: f64,
    pub(crate) scale_max: f64,
    pub(crate) data_base64: String,
}
