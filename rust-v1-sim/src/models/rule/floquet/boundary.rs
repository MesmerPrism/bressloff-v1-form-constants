use std::collections::HashMap;

use super::dynamics::{
    floquet_mode_margin, rule_floquet_mode, rule_floquet_orbit_steps, rule_wave_number_for_cycles,
    RuleOrbitStep,
};
use crate::models::rule::sweep::rule_sweep_params;
use crate::*;

pub(crate) fn rule_floquet_boundary_candidates(
    points: &[RuleFloquetGridPoint],
    periods: &[f64],
    amplitudes: &[f64],
    stim_i_fractions: &[f64],
) -> Vec<RuleFloquetBoundaryCandidate> {
    let periods = sorted_unique_f64(periods);
    let amplitudes = sorted_unique_f64(amplitudes);
    let stim_i_fractions = sorted_unique_f64(stim_i_fractions);
    let mut candidates = Vec::new();
    for stim_i_fraction in &stim_i_fractions {
        for amplitude in &amplitudes {
            for pair in periods.windows(2) {
                if let (Some(from), Some(to)) = (
                    find_rule_floquet_point(points, pair[0], *amplitude, *stim_i_fraction),
                    find_rule_floquet_point(points, pair[1], *amplitude, *stim_i_fraction),
                ) {
                    candidates.extend(rule_floquet_boundary_between(from, to, "period"));
                }
            }
        }
        for period in &periods {
            for pair in amplitudes.windows(2) {
                if let (Some(from), Some(to)) = (
                    find_rule_floquet_point(points, *period, pair[0], *stim_i_fraction),
                    find_rule_floquet_point(points, *period, pair[1], *stim_i_fraction),
                ) {
                    candidates.extend(rule_floquet_boundary_between(from, to, "amplitude"));
                }
            }
        }
    }
    candidates.extend(rule_floquet_beta_boundary_candidates(points));
    candidates.extend(rule_floquet_nearest_boundary_candidates(points, 6));
    candidates.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| a.period_ms.total_cmp(&b.period_ms))
            .then_with(|| a.amplitude.total_cmp(&b.amplitude))
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    candidates
}

fn sorted_unique_f64(values: &[f64]) -> Vec<f64> {
    let mut values = values.to_vec();
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup_by(|a, b| (*a - *b).abs() < 1.0e-9);
    values
}

fn find_rule_floquet_point(
    points: &[RuleFloquetGridPoint],
    period_ms: f64,
    amplitude: f64,
    stim_i_fraction: f64,
) -> Option<&RuleFloquetGridPoint> {
    points.iter().find(|point| {
        (point.period_ms - period_ms).abs() < 1.0e-6
            && (point.amplitude - amplitude).abs() < 1.0e-6
            && (point.stim_i_fraction - stim_i_fraction).abs() < 1.0e-6
    })
}

fn rule_floquet_boundary_between(
    from: &RuleFloquetGridPoint,
    to: &RuleFloquetGridPoint,
    axis: &'static str,
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for from_mode in &from.modes {
        let Some(to_mode) = to
            .modes
            .iter()
            .find(|mode| (mode.beta_cycles - from_mode.beta_cycles).abs() < 1.0e-6)
        else {
            continue;
        };
        for kind in KINDS {
            let margin_from = floquet_mode_margin(from_mode, kind);
            let margin_to = floquet_mode_margin(to_mode, kind);
            if !margin_from.is_finite() || !margin_to.is_finite() {
                continue;
            }
            let crosses =
                (margin_from <= 0.0 && margin_to > 0.0) || (margin_to <= 0.0 && margin_from > 0.0);
            if !crosses {
                continue;
            }
            let denom = margin_to - margin_from;
            let t = if denom.abs() < 1.0e-12 {
                0.5
            } else {
                (-margin_from / denom).clamp(0.0, 1.0)
            };
            candidates.push(RuleFloquetBoundaryCandidate {
                kind,
                evidence: "sign_change",
                beta_cycles: from_mode.beta_cycles,
                axis,
                period_ms: from.period_ms + (to.period_ms - from.period_ms) * t,
                amplitude: from.amplitude + (to.amplitude - from.amplitude) * t,
                stim_i_fraction: from.stim_i_fraction
                    + (to.stim_i_fraction - from.stim_i_fraction) * t,
                from_period_ms: from.period_ms,
                from_amplitude: from.amplitude,
                from_beta_cycles: from_mode.beta_cycles,
                to_period_ms: to.period_ms,
                to_amplitude: to.amplitude,
                to_beta_cycles: to_mode.beta_cycles,
                margin_from,
                margin_to,
                confidence: (margin_to - margin_from).abs().min(1.0),
            });
        }
    }
    candidates
}

fn rule_floquet_beta_boundary_candidates(
    points: &[RuleFloquetGridPoint],
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for point in points {
        let mut modes = point.modes.clone();
        modes.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        for pair in modes.windows(2) {
            let from_mode = pair[0];
            let to_mode = pair[1];
            for kind in KINDS {
                let margin_from = floquet_mode_margin(&from_mode, kind);
                let margin_to = floquet_mode_margin(&to_mode, kind);
                if !margin_from.is_finite() || !margin_to.is_finite() {
                    continue;
                }
                let crosses = (margin_from <= 0.0 && margin_to > 0.0)
                    || (margin_to <= 0.0 && margin_from > 0.0);
                if !crosses {
                    continue;
                }
                let denom = margin_to - margin_from;
                let t = if denom.abs() < 1.0e-12 {
                    0.5
                } else {
                    (-margin_from / denom).clamp(0.0, 1.0)
                };
                candidates.push(RuleFloquetBoundaryCandidate {
                    kind,
                    evidence: "sign_change",
                    beta_cycles: from_mode.beta_cycles
                        + (to_mode.beta_cycles - from_mode.beta_cycles) * t,
                    axis: "beta",
                    period_ms: point.period_ms,
                    amplitude: point.amplitude,
                    stim_i_fraction: point.stim_i_fraction,
                    from_period_ms: point.period_ms,
                    from_amplitude: point.amplitude,
                    from_beta_cycles: from_mode.beta_cycles,
                    to_period_ms: point.period_ms,
                    to_amplitude: point.amplitude,
                    to_beta_cycles: to_mode.beta_cycles,
                    margin_from,
                    margin_to,
                    confidence: (margin_to - margin_from).abs().min(1.0),
                });
            }
        }
    }
    candidates
}

pub(super) fn rule_floquet_beta_boundary_curves(
    points: &[RuleFloquetGridPoint],
    raw: &HashMap<String, String>,
    grid: &RuleSweepGridConfig,
    tolerance: f64,
    max_steps: usize,
) -> Vec<RuleFloquetBoundaryCurve> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut refined_points = Vec::new();
    for point in points {
        let params = rule_sweep_params(
            raw,
            grid,
            point.period_ms,
            point.amplitude,
            point.stim_i_fraction,
        );
        let (orbit_params, _, steps) = rule_floquet_orbit_steps(params);
        let mut modes = point.modes.clone();
        modes.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        for pair in modes.windows(2) {
            let from_mode = pair[0];
            let to_mode = pair[1];
            for kind in KINDS {
                let margin_from = floquet_mode_margin(&from_mode, kind);
                let margin_to = floquet_mode_margin(&to_mode, kind);
                if !floquet_margins_cross(margin_from, margin_to) {
                    continue;
                }
                if let Some(refined) = refine_rule_floquet_beta_boundary(
                    orbit_params,
                    &steps,
                    kind,
                    from_mode.beta_cycles,
                    to_mode.beta_cycles,
                    tolerance,
                    max_steps,
                ) {
                    refined_points.push(refined);
                }
            }
        }
    }
    rule_floquet_boundary_curves_from_points(refined_points)
}

fn refine_rule_floquet_beta_boundary(
    params: FrameParams,
    steps: &[RuleOrbitStep],
    kind: &'static str,
    beta_low: f64,
    beta_high: f64,
    tolerance: f64,
    max_steps: usize,
) -> Option<RuleFloquetBoundaryCurvePoint> {
    let eval = |beta: f64| floquet_mode_margin(&rule_floquet_mode(params, steps, beta), kind);
    let (beta_cycles, margin, iterations) =
        refine_scalar_sign_change(beta_low, beta_high, tolerance, max_steps, eval)?;
    let bracket_low_beta_cycles = beta_low.min(beta_high);
    let bracket_high_beta_cycles = beta_low.max(beta_high);
    Some(RuleFloquetBoundaryCurvePoint {
        kind,
        branch_label: rule_floquet_branch_label(kind),
        branch_periodicity: rule_floquet_branch_periodicity(kind),
        axis: "beta",
        period_ms: params.rule_stim_period_ms,
        stimulus_frequency_hz: 1000.0 / params.rule_stim_period_ms.max(1.0e-9),
        amplitude: params.rule_stim_amplitude,
        stim_i_fraction: params.rule_stim_i_fraction,
        beta_cycles,
        wave_number_radians: rule_wave_number_for_cycles(beta_cycles, params.n),
        bracket_low_beta_cycles,
        bracket_high_beta_cycles,
        bracket_width_beta_cycles: bracket_high_beta_cycles - bracket_low_beta_cycles,
        margin,
        condition_value: -margin,
        iterations,
        residual_abs: margin.abs(),
    })
}

pub(crate) fn rule_floquet_branch_label(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "-1 period-doubling",
        "plus_one_to_one" => "+1 one-to-one",
        "unstable_complex" => "complex unit-circle",
        _ => "unknown",
    }
}

pub(crate) fn rule_floquet_branch_periodicity(kind: &str) -> &'static str {
    match kind {
        "minus_period_doubling" => "2T",
        "plus_one_to_one" => "T",
        "unstable_complex" => "complex",
        _ => "unknown",
    }
}

pub(crate) fn refine_scalar_sign_change<F>(
    low: f64,
    high: f64,
    tolerance: f64,
    max_iterations: usize,
    mut eval: F,
) -> Option<(f64, f64, usize)>
where
    F: FnMut(f64) -> f64,
{
    let mut lo = low.min(high);
    let mut hi = low.max(high);
    let mut f_lo = eval(lo);
    let f_hi = eval(hi);
    if !floquet_margins_cross(f_lo, f_hi) {
        return None;
    }
    let mut best_x = lo;
    let mut best_f = f_lo;
    for iteration in 1..=max_iterations {
        let mid = 0.5 * (lo + hi);
        let f_mid = eval(mid);
        if f_mid.abs() < best_f.abs() {
            best_x = mid;
            best_f = f_mid;
        }
        if f_mid.abs() <= tolerance || (hi - lo).abs() <= tolerance {
            return Some((best_x, best_f, iteration));
        }
        if floquet_margins_cross(f_lo, f_mid) {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }
    Some((best_x, best_f, max_iterations))
}

fn floquet_margins_cross(from: f64, to: f64) -> bool {
    from.is_finite() && to.is_finite() && ((from <= 0.0 && to > 0.0) || (to <= 0.0 && from > 0.0))
}

fn rule_floquet_boundary_curves_from_points(
    mut points: Vec<RuleFloquetBoundaryCurvePoint>,
) -> Vec<RuleFloquetBoundaryCurve> {
    points.sort_by(|a, b| {
        a.kind
            .cmp(b.kind)
            .then_with(|| a.stim_i_fraction.total_cmp(&b.stim_i_fraction))
            .then_with(|| a.amplitude.total_cmp(&b.amplitude))
            .then_with(|| a.period_ms.total_cmp(&b.period_ms))
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    let mut curves: Vec<RuleFloquetBoundaryCurve> = Vec::new();
    let mut index = 0usize;
    while index < points.len() {
        let start = index;
        let key = (
            points[index].kind,
            points[index].amplitude,
            points[index].stim_i_fraction,
        );
        while index < points.len()
            && points[index].kind == key.0
            && (points[index].amplitude - key.1).abs() < 1.0e-9
            && (points[index].stim_i_fraction - key.2).abs() < 1.0e-9
        {
            index += 1;
        }
        let branches = split_rule_floquet_boundary_branches(points[start..index].to_vec());
        for (branch_index, branch_points) in branches.into_iter().enumerate() {
            if let Some(point) = branch_points.first().copied() {
                curves.push(RuleFloquetBoundaryCurve {
                    curve_id: format!(
                        "{}-branch-{:02}-amp-{:.3}-stim-i-{:.3}",
                        point.kind,
                        branch_index + 1,
                        point.amplitude,
                        point.stim_i_fraction
                    ),
                    kind: point.kind,
                    branch_label: format!("{} branch {}", point.branch_label, branch_index + 1),
                    branch_periodicity: point.branch_periodicity,
                    axis: "beta",
                    source_axis: "wave_number_vs_forcing_period",
                    amplitude: point.amplitude,
                    stim_i_fraction: point.stim_i_fraction,
                    point_count: 0,
                    period_min_ms: point.period_ms,
                    period_max_ms: point.period_ms,
                    beta_min_cycles: point.beta_cycles,
                    beta_max_cycles: point.beta_cycles,
                    wave_number_min_radians: point.wave_number_radians,
                    wave_number_max_radians: point.wave_number_radians,
                    mean_residual_abs: 0.0,
                    max_residual_abs: 0.0,
                    mean_bracket_width_beta_cycles: 0.0,
                    max_bracket_width_beta_cycles: 0.0,
                    mean_period_gap_ms: 0.0,
                    max_period_gap_ms: 0.0,
                    continuity_score: 0.0,
                    fit: empty_rule_floquet_curve_fit(),
                    source_comparison: RuleFloquetBoundarySourceComparison::missing(),
                    points: branch_points,
                });
            }
        }
    }

    for curve in &mut curves {
        curve.points.sort_by(|a, b| {
            a.period_ms
                .total_cmp(&b.period_ms)
                .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
        });
        curve.point_count = curve.points.len();
        if let Some(first) = curve.points.first() {
            curve.period_min_ms = first.period_ms;
            curve.period_max_ms = first.period_ms;
            curve.beta_min_cycles = first.beta_cycles;
            curve.beta_max_cycles = first.beta_cycles;
            curve.wave_number_min_radians = first.wave_number_radians;
            curve.wave_number_max_radians = first.wave_number_radians;
        }
        for point in &curve.points {
            curve.period_min_ms = curve.period_min_ms.min(point.period_ms);
            curve.period_max_ms = curve.period_max_ms.max(point.period_ms);
            curve.beta_min_cycles = curve.beta_min_cycles.min(point.beta_cycles);
            curve.beta_max_cycles = curve.beta_max_cycles.max(point.beta_cycles);
            curve.wave_number_min_radians =
                curve.wave_number_min_radians.min(point.wave_number_radians);
            curve.wave_number_max_radians =
                curve.wave_number_max_radians.max(point.wave_number_radians);
        }
        update_rule_floquet_curve_quality(curve);
        curve.fit = fit_rule_floquet_boundary_curve(&curve.points);
    }
    curves
}

fn split_rule_floquet_boundary_branches(
    mut points: Vec<RuleFloquetBoundaryCurvePoint>,
) -> Vec<Vec<RuleFloquetBoundaryCurvePoint>> {
    points.sort_by(|a, b| {
        a.period_ms
            .total_cmp(&b.period_ms)
            .then_with(|| a.beta_cycles.total_cmp(&b.beta_cycles))
    });
    let mut branches: Vec<Vec<RuleFloquetBoundaryCurvePoint>> = Vec::new();
    let mut index = 0usize;
    while index < points.len() {
        let period = points[index].period_ms;
        let start = index;
        while index < points.len() && (points[index].period_ms - period).abs() < 1.0e-9 {
            index += 1;
        }
        let mut period_points = points[start..index].to_vec();
        period_points.sort_by(|a, b| a.beta_cycles.total_cmp(&b.beta_cycles));
        let mut available = (0..branches.len()).collect::<Vec<_>>();
        for point in period_points {
            if available.is_empty() {
                branches.push(vec![point]);
                continue;
            }
            let best_available_index = available
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let a_last = branches[**a]
                        .last()
                        .map(|last| last.beta_cycles)
                        .unwrap_or(0.0);
                    let b_last = branches[**b]
                        .last()
                        .map(|last| last.beta_cycles)
                        .unwrap_or(0.0);
                    (a_last - point.beta_cycles)
                        .abs()
                        .total_cmp(&(b_last - point.beta_cycles).abs())
                })
                .map(|(available_index, _)| available_index)
                .unwrap_or(0);
            let branch_index = available.remove(best_available_index);
            branches[branch_index].push(point);
        }
    }
    branches.sort_by(|a, b| {
        let a_first = a.first().map(|point| point.beta_cycles).unwrap_or(0.0);
        let b_first = b.first().map(|point| point.beta_cycles).unwrap_or(0.0);
        a_first.total_cmp(&b_first)
    });
    branches
}

pub(crate) fn empty_rule_floquet_curve_fit() -> RuleFloquetBoundaryCurveFit {
    RuleFloquetBoundaryCurveFit {
        model: "polynomial_period_to_wave_number",
        degree: 0,
        x_axis: "forcing_period_ms",
        y_axis: "wave_number_radians",
        x_origin_ms: 0.0,
        x_scale_ms: 1.0,
        coefficients: vec![0.0],
        rms_residual: 0.0,
        max_abs_residual: 0.0,
    }
}

fn fit_rule_floquet_boundary_curve(
    points: &[RuleFloquetBoundaryCurvePoint],
) -> RuleFloquetBoundaryCurveFit {
    if points.is_empty() {
        return empty_rule_floquet_curve_fit();
    }

    let x_origin_ms = points.iter().map(|point| point.period_ms).sum::<f64>() / points.len() as f64;
    let x_scale_ms = points
        .iter()
        .map(|point| (point.period_ms - x_origin_ms).abs())
        .fold(0.0, f64::max)
        .max(1.0);
    let requested_degree = if points.len() >= 3 {
        2usize
    } else if points.len() >= 2 {
        1usize
    } else {
        0usize
    };

    let coefficients = polynomial_fit_normalized(
        points
            .iter()
            .map(|point| {
                (
                    (point.period_ms - x_origin_ms) / x_scale_ms,
                    point.wave_number_radians,
                )
            })
            .collect::<Vec<_>>()
            .as_slice(),
        requested_degree,
    )
    .unwrap_or_else(|| {
        vec![
            points
                .iter()
                .map(|point| point.wave_number_radians)
                .sum::<f64>()
                / points.len() as f64,
        ]
    });
    let degree = coefficients.len().saturating_sub(1);
    let residuals = points
        .iter()
        .map(|point| {
            let x = (point.period_ms - x_origin_ms) / x_scale_ms;
            polynomial_value(&coefficients, x) - point.wave_number_radians
        })
        .collect::<Vec<_>>();
    let rms_residual = (residuals
        .iter()
        .map(|residual| residual * residual)
        .sum::<f64>()
        / residuals.len().max(1) as f64)
        .sqrt();
    let max_abs_residual = residuals
        .iter()
        .map(|residual| residual.abs())
        .fold(0.0, f64::max);

    RuleFloquetBoundaryCurveFit {
        model: "polynomial_period_to_wave_number",
        degree,
        x_axis: "forcing_period_ms",
        y_axis: "wave_number_radians",
        x_origin_ms,
        x_scale_ms,
        coefficients,
        rms_residual,
        max_abs_residual,
    }
}

fn polynomial_fit_normalized(points: &[(f64, f64)], degree: usize) -> Option<Vec<f64>> {
    let terms = degree + 1;
    if points.len() < terms {
        return None;
    }
    let mut matrix = vec![vec![0.0; terms]; terms];
    let mut rhs = vec![0.0; terms];
    for (x, y) in points {
        let mut powers = vec![1.0; terms * 2];
        for i in 1..powers.len() {
            powers[i] = powers[i - 1] * x;
        }
        for row in 0..terms {
            rhs[row] += y * powers[row];
            for col in 0..terms {
                matrix[row][col] += powers[row + col];
            }
        }
    }
    solve_linear_system(matrix, rhs)
}

fn solve_linear_system(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    for col in 0..n {
        let pivot =
            (col..n).max_by(|a, b| matrix[*a][col].abs().total_cmp(&matrix[*b][col].abs()))?;
        if matrix[pivot][col].abs() < 1.0e-12 {
            return None;
        }
        matrix.swap(col, pivot);
        rhs.swap(col, pivot);
        let pivot_value = matrix[col][col];
        for value in matrix[col].iter_mut().take(n).skip(col) {
            *value /= pivot_value;
        }
        rhs[col] /= pivot_value;
        let pivot_row = matrix[col].clone();
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = matrix[row][col];
            for (cell, pivot_cell) in matrix[row]
                .iter_mut()
                .zip(pivot_row.iter())
                .take(n)
                .skip(col)
            {
                *cell -= factor * *pivot_cell;
            }
            rhs[row] -= factor * rhs[col];
        }
    }
    Some(rhs)
}

fn polynomial_value(coefficients: &[f64], x: f64) -> f64 {
    coefficients
        .iter()
        .rev()
        .fold(0.0, |acc, coefficient| acc * x + coefficient)
}

fn update_rule_floquet_curve_quality(curve: &mut RuleFloquetBoundaryCurve) {
    let point_count = curve.points.len();
    if point_count == 0 {
        return;
    }

    curve.mean_residual_abs = curve
        .points
        .iter()
        .map(|point| point.residual_abs)
        .sum::<f64>()
        / point_count as f64;
    curve.max_residual_abs = curve
        .points
        .iter()
        .map(|point| point.residual_abs)
        .fold(0.0, f64::max);
    curve.mean_bracket_width_beta_cycles = curve
        .points
        .iter()
        .map(|point| point.bracket_width_beta_cycles)
        .sum::<f64>()
        / point_count as f64;
    curve.max_bracket_width_beta_cycles = curve
        .points
        .iter()
        .map(|point| point.bracket_width_beta_cycles)
        .fold(0.0, f64::max);

    let gaps = curve
        .points
        .windows(2)
        .map(|pair| (pair[1].period_ms - pair[0].period_ms).abs())
        .collect::<Vec<_>>();
    if gaps.is_empty() {
        curve.mean_period_gap_ms = 0.0;
        curve.max_period_gap_ms = 0.0;
        curve.continuity_score = 1.0;
        return;
    }

    curve.mean_period_gap_ms = gaps.iter().sum::<f64>() / gaps.len() as f64;
    curve.max_period_gap_ms = gaps.iter().copied().fold(0.0, f64::max);
    let span = (curve.period_max_ms - curve.period_min_ms).abs();
    curve.continuity_score = if span <= 1.0e-9 {
        1.0
    } else {
        (1.0 - (curve.max_period_gap_ms / span).clamp(0.0, 1.0)).clamp(0.0, 1.0)
    };
}

fn rule_floquet_nearest_boundary_candidates(
    points: &[RuleFloquetGridPoint],
    limit_per_kind: usize,
) -> Vec<RuleFloquetBoundaryCandidate> {
    const KINDS: [&str; 3] = [
        "minus_period_doubling",
        "plus_one_to_one",
        "unstable_complex",
    ];
    let mut candidates = Vec::new();
    for kind in KINDS {
        let mut scored = Vec::new();
        for point in points {
            for mode in &point.modes {
                let margin = floquet_mode_margin(mode, kind);
                if !margin.is_finite() {
                    continue;
                }
                scored.push((
                    margin.abs(),
                    RuleFloquetBoundaryCandidate {
                        kind,
                        evidence: "nearest_margin",
                        beta_cycles: mode.beta_cycles,
                        axis: "nearest",
                        period_ms: point.period_ms,
                        amplitude: point.amplitude,
                        stim_i_fraction: point.stim_i_fraction,
                        from_period_ms: point.period_ms,
                        from_amplitude: point.amplitude,
                        from_beta_cycles: mode.beta_cycles,
                        to_period_ms: point.period_ms,
                        to_amplitude: point.amplitude,
                        to_beta_cycles: mode.beta_cycles,
                        margin_from: margin,
                        margin_to: margin,
                        confidence: (1.0 - margin.abs().min(1.0)).max(0.0),
                    },
                ));
            }
        }
        scored.sort_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then_with(|| b.1.confidence.total_cmp(&a.1.confidence))
                .then_with(|| a.1.period_ms.total_cmp(&b.1.period_ms))
                .then_with(|| a.1.amplitude.total_cmp(&b.1.amplitude))
        });
        candidates.extend(
            scored
                .into_iter()
                .take(limit_per_kind)
                .map(|(_, candidate)| candidate),
        );
    }
    candidates
}
