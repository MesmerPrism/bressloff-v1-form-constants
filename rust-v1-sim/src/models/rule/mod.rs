pub(crate) mod fit;
pub(crate) mod floquet;
pub(crate) mod presets;
pub(crate) mod reports;
pub(crate) mod sweep;

use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct RuleGaussianKernel {
    pub(crate) radius: usize,
    pub(crate) weights: Vec<f64>,
}

pub(crate) fn simulate_rule_flicker_frames(params: FrameParams) -> (Vec<f32>, Vec<f64>) {
    let frame_count = params.frames.max(1);
    let frame_size = params.n * params.n;
    let mut frames = Vec::with_capacity(frame_count * frame_size);
    let mut times = Vec::with_capacity(frame_count);
    let (mut ue, mut ui) = initialize_rule_state(params);
    let kernel_e = rule_gaussian_kernel(params.rule_sigma_e);
    let kernel_i = rule_gaussian_kernel(params.rule_sigma_i);
    let mut tmp_e = vec![0.0; frame_size];
    let mut tmp_i = vec![0.0; frame_size];
    let mut conv_e = vec![0.0; frame_size];
    let mut conv_i = vec![0.0; frame_size];
    let mut next_e = vec![0.0; frame_size];
    let mut next_i = vec![0.0; frame_size];
    let step = match params.solver {
        Solver::Preview => params.preview_step,
        Solver::Accurate => params.preview_step.min(0.1),
    }
    .clamp(0.02, 1.0);
    let mut current_t = 0.0;

    for frame_index in 0..frame_count {
        let target_t = if frame_count <= 1 {
            0.0
        } else {
            params.t * frame_index as f64 / (frame_count - 1) as f64
        };

        while current_t + 1.0e-12 < target_t {
            let dt = step.min(target_t - current_t);
            convolve_periodic_separable(&ue, params.n, &kernel_e, &mut tmp_e, &mut conv_e);
            convolve_periodic_separable(&ui, params.n, &kernel_i, &mut tmp_i, &mut conv_i);
            let stim = params.rule_stim_amplitude * rule_stimulus(params, current_t);
            for i in 0..frame_size {
                let input_e =
                    params.rule_aee * conv_e[i] - params.rule_aie * conv_i[i] - params.rule_theta_e
                        + stim;
                let input_i =
                    params.rule_aei * conv_e[i] - params.rule_aii * conv_i[i] - params.rule_theta_i
                        + params.rule_stim_i_fraction * stim;
                let target_e = rule_sigmoid(input_e);
                let target_i = rule_sigmoid(input_i);
                next_e[i] =
                    (ue[i] + (dt / params.rule_tau_e_ms) * (-ue[i] + target_e)).clamp(0.0, 1.0);
                next_i[i] =
                    (ui[i] + (dt / params.rule_tau_i_ms) * (-ui[i] + target_i)).clamp(0.0, 1.0);
            }
            std::mem::swap(&mut ue, &mut next_e);
            std::mem::swap(&mut ui, &mut next_i);
            current_t += dt;
        }

        times.push(target_t);
        frames.extend(ue.iter().map(|value| *value as f32));
    }

    (frames, times)
}

fn initialize_rule_state(params: FrameParams) -> (Vec<f64>, Vec<f64>) {
    let frame_size = params.n * params.n;
    let (base_e, base_i) = rule_rest_state(params);
    let mut rng = SplitMix64::new(params.seed);
    let mut ue = vec![base_e; frame_size];
    let mut ui = vec![base_i; frame_size];

    if params.rule_seed_strength <= 0.0 {
        return (ue, ui);
    }

    for row in 0..params.n {
        for col in 0..params.n {
            let i = row * params.n + col;
            let structured = rule_seed_value(params, row, col);
            let noise = (rng.next_f64() * 2.0 - 1.0) * 0.2;
            let perturbation = params.rule_seed_strength * (structured + noise);
            ue[i] = (base_e + perturbation).clamp(0.0, 1.0);
            ui[i] = (base_i + 0.35 * perturbation).clamp(0.0, 1.0);
        }
    }
    (ue, ui)
}

pub(crate) fn rule_rest_state(params: FrameParams) -> (f64, f64) {
    let mut ue = 0.1;
    let mut ui = 0.1;
    for _ in 0..2000 {
        let target_e =
            rule_sigmoid(params.rule_aee * ue - params.rule_aie * ui - params.rule_theta_e);
        let target_i =
            rule_sigmoid(params.rule_aei * ue - params.rule_aii * ui - params.rule_theta_i);
        ue += 0.05 * (target_e - ue);
        ui += 0.05 * (target_i - ui);
    }
    (ue, ui)
}

fn rule_seed_value(params: FrameParams, row: usize, col: usize) -> f64 {
    let x = col as f64 / params.n as f64 - 0.5;
    let y = row as f64 / params.n as f64 - 0.5;
    let cycles = if params.rule_stim_period_ms < 80.0 {
        4.0
    } else {
        5.0
    };
    let q = 2.0 * PI * cycles;
    match params.rule_seed_pattern {
        RuleSeedPattern::Random => 0.0,
        RuleSeedPattern::Stripes => (q * x).cos(),
        RuleSeedPattern::Hexagonal => {
            let a = (q * x).cos();
            let b = (q * (-0.5 * x + 0.866_025_403_784_438_6 * y)).cos();
            let c = (q * (-0.5 * x - 0.866_025_403_784_438_6 * y)).cos();
            (a + b + c) / 3.0
        }
    }
}

pub(crate) fn rule_gaussian_kernel(sigma: f64) -> RuleGaussianKernel {
    let sigma = sigma.max(0.1);
    let radius = (3.0 * sigma).ceil() as usize;
    let mut weights: Vec<f64> = (0..=2 * radius)
        .map(|i| {
            let offset = i as isize - radius as isize;
            (-(offset as f64).powi(2) / (sigma * sigma)).exp()
        })
        .collect();
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        for weight in &mut weights {
            *weight /= sum;
        }
    }
    RuleGaussianKernel { radius, weights }
}

fn convolve_periodic_separable(
    input: &[f64],
    n: usize,
    kernel: &RuleGaussianKernel,
    tmp: &mut [f64],
    output: &mut [f64],
) {
    for row in 0..n {
        for col in 0..n {
            let mut sum = 0.0;
            for (k, weight) in kernel.weights.iter().enumerate() {
                let delta = k as isize - kernel.radius as isize;
                let source_col = wrap_index(col, delta, n);
                sum += weight * input[row * n + source_col];
            }
            tmp[row * n + col] = sum;
        }
    }

    for row in 0..n {
        for col in 0..n {
            let mut sum = 0.0;
            for (k, weight) in kernel.weights.iter().enumerate() {
                let delta = k as isize - kernel.radius as isize;
                let source_row = wrap_index(row, delta, n);
                sum += weight * tmp[source_row * n + col];
            }
            output[row * n + col] = sum;
        }
    }
}

pub(crate) fn rule_stimulus(params: FrameParams, time_ms: f64) -> f64 {
    let phase = (2.0 * PI * time_ms / params.rule_stim_period_ms.max(1.0e-9)).sin()
        - params.rule_stim_threshold;
    if params.rule_stim_smoothing <= 0.0 {
        if phase > 0.0 {
            1.0
        } else {
            0.0
        }
    } else {
        rule_sigmoid(params.rule_stim_smoothing * phase)
    }
}

pub(crate) fn rule_sigmoid(x: f64) -> f64 {
    if x <= -50.0 {
        0.0
    } else if x >= 50.0 {
        1.0
    } else {
        1.0 / (1.0 + (-x).exp())
    }
}
