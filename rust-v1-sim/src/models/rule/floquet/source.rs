use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use super::normalization::rule_source_beta_for_model_cycles;
use crate::*;

pub(crate) fn load_rule_figure8_source_curves(
    path: &Path,
) -> Result<RuleFigure8SourceCurves, Box<dyn std::error::Error>> {
    let body = fs::read_to_string(path)?;
    let source = serde_json::from_str::<RuleFigure8SourceCurves>(&body)?;
    if source.format != "rule-2011-figure8-source-curves-v1" {
        return Err(format!(
            "unexpected Rule Figure 8 source curve format: {}",
            source.format
        )
        .into());
    }
    if source.source_key != "rule-2011" {
        return Err(format!("unexpected Rule Figure 8 source key: {}", source.source_key).into());
    }
    if source.figure != "Figure 8C" {
        return Err(format!("unexpected Rule Figure 8 source figure: {}", source.figure).into());
    }
    Ok(source)
}

pub(crate) fn apply_rule_figure8_source_comparison(
    curves: &mut [RuleFloquetBoundaryCurve],
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
    source_curves: Option<&RuleFigure8SourceCurves>,
    source_curve_file: Option<&PathBuf>,
) -> RuleFloquetSourceCurveComparisonSummary {
    let Some(source) = source_curves else {
        for curve in &mut *curves {
            curve.source_comparison = if source_curve_file.is_some() {
                RuleFloquetBoundarySourceComparison::missing()
            } else {
                RuleFloquetBoundarySourceComparison::disabled()
            };
        }
        let status = if source_curve_file.is_some() {
            "source-curve-file-missing"
        } else {
            "source-curve-comparison-disabled"
        };
        return RuleFloquetSourceCurveComparisonSummary {
            status,
            source_curve_file: source_curve_file.map(|path| path.display().to_string()),
            source_curve_count: 0,
            compared_curve_count: 0,
            mean_rms_wave_number_error: None,
            max_rms_wave_number_error: None,
            domain_beta_mapping: None,
            raw_beta_mapping: None,
            scale_only_beta_mapping: None,
            affine_beta_mapping: None,
            fit_objective: None,
        };
    };

    let mut rms_values = Vec::new();
    for curve in &mut *curves {
        curve.source_comparison =
            best_rule_source_curve_comparison(curve, &source.curves, wave_number_normalization);
        if let Some(rms) = curve.source_comparison.rms_wave_number_error {
            rms_values.push(rms);
        }
    }
    let compared_curve_count = rms_values.len();
    let mean_rms_wave_number_error =
        (!rms_values.is_empty()).then(|| rms_values.iter().sum::<f64>() / rms_values.len() as f64);
    let max_rms_wave_number_error = rms_values.iter().copied().reduce(f64::max);
    let samples = rule_figure8_comparison_samples(curves, &source.curves);
    let domain_beta_mapping = rule_beta_axis_mapping(
        "domain_normalized",
        &samples,
        wave_number_normalization.source_beta_per_model_cycle,
        wave_number_normalization.source_beta_offset,
    );
    let raw_beta_mapping = rule_beta_axis_mapping("identity", &samples, 1.0, 0.0);
    let scale_only_beta_mapping = fit_rule_beta_scale_only_mapping(&samples);
    let affine_beta_mapping = fit_rule_beta_affine_mapping(&samples);
    let fit_objective = rule_figure8_fit_objective(
        curves,
        &source.curves,
        domain_beta_mapping.as_ref(),
        raw_beta_mapping.as_ref(),
        scale_only_beta_mapping.as_ref(),
        affine_beta_mapping.as_ref(),
    );

    RuleFloquetSourceCurveComparisonSummary {
        status: if compared_curve_count > 0 {
            "compared"
        } else {
            "no-overlapping-source-curves"
        },
        source_curve_file: source_curve_file.map(|path| path.display().to_string()),
        source_curve_count: source.curves.len(),
        compared_curve_count,
        mean_rms_wave_number_error,
        max_rms_wave_number_error,
        domain_beta_mapping,
        raw_beta_mapping,
        scale_only_beta_mapping,
        affine_beta_mapping,
        fit_objective,
    }
}

fn best_rule_source_curve_comparison(
    curve: &RuleFloquetBoundaryCurve,
    source_curves: &[RuleFigure8SourceCurve],
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
) -> RuleFloquetBoundarySourceComparison {
    source_curves
        .iter()
        .filter(|source| source.kind == curve.kind)
        .filter_map(|source| compare_rule_source_curve(curve, source, wave_number_normalization))
        .min_by(|a, b| {
            a.rms_wave_number_error
                .unwrap_or(f64::INFINITY)
                .total_cmp(&b.rms_wave_number_error.unwrap_or(f64::INFINITY))
        })
        .unwrap_or_else(RuleFloquetBoundarySourceComparison::no_overlap)
}

fn compare_rule_source_curve(
    curve: &RuleFloquetBoundaryCurve,
    source: &RuleFigure8SourceCurve,
    wave_number_normalization: RuleFigure8WaveNumberNormalization,
) -> Option<RuleFloquetBoundarySourceComparison> {
    let mut source_points = source.points.clone();
    source_points.sort_by(|a, b| a.period_ms.total_cmp(&b.period_ms));
    let source_min = source_points.first()?.period_ms;
    let source_max = source_points.last()?.period_ms;
    let mut errors: Vec<f64> = Vec::new();
    let mut matched_periods: Vec<f64> = Vec::new();
    for point in &curve.points {
        if point.period_ms < source_min || point.period_ms > source_max {
            continue;
        }
        let Some(source_wave) =
            interpolate_rule_source_wave_number(&source_points, point.period_ms)
        else {
            continue;
        };
        let normalized_beta =
            rule_source_beta_for_model_cycles(point.beta_cycles, wave_number_normalization);
        errors.push(normalized_beta - source_wave);
        matched_periods.push(point.period_ms);
    }
    if errors.is_empty() {
        return None;
    }
    let mean_abs_wave_number_error =
        errors.iter().map(|error| error.abs()).sum::<f64>() / errors.len() as f64;
    let rms_wave_number_error =
        (errors.iter().map(|error| error * error).sum::<f64>() / errors.len() as f64).sqrt();
    let max_abs_wave_number_error = errors.iter().map(|error| error.abs()).fold(0.0, f64::max);
    Some(RuleFloquetBoundarySourceComparison {
        status: "compared",
        source_curve_id: Some(source.curve_id.clone()),
        source_branch_label: Some(source.branch_label.clone()),
        overlap_point_count: errors.len(),
        period_overlap_min_ms: matched_periods.iter().copied().reduce(f64::min),
        period_overlap_max_ms: matched_periods.iter().copied().reduce(f64::max),
        mean_abs_wave_number_error: Some(mean_abs_wave_number_error),
        rms_wave_number_error: Some(rms_wave_number_error),
        max_abs_wave_number_error: Some(max_abs_wave_number_error),
    })
}

fn interpolate_rule_source_wave_number(
    points: &[RuleFigure8SourcePoint],
    period_ms: f64,
) -> Option<f64> {
    for pair in points.windows(2) {
        let from = pair[0];
        let to = pair[1];
        if period_ms < from.period_ms || period_ms > to.period_ms {
            continue;
        }
        let span = (to.period_ms - from.period_ms).abs();
        if span <= 1.0e-9 {
            return Some(from.wave_number_beta);
        }
        let t = (period_ms - from.period_ms) / (to.period_ms - from.period_ms);
        return Some(from.wave_number_beta + t * (to.wave_number_beta - from.wave_number_beta));
    }
    None
}

fn rule_figure8_comparison_samples(
    curves: &[RuleFloquetBoundaryCurve],
    source_curves: &[RuleFigure8SourceCurve],
) -> Vec<RuleFigure8ComparisonSample> {
    let mut samples = Vec::new();
    for curve in curves {
        let Some(source_curve_id) = curve.source_comparison.source_curve_id.as_deref() else {
            continue;
        };
        let Some(source) = source_curves
            .iter()
            .find(|candidate| candidate.curve_id == source_curve_id)
        else {
            continue;
        };
        let mut source_points = source.points.clone();
        source_points.sort_by(|a, b| a.period_ms.total_cmp(&b.period_ms));
        let Some(source_min) = source_points.first().map(|point| point.period_ms) else {
            continue;
        };
        let Some(source_max) = source_points.last().map(|point| point.period_ms) else {
            continue;
        };
        for point in &curve.points {
            if point.period_ms < source_min || point.period_ms > source_max {
                continue;
            }
            if let Some(source_beta) =
                interpolate_rule_source_wave_number(&source_points, point.period_ms)
            {
                samples.push(RuleFigure8ComparisonSample {
                    generated_beta_cycles: point.beta_cycles,
                    source_beta,
                });
            }
        }
    }
    samples
}

fn rule_beta_axis_mapping(
    model: &'static str,
    samples: &[RuleFigure8ComparisonSample],
    scale: f64,
    offset: f64,
) -> Option<RuleFloquetBetaAxisMapping> {
    if samples.is_empty() {
        return None;
    }
    let mut abs_sum = 0.0;
    let mut square_sum = 0.0;
    let mut max_abs = 0.0;
    for sample in samples {
        let predicted = scale * sample.generated_beta_cycles + offset;
        let error = predicted - sample.source_beta;
        let abs = error.abs();
        abs_sum += abs;
        square_sum += error * error;
        max_abs = f64::max(max_abs, abs);
    }
    let sample_count = samples.len();
    Some(RuleFloquetBetaAxisMapping {
        model,
        generated_axis: "generated_beta_cycles",
        source_axis: "source_figure_beta",
        scale,
        offset,
        sample_count,
        mean_abs_error: abs_sum / sample_count as f64,
        rms_error: (square_sum / sample_count as f64).sqrt(),
        max_abs_error: max_abs,
    })
}

fn fit_rule_beta_scale_only_mapping(
    samples: &[RuleFigure8ComparisonSample],
) -> Option<RuleFloquetBetaAxisMapping> {
    let denominator = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles * sample.generated_beta_cycles)
        .sum::<f64>();
    if denominator.abs() <= 1.0e-12 {
        return rule_beta_axis_mapping("scale_only", samples, 1.0, 0.0);
    }
    let numerator = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles * sample.source_beta)
        .sum::<f64>();
    rule_beta_axis_mapping("scale_only", samples, numerator / denominator, 0.0)
}

fn fit_rule_beta_affine_mapping(
    samples: &[RuleFigure8ComparisonSample],
) -> Option<RuleFloquetBetaAxisMapping> {
    if samples.is_empty() {
        return None;
    }
    let sample_count = samples.len() as f64;
    let mean_x = samples
        .iter()
        .map(|sample| sample.generated_beta_cycles)
        .sum::<f64>()
        / sample_count;
    let mean_y = samples.iter().map(|sample| sample.source_beta).sum::<f64>() / sample_count;
    let variance_x = samples
        .iter()
        .map(|sample| {
            let dx = sample.generated_beta_cycles - mean_x;
            dx * dx
        })
        .sum::<f64>();
    if variance_x.abs() <= 1.0e-12 {
        return rule_beta_axis_mapping("affine", samples, 1.0, mean_y - mean_x);
    }
    let covariance_xy = samples
        .iter()
        .map(|sample| {
            let dx = sample.generated_beta_cycles - mean_x;
            let dy = sample.source_beta - mean_y;
            dx * dy
        })
        .sum::<f64>();
    let scale = covariance_xy / variance_x;
    let offset = mean_y - scale * mean_x;
    rule_beta_axis_mapping("affine", samples, scale, offset)
}

fn rule_figure8_fit_objective(
    curves: &[RuleFloquetBoundaryCurve],
    source_curves: &[RuleFigure8SourceCurve],
    domain_mapping: Option<&RuleFloquetBetaAxisMapping>,
    raw_mapping: Option<&RuleFloquetBetaAxisMapping>,
    scale_only_mapping: Option<&RuleFloquetBetaAxisMapping>,
    affine_mapping: Option<&RuleFloquetBetaAxisMapping>,
) -> Option<RuleFigure8FitObjective> {
    let raw = raw_mapping?;
    let domain = domain_mapping.unwrap_or(raw);
    let scale_only = scale_only_mapping.unwrap_or(raw);
    let affine = affine_mapping.unwrap_or(raw);
    let compared_curves = curves
        .iter()
        .filter(|curve| curve.source_comparison.status == "compared")
        .collect::<Vec<_>>();
    let matched_source_curve_count = compared_curves
        .iter()
        .filter_map(|curve| curve.source_comparison.source_curve_id.as_deref())
        .collect::<HashSet<_>>()
        .len();
    let source_curve_count = source_curves.len();
    let compared_curve_count = compared_curves.len();
    let generated_curve_coverage = if curves.is_empty() {
        0.0
    } else {
        compared_curve_count as f64 / curves.len() as f64
    };
    let source_branch_coverage = if source_curve_count == 0 {
        0.0
    } else {
        matched_source_curve_count as f64 / source_curve_count as f64
    };
    let total_curve_points = curves.iter().map(|curve| curve.point_count).sum::<usize>();
    let overlap_point_count = compared_curves
        .iter()
        .map(|curve| curve.source_comparison.overlap_point_count)
        .sum::<usize>();
    let overlap_point_coverage = if total_curve_points == 0 {
        0.0
    } else {
        overlap_point_count as f64 / total_curve_points as f64
    };
    let continuity_score = if compared_curves.is_empty() {
        0.0
    } else {
        compared_curves
            .iter()
            .map(|curve| curve.continuity_score.clamp(0.0, 1.0))
            .sum::<f64>()
            / compared_curves.len() as f64
    };
    let underresolved_branch_count = curves.iter().filter(|curve| curve.point_count < 2).count();
    let underresolved_branch_fraction = if curves.is_empty() {
        1.0
    } else {
        underresolved_branch_count as f64 / curves.len() as f64
    };
    let ordering_score = rule_figure8_kind_ordering_score(curves);
    let coverage_score = (0.45 * source_branch_coverage
        + 0.35 * generated_curve_coverage
        + 0.20 * overlap_point_coverage)
        .clamp(0.0, 1.0);
    let score = 0.55 * domain.rms_error
        + 0.15 * affine.rms_error
        + 0.05 * scale_only.rms_error
        + 0.85 * (1.0 - coverage_score)
        + 0.35 * (1.0 - continuity_score)
        + 0.45 * (1.0 - ordering_score)
        + 0.25 * underresolved_branch_fraction;

    const DOMAIN_RMS_BETA_ERROR_MAX: f64 = 0.10;
    const SOURCE_BRANCH_COVERAGE_MIN: f64 = 1.0;
    const GENERATED_CURVE_COVERAGE_MIN: f64 = 0.95;
    const OVERLAP_POINT_COVERAGE_MIN: f64 = 0.75;
    const CONTINUITY_SCORE_MIN: f64 = 0.75;
    const ORDERING_SCORE_MIN: f64 = 1.0;
    const UNDERRESOLVED_BRANCH_FRACTION_MAX: f64 = 0.0;

    let mut failed_acceptance_checks = Vec::new();
    let mut check_acceptance = |name: &'static str, passes: bool| {
        if !passes {
            failed_acceptance_checks.push(name);
        }
    };
    check_acceptance(
        "domain_normalized_rms_beta_error",
        domain.rms_error <= DOMAIN_RMS_BETA_ERROR_MAX,
    );
    check_acceptance(
        "source_branch_coverage",
        source_branch_coverage >= SOURCE_BRANCH_COVERAGE_MIN,
    );
    check_acceptance(
        "generated_curve_coverage",
        generated_curve_coverage >= GENERATED_CURVE_COVERAGE_MIN,
    );
    check_acceptance(
        "overlap_point_coverage",
        overlap_point_coverage >= OVERLAP_POINT_COVERAGE_MIN,
    );
    check_acceptance("continuity_score", continuity_score >= CONTINUITY_SCORE_MIN);
    check_acceptance("ordering_score", ordering_score >= ORDERING_SCORE_MIN);
    check_acceptance(
        "underresolved_branch_fraction",
        underresolved_branch_fraction <= UNDERRESOLVED_BRANCH_FRACTION_MAX,
    );
    let acceptance_status = if failed_acceptance_checks.is_empty() {
        "threshold-accepted-diagnostic"
    } else {
        "outside-threshold-diagnostic"
    };

    Some(RuleFigure8FitObjective {
        status: "scored",
        acceptance_status,
        calibration_claim_allowed: false,
        score,
        domain_normalized_rms_beta_error_max: DOMAIN_RMS_BETA_ERROR_MAX,
        source_branch_coverage_min: SOURCE_BRANCH_COVERAGE_MIN,
        generated_curve_coverage_min: GENERATED_CURVE_COVERAGE_MIN,
        overlap_point_coverage_min: OVERLAP_POINT_COVERAGE_MIN,
        continuity_score_min: CONTINUITY_SCORE_MIN,
        ordering_score_min: ORDERING_SCORE_MIN,
        underresolved_branch_fraction_max: UNDERRESOLVED_BRANCH_FRACTION_MAX,
        failed_acceptance_checks,
        domain_normalized_rms_beta_error: domain.rms_error,
        raw_rms_beta_error: raw.rms_error,
        affine_rms_beta_error: affine.rms_error,
        scale_only_rms_beta_error: scale_only.rms_error,
        source_branch_coverage,
        generated_curve_coverage,
        overlap_point_coverage,
        continuity_score,
        ordering_score,
        underresolved_branch_fraction,
        compared_curve_count,
        source_curve_count,
        matched_source_curve_count,
        overlap_point_count,
    })
}

fn rule_figure8_kind_ordering_score(curves: &[RuleFloquetBoundaryCurve]) -> f64 {
    let minus_mean = mean_curve_period_for_kind(curves, "minus_period_doubling");
    let plus_mean = mean_curve_period_for_kind(curves, "plus_one_to_one");
    match (minus_mean, plus_mean) {
        (Some(minus), Some(plus)) if minus < plus => 1.0,
        (Some(_), Some(_)) => 0.25,
        _ => 0.0,
    }
}

fn mean_curve_period_for_kind(curves: &[RuleFloquetBoundaryCurve], kind: &str) -> Option<f64> {
    let mut count = 0usize;
    let mut sum = 0.0;
    for point in curves
        .iter()
        .filter(|curve| curve.kind == kind)
        .flat_map(|curve| curve.points.iter())
    {
        sum += point.period_ms;
        count += 1;
    }
    (count > 0).then(|| sum / count as f64)
}
