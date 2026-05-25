use std::error::Error;

use base64::{engine::general_purpose, Engine as _};

use super::reports::{
    MackayExampleKernel, MackayExampleParameters, MackayFieldMetrics, MackayFieldThumbnail,
    MackayFixedPointDiagnostics, MackayGeneratedExample, MackayLocalizedInputReport,
    MackayReportConfig, MackayReportParameters, MackayReportSolver,
};
use crate::{
    numeric::{convolution, metrics},
    MODEL_FAMILY_MACKAY, PI,
};

#[derive(Clone, Copy, Debug)]
enum MackayInputKind {
    Rays,
    Target,
}

#[derive(Clone, Copy, Debug)]
enum MackayKernelKind {
    DifferenceOfGaussians,
    GaussianOnly,
}

#[derive(Clone, Copy, Debug)]
struct MackayExampleSpec {
    id: &'static str,
    label: &'static str,
    input_kind: MackayInputKind,
    kernel_kind: MackayKernelKind,
    expected_behavior: &'static str,
    public_claim_level: &'static str,
    note: &'static str,
}

pub(crate) fn mackay_localized_input_report(
    config: MackayReportConfig,
) -> Result<MackayLocalizedInputReport, Box<dyn Error>> {
    if config.n < 2 {
        return Err("mackay report grid must have at least two samples".into());
    }
    if config.domain_max <= config.domain_min {
        return Err("mackay report domain-max must be larger than domain-min".into());
    }
    let parameters = mackay_report_parameters(config);
    let examples = mackay_example_specs()
        .into_iter()
        .map(|spec| mackay_generated_example(spec, config, parameters))
        .collect();
    Ok(MackayLocalizedInputReport {
        format: "mackay-localized-input-report-v1",
        model_family: MODEL_FAMILY_MACKAY,
        source_key: "tamekue-prandi-chitour-2024",
        status: "generated-first-pass-diagnostic",
        note: "Generated numeric diagnostics for localized-input MacKay-style neural fields. This is not a reproduction claim; source figures and extraction artifacts remain private.",
        rights_status: "generated outputs only; no copied paper figures or full text",
        solver: MackayReportSolver {
            method: "fixed-point iteration of stationary neural-field equation",
            boundary_model: "finite square grid with zero-padded convolution outside the sampled domain",
            transfer_function: "linear f(a)=a",
            claim_level: "first-pass diagnostic",
        },
        parameters,
        examples,
    })
}

fn mackay_report_parameters(config: MackayReportConfig) -> MackayReportParameters {
    MackayReportParameters {
        n: config.n,
        domain_min: config.domain_min,
        domain_max: config.domain_max,
        dx: mackay_dx(config),
        iterations: config.iterations,
        tolerance: config.tolerance,
        mu: config.mu,
        epsilon: config.epsilon,
        kappa: config.kappa,
        sigma1: 1.0 / (2.0_f64.sqrt() * PI),
        sigma2: 1.0 / PI,
    }
}

fn mackay_example_specs() -> Vec<MackayExampleSpec> {
    vec![
        MackayExampleSpec {
            id: "mackay_rays_linear_stationary",
            label: "MacKay rays from localized half-space input",
            input_kind: MackayInputKind::Rays,
            kernel_kind: MackayKernelKind::DifferenceOfGaussians,
            expected_behavior: "ray-like modulation near a localized half-space perturbation",
            public_claim_level: "first-pass diagnostic",
            note: "Uses the extracted linear stationary MacKay-style example and generated diagnostics only.",
        },
        MackayExampleSpec {
            id: "mackay_target_linear_stationary",
            label: "MacKay target from localized strip input",
            input_kind: MackayInputKind::Target,
            kernel_kind: MackayKernelKind::DifferenceOfGaussians,
            expected_behavior: "target-like modulation around the localized strip input",
            public_claim_level: "first-pass diagnostic",
            note: "Uses the extracted linear stationary MacKay-style example and generated diagnostics only.",
        },
        MackayExampleSpec {
            id: "mackay_gaussian_kernel_negative_check",
            label: "Gaussian-only localized-input negative check",
            input_kind: MackayInputKind::Rays,
            kernel_kind: MackayKernelKind::GaussianOnly,
            expected_behavior: "smoother control response without the balanced DoG edge-selective component",
            public_claim_level: "diagnostic control",
            note: "Control run for separating localized-input effects from the difference-of-Gaussians kernel choice.",
        },
    ]
}

fn mackay_generated_example(
    spec: MackayExampleSpec,
    config: MackayReportConfig,
    parameters: MackayReportParameters,
) -> MackayGeneratedExample {
    let input = mackay_input_field(config, spec.input_kind);
    let (output, fixed_point) = mackay_fixed_point(&input, config, parameters, spec.kernel_kind);
    let field_metrics = mackay_field_metrics(&input, &output, config.n);
    MackayGeneratedExample {
        id: spec.id,
        label: spec.label,
        registry_id: spec.id,
        source_key: "tamekue-prandi-chitour-2024",
        model_family: MODEL_FAMILY_MACKAY,
        mathematical_object: "stationary scalar neural field a = I + mu * omega * f(a)",
        domain: format!(
            "finite square diagnostic grid on [{}, {}]^2",
            config.domain_min, config.domain_max
        ),
        input_formula: spec.input_kind.formula(),
        kernel: spec.kernel_kind.details(parameters),
        parameters: MackayExampleParameters {
            n: config.n,
            domain_min: config.domain_min,
            domain_max: config.domain_max,
            dx: parameters.dx,
            mu: config.mu,
            epsilon: config.epsilon,
            transfer_function: "linear f(a)=a",
            boundary_model: "zero-padded finite-domain convolution",
        },
        fixed_point,
        metrics: field_metrics,
        input_thumbnail: mackay_field_thumbnail(&input, config.n),
        output_thumbnail: mackay_field_thumbnail(&output, config.n),
        status: "generated",
        expected_behavior: spec.expected_behavior,
        public_claim_level: spec.public_claim_level,
        note: spec.note,
    }
}

impl MackayInputKind {
    fn formula(self) -> &'static str {
        match self {
            MackayInputKind::Rays => "I(x1,x2)=cos(5*pi*x2)+epsilon*H(2-x1)",
            MackayInputKind::Target => {
                "I(x1,x2)=cos(5*pi*x1)+epsilon*(H(-x2-9.75)+H(x2-9.75)+H(0.25-|x2|))"
            }
        }
    }
}

impl MackayKernelKind {
    fn details(self, parameters: MackayReportParameters) -> MackayExampleKernel {
        match self {
            MackayKernelKind::DifferenceOfGaussians => MackayExampleKernel {
                family: "difference_of_gaussians",
                formula: "omega(r)=G_sigma1(r)-kappa*G_sigma2(r)",
                sigma1: parameters.sigma1,
                sigma2: Some(parameters.sigma2),
                kappa: Some(parameters.kappa),
            },
            MackayKernelKind::GaussianOnly => MackayExampleKernel {
                family: "gaussian_only",
                formula: "omega(r)=G_sigma1(r)",
                sigma1: parameters.sigma1,
                sigma2: None,
                kappa: None,
            },
        }
    }
}

fn mackay_input_field(config: MackayReportConfig, kind: MackayInputKind) -> Vec<f64> {
    let mut input = vec![0.0; config.n * config.n];
    let dx = mackay_dx(config);
    for row in 0..config.n {
        let x2 = config.domain_min + row as f64 * dx;
        for col in 0..config.n {
            let x1 = config.domain_min + col as f64 * dx;
            input[row * config.n + col] = match kind {
                MackayInputKind::Rays => {
                    (5.0 * PI * x2).cos() + config.epsilon * heaviside(2.0 - x1)
                }
                MackayInputKind::Target => {
                    (5.0 * PI * x1).cos()
                        + config.epsilon
                            * (heaviside(-x2 - 9.75)
                                + heaviside(x2 - 9.75)
                                + heaviside(0.25 - x2.abs()))
                }
            };
        }
    }
    input
}

fn mackay_fixed_point(
    input: &[f64],
    config: MackayReportConfig,
    parameters: MackayReportParameters,
    kernel_kind: MackayKernelKind,
) -> (Vec<f64>, MackayFixedPointDiagnostics) {
    let n = config.n;
    let weights1 = convolution::gaussian_weights(n, parameters.dx, parameters.sigma1);
    let weights2 = convolution::gaussian_weights(n, parameters.dx, parameters.sigma2);
    let mut state = input.to_vec();
    let mut scratch = vec![0.0; input.len()];
    let mut conv1 = vec![0.0; input.len()];
    let mut conv2 = vec![0.0; input.len()];
    let mut next = vec![0.0; input.len()];
    let mut residual = f64::INFINITY;
    let mut completed_iterations = 0usize;

    for iteration in 1..=config.iterations {
        convolution::gaussian_convolve_2d_into(
            &state,
            n,
            &weights1,
            parameters.dx,
            &mut scratch,
            &mut conv1,
        );
        if matches!(kernel_kind, MackayKernelKind::DifferenceOfGaussians) {
            convolution::gaussian_convolve_2d_into(
                &state,
                n,
                &weights2,
                parameters.dx,
                &mut scratch,
                &mut conv2,
            );
        }

        residual = 0.0;
        for index in 0..state.len() {
            let coupling = match kernel_kind {
                MackayKernelKind::DifferenceOfGaussians => {
                    conv1[index] - parameters.kappa * conv2[index]
                }
                MackayKernelKind::GaussianOnly => conv1[index],
            };
            next[index] = input[index] + config.mu * coupling;
            residual = residual.max((next[index] - state[index]).abs());
        }
        std::mem::swap(&mut state, &mut next);
        completed_iterations = iteration;
        if residual <= config.tolerance {
            break;
        }
    }

    (
        state,
        MackayFixedPointDiagnostics {
            iterations: completed_iterations,
            residual_linf: residual,
            converged: residual <= config.tolerance,
        },
    )
}

fn mackay_field_metrics(input: &[f64], output: &[f64], n: usize) -> MackayFieldMetrics {
    let input_stats = metrics::stats(input);
    let output_stats = metrics::stats(output);
    let output_active_fraction =
        output.iter().filter(|value| **value >= 0.0).count() as f64 / output.len().max(1) as f64;
    let delta = output
        .iter()
        .zip(input.iter())
        .map(|(out, inp)| out - inp)
        .collect::<Vec<_>>();
    MackayFieldMetrics {
        input_mean: input_stats.0,
        input_std: input_stats.1,
        input_min: input_stats.2,
        input_max: input_stats.3,
        output_mean: output_stats.0,
        output_std: output_stats.1,
        output_min: output_stats.2,
        output_max: output_stats.3,
        output_active_fraction,
        zero_crossings_along_x_mean: metrics::zero_crossings_along_x(output, n),
        zero_crossings_along_y_mean: metrics::zero_crossings_along_y(output, n),
        input_output_correlation: metrics::correlation(input, output),
        output_input_delta_std: metrics::stats(&delta).1,
    }
}

fn mackay_field_thumbnail(field: &[f64], n: usize) -> MackayFieldThumbnail {
    let (_, _, min, max) = metrics::stats(field);
    let denom = (max - min).max(1.0e-12);
    let bytes = field
        .iter()
        .map(|value| (((value - min) / denom) * 255.0).clamp(0.0, 255.0) as u8)
        .collect::<Vec<_>>();
    MackayFieldThumbnail {
        format: "u8-field-v1",
        encoding: "base64",
        color_space: "normalized-luma",
        width: n,
        height: n,
        scale_min: min,
        scale_max: max,
        data_base64: general_purpose::STANDARD.encode(bytes),
    }
}

fn mackay_dx(config: MackayReportConfig) -> f64 {
    (config.domain_max - config.domain_min) / (config.n.saturating_sub(1).max(1) as f64)
}

fn heaviside(value: f64) -> f64 {
    if value >= 0.0 {
        1.0
    } else {
        0.0
    }
}
