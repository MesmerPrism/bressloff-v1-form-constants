use std::error::Error;

use base64::{engine::general_purpose, Engine as _};

use super::reports::{
    NicksAmplitudeSolution, NicksFieldThumbnail, NicksGeneratedExample, NicksOrthogonalMetrics,
    NicksOrthogonalResponseReport, NicksReportConfig, NicksReportParameters, NicksReportSolver,
    NicksSweepRow, NicksWavevectorDiagnostics,
};
use crate::{numeric::metrics, MODEL_FAMILY_DRIVEN_ORTHOGONAL, PI};

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
            &nicks_response_run(&config, representative_gamma, 0.5),
            &config,
        ),
        nicks_generated_example(
            "nicks_billock_tsou_generated_map",
            "Nicks Billock-Tsou-style orthogonal map diagnostic",
            "nicks_billock_tsou_generated_map",
            &nicks_response_run(&config, representative_gamma, 0.9),
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
                status: "generated-first-pass-diagnostic",
                note: "Generated reduced-amplitude row; source-derived coefficient normalization and acceptance thresholds remain deferred.",
            });
        }
    }

    Ok(NicksOrthogonalResponseReport {
        format: "nicks-orthogonal-response-report-v1",
        model_family: MODEL_FAMILY_DRIVEN_ORTHOGONAL,
        source_key: "nicks-et-al-2021",
        status: "generated-first-pass-diagnostic",
        note: "Generated Nicks-style 2:1 spatial-forcing diagnostics. This report draws cortical and inverse-log-polar response frames from a normalized reduced-amplitude geometry; it is not a source-figure reproduction claim.",
        rights_status: "generated outputs only; no copied paper figures or full text",
        solver: NicksReportSolver {
            method: "normalized two-amplitude 2:1 spatial-resonance diagnostic",
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
    if config.self_interaction + config.cross_interaction <= 0.0 {
        return Err("nicks report amplitude interactions must have positive sum".into());
    }
    if !config.sigma.is_finite() || config.sigma <= 0.0 {
        return Err("nicks report sigma must be finite and positive".into());
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
        self_interaction: config.self_interaction,
        cross_interaction: config.cross_interaction,
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
            "0=(epsilon^2*delta+Gamma)*rho-(Phi1+Phi4)*rho^3 on the symmetric reduced branch",
        wavevectors: run.wavevectors,
        amplitude_solution: run.amplitude_solution,
        metrics: run.metrics,
        forcing_thumbnail: nicks_field_thumbnail(&run.forcing_field, config.n),
        cortical_response_thumbnail: nicks_field_thumbnail(&run.cortical_response_field, config.n),
        retinal_response_thumbnail: nicks_field_thumbnail(&run.retinal_response_field, config.n),
        status: "generated-first-pass-diagnostic",
        expected_behavior: run.metrics.classification,
        public_claim_level: "first-pass diagnostic",
        note: "Uses generated mode geometry and metrics only; private source panels and source-derived acceptance thresholds remain out of the public report.",
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
    NicksRunOutput {
        forcing_strength_gamma,
        detuning_fraction,
        forcing_field,
        cortical_response_field,
        retinal_response_field,
        wavevectors,
        amplitude_solution,
        metrics,
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
    let threshold_penalty = 0.15 * config.h.abs();
    let detuning_bonus = 0.25 * detuning_fraction;
    let growth_term = (config.epsilon2_delta + detuning_bonus - threshold_penalty).max(0.0);
    let coupling_term = forcing_strength_gamma * (1.0 + 0.5 * detuning_fraction);
    let denominator = config.self_interaction + config.cross_interaction;
    let rho = ((growth_term + coupling_term) / denominator)
        .max(0.0)
        .sqrt();
    let residual = (growth_term + coupling_term) * rho - denominator * rho.powi(3);
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
        source_target_comparison: false,
        calibrated: false,
    }
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
