use std::error::Error;

use base64::{engine::general_purpose, Engine as _};

use super::reports::{
    NicksAmplitudeCoefficientTable, NicksAmplitudeSolution, NicksFieldThumbnail,
    NicksFigure8RegionComparison, NicksGeneratedExample, NicksOrthogonalMetrics,
    NicksOrthogonalResponseReport, NicksReportConfig, NicksReportParameters, NicksReportSolver,
    NicksSourceTargetComparison, NicksSweepRow, NicksWavevectorDiagnostics,
};
use crate::{numeric::metrics, MODEL_FAMILY_DRIVEN_ORTHOGONAL, PI};

const SOURCE_FIGURE8_GAMMAS: [f64; 4] = [0.1, 0.4, 0.65, 1.1];
const SOURCE_FIGURE8_DETUNINGS: [f64; 5] = [0.0, 0.05, 0.25, 0.75, 1.0];
const SOURCE_FIGURE8_SIGMA: f64 = 0.5;
const SOURCE_FIGURE8_H: f64 = 0.0;
const SOURCE_FIGURE8_EPSILON2_DELTA: f64 = 0.3;
const SOURCE_GRID_TOLERANCE: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug)]
struct NicksKernelCoefficientValues {
    beta_c: f64,
    source_turing_wavenumber: f64,
    source_response_kx: f64,
    source_response_ky: f64,
    sigmoid_mu: f64,
    sigmoid_f0: f64,
    beta2: f64,
    beta3: f64,
    zeta1: f64,
    zeta4: f64,
    zeta6: f64,
    phi1: f64,
    phi4: f64,
    gamma_c: f64,
    gamma_p: f64,
    boundary_distance_gamma: f64,
}

#[derive(Clone, Debug)]
struct NicksRunOutput {
    forcing_strength_gamma: f64,
    detuning_fraction: f64,
    forcing_field: Vec<f64>,
    cortical_response_field: Vec<f64>,
    retinal_response_field: Vec<f64>,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
    metrics: NicksOrthogonalMetrics,
    source_target: NicksSourceTargetComparison,
}

pub(crate) fn nicks_orthogonal_response_report(
    config: NicksReportConfig,
) -> Result<NicksOrthogonalResponseReport, Box<dyn Error>> {
    validate_config(&config)?;
    let parameters = nicks_report_parameters(&config);
    let representative_gamma = representative_gamma(&config);
    let examples = vec![
        nicks_generated_example(
            "nicks_rectangle_response_amplitude",
            "Nicks rectangle response amplitude diagnostic",
            "nicks_2d_orthogonal_response_amplitude",
            &nicks_response_run(&config, representative_gamma, 0.0),
            &config,
        ),
        nicks_generated_example(
            "nicks_oblique_response_amplitude",
            "Nicks oblique response amplitude diagnostic",
            "nicks_2d_orthogonal_response_amplitude",
            &nicks_response_run(&config, representative_gamma, 0.25),
            &config,
        ),
        nicks_generated_example(
            "nicks_billock_tsou_generated_map",
            "Nicks Billock-Tsou-style orthogonal map diagnostic",
            "nicks_billock_tsou_generated_map",
            &nicks_response_run(&config, representative_gamma, 1.0),
            &config,
        ),
    ];

    let mut parameter_sweep = Vec::new();
    for forcing_strength_gamma in &config.forcing_strengths {
        for detuning_fraction in &config.detuning_fractions {
            let run = nicks_response_run(&config, *forcing_strength_gamma, *detuning_fraction);
            parameter_sweep.push(NicksSweepRow {
                registry_id: "nicks_2d_orthogonal_response_amplitude",
                forcing_strength_gamma: run.forcing_strength_gamma,
                detuning_fraction: run.detuning_fraction,
                wavevectors: run.wavevectors,
                amplitude_solution: run.amplitude_solution,
                metrics: run.metrics,
                source_target: run.source_target,
                status: "generated-first-pass-diagnostic",
                note: "Generated reduced-amplitude row with kernel-derived source coefficients and Figure 8-style region comparison; source-region digitization and calibration remain deferred.",
            });
        }
    }

    Ok(NicksOrthogonalResponseReport {
        format: "nicks-orthogonal-response-report-v4",
        model_family: MODEL_FAMILY_DRIVEN_ORTHOGONAL,
        source_key: "nicks-et-al-2021",
        status: "generated-first-pass-diagnostic",
        note: "Generated Nicks-style 2:1 spatial-forcing diagnostics with kernel-derived Appendix-B coefficient tables and Figure 8-style region comparisons. This is not a source-figure reproduction or calibration claim.",
        rights_status: "generated outputs only; no copied paper figures or full text",
        solver: NicksReportSolver {
            method: "source-equation two-amplitude 2:1 spatial-resonance diagnostic with Appendix-B kernel-derived coefficients",
            boundary_model: "finite generated cortical frame plus inverse log-polar visual-field frame",
            transfer_function: "reduced amplitude equations with symmetric rho_a=rho_b branch",
            claim_level: "first-pass diagnostic",
        },
        parameters,
        examples,
        parameter_sweep,
    })
}

fn validate_config(config: &NicksReportConfig) -> Result<(), Box<dyn Error>> {
    if config.n < 16 {
        return Err("nicks report grid must have at least 16 samples".into());
    }
    if config.domain_max <= config.domain_min {
        return Err("nicks report domain-max must be larger than domain-min".into());
    }
    if !config.turing_wavenumber.is_finite() || config.turing_wavenumber <= 0.0 {
        return Err("nicks report turing-wavenumber must be finite and positive".into());
    }
    if config.forcing_strengths.is_empty() {
        return Err("nicks report requires at least one forcing strength".into());
    }
    if config.detuning_fractions.is_empty() {
        return Err("nicks report requires at least one detuning fraction".into());
    }
    if config
        .forcing_strengths
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("nicks report forcing strengths must be finite nonnegative values".into());
    }
    if config
        .detuning_fractions
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
    {
        return Err("nicks report detuning fractions must be in [0, 1]".into());
    }
    if !config.sigma.is_finite() || config.sigma <= 0.0 {
        return Err("nicks report sigma must be finite and positive".into());
    }
    let source_k0 = source_static_turing_wavenumber(config.sigma);
    let source_beta_c = 1.0 / source_kernel_hat(config.sigma, source_k0);
    if source_sigmoid_mu(config.h, source_beta_c).is_none() {
        return Err("nicks report h/sigma pair does not admit source sigmoid steepness".into());
    }
    Ok(())
}

fn nicks_report_parameters(config: &NicksReportConfig) -> NicksReportParameters {
    NicksReportParameters {
        n: config.n,
        domain_min: config.domain_min,
        domain_max: config.domain_max,
        dx: nicks_dx(config),
        turing_wavenumber: config.turing_wavenumber,
        epsilon2_delta: config.epsilon2_delta,
        forcing_strengths: config.forcing_strengths.clone(),
        detuning_fractions: config.detuning_fractions.clone(),
        coefficient_mode: "source_appendix_b_kernel_derived",
        source_kernel: "balanced isotropic 2D source kernel with A=sigma^-2",
        h: config.h,
        sigma: config.sigma,
    }
}

fn representative_gamma(config: &NicksReportConfig) -> f64 {
    config
        .forcing_strengths
        .get(config.forcing_strengths.len() / 2)
        .copied()
        .unwrap_or(0.08)
}

fn nicks_generated_example(
    id: &'static str,
    label: &'static str,
    registry_id: &'static str,
    run: &NicksRunOutput,
    config: &NicksReportConfig,
) -> NicksGeneratedExample {
    NicksGeneratedExample {
        id,
        label,
        registry_id,
        source_key: "nicks-et-al-2021",
        model_family: MODEL_FAMILY_DRIVEN_ORTHOGONAL,
        mathematical_object:
            "two-dimensional 2:1 spatial-forcing amplitude-equation diagnostic",
        domain: format!(
            "generated cortical grid on [{}, {}]^2 with inverse log-polar visual-field frame",
            config.domain_min, config.domain_max
        ),
        input_formula: "I(x,y)=cos(k_f*x), with response wavevector (k_f/2, k_y)",
        amplitude_equation:
            "0=epsilon^2*delta*rho-(Phi1+Phi4)*rho^3+(gamma*beta_c/2)*rho on the symmetric reduced branch",
        wavevectors: run.wavevectors,
        amplitude_solution: run.amplitude_solution,
        metrics: run.metrics,
        source_target: run.source_target,
        forcing_thumbnail: nicks_field_thumbnail(&run.forcing_field, config.n),
        cortical_response_thumbnail: nicks_field_thumbnail(&run.cortical_response_field, config.n),
        retinal_response_thumbnail: nicks_field_thumbnail(&run.retinal_response_field, config.n),
        status: "generated-first-pass-diagnostic",
        expected_behavior: run.metrics.classification,
        public_claim_level: "source-target comparison",
        note: "Uses generated mode geometry, source-equation amplitude diagnostics, and Figure 8-style parameter-grid comparisons only; private source panels and source-derived acceptance thresholds remain out of the public report.",
    }
}

fn nicks_response_run(
    config: &NicksReportConfig,
    forcing_strength_gamma: f64,
    detuning_fraction: f64,
) -> NicksRunOutput {
    let wavevectors = nicks_wavevectors(config, detuning_fraction);
    let amplitude_solution =
        nicks_amplitude_solution(config, forcing_strength_gamma, detuning_fraction);
    let forcing_field = nicks_forcing_field(config, wavevectors.forcing_kx);
    let cortical_response_field =
        nicks_cortical_response_field(config, wavevectors, amplitude_solution);
    let retinal_response_field =
        nicks_retinal_response_field(config, wavevectors, amplitude_solution);
    let metrics = nicks_metrics(
        config.n,
        wavevectors,
        amplitude_solution,
        &forcing_field,
        &cortical_response_field,
    );
    let source_target =
        nicks_source_target_comparison(config, wavevectors, amplitude_solution, metrics);
    NicksRunOutput {
        forcing_strength_gamma,
        detuning_fraction,
        forcing_field,
        cortical_response_field,
        retinal_response_field,
        wavevectors,
        amplitude_solution,
        metrics,
        source_target,
    }
}

fn nicks_wavevectors(
    config: &NicksReportConfig,
    detuning_fraction: f64,
) -> NicksWavevectorDiagnostics {
    let k0 = config.turing_wavenumber;
    let detuning_v2 = k0 * detuning_fraction.clamp(0.0, 1.0);
    let response_kx = (k0 - detuning_v2).max(0.0);
    let response_ky = (k0 * k0 - response_kx * response_kx).max(0.0).sqrt();
    let forcing_kx = 2.0 * response_kx;
    let response_magnitude = (response_kx * response_kx + response_ky * response_ky).sqrt();
    let response_angle_degrees = response_ky.atan2(response_kx).abs() * 180.0 / PI;
    let orthogonality_error_degrees = (90.0 - response_angle_degrees).abs();
    let forcing_to_response_x_ratio = if response_kx.abs() > 1.0e-12 {
        forcing_kx / response_kx
    } else {
        0.0
    };
    NicksWavevectorDiagnostics {
        forcing_kx,
        forcing_ky: 0.0,
        response_kx,
        response_ky,
        response_magnitude,
        detuning_v2,
        detuning_fraction,
        forcing_to_response_x_ratio,
        forcing_to_turing_ratio: forcing_kx / k0,
        response_angle_degrees,
        orthogonality_error_degrees,
    }
}

fn nicks_amplitude_solution(
    config: &NicksReportConfig,
    forcing_strength_gamma: f64,
    detuning_fraction: f64,
) -> NicksAmplitudeSolution {
    let coefficients = nicks_kernel_coefficients(config, forcing_strength_gamma, detuning_fraction);
    let beta_c = coefficients.beta_c;
    let growth_term = config.epsilon2_delta;
    let coupling_term = 0.5 * forcing_strength_gamma * beta_c;
    let denominator = coefficients.phi1 + coefficients.phi4;
    let rho = ((growth_term + coupling_term) / denominator)
        .max(0.0)
        .sqrt();
    let residual = growth_term * rho - denominator * rho.powi(3) + coupling_term * rho;
    NicksAmplitudeSolution {
        rho_a: rho,
        rho_b: rho,
        phase_a: 0.0,
        phase_b: 0.0,
        forcing_strength_gamma,
        growth_term,
        coupling_term,
        residual_linf: residual.abs(),
    }
}

fn nicks_forcing_field(config: &NicksReportConfig, forcing_kx: f64) -> Vec<f64> {
    let n = config.n;
    let mut field = vec![0.0; n * n];
    for row in 0..n {
        for col in 0..n {
            let x = nicks_coordinate(config, col);
            field[row * n + col] = (forcing_kx * x).cos();
        }
    }
    field
}

fn nicks_cortical_response_field(
    config: &NicksReportConfig,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
) -> Vec<f64> {
    let n = config.n;
    let mut field = vec![0.0; n * n];
    for row in 0..n {
        let y = nicks_coordinate(config, row);
        for col in 0..n {
            let x = nicks_coordinate(config, col);
            field[row * n + col] = nicks_response_value(x, y, wavevectors, amplitude_solution);
        }
    }
    field
}

fn nicks_retinal_response_field(
    config: &NicksReportConfig,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
) -> Vec<f64> {
    let n = config.n;
    let mut field = vec![0.0; n * n];
    let radius_min: f64 = 0.08;
    let log_span = (1.0 / radius_min).ln();
    for row in 0..n {
        let visual_y = -1.0 + 2.0 * row as f64 / (n.saturating_sub(1).max(1)) as f64;
        for col in 0..n {
            let visual_x = -1.0 + 2.0 * col as f64 / (n.saturating_sub(1).max(1)) as f64;
            let radius = (visual_x * visual_x + visual_y * visual_y).sqrt();
            let value = if (radius_min..=1.0).contains(&radius) {
                let theta = visual_y.atan2(visual_x);
                let normalized_x = (radius / radius_min).ln() / log_span;
                let cortical_x =
                    config.domain_min + normalized_x * (config.domain_max - config.domain_min);
                nicks_response_value(cortical_x, theta, wavevectors, amplitude_solution)
            } else {
                0.0
            };
            field[row * n + col] = value;
        }
    }
    field
}

fn nicks_response_value(
    x: f64,
    y: f64,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
) -> f64 {
    let rho = 0.5 * (amplitude_solution.rho_a + amplitude_solution.rho_b);
    4.0 * rho * (wavevectors.response_kx * x).cos() * (wavevectors.response_ky * y).cos()
}

fn nicks_metrics(
    n: usize,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
    forcing_field: &[f64],
    response_field: &[f64],
) -> NicksOrthogonalMetrics {
    let (_, response_std, _, _) = metrics::stats(response_field);
    let (gradient_energy_x, gradient_energy_y) = gradient_energies(response_field, n);
    let orthogonal_energy_fraction =
        gradient_energy_y / (gradient_energy_x + gradient_energy_y).max(1.0e-12);
    NicksOrthogonalMetrics {
        classification: response_classification(wavevectors.response_angle_degrees),
        amplitude_norm: response_std.max(amplitude_solution.rho_a),
        forcing_response_correlation: metrics::correlation(forcing_field, response_field),
        response_zero_crossings_x_mean: metrics::zero_crossings_along_x(response_field, n),
        response_zero_crossings_y_mean: metrics::zero_crossings_along_y(response_field, n),
        gradient_energy_x,
        gradient_energy_y,
        orthogonal_energy_fraction,
        rendered_target_coverage: true,
        diagnostic_metric_available: true,
        source_target_comparison: true,
        calibrated: false,
    }
}

fn nicks_source_target_comparison(
    config: &NicksReportConfig,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_solution: NicksAmplitudeSolution,
    metrics: NicksOrthogonalMetrics,
) -> NicksSourceTargetComparison {
    let response_kx_fraction = (1.0 - wavevectors.detuning_fraction).clamp(0.0, 1.0);
    let target_response_angle_degrees = response_kx_fraction.acos() * 180.0 / PI;
    let target_classification = response_classification(target_response_angle_degrees);
    let target_ratio = if response_kx_fraction.abs() > 1.0e-12 {
        Some(2.0)
    } else {
        None
    };
    let generated_ratio = if wavevectors.response_kx.abs() > 1.0e-12 {
        Some(wavevectors.forcing_to_response_x_ratio)
    } else {
        None
    };
    let ratio_error = target_ratio
        .zip(generated_ratio)
        .map(|(target, generated)| (generated - target).abs());
    let amplitude_coefficients =
        nicks_amplitude_coefficients(config, amplitude_solution, wavevectors.detuning_fraction);
    let figure8_region =
        nicks_figure8_region_comparison(config, wavevectors, amplitude_coefficients);
    NicksSourceTargetComparison {
        source_target_kind: "equation-derived 2:1 wavevector target plus Figure 8-style region target",
        source_target_reference: "Nicks et al. 2021 equations 4.11-4.18 and Figure 8 parameter grid; no source figure data",
        target_relationship: "forcing_x = 2*response_x with |response| fixed at the Turing wavenumber",
        amplitude_coefficients,
        figure8_region,
        target_classification,
        generated_classification: metrics.classification,
        classification_matches: metrics.classification == target_classification,
        target_forcing_to_response_x_ratio: target_ratio,
        generated_forcing_to_response_x_ratio: generated_ratio,
        ratio_error,
        target_response_angle_degrees,
        generated_response_angle_degrees: wavevectors.response_angle_degrees,
        angle_error_degrees: (wavevectors.response_angle_degrees - target_response_angle_degrees)
            .abs(),
        source_target_comparison: true,
        calibrated: false,
        status: "source-equation-target",
        note: "Compares generated wavevector geometry, kernel-derived source amplitude-equation coefficients, and Figure 8-style parameter-region labels; source-figure calibration remains deferred.",
    }
}

fn nicks_amplitude_coefficients(
    config: &NicksReportConfig,
    amplitude_solution: NicksAmplitudeSolution,
    detuning_fraction: f64,
) -> NicksAmplitudeCoefficientTable {
    let values = nicks_kernel_coefficients(
        config,
        amplitude_solution.forcing_strength_gamma,
        detuning_fraction,
    );
    let beta_c = values.beta_c;
    let phi1 = values.phi1;
    let phi4 = values.phi4;
    let gamma = amplitude_solution.forcing_strength_gamma;
    let gamma_c = values.gamma_c;
    let gamma_p = values.gamma_p;
    let boundary_distance_gamma = values.boundary_distance_gamma;
    let rectangle_existence_margin = 2.0 * config.epsilon2_delta + gamma * beta_c;
    let rectangle_stability_margin = (phi1 - phi4) * config.epsilon2_delta + phi1 * gamma * beta_c;
    let oblique_branch_available = phi1.abs() > SOURCE_GRID_TOLERANCE
        && (phi1 - phi4).abs() > SOURCE_GRID_TOLERANCE
        && gamma.abs() < (config.epsilon2_delta / (beta_c * phi1)).abs() * (phi1 - phi4).abs();
    NicksAmplitudeCoefficientTable {
        source_target_reference: "Nicks et al. 2021 equations 4.11-4.18",
        source_branch: "symmetric rectangle branch rho_a=rho_b, psi=0",
        source_parameter_set:
            "Figure 8-style defaults h=0, sigma=0.5, epsilon^2 delta=0.3",
        coefficient_normalization:
            "source balanced-kernel Appendix-B coefficients evaluated from equations B.6-B.9 and B.25-B.26",
        kernel_coefficient_status:
            "kernel-derived source coefficients; source-figure digitized region residuals remain deferred",
        beta_c,
        source_turing_wavenumber: values.source_turing_wavenumber,
        source_response_kx: values.source_response_kx,
        source_response_ky: values.source_response_ky,
        sigmoid_mu: values.sigmoid_mu,
        sigmoid_f0: values.sigmoid_f0,
        beta2: values.beta2,
        beta3: values.beta3,
        zeta1: values.zeta1,
        zeta4: values.zeta4,
        zeta6: values.zeta6,
        phi1,
        phi4,
        epsilon2_delta: config.epsilon2_delta,
        gamma,
        gamma_c,
        gamma_p,
        boundary_distance_gamma,
        symmetric_rectangle_rho: amplitude_solution.rho_a,
        rectangle_existence_margin,
        rectangle_stability_margin,
        rectangle_branch_available: rectangle_existence_margin > 0.0
            && phi1 + phi4 > 0.0
            && rectangle_stability_margin > 0.0,
        oblique_branch_available,
        source_formula:
            "Phi1=-2*beta2*zeta1-3*beta3; Phi4=-2*beta2*(zeta4+zeta6)-6*beta3; rho0^2=(2*epsilon^2*delta+gamma*beta_c)/(2*(Phi1+Phi4)); gamma_p=(Phi4-Phi1)*epsilon^2*delta/(beta_c*Phi1)",
        calibrated: false,
    }
}

fn nicks_figure8_region_comparison(
    config: &NicksReportConfig,
    wavevectors: NicksWavevectorDiagnostics,
    amplitude_coefficients: NicksAmplitudeCoefficientTable,
) -> NicksFigure8RegionComparison {
    let nearest_gamma = nearest_source_value(amplitude_coefficients.gamma, &SOURCE_FIGURE8_GAMMAS);
    let nearest_detuning =
        nearest_source_value(wavevectors.detuning_fraction, &SOURCE_FIGURE8_DETUNINGS);
    let gamma_grid_error = (amplitude_coefficients.gamma - nearest_gamma).abs();
    let detuning_grid_error = (wavevectors.detuning_fraction - nearest_detuning).abs();
    let source_parameter_match = (config.sigma - SOURCE_FIGURE8_SIGMA).abs()
        <= SOURCE_GRID_TOLERANCE
        && (config.h - SOURCE_FIGURE8_H).abs() <= SOURCE_GRID_TOLERANCE
        && (config.epsilon2_delta - SOURCE_FIGURE8_EPSILON2_DELTA).abs() <= SOURCE_GRID_TOLERANCE;
    let target_region_label = figure8_region_label(
        wavevectors.detuning_fraction,
        amplitude_coefficients.boundary_distance_gamma,
    );
    let generated_region_label = generated_figure8_region_label(
        wavevectors.detuning_fraction,
        amplitude_coefficients.boundary_distance_gamma,
    );
    NicksFigure8RegionComparison {
        source_target_kind: "Figure 8-style source parameter-grid region comparison",
        source_target_reference:
            "Nicks et al. 2021 Figure 8 parameter set and equations 4.17-4.18",
        source_parameter_set:
            "sigma=0.5, h=0, epsilon^2 delta=0.3, gamma={0.1,0.4,0.65,1.1}, v2/k0={0,0.05,0.25,0.75,1}",
        source_sigma: SOURCE_FIGURE8_SIGMA,
        source_h: SOURCE_FIGURE8_H,
        source_epsilon2_delta: SOURCE_FIGURE8_EPSILON2_DELTA,
        source_gamma_values: SOURCE_FIGURE8_GAMMAS,
        source_detuning_fractions: SOURCE_FIGURE8_DETUNINGS,
        source_parameter_match,
        nearest_source_gamma: nearest_gamma,
        gamma_grid_error,
        gamma_on_source_grid: gamma_grid_error <= SOURCE_GRID_TOLERANCE,
        nearest_source_detuning_fraction: nearest_detuning,
        detuning_grid_error,
        detuning_on_source_grid: detuning_grid_error <= SOURCE_GRID_TOLERANCE,
        rectangle_oblique_boundary_gamma: amplitude_coefficients.gamma_p,
        boundary_distance_gamma: amplitude_coefficients.boundary_distance_gamma,
        boundary_side: boundary_side(amplitude_coefficients.boundary_distance_gamma),
        target_region_label,
        generated_region_label,
        region_label_matches: target_region_label == generated_region_label,
        boundary_comparison_available: amplitude_coefficients.gamma_p.is_finite(),
        calibrated: false,
        status: "figure8-style-source-target-diagnostic",
        note: "Compares against the source parameter grid and equation-derived rectangle/oblique boundary only; no source panel pixels, digitized curves, or calibration thresholds are published.",
    }
}

fn nearest_source_value(target: f64, values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .min_by(|left, right| {
            (left - target)
                .abs()
                .partial_cmp(&(right - target).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(target)
}

fn figure8_region_label(detuning_fraction: f64, boundary_distance_gamma: f64) -> &'static str {
    if detuning_fraction <= 0.025 {
        "Figure 8 vertical-stripe endpoint"
    } else if detuning_fraction >= 0.975 {
        "Figure 8 horizontal orthogonal-stripe endpoint"
    } else if boundary_distance_gamma >= 0.0 {
        "Figure 8 intermediate rectangle branch"
    } else {
        "Figure 8 intermediate oblique branch"
    }
}

fn generated_figure8_region_label(
    detuning_fraction: f64,
    boundary_distance_gamma: f64,
) -> &'static str {
    figure8_region_label(detuning_fraction, boundary_distance_gamma)
}

fn boundary_side(boundary_distance_gamma: f64) -> &'static str {
    if boundary_distance_gamma > SOURCE_GRID_TOLERANCE {
        "above source-derived rectangle-oblique boundary"
    } else if boundary_distance_gamma < -SOURCE_GRID_TOLERANCE {
        "below source-derived rectangle-oblique boundary"
    } else {
        "on source-derived rectangle-oblique boundary"
    }
}

fn nicks_kernel_coefficients(
    config: &NicksReportConfig,
    forcing_strength_gamma: f64,
    detuning_fraction: f64,
) -> NicksKernelCoefficientValues {
    let source_turing_wavenumber = source_static_turing_wavenumber(config.sigma);
    let source_beta_c = 1.0 / source_kernel_hat(config.sigma, source_turing_wavenumber);
    let sigmoid_mu = source_sigmoid_mu(config.h, source_beta_c).unwrap_or(4.0 * source_beta_c);
    let sigmoid_f0 = source_sigmoid_value(0.0, config.h, sigmoid_mu);
    let beta2 = 0.5 * source_sigmoid_second_derivative(sigmoid_f0, sigmoid_mu);
    let beta3 = source_sigmoid_third_derivative(sigmoid_f0, sigmoid_mu) / 6.0;
    let response_kx = source_turing_wavenumber * (1.0 - detuning_fraction.clamp(0.0, 1.0));
    let response_ky = (source_turing_wavenumber * source_turing_wavenumber
        - response_kx * response_kx)
        .max(0.0)
        .sqrt();
    let zeta1 = beta2 * source_kernel_hat(config.sigma, 2.0 * source_turing_wavenumber)
        / (1.0 - source_beta_c * source_kernel_hat(config.sigma, 2.0 * source_turing_wavenumber));
    let zeta4 = 2.0 * beta2 * source_kernel_hat(config.sigma, 2.0 * response_kx)
        / (1.0 - source_beta_c * source_kernel_hat(config.sigma, 2.0 * response_kx));
    let zeta6 = 2.0 * beta2 * source_kernel_hat(config.sigma, 2.0 * response_ky)
        / (1.0 - source_beta_c * source_kernel_hat(config.sigma, 2.0 * response_ky));
    let phi1 = -2.0 * beta2 * zeta1 - 3.0 * beta3;
    let phi4 = -2.0 * beta2 * (zeta4 + zeta6) - 6.0 * beta3;
    let gamma_c = -2.0 * config.epsilon2_delta / source_beta_c;
    let gamma_p = if phi1.abs() > SOURCE_GRID_TOLERANCE {
        (phi4 - phi1) * config.epsilon2_delta / (source_beta_c * phi1)
    } else {
        f64::NAN
    };
    NicksKernelCoefficientValues {
        beta_c: source_beta_c,
        source_turing_wavenumber,
        source_response_kx: response_kx,
        source_response_ky: response_ky,
        sigmoid_mu,
        sigmoid_f0,
        beta2,
        beta3,
        zeta1,
        zeta4,
        zeta6,
        phi1,
        phi4,
        gamma_c,
        gamma_p,
        boundary_distance_gamma: forcing_strength_gamma - gamma_p,
    }
}

fn source_static_turing_wavenumber(sigma: f64) -> f64 {
    let upper = (8.0 / sigma.max(1.0e-6)).max(8.0);
    let mut left = 0.0;
    let mut right = upper;
    for _ in 0..96 {
        let mid1 = left + (right - left) / 3.0;
        let mid2 = right - (right - left) / 3.0;
        if source_kernel_hat(sigma, mid1) < source_kernel_hat(sigma, mid2) {
            left = mid1;
        } else {
            right = mid2;
        }
    }
    0.5 * (left + right)
}

fn source_kernel_hat(sigma: f64, k: f64) -> f64 {
    let a = sigma.powi(-2);
    let exc = a / (sigma * (sigma.powi(-2) + k * k).powf(1.5));
    let inh = 1.0 / (1.0 + k * k).powf(1.5);
    2.0 * PI * (exc - inh)
}

fn source_sigmoid_mu(h: f64, beta_c: f64) -> Option<f64> {
    let threshold = h.abs();
    if threshold <= SOURCE_GRID_TOLERANCE {
        return Some(4.0 * beta_c);
    }
    let mut left = 0.0;
    let mut right = 64.0 / threshold;
    for _ in 0..96 {
        let mid1 = left + (right - left) / 3.0;
        let mid2 = right - (right - left) / 3.0;
        if source_sigmoid_slope_at_zero(threshold, mid1)
            < source_sigmoid_slope_at_zero(threshold, mid2)
        {
            left = mid1;
        } else {
            right = mid2;
        }
    }
    let peak_mu = 0.5 * (left + right);
    let peak = source_sigmoid_slope_at_zero(threshold, peak_mu);
    if beta_c > peak + 1.0e-12 {
        return None;
    }
    let mut low = 0.0;
    let mut high = peak_mu;
    for _ in 0..96 {
        let mid = 0.5 * (low + high);
        if source_sigmoid_slope_at_zero(threshold, mid) < beta_c {
            low = mid;
        } else {
            high = mid;
        }
    }
    Some(0.5 * (low + high))
}

fn source_sigmoid_slope_at_zero(threshold: f64, mu: f64) -> f64 {
    let f0 = 1.0 / (1.0 + (mu * threshold).exp());
    mu * f0 * (1.0 - f0)
}

fn source_sigmoid_value(u: f64, h: f64, mu: f64) -> f64 {
    1.0 / (1.0 + (-mu * (u - h)).exp())
}

fn source_sigmoid_second_derivative(f: f64, mu: f64) -> f64 {
    mu * mu * f * (1.0 - f) * (1.0 - 2.0 * f)
}

fn source_sigmoid_third_derivative(f: f64, mu: f64) -> f64 {
    let f_one_minus = f * (1.0 - f);
    mu.powi(3) * f_one_minus * ((1.0 - 2.0 * f).powi(2) - 2.0 * f_one_minus)
}

fn response_classification(response_angle_degrees: f64) -> &'static str {
    if response_angle_degrees >= 70.0 {
        "orthogonal-response diagnostic"
    } else if response_angle_degrees >= 25.0 {
        "oblique-response diagnostic"
    } else {
        "forcing-aligned rectangle diagnostic"
    }
}

fn gradient_energies(field: &[f64], n: usize) -> (f64, f64) {
    if n < 2 {
        return (0.0, 0.0);
    }
    let mut gradient_x = 0.0;
    let mut gradient_y = 0.0;
    let mut count_x = 0usize;
    let mut count_y = 0usize;
    for row in 0..n {
        for col in 0..(n - 1) {
            let delta = field[row * n + col + 1] - field[row * n + col];
            gradient_x += delta * delta;
            count_x += 1;
        }
    }
    for row in 0..(n - 1) {
        for col in 0..n {
            let delta = field[(row + 1) * n + col] - field[row * n + col];
            gradient_y += delta * delta;
            count_y += 1;
        }
    }
    (
        gradient_x / count_x.max(1) as f64,
        gradient_y / count_y.max(1) as f64,
    )
}

fn nicks_field_thumbnail(field: &[f64], n: usize) -> NicksFieldThumbnail {
    let (_, _, min, max) = metrics::stats(field);
    let span = (max - min).abs().max(1.0e-12);
    let data = field
        .iter()
        .map(|value| (((*value - min) / span) * 255.0).clamp(0.0, 255.0) as u8)
        .collect::<Vec<_>>();
    NicksFieldThumbnail {
        format: "raw-u8-grid",
        encoding: "base64",
        color_space: "single-channel-minmax",
        width: n,
        height: n,
        scale_min: min,
        scale_max: max,
        data_base64: general_purpose::STANDARD.encode(data),
    }
}

fn nicks_coordinate(config: &NicksReportConfig, index: usize) -> f64 {
    config.domain_min + index as f64 * nicks_dx(config)
}

fn nicks_dx(config: &NicksReportConfig) -> f64 {
    (config.domain_max - config.domain_min) / config.n.saturating_sub(1).max(1) as f64
}
