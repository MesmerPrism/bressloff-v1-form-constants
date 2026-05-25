use crate::*;

pub(crate) fn generate_planform_frames(
    params: FrameParams,
) -> (Vec<f32>, Vec<f64>, Option<Vec<f32>>) {
    let frame_count = params.frames.max(1);
    let frame_size = params.n * params.n;
    let mut frames = vec![0.0_f32; frame_count * frame_size];
    let mut orientation_frames = params
        .export_orientation_channels
        .then(|| vec![0.0_f32; frame_count * frame_size * params.m]);
    let cell_mm = cell_mm_for(params);
    let extent = params.n as f64 * cell_mm;
    let half = extent / 2.0;
    let stability = stability_scan(params);
    let planform_params = effective_planform_params(params, &stability);
    let wave_number = planform_wave_number(params, cell_mm, Some(&stability));
    let q = wave_number * params.hypercolumn_mm;
    let eigen = orientation_eigen_details(planform_params, q);
    let branch_selection = branch_selection(planform_params, &stability);
    let effective_pattern = effective_pattern(params, &branch_selection);
    let modes = planform_modes(planform_params, effective_pattern);
    let times: Vec<f64> = (0..frame_count)
        .map(|frame_index| {
            let progress = if frame_count <= 1 {
                0.0
            } else {
                frame_index as f64 / (frame_count - 1) as f64
            };
            params.t * progress
        })
        .collect();

    match orientation_frames.as_mut() {
        Some(channels) => frames
            .par_chunks_mut(frame_size)
            .zip(channels.par_chunks_mut(frame_size * params.m))
            .enumerate()
            .for_each(|(frame_index, (frame, channel_frame))| {
                let progress = if frame_count <= 1 {
                    0.0
                } else {
                    frame_index as f64 / (frame_count - 1) as f64
                };
                let phase = 2.0 * PI * params.drift * progress;

                for row in 0..params.n {
                    let y = (row as f64 + 0.5) * cell_mm - half;
                    for col in 0..params.n {
                        let x = (col as f64 + 0.5) * cell_mm - half;
                        let cell = row * params.n + col;
                        if planform_params.contour_mode == ContourMode::Noncontoured {
                            let value = planform_scalar_activity(x, y, wave_number, phase, &modes);
                            let output = (value * params.sharpness).tanh() as f32;
                            for k in 0..params.m {
                                channel_frame[cell * params.m + k] = output;
                            }
                            frame[cell] = output;
                            continue;
                        }

                        let mut best = 0.0_f64;
                        for k in 0..params.m {
                            let phi = PI * k as f64 / params.m as f64;
                            let value = orientation_planform_activity(
                                x,
                                y,
                                phi,
                                wave_number,
                                phase,
                                &modes,
                                &eigen,
                            );
                            channel_frame[cell * params.m + k] =
                                (value * params.sharpness).tanh() as f32;
                            if value.abs() > best.abs() {
                                best = value;
                            }
                        }
                        frame[cell] = (best * params.sharpness).tanh() as f32;
                    }
                }
            }),
        None => frames
            .par_chunks_mut(frame_size)
            .enumerate()
            .for_each(|(frame_index, frame)| {
                let progress = if frame_count <= 1 {
                    0.0
                } else {
                    frame_index as f64 / (frame_count - 1) as f64
                };
                let phase = 2.0 * PI * params.drift * progress;

                for row in 0..params.n {
                    let y = (row as f64 + 0.5) * cell_mm - half;
                    for col in 0..params.n {
                        let x = (col as f64 + 0.5) * cell_mm - half;
                        let value = planform_value(
                            planform_params,
                            x,
                            y,
                            wave_number,
                            phase,
                            &modes,
                            &eigen,
                        );
                        frame[row * params.n + col] = (value * params.sharpness).tanh() as f32;
                    }
                }
            }),
    }

    (frames, times, orientation_frames)
}

pub(crate) fn cell_mm_for(params: FrameParams) -> f64 {
    match params.generator {
        Generator::Dynamics | Generator::RuleFlicker => DYNAMIC_CELL_MM,
        Generator::Planform => (2.0 * PI * RETINO_BETA / RETINO_EPS) / params.n as f64,
    }
}

pub(crate) fn orientation_count_for(params: FrameParams) -> usize {
    match params.generator {
        Generator::RuleFlicker => 1,
        Generator::Dynamics | Generator::Planform => params.m,
    }
}

pub(crate) fn planform_details(params: FrameParams, cell_mm: f64) -> PlanformDetails {
    let stability = stability_scan(params);
    let planform_params = effective_planform_params(params, &stability);
    let wave_number = planform_wave_number(params, cell_mm, Some(&stability));
    let q = wave_number * params.hypercolumn_mm;
    let branch_selection = branch_selection(planform_params, &stability);
    let effective_pattern = effective_pattern(params, &branch_selection);
    PlanformDetails {
        contour_mode: params.contour_mode.as_str(),
        parity: planform_params.parity.as_str(),
        q,
        wave_number,
        phase_base: 2.0 * PI * params.drift,
        modes: planform_modes(planform_params, effective_pattern),
        eigen: orientation_eigen_details(planform_params, q),
        stability,
        branch_selection,
        kernel: kernel_details(params),
    }
}

pub(crate) fn effective_pattern_from_params(
    params: FrameParams,
    planform: &PlanformDetails,
) -> PatternPreset {
    match params.pattern {
        PatternPreset::Auto => match planform.branch_selection.selected_pattern {
            "honeycomb" => PatternPreset::Honeycomb,
            "hex_pi" => PatternPreset::HexPi,
            "triangle" => PatternPreset::Triangle,
            "rhombic" => PatternPreset::Rhombic,
            "spiral" => PatternPreset::Spiral,
            "rings" => PatternPreset::Rings,
            _ => PatternPreset::Cobweb,
        },
        other => other,
    }
}

pub(crate) fn pattern_family(pattern: PatternPreset) -> &'static str {
    match pattern {
        PatternPreset::Auto => "branch-selected",
        PatternPreset::Rings | PatternPreset::Rays | PatternPreset::Spiral => "roll",
        PatternPreset::Cobweb => "square",
        PatternPreset::Honeycomb | PatternPreset::HexPi | PatternPreset::Triangle => "hexagonal",
        PatternPreset::Rhombic => "rhombic",
    }
}

pub(crate) fn planform_wave_number(
    params: FrameParams,
    cell_mm: f64,
    stability: Option<&StabilityDetails>,
) -> f64 {
    if params.pattern == PatternPreset::Auto {
        let critical_q = stability
            .map(|details| details.critical_q)
            .unwrap_or_else(|| stability_scan(params).critical_q);
        return critical_q / params.hypercolumn_mm.max(1.0e-9);
    }
    let extent = params.n as f64 * cell_mm;
    2.0 * PI * params.wave_count / extent.max(1.0e-9)
}

fn effective_planform_params(mut params: FrameParams, stability: &StabilityDetails) -> FrameParams {
    if params.pattern == PatternPreset::Auto {
        params.parity = parity_from_branch(stability.critical_branch);
    }
    params
}

fn parity_from_branch(branch: &str) -> Parity {
    if branch == "odd" {
        Parity::Odd
    } else {
        Parity::Even
    }
}

fn effective_pattern(
    params: FrameParams,
    branch_selection: &BranchSelectionDetails,
) -> PatternPreset {
    if params.pattern != PatternPreset::Auto {
        return params.pattern;
    }
    match branch_selection.selected_pattern {
        "honeycomb" => PatternPreset::Honeycomb,
        "hex_pi" => PatternPreset::HexPi,
        "triangle" => PatternPreset::Triangle,
        "rhombic" => PatternPreset::Rhombic,
        "spiral" => PatternPreset::Spiral,
        "rings" => PatternPreset::Rings,
        _ => PatternPreset::Cobweb,
    }
}

fn kernel_details(params: FrameParams) -> KernelDetails {
    KernelDetails {
        local_sigma_deg: params.local_sigma_deg,
        local_wide_sigma_deg: params.local_wide_sigma_deg,
        local_inhibition: params.local_inhibition,
        lateral_sigma: params.lateral_sigma,
        lateral_wide_sigma: params.lateral_wide_sigma,
        lateral_inhibition: params.lateral_inhibition,
        lateral_spread_deg: params.lateral_spread_deg,
    }
}

fn stability_scan(params: FrameParams) -> StabilityDetails {
    let samples = params.stability_samples.max(2);
    let q_min = params.stability_q_min.min(params.stability_q_max);
    let q_max = params.stability_q_max.max(q_min + 1.0e-6);
    let mut points = Vec::with_capacity(samples);
    let mut critical_q = q_min;
    let mut critical_branch = "even";
    let mut critical_growth = f64::NEG_INFINITY;

    for i in 0..samples {
        let q = if samples <= 1 {
            q_min
        } else {
            q_min + (q_max - q_min) * i as f64 / (samples - 1) as f64
        };
        let even_growth = branch_growth(params, Parity::Even, q);
        let odd_growth = branch_growth(params, Parity::Odd, q);
        if even_growth >= critical_growth {
            critical_growth = even_growth;
            critical_q = q;
            critical_branch = "even";
        }
        if odd_growth >= critical_growth {
            critical_growth = odd_growth;
            critical_q = q;
            critical_branch = "odd";
        }
        points.push(StabilityPoint {
            q,
            even_growth,
            odd_growth,
        });
    }

    let mut branch_params = params;
    branch_params.parity = parity_from_branch(critical_branch);
    let selected_pattern =
        branch_selection_for(branch_params, critical_q, critical_growth).selected_pattern;
    StabilityDetails {
        q_min,
        q_max,
        samples,
        critical_q,
        critical_branch,
        critical_growth,
        selected_pattern,
        points,
    }
}

fn branch_growth(params: FrameParams, parity: Parity, q: f64) -> f64 {
    let beta = params.eigen_beta;
    local_weight_coeff(params, 1)
        + beta * signed_lateral_pair(params, parity, 0, 2, q)
        + beta * beta * branch_coupling_sum(params, parity, q, 10)
}

fn branch_coupling_sum(params: FrameParams, parity: Parity, q: f64, harmonics: usize) -> f64 {
    let w1 = local_weight_coeff(params, 1);
    (0..=harmonics)
        .filter(|&m| m != 1)
        .map(|m| {
            let left = if m == 0 {
                lateral_weight_coeff(params, 1, q)
            } else {
                lateral_weight_coeff(params, m - 1, q)
            };
            let right = lateral_weight_coeff(params, m + 1, q);
            let numerator = match parity {
                Parity::Even => left + right,
                Parity::Odd => left - right,
            };
            numerator * numerator / safe_denominator(w1 - local_weight_coeff(params, m))
        })
        .sum()
}

fn signed_lateral_pair(
    params: FrameParams,
    parity: Parity,
    left_harmonic: usize,
    right_harmonic: usize,
    q: f64,
) -> f64 {
    let left = lateral_weight_coeff(params, left_harmonic, q);
    let right = lateral_weight_coeff(params, right_harmonic, q);
    match parity {
        Parity::Even => left + right,
        Parity::Odd => left - right,
    }
}

fn branch_selection(params: FrameParams, stability: &StabilityDetails) -> BranchSelectionDetails {
    branch_selection_for(params, stability.critical_q, stability.critical_growth)
}

fn branch_selection_for(params: FrameParams, q: f64, growth: f64) -> BranchSelectionDetails {
    let lambda = growth.max(0.0);
    let eigen = orientation_eigen_details(params, q);
    let square_theta = PI / 2.0;
    let rhombic_theta = params
        .pattern_angle
        .to_radians()
        .clamp(PI / 12.0, 5.0 * PI / 12.0);
    let hex_theta = 2.0 * PI / 3.0;
    let gamma0 = amplitude_gamma3(0.0, &eigen);
    let gamma_square = amplitude_gamma3(square_theta, &eigen);
    let gamma_rhombic = amplitude_gamma3(rhombic_theta, &eigen);
    let gamma_hex = amplitude_gamma3(hex_theta, &eigen);
    let eta_hex = match params.parity {
        Parity::Even => amplitude_gamma2(&eigen),
        Parity::Odd => 0.0,
    };

    let roll_stable = gamma0 > 0.0
        && 2.0 * gamma_square > gamma0
        && 2.0 * gamma_rhombic > gamma0
        && 2.0 * gamma_hex > gamma0;
    let roll = branch_candidate(BranchCandidateSpec {
        family: "roll",
        pattern: "spiral",
        mode_count: 1,
        theta_rad: 0.0,
        gamma0,
        gamma_cross: 0.0,
        lambda,
        denominator: gamma0,
        eta: 0.0,
        stable: roll_stable,
        note: "single active wavevector",
    });
    let square = branch_candidate(BranchCandidateSpec {
        family: "square",
        pattern: "cobweb",
        mode_count: 2,
        theta_rad: square_theta,
        gamma0,
        gamma_cross: gamma_square,
        lambda,
        denominator: gamma0 + 2.0 * gamma_square,
        eta: 0.0,
        stable: gamma_square > 0.0 && 2.0 * gamma_square < gamma0,
        note: "two equal amplitudes on a square lattice",
    });
    let rhombic = branch_candidate(BranchCandidateSpec {
        family: "rhombic",
        pattern: "rhombic",
        mode_count: 2,
        theta_rad: rhombic_theta,
        gamma0,
        gamma_cross: gamma_rhombic,
        lambda,
        denominator: gamma0 + 2.0 * gamma_rhombic,
        eta: 0.0,
        stable: gamma_rhombic > 0.0 && 2.0 * gamma_rhombic < gamma0,
        note: "two equal amplitudes on an oblique lattice",
    });
    let hex_pattern = if eta_hex < 0.0 { "hex_pi" } else { "honeycomb" };
    let hex_note = match params.parity {
        Parity::Even => "three-wave hexagonal branch with quadratic term",
        Parity::Odd => "odd hexagonal branch has zero quadratic term at cubic order",
    };
    let hex = branch_candidate(BranchCandidateSpec {
        family: "hexagonal",
        pattern: hex_pattern,
        mode_count: 3,
        theta_rad: hex_theta,
        gamma0,
        gamma_cross: gamma_hex,
        lambda,
        denominator: gamma0 + 4.0 * gamma_hex,
        eta: eta_hex,
        stable: gamma_hex > 0.0
            && (params.parity == Parity::Even || 2.0 * gamma_hex < gamma0)
            && gamma0 + 4.0 * gamma_hex > 0.0,
        note: hex_note,
    });
    let mut candidates = vec![roll, square, rhombic, hex];
    candidates.sort_by(|a, b| {
        b.stable
            .cmp(&a.stable)
            .then_with(|| b.score.total_cmp(&a.score))
    });
    let global_selected = candidates.first().copied().unwrap_or(roll);
    let target_lattice = branch_target_lattice(params.pattern);
    let (selected, selected_scope, selected_lattice_stable) = select_lattice_branch(
        &candidates,
        target_lattice,
        gamma0,
        gamma_square,
        gamma_rhombic,
        gamma_hex,
        global_selected,
    );

    BranchSelectionDetails {
        model: "cubic-amplitude-equation",
        lambda,
        gamma0,
        gamma_square,
        gamma_rhombic,
        gamma_hex,
        eta_hex,
        target_lattice,
        selected_scope,
        selected_family: selected.family,
        selected_pattern: selected.pattern,
        selected_lattice_stable,
        global_selected_family: global_selected.family,
        global_selected_pattern: global_selected.pattern,
        global_selected_stable: global_selected.stable,
        candidates,
    }
}

fn branch_target_lattice(pattern: PatternPreset) -> &'static str {
    match pattern {
        PatternPreset::Auto => "global",
        PatternPreset::Cobweb => "square",
        PatternPreset::Rhombic => "rhombic",
        PatternPreset::Honeycomb | PatternPreset::HexPi | PatternPreset::Triangle => "hexagonal",
        PatternPreset::Rings | PatternPreset::Rays | PatternPreset::Spiral => "roll",
    }
}

fn select_lattice_branch(
    candidates: &[BranchCandidate],
    target_lattice: &'static str,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
    global_selected: BranchCandidate,
) -> (BranchCandidate, &'static str, bool) {
    if target_lattice == "global" {
        return (global_selected, "global", global_selected.stable);
    }

    let mut scoped: Vec<(BranchCandidate, bool)> = candidates
        .iter()
        .copied()
        .filter(|candidate| candidate_in_lattice(*candidate, target_lattice))
        .map(|candidate| {
            let stable = lattice_local_stable(
                candidate,
                target_lattice,
                gamma0,
                gamma_square,
                gamma_rhombic,
                gamma_hex,
            );
            (candidate, stable)
        })
        .collect();

    scoped.sort_by(|(a, a_stable), (b, b_stable)| {
        b_stable
            .cmp(a_stable)
            .then_with(|| b.score.total_cmp(&a.score))
    });

    scoped
        .first()
        .copied()
        .map(|(candidate, stable)| (candidate, "lattice-local", stable))
        .unwrap_or((global_selected, "global-fallback", global_selected.stable))
}

fn candidate_in_lattice(candidate: BranchCandidate, target_lattice: &str) -> bool {
    match target_lattice {
        "roll" => candidate.family == "roll",
        "square" => candidate.family == "square" || candidate.family == "roll",
        "rhombic" => candidate.family == "rhombic" || candidate.family == "roll",
        "hexagonal" => candidate.family == "hexagonal" || candidate.family == "roll",
        _ => true,
    }
}

fn lattice_local_stable(
    candidate: BranchCandidate,
    target_lattice: &str,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
) -> bool {
    if candidate.family == "roll" {
        return match target_lattice {
            "square" => gamma0 > 0.0 && 2.0 * gamma_square > gamma0,
            "rhombic" => gamma0 > 0.0 && 2.0 * gamma_rhombic > gamma0,
            "hexagonal" => gamma0 > 0.0 && 2.0 * gamma_hex > gamma0,
            "roll" => gamma0 > 0.0,
            _ => candidate.stable,
        };
    }

    match candidate.family {
        "square" => gamma_square > 0.0 && 2.0 * gamma_square < gamma0,
        "rhombic" => gamma_rhombic > 0.0 && 2.0 * gamma_rhombic < gamma0,
        "hexagonal" => candidate.stable,
        _ => candidate.stable,
    }
}

fn branch_candidate(spec: BranchCandidateSpec) -> BranchCandidate {
    let denominator = spec.denominator.max(1.0e-9);
    let lambda = spec.lambda.max(0.0);
    let amplitude = if lambda <= 0.0 {
        0.0
    } else if spec.eta.abs() > 1.0e-9 {
        ((spec.eta.abs() + (spec.eta * spec.eta + 4.0 * denominator * lambda).sqrt())
            / (2.0 * denominator))
            .max(0.0)
    } else {
        (lambda / denominator).sqrt()
    };
    let score = if lambda <= 0.0 {
        f64::NEG_INFINITY
    } else if spec.mode_count == 1 {
        lambda * lambda / (4.0 * spec.gamma0.max(1.0e-9))
    } else {
        spec.mode_count as f64
            * (0.5 * lambda * amplitude * amplitude + spec.eta.abs() * amplitude.powi(3) / 3.0
                - 0.25 * denominator * amplitude.powi(4))
    };
    BranchCandidate {
        family: spec.family,
        pattern: spec.pattern,
        mode_count: spec.mode_count,
        theta_rad: spec.theta_rad,
        gamma_cross: spec.gamma_cross,
        eta: spec.eta,
        amplitude,
        score,
        stable: spec.stable && lambda > 0.0 && amplitude.is_finite(),
        note: spec.note,
    }
}

fn amplitude_gamma2(eigen: &OrientationEigenDetails) -> f64 {
    const SAMPLES: usize = 720;
    (0..SAMPLES)
        .map(|i| {
            let phi = PI * (i as f64 + 0.5) / SAMPLES as f64;
            orientation_eigen_value(phi, eigen)
                * orientation_eigen_value(phi - 2.0 * PI / 3.0, eigen)
                * orientation_eigen_value(phi + 2.0 * PI / 3.0, eigen)
        })
        .sum::<f64>()
        / SAMPLES as f64
}

fn amplitude_gamma3(theta: f64, eigen: &OrientationEigenDetails) -> f64 {
    const SAMPLES: usize = 720;
    (0..SAMPLES)
        .map(|i| {
            let phi = PI * (i as f64 + 0.5) / SAMPLES as f64;
            let shifted = orientation_eigen_value(phi - theta, eigen);
            let base = orientation_eigen_value(phi, eigen);
            shifted * shifted * base * base
        })
        .sum::<f64>()
        / SAMPLES as f64
}

pub(crate) fn planform_modes(
    params: FrameParams,
    pattern: PatternPreset,
) -> Vec<PlanformModeDetails> {
    let angle = params.pattern_angle.to_radians();
    match pattern {
        PatternPreset::Auto => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(PI / 2.0, 0.35, 1.0),
        ],
        PatternPreset::Rings => vec![planform_mode(0.0, 1.0, 1.0)],
        PatternPreset::Rays => vec![planform_mode(PI / 2.0, 1.0, 1.0)],
        PatternPreset::Spiral => vec![planform_mode(angle, 1.0, 1.0)],
        PatternPreset::Cobweb => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(PI / 2.0, 0.35, 1.0),
        ],
        PatternPreset::Honeycomb => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(2.0 * PI / 3.0, 1.0, 1.0),
            planform_mode(-2.0 * PI / 3.0, 1.0, 1.0),
        ],
        PatternPreset::Rhombic => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(angle, -0.25, 1.0),
        ],
        PatternPreset::HexPi => vec![
            planform_mode(0.0, 1.0, 1.0),
            planform_mode(2.0 * PI / 3.0, -1.0, -1.0),
            planform_mode(-2.0 * PI / 3.0, 0.5, 1.0),
        ],
        PatternPreset::Triangle => vec![
            planform_mode_with_phase(0.0, 1.0, -PI / 2.0, 1.0),
            planform_mode_with_phase(2.0 * PI / 3.0, 1.0, -PI / 2.0, 1.0),
            planform_mode_with_phase(-2.0 * PI / 3.0, 1.0, -PI / 2.0, 1.0),
        ],
    }
}

fn planform_mode(normal_angle: f64, phase_scale: f64, amplitude: f64) -> PlanformModeDetails {
    planform_mode_with_phase(normal_angle, phase_scale, 0.0, amplitude)
}

fn planform_mode_with_phase(
    normal_angle: f64,
    phase_scale: f64,
    phase_offset: f64,
    amplitude: f64,
) -> PlanformModeDetails {
    PlanformModeDetails {
        normal_angle,
        phase_scale,
        phase_offset,
        amplitude,
    }
}

fn planform_value(
    params: FrameParams,
    x: f64,
    y: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
    eigen: &OrientationEigenDetails,
) -> f64 {
    if params.contour_mode == ContourMode::Noncontoured {
        return planform_scalar_activity(x, y, wave_number, phase, modes);
    }

    let samples = params.m.max(8);
    let mut best = 0.0_f64;
    for k in 0..samples {
        let phi = PI * k as f64 / samples as f64;
        let value = orientation_planform_activity(x, y, phi, wave_number, phase, modes, eigen);
        if value.abs() > best.abs() {
            best = value;
        }
    }
    best
}

fn planform_scalar_activity(
    x: f64,
    y: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
) -> f64 {
    modes
        .iter()
        .map(|mode| {
            let projection = x * mode.normal_angle.cos() + y * mode.normal_angle.sin();
            mode.amplitude
                * (wave_number * projection + phase * mode.phase_scale + mode.phase_offset).cos()
        })
        .sum()
}

fn orientation_planform_activity(
    x: f64,
    y: f64,
    phi: f64,
    wave_number: f64,
    phase: f64,
    modes: &[PlanformModeDetails],
    eigen: &OrientationEigenDetails,
) -> f64 {
    modes
        .iter()
        .map(|mode| {
            let projection = x * mode.normal_angle.cos() + y * mode.normal_angle.sin();
            let spatial = mode.amplitude
                * (wave_number * projection + phase * mode.phase_scale + mode.phase_offset).cos();
            let tangent_center = mode.normal_angle + PI / 2.0;
            spatial * orientation_eigen_value(phi - tangent_center, eigen)
        })
        .sum()
}

fn orientation_eigen_value(delta: f64, eigen: &OrientationEigenDetails) -> f64 {
    let cos_part = eigen
        .cos_coefficients
        .iter()
        .map(|[harmonic, coefficient]| coefficient * (2.0 * harmonic * delta).cos())
        .sum::<f64>();
    let sin_part = eigen
        .sin_coefficients
        .iter()
        .map(|[harmonic, coefficient]| coefficient * (2.0 * harmonic * delta).sin())
        .sum::<f64>();
    cos_part + sin_part
}

fn orientation_eigen_details(params: FrameParams, q: f64) -> OrientationEigenDetails {
    let max_harmonic = 4;
    let mut cos_coefficients = Vec::new();
    let mut sin_coefficients = Vec::new();
    match params.parity {
        Parity::Even => {
            cos_coefficients.push([1.0, 1.0]);
            let u0 = lateral_weight_coeff(params, 1, q)
                / safe_denominator(local_weight_coeff(params, 1) - local_weight_coeff(params, 0));
            cos_coefficients.push([0.0, (params.eigen_beta * u0).clamp(-1.5, 1.5)]);
            for m in 2..=max_harmonic {
                let coeff = (lateral_weight_coeff(params, m - 1, q)
                    + lateral_weight_coeff(params, m + 1, q))
                    / safe_denominator(
                        local_weight_coeff(params, 1) - local_weight_coeff(params, m),
                    );
                cos_coefficients.push([m as f64, (params.eigen_beta * coeff).clamp(-1.5, 1.5)]);
            }
        }
        Parity::Odd => {
            sin_coefficients.push([1.0, 1.0]);
            for m in 2..=max_harmonic {
                let coeff = (lateral_weight_coeff(params, m - 1, q)
                    - lateral_weight_coeff(params, m + 1, q))
                    / safe_denominator(
                        local_weight_coeff(params, 1) - local_weight_coeff(params, m),
                    );
                sin_coefficients.push([m as f64, (params.eigen_beta * coeff).clamp(-1.5, 1.5)]);
            }
        }
    }

    OrientationEigenDetails {
        parity: params.parity.as_str(),
        beta: params.eigen_beta,
        cos_coefficients,
        sin_coefficients,
    }
}

fn safe_denominator(value: f64) -> f64 {
    if value.abs() < 1.0e-6 {
        if value.is_sign_negative() {
            -1.0e-6
        } else {
            1.0e-6
        }
    } else {
        value
    }
}

fn local_weight_coeff(params: FrameParams, n: usize) -> f64 {
    let xi = params.local_sigma_deg.to_radians();
    let xi_hat = params.local_wide_sigma_deg.to_radians();
    let inhibition = params.local_inhibition;
    (-2.0 * (n as f64).powi(2) * xi * xi).exp()
        - inhibition * (-2.0 * (n as f64).powi(2) * xi_hat * xi_hat).exp()
}

fn lateral_weight_coeff(params: FrameParams, n: usize, q: f64) -> f64 {
    let xi = params.lateral_sigma;
    let xi_hat = params.lateral_wide_sigma;
    let inhibition = params.lateral_inhibition;
    let narrow = 0.25 * xi * xi * q * q;
    let broad = 0.25 * xi_hat * xi_hat * q * q;
    let sign = if n.is_multiple_of(2) { 1.0 } else { -1.0 };
    lateral_spread_factor(params, n)
        * 0.5
        * sign
        * ((-narrow).exp() * modified_bessel_i(n, narrow)
            - inhibition * (-broad).exp() * modified_bessel_i(n, broad))
}

pub(crate) fn lateral_spread_factor(params: FrameParams, n: usize) -> f64 {
    let theta0 = params.lateral_spread_deg.to_radians();
    if n == 0 || theta0.abs() < 1.0e-9 {
        return 1.0;
    }
    let x = 2.0 * n as f64 * theta0;
    x.sin() / x
}

fn modified_bessel_i(n: usize, x: f64) -> f64 {
    if x.abs() < 1.0e-12 {
        return if n == 0 { 1.0 } else { 0.0 };
    }
    let half_x = 0.5 * x;
    let mut factorial = 1.0;
    for value in 1..=n {
        factorial *= value as f64;
    }
    let mut term = half_x.powi(n as i32) / factorial;
    let mut sum = term;
    for k in 1..80 {
        term *= half_x * half_x / (k as f64 * (k + n) as f64);
        sum += term;
        if term.abs() < sum.abs().max(1.0) * 1.0e-13 {
            break;
        }
    }
    sum
}
