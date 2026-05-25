use std::{
    error::Error,
    ops::{Div, Mul, Sub},
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
            note: "Generated frequency-sweep row with an equation-derived principal-pole width comparison; calibration thresholds remain deferred.",
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
        format: "bolelli-time-periodic-input-report-v2",
        model_family: MODEL_FAMILY_LOCALIZED_PERIODIC,
        source_key: "bolelli-prandi-2025",
        status: "generated-first-pass-diagnostic",
        note: "Generated Bolelli-Prandi-style localized time-periodic input diagnostics. This report checks period locking, phase, and generated stripe-width metrics; it is not a source-figure reproduction claim.",
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
        note: "Uses generated profiles, numeric period metrics, and an equation-derived pole-width target only; source figure comparison remains deferred.",
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
    let source_target =
        bolelli_pole_width_comparison(config, frequency_lambda, metrics.stripe_width_half_max);
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
) -> BolelliPoleWidthComparison {
    if let Some((pole, pole_residual)) = principal_pole_root(config, frequency_lambda) {
        let target_width = 1.0 / (2.0 * pole.re);
        let absolute_width_error = (generated_width_half_max - target_width).abs();
        BolelliPoleWidthComparison {
            source_target_kind: "equation-derived principal-pole width",
            source_target_reference:
                "Bolelli-Prandi 2025 principal-pole relation; no source figure data",
            pole_equation: "1 +/- i*lambda = omega_hat(z)",
            pole_real: Some(pole.re),
            pole_imaginary: Some(pole.im),
            pole_residual: Some(pole_residual),
            target_width_principal_pole: Some(target_width),
            generated_width_half_max,
            absolute_width_error: Some(absolute_width_error),
            relative_width_error: Some(absolute_width_error / target_width.abs().max(1.0e-12)),
            source_target_comparison: true,
            calibrated: false,
            status: "equation-derived-target",
            note: "Compares the generated half-max width with the principal-pole asymptotic width. This is a source-target diagnostic, not a calibrated source-figure match.",
        }
    } else {
        BolelliPoleWidthComparison {
            source_target_kind: "equation-derived principal-pole width",
            source_target_reference:
                "Bolelli-Prandi 2025 principal-pole relation; no source figure data",
            pole_equation: "1 +/- i*lambda = omega_hat(z)",
            pole_real: None,
            pole_imaginary: None,
            pole_residual: None,
            target_width_principal_pole: None,
            generated_width_half_max,
            absolute_width_error: None,
            relative_width_error: None,
            source_target_comparison: false,
            calibrated: false,
            status: "root-unresolved",
            note: "The report generated the periodic state, but the principal-pole root finder did not converge for this frequency.",
        }
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
    let exc_derivative = z.scale(-config.sigma_exc * config.sigma_exc) * exc;
    let inh_derivative = z.scale(-config.sigma_inh * config.sigma_inh) * inh;
    exc_derivative - inh_derivative.scale(config.inhibition)
}

fn gaussian_hat_complex(sigma: f64, z: Complex) -> Complex {
    (z * z).scale(-0.5 * sigma * sigma).exp()
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

fn heaviside(value: f64) -> f64 {
    if value >= 0.0 {
        1.0
    } else {
        0.0
    }
}
