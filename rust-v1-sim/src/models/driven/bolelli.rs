use std::{
    error::Error,
    ops::{Div, Mul, Neg, Sub},
};

use base64::{engine::general_purpose, Engine as _};

use super::reports::{
    BolelliExampleKernel, BolelliFrequencySweepRow, BolelliGeneratedExample, BolelliPeriodMetrics,
    BolelliPoleWidthComparison, BolelliProfileThumbnail, BolelliReportConfig,
    BolelliReportParameters, BolelliReportSolver, BolelliTimePeriodicInputReport,
};
use crate::{numeric::metrics, MODEL_FAMILY_LOCALIZED_PERIODIC, PI};

#[derive(Clone, Debug)]
struct BolelliRunOutput {
    frequency_lambda: f64,
    period: f64,
    input_profile: Vec<f64>,
    mean_profile: Vec<f64>,
    amplitude_profile: Vec<f64>,
    metrics: BolelliPeriodMetrics,
    source_target: BolelliPoleWidthComparison,
}

struct BolelliPeriodMetricInput<'a> {
    previous_period: &'a [f64],
    last_period: &'a [f64],
    mean_profile: &'a [f64],
    amplitude_profile: &'a [f64],
    localized_mask: &'a [f64],
    config: &'a BolelliReportConfig,
    dt: f64,
    total_steps: usize,
}

#[derive(Clone, Copy, Debug)]
struct BolelliGeneratedWidthEstimate {
    width: Option<f64>,
    decay_rate: Option<f64>,
    fit_r_squared: Option<f64>,
    fit_points: usize,
    fit_pass: bool,
}

const GENERATED_DECAY_WIDTH_MIN_R_SQUARED: f64 = 0.70;
const GENERATED_DECAY_WIDTH_MIN_POINTS: usize = 8;

pub(crate) fn bolelli_time_periodic_report(
    config: BolelliReportConfig,
) -> Result<BolelliTimePeriodicInputReport, Box<dyn Error>> {
    validate_config(&config)?;
    let parameters = bolelli_report_parameters(&config);
    let representative_frequency = representative_frequency(&config);
    let mut representative_run = None;
    let mut frequency_sweep = Vec::new();

    for frequency_lambda in &config.frequencies {
        let run = bolelli_frequency_run(&config, &parameters, *frequency_lambda);
        if (*frequency_lambda - representative_frequency).abs() < 1.0e-9 {
            representative_run = Some(run.clone());
        }
        frequency_sweep.push(BolelliFrequencySweepRow {
            registry_id: "bolelli_contour_width_frequency_sweep",
            frequency_lambda: run.frequency_lambda,
            period: run.period,
            metrics: run.metrics,
            source_target: run.source_target,
            status: if run.metrics.locked {
                "period-locked-diagnostic"
            } else {
                "warmup-insufficient-diagnostic"
            },
            note: "Generated frequency-sweep row with an accepted source-side principal-pole width convention; generated half-max width remains an auxiliary diagnostic.",
        });
    }

    let representative_run = representative_run
        .or_else(|| {
            config
                .frequencies
                .first()
                .map(|frequency| bolelli_frequency_run(&config, &parameters, *frequency))
        })
        .ok_or("bolelli report requires at least one frequency")?;

    let examples = vec![bolelli_generated_example(
        &representative_run,
        &config,
        &parameters,
    )];

    Ok(BolelliTimePeriodicInputReport {
        format: "bolelli-time-periodic-input-report-v5",
        model_family: MODEL_FAMILY_LOCALIZED_PERIODIC,
        source_key: "bolelli-prandi-2025",
        status: "generated-first-pass-diagnostic",
        note: "Generated Bolelli-Prandi-style localized time-periodic input diagnostics. This report checks period locking, phase, generated stripe-width metrics, an accepted source-side principal-pole width convention, and a generated decay-width estimate in the same pole convention; it is not a source-figure reproduction or generated-width calibration claim.",
        rights_status: "generated outputs only; no copied paper figures or full text",
        solver: BolelliReportSolver {
            method: "explicit Euler integration to a periodic state after warmup",
            boundary_model: "one-dimensional periodic cortical coordinate",
            transfer_function: "linear response with localized time-periodic external input",
            claim_level: "first-pass diagnostic",
        },
        parameters,
        examples,
        frequency_sweep,
    })
}

fn validate_config(config: &BolelliReportConfig) -> Result<(), Box<dyn Error>> {
    if config.n < 16 {
        return Err("bolelli report grid must have at least 16 samples".into());
    }
    if config.domain_max <= config.domain_min {
        return Err("bolelli report domain-max must be larger than domain-min".into());
    }
    if config.samples_per_period < 16 {
        return Err("bolelli report samples-per-period must be at least 16".into());
    }
    if config.frequencies.is_empty() {
        return Err("bolelli report requires at least one frequency".into());
    }
    if config
        .frequencies
        .iter()
        .any(|frequency| !frequency.is_finite() || *frequency <= 0.0)
    {
        return Err("bolelli report frequencies must be finite positive lambda values".into());
    }
    Ok(())
}

fn representative_frequency(config: &BolelliReportConfig) -> f64 {
    config
        .frequencies
        .iter()
        .copied()
        .min_by(|left, right| (left - 20.0).abs().total_cmp(&(right - 20.0).abs()))
        .unwrap_or(20.0)
}

fn bolelli_report_parameters(config: &BolelliReportConfig) -> BolelliReportParameters {
    let dx = bolelli_dx(config);
    let kernel = bolelli_kernel_offsets(dx, config.sigma_exc, config.sigma_inh, config.inhibition);
    let approximate_kernel_l1 = kernel
        .iter()
        .map(|(_, weight)| weight.abs() * dx)
        .sum::<f64>();
    BolelliReportParameters {
        n: config.n,
        domain_min: config.domain_min,
        domain_max: config.domain_max,
        dx,
        samples_per_period: config.samples_per_period,
        warmup_periods: config.warmup_periods,
        periodic_tolerance: config.periodic_tolerance,
        mu: config.mu,
        drive_amplitude: config.drive_amplitude,
        static_bias: config.static_bias,
        spatial_frequency: config.spatial_frequency,
        sigma_exc: config.sigma_exc,
        sigma_inh: config.sigma_inh,
        inhibition: config.inhibition,
        approximate_kernel_l1,
        contraction_mu_l1: config.mu * approximate_kernel_l1,
    }
}

fn bolelli_generated_example(
    run: &BolelliRunOutput,
    config: &BolelliReportConfig,
    parameters: &BolelliReportParameters,
) -> BolelliGeneratedExample {
    BolelliGeneratedExample {
        id: "bolelli_heaviside_flicker_periodic_state",
        label: "Bolelli Heaviside flicker periodic-state diagnostic",
        registry_id: "bolelli_heaviside_flicker_stripes",
        source_key: "bolelli-prandi-2025",
        model_family: MODEL_FAMILY_LOCALIZED_PERIODIC,
        mathematical_object:
            "linear time-periodic neural field with localized Heaviside external input",
        domain: format!(
            "one-dimensional periodic diagnostic grid on [{}, {}]",
            config.domain_min, config.domain_max
        ),
        input_formula: "I(x,t)=static_bias*cos(k*x)+drive_amplitude*H(-x)*cos(lambda*t)",
        kernel: BolelliExampleKernel {
            family: "difference_of_gaussians_1d",
            formula: "omega(x)=G_sigma_exc(x)-inhibition*G_sigma_inh(x)",
            sigma_exc: parameters.sigma_exc,
            sigma_inh: parameters.sigma_inh,
            inhibition: parameters.inhibition,
            approximate_l1: parameters.approximate_kernel_l1,
        },
        frequency_lambda: run.frequency_lambda,
        period: run.period,
        metrics: run.metrics,
        source_target: run.source_target,
        input_thumbnail: bolelli_profile_thumbnail(&run.input_profile),
        mean_response_thumbnail: bolelli_profile_thumbnail(&run.mean_profile),
        amplitude_thumbnail: bolelli_profile_thumbnail(&run.amplitude_profile),
        status: if run.metrics.locked {
            "period-locked-diagnostic"
        } else {
            "warmup-insufficient-diagnostic"
        },
        expected_behavior: "periodic response locked to localized flicker with a generated stripe-width diagnostic",
        public_claim_level: "source-target comparison",
        note: "Uses generated profiles, numeric period metrics, an equation-derived pole-width target, and a generated decay-width estimate in the same pole convention; source figure comparison remains deferred.",
    }
}

fn bolelli_frequency_run(
    config: &BolelliReportConfig,
    parameters: &BolelliReportParameters,
    frequency_lambda: f64,
) -> BolelliRunOutput {
    let n = config.n;
    let period = 2.0 * PI / frequency_lambda;
    let dt = period / config.samples_per_period as f64;
    let total_periods = config.warmup_periods + 2;
    let total_steps = total_periods * config.samples_per_period;
    let x_values = bolelli_x_values(config);
    let localized_mask = x_values.iter().map(|x| heaviside(-*x)).collect::<Vec<_>>();
    let static_profile = x_values
        .iter()
        .map(|x| config.static_bias * (config.spatial_frequency * x).cos())
        .collect::<Vec<_>>();
    let input_profile = static_profile
        .iter()
        .zip(localized_mask.iter())
        .map(|(static_value, mask)| static_value + config.drive_amplitude * mask)
        .collect::<Vec<_>>();
    let kernel = bolelli_kernel_offsets(
        parameters.dx,
        parameters.sigma_exc,
        parameters.sigma_inh,
        parameters.inhibition,
    );
    let mut state = vec![0.0; n];
    let mut next = vec![0.0; n];
    let mut convolution = vec![0.0; n];
    let mut previous_period = vec![0.0; config.samples_per_period * n];
    let mut last_period = vec![0.0; config.samples_per_period * n];

    for step_index in 0..total_steps {
        let time = step_index as f64 * dt;
        let drive_phase = (frequency_lambda * time).cos();
        convolve_periodic_1d(&state, &kernel, &mut convolution);
        for index in 0..n {
            let forcing = static_profile[index]
                + config.drive_amplitude * localized_mask[index] * drive_phase;
            let derivative = -state[index] + config.mu * convolution[index] + forcing;
            next[index] = state[index] + dt * derivative;
        }
        std::mem::swap(&mut state, &mut next);

        let period_index = step_index / config.samples_per_period;
        let sample_index = step_index % config.samples_per_period;
        let target = if period_index == config.warmup_periods {
            Some(&mut previous_period)
        } else if period_index == config.warmup_periods + 1 {
            Some(&mut last_period)
        } else {
            None
        };
        if let Some(target) = target {
            let offset = sample_index * n;
            target[offset..offset + n].copy_from_slice(&state);
        }
    }

    let (mean_profile, amplitude_profile) =
        period_profiles(&last_period, config.samples_per_period, n);
    let mut metrics = period_metrics(BolelliPeriodMetricInput {
        previous_period: &previous_period,
        last_period: &last_period,
        mean_profile: &mean_profile,
        amplitude_profile: &amplitude_profile,
        localized_mask: &localized_mask,
        config,
        dt,
        total_steps,
    });
    let source_target = bolelli_pole_width_comparison(
        config,
        frequency_lambda,
        metrics.stripe_width_half_max,
        &amplitude_profile,
    );
    metrics.source_target_comparison = source_target.source_target_comparison;
    metrics.calibrated = source_target.calibrated;
    BolelliRunOutput {
        frequency_lambda,
        period,
        input_profile,
        mean_profile,
        amplitude_profile,
        metrics,
        source_target,
    }
}

fn period_profiles(
    period_states: &[f64],
    samples_per_period: usize,
    n: usize,
) -> (Vec<f64>, Vec<f64>) {
    let mut mean_profile = vec![0.0; n];
    let mut cos_projection = vec![0.0; n];
    let mut sin_projection = vec![0.0; n];
    for sample in 0..samples_per_period {
        let phase = 2.0 * PI * (sample as f64 + 1.0) / samples_per_period as f64;
        let cos_phase = phase.cos();
        let sin_phase = phase.sin();
        for index in 0..n {
            let value = period_states[sample * n + index];
            mean_profile[index] += value;
            cos_projection[index] += value * cos_phase;
            sin_projection[index] += value * sin_phase;
        }
    }
    let inv_samples = 1.0 / samples_per_period as f64;
    let amplitude_profile = (0..n)
        .map(|index| {
            mean_profile[index] *= inv_samples;
            2.0 * inv_samples * cos_projection[index].hypot(sin_projection[index])
        })
        .collect();
    (mean_profile, amplitude_profile)
}

fn period_metrics(input: BolelliPeriodMetricInput<'_>) -> BolelliPeriodMetrics {
    let periodic_residual_linf = input
        .previous_period
        .iter()
        .zip(input.last_period.iter())
        .map(|(previous, last)| (last - previous).abs())
        .fold(0.0, f64::max);
    let period_correlation = metrics::correlation(input.previous_period, input.last_period);
    let (response_phase_radians, response_amplitude) = masked_period_phase(
        input.last_period,
        input.localized_mask,
        input.config.samples_per_period,
        input.config.n,
    );
    let (stripe_width_half_max, active_fraction_half_max) =
        half_max_width(input.amplitude_profile, bolelli_dx(input.config));
    let max_abs_response = input
        .last_period
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    BolelliPeriodMetrics {
        time_step: input.dt,
        total_steps: input.total_steps,
        warmup_periods: input.config.warmup_periods,
        periodic_residual_linf,
        period_correlation,
        response_phase_radians,
        response_amplitude,
        stripe_width_half_max,
        active_fraction_half_max,
        max_abs_response,
        mean_zero_crossings: zero_crossings_1d(input.mean_profile),
        locked: periodic_residual_linf <= input.config.periodic_tolerance,
        rendered_target_coverage: true,
        diagnostic_metric_available: true,
        source_target_comparison: false,
        calibrated: false,
    }
}

fn bolelli_pole_width_comparison(
    config: &BolelliReportConfig,
    frequency_lambda: f64,
    generated_width_half_max: f64,
    amplitude_profile: &[f64],
) -> BolelliPoleWidthComparison {
    let source_parameter_match = bolelli_source_parameter_match(config);
    let lambda_in_source_range = (2.0..=100.0).contains(&frequency_lambda);
    let pole_residual_tolerance = 1.0e-8;
    let generated_width_estimate = generated_pole_convention_width(config, amplitude_profile);
    if let Some((pole, pole_residual)) = principal_pole_root(config, frequency_lambda) {
        let pole = Complex::new(pole.re, pole.im.abs());
        let target_width = 1.0 / (2.0 * pole.re);
        let pole_residual_pass = pole_residual <= pole_residual_tolerance;
        let asymptotic_width =
            dominant_inhibitory_width_approximation(config, frequency_lambda, pole.im.signum());
        let asymptotic_relative_width_error = asymptotic_width
            .map(|width| (width - target_width).abs() / target_width.abs().max(1.0e-12));
        let source_width_convention_accepted = source_parameter_match
            && lambda_in_source_range
            && pole_residual_pass
            && target_width.is_finite()
            && target_width > 0.0;
        let generated_width_comparable =
            source_width_convention_accepted && generated_width_estimate.fit_pass;
        let absolute_width_error = generated_width_estimate
            .width
            .filter(|_| generated_width_comparable)
            .map(|width| (width - target_width).abs());
        let relative_width_error =
            absolute_width_error.map(|error| error / target_width.abs().max(1.0e-12));
        BolelliPoleWidthComparison {
            source_target_kind: "equation-derived principal-pole width",
            source_target_reference:
                "Bolelli-Prandi 2025 Section 4.3.1 / Proposition 4.17; no source figure data",
            pole_equation: "1 +/- i*lambda = omega_hat(z)",
            fourier_convention: "source convention: omega_hat(z)=exp(-2*pi^2*sigma_exc^2*z^2)-k*exp(-2*pi^2*sigma_inh^2*z^2)",
            source_parameter_set: "Figure 5 source families with k=1 and (sigma_exc,sigma_inh) in {(0.2,0.4),(1,2),(0.4,0.8)}/sqrt(2*pi)",
            source_parameter_match,
            source_lambda_range: [2.0, 100.0],
            lambda_in_source_range,
            accepted_width_convention:
                "source-side vertical-stripe width is 1/(2*Re z0(lambda)) from the principal pole in P1+",
            accepted_width_residual_kind:
                "complex principal-pole equation residual under the paper Fourier convention",
            accepted_width_residual: Some(pole_residual),
            accepted_width_residual_tolerance: pole_residual_tolerance,
            source_width_convention_accepted,
            pole_residual_tolerance,
            pole_residual_pass,
            pole_real: Some(pole.re),
            pole_imaginary: Some(pole.im),
            pole_residual: Some(pole_residual),
            target_width_principal_pole: Some(target_width),
            asymptotic_width_principal_pole: asymptotic_width,
            asymptotic_relative_width_error,
            generated_width_half_max,
            generated_width_pole_convention: generated_width_estimate.width,
            generated_width_decay_rate: generated_width_estimate.decay_rate,
            generated_width_fit_r_squared: generated_width_estimate.fit_r_squared,
            generated_width_fit_points: generated_width_estimate.fit_points,
            generated_width_fit_min_r_squared: GENERATED_DECAY_WIDTH_MIN_R_SQUARED,
            generated_width_comparison_convention: "generated decay-width estimate uses the source pole convention width=1/(2*alpha), where alpha is fitted from the generated unforced-side amplitude envelope; generated half-max support remains an auxiliary finite-domain renderer metric",
            generated_width_comparable,
            generated_width_residual_status: if generated_width_comparable {
                "diagnostic-comparable: generated decay-width estimate shares the source pole-width convention, but calibration remains disallowed without source-figure residuals"
            } else {
                "not-accepted: generated decay-width fit did not meet the source-convention diagnostic quality gate"
            },
            absolute_width_error,
            relative_width_error,
            source_target_comparison: true,
            calibration_claim_allowed: false,
            calibrated: false,
            status: if source_width_convention_accepted {
                "accepted-source-pole-width-convention"
            } else {
                "source-pole-target-outside-validation-window"
            },
            note: "Accepts the source-side pole equation and width convention. The generated decay-width estimate now uses the same width convention when the envelope fit passes; the generated half-max support remains auxiliary and no calibration claim is allowed.",
        }
    } else {
        BolelliPoleWidthComparison {
            source_target_kind: "equation-derived principal-pole width",
            source_target_reference:
                "Bolelli-Prandi 2025 Section 4.3.1 / Proposition 4.17; no source figure data",
            pole_equation: "1 +/- i*lambda = omega_hat(z)",
            fourier_convention: "source convention: omega_hat(z)=exp(-2*pi^2*sigma_exc^2*z^2)-k*exp(-2*pi^2*sigma_inh^2*z^2)",
            source_parameter_set: "Figure 5 source families with k=1 and (sigma_exc,sigma_inh) in {(0.2,0.4),(1,2),(0.4,0.8)}/sqrt(2*pi)",
            source_parameter_match,
            source_lambda_range: [2.0, 100.0],
            lambda_in_source_range,
            accepted_width_convention:
                "source-side vertical-stripe width is 1/(2*Re z0(lambda)) from the principal pole in P1+",
            accepted_width_residual_kind:
                "complex principal-pole equation residual under the paper Fourier convention",
            accepted_width_residual: None,
            accepted_width_residual_tolerance: pole_residual_tolerance,
            source_width_convention_accepted: false,
            pole_residual_tolerance,
            pole_residual_pass: false,
            pole_real: None,
            pole_imaginary: None,
            pole_residual: None,
            target_width_principal_pole: None,
            asymptotic_width_principal_pole: None,
            asymptotic_relative_width_error: None,
            generated_width_half_max,
            generated_width_pole_convention: generated_width_estimate.width,
            generated_width_decay_rate: generated_width_estimate.decay_rate,
            generated_width_fit_r_squared: generated_width_estimate.fit_r_squared,
            generated_width_fit_points: generated_width_estimate.fit_points,
            generated_width_fit_min_r_squared: GENERATED_DECAY_WIDTH_MIN_R_SQUARED,
            generated_width_comparison_convention: "generated decay-width estimate uses the source pole convention width=1/(2*alpha), where alpha is fitted from the generated unforced-side amplitude envelope; generated half-max support remains an auxiliary finite-domain renderer metric",
            generated_width_comparable: false,
            generated_width_residual_status:
                "not-accepted: source principal-pole target was unresolved",
            absolute_width_error: None,
            relative_width_error: None,
            source_target_comparison: false,
            calibration_claim_allowed: false,
            calibrated: false,
            status: "root-unresolved",
            note: "The report generated the periodic state, but the principal-pole root finder did not converge for this frequency.",
        }
    }
}

fn bolelli_source_parameter_match(config: &BolelliReportConfig) -> bool {
    if (config.inhibition - 1.0).abs() > 1.0e-10 {
        return false;
    }
    let source_pairs = [(0.2, 0.4), (1.0, 2.0), (0.4, 0.8)];
    source_pairs.iter().any(|(exc, inh)| {
        (config.sigma_exc - source_sigma(*exc)).abs() <= 1.0e-10
            && (config.sigma_inh - source_sigma(*inh)).abs() <= 1.0e-10
    })
}

fn source_sigma(scale: f64) -> f64 {
    scale / (2.0 * PI).sqrt()
}

fn dominant_inhibitory_width_approximation(
    config: &BolelliReportConfig,
    frequency_lambda: f64,
    imaginary_sign: f64,
) -> Option<f64> {
    dominant_inhibitory_pole_approximation(config, frequency_lambda, imaginary_sign)
        .filter(|pole| pole.re > 1.0e-12)
        .map(|pole| 1.0 / (2.0 * pole.re))
}

fn dominant_inhibitory_pole_approximation(
    config: &BolelliReportConfig,
    frequency_lambda: f64,
    imaginary_sign: f64,
) -> Option<Complex> {
    if config.inhibition <= 0.0 || !config.inhibition.is_finite() {
        return None;
    }
    let target = Complex::new(-1.0, -imaginary_sign.signum() * frequency_lambda)
        .scale(1.0 / config.inhibition);
    let log_target = Complex::new(target.abs().ln(), target.im.atan2(target.re));
    let z_squared = (-log_target).scale(1.0 / source_fourier_coefficient(config.sigma_inh));
    let root = z_squared.sqrt();
    if root.is_finite() {
        Some(Complex::new(root.re.abs(), root.im.abs()))
    } else {
        None
    }
}

fn principal_pole_root(
    config: &BolelliReportConfig,
    frequency_lambda: f64,
) -> Option<(Complex, f64)> {
    let mut roots = Vec::new();
    let y_max = (8.0 + 1.6 * frequency_lambda.sqrt()).max(24.0);
    let x_min: f64 = 0.005;
    let x_max: f64 = 12.0;
    let x_samples = 42usize;
    let y_samples = 86usize;

    for sign in [-1.0, 1.0] {
        let target = Complex::new(1.0, sign * frequency_lambda);
        let mut seeds = Vec::new();
        if let Some(approx_seed) =
            dominant_inhibitory_pole_approximation(config, frequency_lambda, sign)
        {
            seeds.push((
                approx_seed,
                bolelli_pole_residual(config, approx_seed, target),
            ));
            let reflected_seed = Complex::new(approx_seed.re, -approx_seed.im);
            seeds.push((
                reflected_seed,
                bolelli_pole_residual(config, reflected_seed, target),
            ));
        }
        for xi in 0..x_samples {
            let fraction = xi as f64 / x_samples.saturating_sub(1).max(1) as f64;
            let x = x_min * (x_max / x_min).powf(fraction);
            for yi in 0..y_samples {
                let y_fraction = yi as f64 / y_samples.saturating_sub(1).max(1) as f64;
                let y = -y_max + 2.0 * y_max * y_fraction;
                let seed = Complex::new(x, y);
                let residual = bolelli_pole_residual(config, seed, target);
                if residual.is_finite() {
                    seeds.push((seed, residual));
                }
            }
        }
        seeds.sort_by(|left, right| left.1.total_cmp(&right.1));
        for (seed, _) in seeds.into_iter().take(12) {
            if let Some((root, residual)) = refine_bolelli_pole(config, target, seed) {
                roots.push((root, residual));
            }
        }
    }

    roots
        .into_iter()
        .filter(|(root, residual)| {
            root.re > 1.0e-7 && root.re.is_finite() && root.im.is_finite() && *residual < 1.0e-6
        })
        .min_by(|left, right| {
            left.0
                .re
                .total_cmp(&right.0.re)
                .then_with(|| left.1.total_cmp(&right.1))
        })
}

fn refine_bolelli_pole(
    config: &BolelliReportConfig,
    target: Complex,
    seed: Complex,
) -> Option<(Complex, f64)> {
    let mut z = seed;
    for _ in 0..50 {
        let value = bolelli_kernel_hat(config, z) - target;
        let residual = value.abs();
        if residual < 1.0e-10 {
            return Some((z, residual));
        }
        let derivative = bolelli_kernel_hat_derivative(config, z);
        if derivative.abs() < 1.0e-14 {
            return None;
        }
        let step = value / derivative;
        z = z - step;
        if !z.is_finite() || z.re <= 0.0 || z.abs() > 1.0e4 {
            return None;
        }
        if step.abs() < 1.0e-11 {
            break;
        }
    }
    let residual = bolelli_pole_residual(config, z, target);
    if residual.is_finite() {
        Some((z, residual))
    } else {
        None
    }
}

fn bolelli_pole_residual(config: &BolelliReportConfig, z: Complex, target: Complex) -> f64 {
    (bolelli_kernel_hat(config, z) - target).abs()
}

fn bolelli_kernel_hat(config: &BolelliReportConfig, z: Complex) -> Complex {
    let exc = gaussian_hat_complex(config.sigma_exc, z);
    let inh = gaussian_hat_complex(config.sigma_inh, z);
    exc - inh.scale(config.inhibition)
}

fn bolelli_kernel_hat_derivative(config: &BolelliReportConfig, z: Complex) -> Complex {
    let exc = gaussian_hat_complex(config.sigma_exc, z);
    let inh = gaussian_hat_complex(config.sigma_inh, z);
    let exc_derivative = z.scale(-2.0 * source_fourier_coefficient(config.sigma_exc)) * exc;
    let inh_derivative = z.scale(-2.0 * source_fourier_coefficient(config.sigma_inh)) * inh;
    exc_derivative - inh_derivative.scale(config.inhibition)
}

fn gaussian_hat_complex(sigma: f64, z: Complex) -> Complex {
    (z * z).scale(-source_fourier_coefficient(sigma)).exp()
}

fn source_fourier_coefficient(sigma: f64) -> f64 {
    2.0 * PI * PI * sigma * sigma
}

fn masked_period_phase(
    period_states: &[f64],
    localized_mask: &[f64],
    samples_per_period: usize,
    n: usize,
) -> (f64, f64) {
    let mask_weight = localized_mask.iter().sum::<f64>().max(1.0e-12);
    let mut cos_projection = 0.0;
    let mut sin_projection = 0.0;
    for sample in 0..samples_per_period {
        let phase = 2.0 * PI * (sample as f64 + 1.0) / samples_per_period as f64;
        let masked_mean = (0..n)
            .map(|index| period_states[sample * n + index] * localized_mask[index])
            .sum::<f64>()
            / mask_weight;
        cos_projection += masked_mean * phase.cos();
        sin_projection += masked_mean * phase.sin();
    }
    let scale = 2.0 / samples_per_period as f64;
    (
        sin_projection.atan2(cos_projection),
        scale * cos_projection.hypot(sin_projection),
    )
}

fn half_max_width(values: &[f64], dx: f64) -> (f64, f64) {
    let max_value = values.iter().copied().fold(0.0, f64::max);
    if max_value <= 1.0e-12 {
        return (0.0, 0.0);
    }
    let threshold = 0.5 * max_value;
    let active_count = values.iter().filter(|value| **value >= threshold).count();
    (
        active_count as f64 * dx,
        active_count as f64 / values.len().max(1) as f64,
    )
}

fn generated_pole_convention_width(
    config: &BolelliReportConfig,
    amplitude_profile: &[f64],
) -> BolelliGeneratedWidthEstimate {
    let x_values = bolelli_x_values(config);
    let dx = bolelli_dx(config);
    let max_amplitude = amplitude_profile.iter().copied().fold(0.0, f64::max);
    if max_amplitude <= 1.0e-12 || x_values.len() != amplitude_profile.len() {
        return empty_generated_width_estimate(0);
    }

    let fit_window_max = (config.domain_max - config.domain_min).abs() * 0.30;
    let candidates = x_values
        .iter()
        .zip(amplitude_profile.iter())
        .filter_map(|(x, amplitude)| {
            if *x >= dx
                && *x <= fit_window_max
                && amplitude.is_finite()
                && *amplitude > max_amplitude * 1.0e-4
            {
                Some((*x, *amplitude))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if candidates.len() < GENERATED_DECAY_WIDTH_MIN_POINTS {
        return empty_generated_width_estimate(candidates.len());
    }

    let baseline = candidates
        .iter()
        .map(|(_, amplitude)| *amplitude)
        .fold(f64::INFINITY, f64::min)
        .min(max_amplitude * 0.05);
    let points = candidates
        .iter()
        .filter_map(|(x, amplitude)| {
            let adjusted = amplitude - baseline;
            if adjusted > max_amplitude * 1.0e-5 {
                Some((*x, adjusted.ln()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if points.len() < GENERATED_DECAY_WIDTH_MIN_POINTS {
        return empty_generated_width_estimate(points.len());
    }

    let count = points.len() as f64;
    let mean_x = points.iter().map(|(x, _)| *x).sum::<f64>() / count;
    let mean_y = points.iter().map(|(_, y)| *y).sum::<f64>() / count;
    let variance_x = points
        .iter()
        .map(|(x, _)| (x - mean_x).powi(2))
        .sum::<f64>();
    if variance_x <= 1.0e-12 {
        return empty_generated_width_estimate(points.len());
    }
    let covariance_xy = points
        .iter()
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>();
    let slope = covariance_xy / variance_x;
    let intercept = mean_y - slope * mean_x;
    let total_sum_squares = points
        .iter()
        .map(|(_, y)| (y - mean_y).powi(2))
        .sum::<f64>();
    let residual_sum_squares = points
        .iter()
        .map(|(x, y)| (y - (intercept + slope * x)).powi(2))
        .sum::<f64>();
    let r_squared = if total_sum_squares > 1.0e-12 {
        1.0 - residual_sum_squares / total_sum_squares
    } else {
        0.0
    };
    let decay_rate = -slope;
    let width = if decay_rate.is_finite() && decay_rate > 1.0e-12 {
        Some(1.0 / (2.0 * decay_rate))
    } else {
        None
    };
    let fit_pass = width.is_some()
        && r_squared.is_finite()
        && r_squared >= GENERATED_DECAY_WIDTH_MIN_R_SQUARED
        && points.len() >= GENERATED_DECAY_WIDTH_MIN_POINTS;
    BolelliGeneratedWidthEstimate {
        width,
        decay_rate: Some(decay_rate),
        fit_r_squared: Some(r_squared),
        fit_points: points.len(),
        fit_pass,
    }
}

fn empty_generated_width_estimate(fit_points: usize) -> BolelliGeneratedWidthEstimate {
    BolelliGeneratedWidthEstimate {
        width: None,
        decay_rate: None,
        fit_r_squared: None,
        fit_points,
        fit_pass: false,
    }
}

fn zero_crossings_1d(values: &[f64]) -> f64 {
    values
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count() as f64
}

fn convolve_periodic_1d(state: &[f64], kernel: &[(isize, f64)], out: &mut [f64]) {
    let n = state.len() as isize;
    for (index, target) in out.iter_mut().enumerate() {
        let center = index as isize;
        let mut sum = 0.0;
        for (offset, weight) in kernel {
            let source = (center + offset).rem_euclid(n) as usize;
            sum += weight * state[source];
        }
        *target = sum;
    }
}

fn bolelli_kernel_offsets(
    dx: f64,
    sigma_exc: f64,
    sigma_inh: f64,
    inhibition: f64,
) -> Vec<(isize, f64)> {
    let radius = ((4.0 * sigma_inh.max(sigma_exc)) / dx).ceil().max(1.0) as isize;
    (-radius..=radius)
        .map(|offset| {
            let x = offset as f64 * dx;
            let weight = (gaussian_1d(x, sigma_exc) - inhibition * gaussian_1d(x, sigma_inh)) * dx;
            (offset, weight)
        })
        .collect()
}

fn gaussian_1d(x: f64, sigma: f64) -> f64 {
    let norm = 1.0 / ((2.0 * PI).sqrt() * sigma);
    norm * (-0.5 * (x / sigma).powi(2)).exp()
}

fn bolelli_profile_thumbnail(profile: &[f64]) -> BolelliProfileThumbnail {
    let (_, _, min, max) = metrics::stats(profile);
    let denom = (max - min).max(1.0e-12);
    let bytes = profile
        .iter()
        .map(|value| (((value - min) / denom) * 255.0).clamp(0.0, 255.0) as u8)
        .collect::<Vec<_>>();
    BolelliProfileThumbnail {
        format: "u8-profile-v1",
        encoding: "base64",
        color_space: "normalized-luma",
        width: profile.len(),
        height: 1,
        scale_min: min,
        scale_max: max,
        data_base64: general_purpose::STANDARD.encode(bytes),
    }
}

fn bolelli_x_values(config: &BolelliReportConfig) -> Vec<f64> {
    let dx = bolelli_dx(config);
    (0..config.n)
        .map(|index| config.domain_min + index as f64 * dx)
        .collect()
}

fn bolelli_dx(config: &BolelliReportConfig) -> f64 {
    (config.domain_max - config.domain_min) / (config.n.saturating_sub(1).max(1) as f64)
}

#[derive(Clone, Copy, Debug)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    fn scale(self, scale: f64) -> Self {
        Self::new(scale * self.re, scale * self.im)
    }

    fn exp(self) -> Self {
        let magnitude = self.re.exp();
        Self::new(magnitude * self.im.cos(), magnitude * self.im.sin())
    }

    fn sqrt(self) -> Self {
        let magnitude = self.abs();
        let re = ((magnitude + self.re) * 0.5).sqrt();
        let im = self.im.signum() * ((magnitude - self.re) * 0.5).sqrt();
        Self::new(re, im)
    }
}

impl Sub for Complex {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl Div for Complex {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let denom = rhs.re * rhs.re + rhs.im * rhs.im;
        Self::new(
            (self.re * rhs.re + self.im * rhs.im) / denom,
            (self.im * rhs.re - self.re * rhs.im) / denom,
        )
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

fn heaviside(value: f64) -> f64 {
    if value >= 0.0 {
        1.0
    } else {
        0.0
    }
}
