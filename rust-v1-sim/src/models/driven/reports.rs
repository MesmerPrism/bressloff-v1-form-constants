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
