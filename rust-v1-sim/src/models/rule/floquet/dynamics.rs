use crate::models::rule::{
    rule_gaussian_kernel, rule_rest_state, rule_sigmoid, rule_stimulus, RuleGaussianKernel,
};
use crate::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct RuleOrbitStep {
    dt: f64,
    time_ms: f64,
    ue: f64,
    ui: f64,
}

pub(crate) fn rule_floquet_report(params: FrameParams, mode_cycles: &[f64]) -> RuleFloquetReport {
    let (orbit_params, orbit, steps) = rule_floquet_orbit_steps(params);
    let modes = mode_cycles
        .iter()
        .map(|cycles| rule_floquet_mode(orbit_params, &steps, *cycles))
        .collect::<Vec<_>>();
    let strongest_mode = modes
        .iter()
        .copied()
        .max_by(|a, b| a.max_abs_multiplier.total_cmp(&b.max_abs_multiplier))
        .unwrap_or(RuleFloquetMode {
            beta_cycles: 0.0,
            wave_number_radians: 0.0,
            multiplier_1_real: 0.0,
            multiplier_1_imag: 0.0,
            multiplier_2_real: 0.0,
            multiplier_2_imag: 0.0,
            max_abs_multiplier: 0.0,
            monodromy_trace: 0.0,
            monodromy_determinant: 0.0,
            plus_condition: 1.0,
            minus_condition: 1.0,
            determinant_condition: 1.0,
            crossing_hint: "no-modes",
        });
    let plus_crossing_modes = modes
        .iter()
        .filter(|mode| mode.crossing_hint == "plus_one_to_one")
        .map(|mode| mode.beta_cycles)
        .collect();
    let minus_crossing_modes = modes
        .iter()
        .filter(|mode| mode.crossing_hint == "minus_period_doubling")
        .map(|mode| mode.beta_cycles)
        .collect();

    RuleFloquetReport {
        period_ms: orbit_params.rule_stim_period_ms,
        amplitude: orbit_params.rule_stim_amplitude,
        stim_i_fraction: orbit_params.rule_stim_i_fraction,
        orbit,
        modes,
        strongest_mode,
        plus_crossing_modes,
        minus_crossing_modes,
    }
}

pub(super) fn rule_floquet_orbit_steps(
    params: FrameParams,
) -> (FrameParams, RuleOrbitSummary, Vec<RuleOrbitStep>) {
    let mut orbit_params = params;
    orbit_params.rule_seed_strength = 0.0;
    orbit_params.n = orbit_params.n.max(32);
    let period = orbit_params.rule_stim_period_ms.max(1.0e-9);
    let dt = orbit_params.preview_step.min(0.1).clamp(0.02, 0.1);
    let (mut ue, mut ui) = rule_rest_state(orbit_params);
    let mut time_ms = 0.0;
    let warmup_end = period * 14.0;
    while time_ms + 1.0e-12 < warmup_end {
        let step = dt.min(warmup_end - time_ms);
        (ue, ui) = rule_homogeneous_step(orbit_params, ue, ui, time_ms, step);
        time_ms += step;
    }

    let mut steps = Vec::new();
    let orbit_end = warmup_end + period;
    while time_ms + 1.0e-12 < orbit_end {
        let step = dt.min(orbit_end - time_ms);
        steps.push(RuleOrbitStep {
            dt: step,
            time_ms,
            ue,
            ui,
        });
        (ue, ui) = rule_homogeneous_step(orbit_params, ue, ui, time_ms, step);
        time_ms += step;
    }

    let orbit = summarize_rule_orbit(period, &steps);
    (orbit_params, orbit, steps)
}

pub(crate) fn rule_floquet_grid_point_for(
    params: FrameParams,
    mode_cycles: &[f64],
) -> RuleFloquetGridPoint {
    let report = rule_floquet_report(params, mode_cycles);
    let plus_margin = max_floquet_margin(&report.modes, "plus_one_to_one");
    let minus_margin = max_floquet_margin(&report.modes, "minus_period_doubling");
    let complex_margin = max_floquet_margin(&report.modes, "unstable_complex");
    let crossing_hint = if minus_margin > 0.0 {
        "minus_period_doubling"
    } else if plus_margin > 0.0 {
        "plus_one_to_one"
    } else if complex_margin > 0.0 {
        "unstable_complex"
    } else {
        "stable"
    };

    RuleFloquetGridPoint {
        period_ms: report.period_ms,
        amplitude: report.amplitude,
        stim_i_fraction: report.stim_i_fraction,
        dominant_beta_cycles: report.strongest_mode.beta_cycles,
        max_abs_multiplier: report.strongest_mode.max_abs_multiplier,
        crossing_hint,
        plus_margin,
        minus_margin,
        complex_margin,
        orbit: report.orbit,
        modes: report.modes,
    }
}

fn max_floquet_margin(modes: &[RuleFloquetMode], kind: &'static str) -> f64 {
    modes
        .iter()
        .map(|mode| floquet_mode_margin(mode, kind))
        .fold(f64::NEG_INFINITY, f64::max)
}

pub(crate) fn floquet_mode_margin(mode: &RuleFloquetMode, kind: &'static str) -> f64 {
    match kind {
        "minus_period_doubling" => -mode.minus_condition,
        "plus_one_to_one" => -mode.plus_condition,
        "unstable_complex" => -mode.determinant_condition,
        _ => mode.max_abs_multiplier - 1.0,
    }
}

fn rule_homogeneous_step(
    params: FrameParams,
    ue: f64,
    ui: f64,
    time_ms: f64,
    dt: f64,
) -> (f64, f64) {
    let (input_e, input_i) = rule_homogeneous_inputs(params, ue, ui, time_ms);
    let target_e = rule_sigmoid(input_e);
    let target_i = rule_sigmoid(input_i);
    (
        (ue + (dt / params.rule_tau_e_ms) * (-ue + target_e)).clamp(0.0, 1.0),
        (ui + (dt / params.rule_tau_i_ms) * (-ui + target_i)).clamp(0.0, 1.0),
    )
}

fn rule_homogeneous_inputs(params: FrameParams, ue: f64, ui: f64, time_ms: f64) -> (f64, f64) {
    let stim = params.rule_stim_amplitude * rule_stimulus(params, time_ms);
    (
        params.rule_aee * ue - params.rule_aie * ui - params.rule_theta_e + stim,
        params.rule_aei * ue - params.rule_aii * ui - params.rule_theta_i
            + params.rule_stim_i_fraction * stim,
    )
}

fn summarize_rule_orbit(period_ms: f64, steps: &[RuleOrbitStep]) -> RuleOrbitSummary {
    if steps.is_empty() {
        return RuleOrbitSummary {
            period_ms,
            samples: 0,
            e_min: 0.0,
            e_max: 0.0,
            e_mean: 0.0,
            i_min: 0.0,
            i_max: 0.0,
            i_mean: 0.0,
        };
    }
    let mut e_min = f64::INFINITY;
    let mut e_max = f64::NEG_INFINITY;
    let mut e_sum = 0.0;
    let mut i_min = f64::INFINITY;
    let mut i_max = f64::NEG_INFINITY;
    let mut i_sum = 0.0;
    for step in steps {
        e_min = e_min.min(step.ue);
        e_max = e_max.max(step.ue);
        e_sum += step.ue;
        i_min = i_min.min(step.ui);
        i_max = i_max.max(step.ui);
        i_sum += step.ui;
    }
    RuleOrbitSummary {
        period_ms,
        samples: steps.len(),
        e_min,
        e_max,
        e_mean: e_sum / steps.len() as f64,
        i_min,
        i_max,
        i_mean: i_sum / steps.len() as f64,
    }
}

pub(super) fn rule_floquet_mode(
    params: FrameParams,
    steps: &[RuleOrbitStep],
    beta_cycles: f64,
) -> RuleFloquetMode {
    let gain_e = rule_kernel_mode_gain(params.rule_sigma_e, beta_cycles, 0.0, params.n);
    let gain_i = rule_kernel_mode_gain(params.rule_sigma_i, beta_cycles, 0.0, params.n);
    let mut m00 = 1.0;
    let mut m01 = 0.0;
    let mut m10 = 0.0;
    let mut m11 = 1.0;

    for step in steps {
        let (input_e, input_i) = rule_homogeneous_inputs(params, step.ue, step.ui, step.time_ms);
        let slope_e = rule_sigmoid_derivative(input_e);
        let slope_i = rule_sigmoid_derivative(input_i);
        let j00 = (-1.0 + slope_e * params.rule_aee * gain_e) / params.rule_tau_e_ms;
        let j01 = (-slope_e * params.rule_aie * gain_i) / params.rule_tau_e_ms;
        let j10 = (slope_i * params.rule_aei * gain_e) / params.rule_tau_i_ms;
        let j11 = (-1.0 - slope_i * params.rule_aii * gain_i) / params.rule_tau_i_ms;
        let a00 = 1.0 + step.dt * j00;
        let a01 = step.dt * j01;
        let a10 = step.dt * j10;
        let a11 = 1.0 + step.dt * j11;
        let next00 = a00 * m00 + a01 * m10;
        let next01 = a00 * m01 + a01 * m11;
        let next10 = a10 * m00 + a11 * m10;
        let next11 = a10 * m01 + a11 * m11;
        m00 = next00;
        m01 = next01;
        m10 = next10;
        m11 = next11;
    }

    floquet_mode_from_matrix(
        beta_cycles,
        rule_wave_number_for_cycles(beta_cycles, params.n),
        m00,
        m01,
        m10,
        m11,
    )
}

pub(crate) fn floquet_mode_from_matrix(
    beta_cycles: f64,
    wave_number_radians: f64,
    m00: f64,
    m01: f64,
    m10: f64,
    m11: f64,
) -> RuleFloquetMode {
    let trace = m00 + m11;
    let determinant = m00 * m11 - m01 * m10;
    let discriminant = trace * trace - 4.0 * determinant;
    let (l1_real, l1_imag, l2_real, l2_imag) = if discriminant >= 0.0 {
        let root = discriminant.sqrt();
        ((trace + root) * 0.5, 0.0, (trace - root) * 0.5, 0.0)
    } else {
        let real = trace * 0.5;
        let imag = (-discriminant).sqrt() * 0.5;
        (real, imag, real, -imag)
    };
    let abs_1 = (l1_real * l1_real + l1_imag * l1_imag).sqrt();
    let abs_2 = (l2_real * l2_real + l2_imag * l2_imag).sqrt();
    let plus_condition = 1.0 - trace + determinant;
    let minus_condition = 1.0 + trace + determinant;
    let determinant_condition = 1.0 - determinant;
    let crossing_hint = if minus_condition < 0.0 {
        "minus_period_doubling"
    } else if plus_condition < 0.0 {
        "plus_one_to_one"
    } else if determinant_condition < 0.0 {
        "unstable_complex"
    } else {
        "stable"
    };
    RuleFloquetMode {
        beta_cycles,
        wave_number_radians,
        multiplier_1_real: l1_real,
        multiplier_1_imag: l1_imag,
        multiplier_2_real: l2_real,
        multiplier_2_imag: l2_imag,
        max_abs_multiplier: abs_1.max(abs_2),
        monodromy_trace: trace,
        monodromy_determinant: determinant,
        plus_condition,
        minus_condition,
        determinant_condition,
        crossing_hint,
    }
}

fn rule_sigmoid_derivative(input: f64) -> f64 {
    let value = rule_sigmoid(input);
    value * (1.0 - value)
}

fn rule_kernel_mode_gain(sigma: f64, beta_cycles: f64, angle: f64, n: usize) -> f64 {
    let kernel = rule_gaussian_kernel(sigma);
    let q = rule_wave_number_for_cycles(beta_cycles, n);
    let qx = q * angle.cos();
    let qy = q * angle.sin();
    rule_kernel_1d_gain(&kernel, qx) * rule_kernel_1d_gain(&kernel, qy)
}

pub(crate) fn rule_wave_number_for_cycles(beta_cycles: f64, n: usize) -> f64 {
    2.0 * PI * beta_cycles / n as f64
}

fn rule_kernel_1d_gain(kernel: &RuleGaussianKernel, q: f64) -> f64 {
    kernel
        .weights
        .iter()
        .enumerate()
        .map(|(index, weight)| {
            let offset = index as isize - kernel.radius as isize;
            weight * (q * offset as f64).cos()
        })
        .sum()
}
