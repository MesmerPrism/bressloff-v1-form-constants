use std::{
    fs,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose, Engine as _};

use super::planform::{cell_mm_for, planform_details};
use super::presets::{
    apply_paper_preset, paper_preset_catalog, PaperPreset, PAPER_PRESET_REGISTRY,
};
use crate::*;
pub(crate) fn calibrate_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/paper-calibration.json");
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--export-orientations" | "--export-orientation-channels" => {
                raw.insert(
                    "export_orientation_channels".to_string(),
                    "true".to_string(),
                );
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    raw.entry("generator".to_string())
        .or_insert_with(|| "planform".to_string());
    raw.entry("n".to_string())
        .or_insert_with(|| "96".to_string());
    raw.entry("m".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("frames".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("t".to_string())
        .or_insert_with(|| "18".to_string());

    let state = ServerState::default();
    let mut runs = Vec::new();
    for preset in paper_preset_catalog()
        .into_iter()
        .map(|details| parse_paper_preset(Some(details.id)))
        .filter(|preset| *preset != PaperPreset::Manual)
    {
        let mut preset_raw = raw.clone();
        preset_raw.insert("paper_preset".to_string(), preset.as_str().to_string());
        let params = coerce_params(&preset_raw);
        let payload = generate_payload(params, &state)?;
        let calibration = payload
            .calibration
            .as_ref()
            .ok_or("calibration report missing for paper preset")?;
        runs.push(CalibrationRun {
            preset: calibration.preset,
            status: calibration.status,
            rendered_contour_mode: calibration.rendered_contour_mode,
            rendered_parity: calibration.rendered_parity,
            rendered_pattern: calibration.rendered_pattern,
            selected_family: calibration.selected_family,
            selected_pattern: calibration.selected_pattern,
            selected_scope: calibration.selected_scope,
            global_selected_family: calibration.global_selected_family,
            global_selected_pattern: calibration.global_selected_pattern,
            target_lattice: calibration.target_lattice,
            critical_q: calibration.critical_q,
            critical_branch: calibration.critical_branch,
            dominant_cycles: calibration.dominant_cycles,
            checks: calibration.checks.clone(),
        });
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let run_count = runs.len();
    let body = serde_json::json!({
        "format": "bressloff-paper-calibration-v4",
        "model_family": MODEL_FAMILY_BRESSLOFF,
        "runs": runs,
        "stability_reports": bressloff_stability_reports(),
    });
    fs::write(&out, serde_json::to_vec_pretty(&body)?)?;
    println!("wrote {} presets={run_count}", out.display());
    Ok(())
}

pub(crate) fn bressloff_geometry_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("reports/figure-targets/bressloff-generated-stills.json");
    let mut source_profile_dir = PathBuf::from("private/figure-targets/derived");
    let mut preset_set = "figures29-36".to_string();
    let mut preset_override: Option<Vec<PaperPreset>> = None;
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
            "--source-profile-dir" => {
                source_profile_dir =
                    PathBuf::from(iter.next().ok_or("--source-profile-dir requires a value")?);
            }
            "--preset-set" => {
                preset_set = iter
                    .next()
                    .ok_or("--preset-set requires a value")?
                    .to_string();
            }
            "--presets" => {
                preset_override = Some(parse_paper_preset_csv(
                    iter.next().ok_or("--presets requires a value")?,
                )?);
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    raw.entry("generator".to_string())
        .or_insert_with(|| "planform".to_string());
    raw.entry("n".to_string())
        .or_insert_with(|| "96".to_string());
    raw.entry("m".to_string())
        .or_insert_with(|| "24".to_string());
    raw.entry("frames".to_string())
        .or_insert_with(|| "8".to_string());
    raw.entry("t".to_string())
        .or_insert_with(|| "18".to_string());

    let state = ServerState::default();
    let presets = preset_override.unwrap_or_else(|| bressloff_geometry_preset_set(&preset_set));
    let mut stills = Vec::new();
    for preset in presets {
        let mut preset_raw = raw.clone();
        preset_raw.insert("paper_preset".to_string(), preset.as_str().to_string());
        let params = coerce_params(&preset_raw);
        let payload = generate_payload(params, &state)?;
        let calibration = payload
            .calibration
            .as_ref()
            .ok_or("geometry still missing Bressloff calibration metadata")?;
        let frame_index = payload.frame_count.saturating_sub(1);
        let frame = payload_frame_u8(&payload, frame_index)?;
        let metrics = bressloff_still_metrics(&frame, payload.width, payload.height);
        let source_profile =
            load_bressloff_source_profile(&source_profile_dir, calibration.preset.id)?;
        let source_comparison =
            bressloff_source_comparison(calibration.preset.id, &metrics, source_profile.as_ref());
        let acceptance = evaluate_bressloff_geometry_acceptance(&source_comparison);
        stills.push(BressloffFigureStill {
            preset: calibration.preset,
            target_mask_status: source_comparison.status,
            target_mask_id: source_comparison.source_mask_id.clone(),
            width: payload.width,
            height: payload.height,
            frame_index,
            rendered_contour_mode: calibration.rendered_contour_mode,
            rendered_pattern: calibration.rendered_pattern,
            selected_family: calibration.selected_family,
            image: BressloffStillImage {
                format: "u8-frame-v1",
                encoding: "base64",
                color_space: "normalized-luma",
                data_base64: general_purpose::STANDARD.encode(frame),
            },
            metrics,
            source_comparison,
            acceptance,
        });
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let still_count = stills.len();
    let compared_still_count = stills
        .iter()
        .filter(|still| still.source_comparison.status == "compared")
        .count();
    let threshold_accepted_still_count = stills
        .iter()
        .filter(|still| still.acceptance.passes_thresholds)
        .count();
    let width = stills.first().map(|still| still.width).unwrap_or(0);
    let height = stills.first().map(|still| still.height).unwrap_or(0);
    let acceptance_policy = bressloff_geometry_acceptance_policy();
    let report = BressloffFigureGeometryReport {
        format: "bressloff-generated-figure-stills-v2",
        model_family: MODEL_FAMILY_BRESSLOFF,
        source_key: "bressloff-2001-2002",
        status: if compared_still_count > 0 {
            "generated-vs-source-derived-comparison"
        } else {
            "generated-targets-ready-for-private-mask-calibration"
        },
        note: "Bressloff figure stills and public-safe geometry metrics. Private source scans/crops stay out of the report; comparisons use only derived numeric masks/profiles when available.",
        acceptance_policy,
        calibration_claim_allowed: acceptance_policy.calibration_claim_allowed,
        source_profile_dir: source_profile_dir.display().to_string(),
        width,
        height,
        still_count,
        compared_still_count,
        threshold_accepted_still_count,
        stills,
    };
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;
    println!("wrote {} generated_stills={still_count}", out.display());
    Ok(())
}

pub(crate) fn parse_paper_preset_csv(
    value: &str,
) -> Result<Vec<PaperPreset>, Box<dyn std::error::Error>> {
    let presets = value
        .split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .map(|part| {
            PAPER_PRESET_REGISTRY
                .iter()
                .find(|entry| entry.details.id == part)
                .map(|entry| entry.preset)
                .ok_or_else(|| format!("unknown paper preset: {part}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if presets.is_empty() {
        Err("preset list must contain at least one paper preset".into())
    } else {
        Ok(presets)
    }
}

pub(crate) fn bressloff_geometry_preset_set(name: &str) -> Vec<PaperPreset> {
    match name {
        "all" => PAPER_PRESET_REGISTRY
            .iter()
            .map(|entry| entry.preset)
            .filter(|preset| *preset != PaperPreset::Manual)
            .collect(),
        _ => vec![
            PaperPreset::Fig29SquareNoncontoured,
            PaperPreset::Fig29RollNoncontoured,
            PaperPreset::Fig30RhombicNoncontoured,
            PaperPreset::Fig30HexNoncontoured,
            PaperPreset::Fig31SquareEven,
            PaperPreset::Fig31SquareEvenRoll,
            PaperPreset::Fig32SquareOdd,
            PaperPreset::Fig32SquareOddRoll,
            PaperPreset::Fig33RhombicEven,
            PaperPreset::Fig33RhombicEvenRoll,
            PaperPreset::Fig34RhombicOdd,
            PaperPreset::Fig34RhombicOddRoll,
            PaperPreset::Fig35HexEven,
            PaperPreset::Fig35HexZeroEven,
            PaperPreset::Fig36TriangleOdd,
            PaperPreset::Fig36HexZeroOdd,
        ],
    }
}

pub(crate) fn payload_frame_u8(
    payload: &Payload,
    frame_index: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let frame_size = payload.width * payload.height;
    let bytes = general_purpose::STANDARD.decode(&payload.data_base64)?;
    let start = frame_index
        .min(payload.frame_count.saturating_sub(1))
        .saturating_mul(frame_size);
    let end = start + frame_size;
    if end > bytes.len() {
        return Err("payload frame index outside encoded data".into());
    }
    Ok(bytes[start..end].to_vec())
}

pub(crate) fn bressloff_still_metrics(
    frame: &[u8],
    width: usize,
    height: usize,
) -> BressloffStillMetrics {
    let len = frame.len().max(1) as f64;
    let values = frame.iter().map(|value| *value as f64 / 255.0);
    let mean = values.clone().sum::<f64>() / len;
    let variance = values
        .clone()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / len;
    let active_fraction = frame.iter().filter(|value| **value >= 128).count() as f64 / len;

    let mut edge_sum = 0.0;
    let mut edge_count = 0usize;
    for y in 0..height {
        for x in 0..width {
            let here = frame[y * width + x] as f64;
            if x + 1 < width {
                edge_sum += (here - frame[y * width + x + 1] as f64).abs() / 255.0;
                edge_count += 1;
            }
            if y + 1 < height {
                edge_sum += (here - frame[(y + 1) * width + x] as f64).abs() / 255.0;
                edge_count += 1;
            }
        }
    }

    let radial_profile = normalized_radial_profile(frame, width, height, 16);
    let angular_profile = normalized_angular_profile(frame, width, height, 24);
    let dominant_angle_degrees = dominant_profile_angle_degrees(&angular_profile);

    BressloffStillMetrics {
        mean_luma: mean,
        std_luma: variance.sqrt(),
        active_fraction,
        edge_density: if edge_count == 0 {
            0.0
        } else {
            edge_sum / edge_count as f64
        },
        dominant_angle_degrees,
        radial_profile,
        angular_profile,
    }
}

pub(crate) fn normalized_radial_profile(
    frame: &[u8],
    width: usize,
    height: usize,
    bins: usize,
) -> Vec<f64> {
    let bins = bins.max(1);
    let mut sums = vec![0.0; bins];
    let mut counts = vec![0usize; bins];
    let cx = (width.saturating_sub(1)) as f64 * 0.5;
    let cy = (height.saturating_sub(1)) as f64 * 0.5;
    let max_radius = (cx * cx + cy * cy).sqrt().max(1.0e-9);
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let bin = (((dx * dx + dy * dy).sqrt() / max_radius) * bins as f64)
                .floor()
                .min((bins - 1) as f64) as usize;
            sums[bin] += frame[y * width + x] as f64 / 255.0;
            counts[bin] += 1;
        }
    }
    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| if count == 0 { 0.0 } else { sum / count as f64 })
        .collect()
}

pub(crate) fn normalized_angular_profile(
    frame: &[u8],
    width: usize,
    height: usize,
    bins: usize,
) -> Vec<f64> {
    let bins = bins.max(1);
    let mut sums = vec![0.0; bins];
    let mut counts = vec![0usize; bins];
    let cx = (width.saturating_sub(1)) as f64 * 0.5;
    let cy = (height.saturating_sub(1)) as f64 * 0.5;
    for y in 0..height {
        for x in 0..width {
            let angle = (y as f64 - cy).atan2(x as f64 - cx).rem_euclid(2.0 * PI);
            let bin = ((angle / (2.0 * PI)) * bins as f64)
                .floor()
                .min((bins - 1) as f64) as usize;
            sums[bin] += frame[y * width + x] as f64 / 255.0;
            counts[bin] += 1;
        }
    }
    sums.into_iter()
        .zip(counts)
        .map(|(sum, count)| if count == 0 { 0.0 } else { sum / count as f64 })
        .collect()
}

pub(crate) fn dominant_profile_angle_degrees(profile: &[f64]) -> f64 {
    if profile.is_empty() {
        return 0.0;
    }
    let (index, _) = profile
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap_or((0, &0.0));
    (index as f64 + 0.5) * 360.0 / profile.len() as f64
}

pub(crate) fn load_bressloff_source_profile(
    source_profile_dir: &Path,
    preset_id: &str,
) -> Result<Option<BressloffSourceProfile>, Box<dyn std::error::Error>> {
    let path = source_profile_dir.join(format!("{preset_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let body = fs::read_to_string(path)?;
    let profile = serde_json::from_str::<BressloffSourceProfile>(&body)?;
    if profile.preset_id != preset_id {
        return Err(format!(
            "source profile preset mismatch: expected {preset_id}, found {}",
            profile.preset_id
        )
        .into());
    }
    Ok(Some(profile))
}

pub(crate) fn bressloff_source_comparison(
    preset_id: &str,
    metrics: &BressloffStillMetrics,
    source_profile: Option<&BressloffSourceProfile>,
) -> BressloffSourceComparison {
    let Some(source) = source_profile else {
        return BressloffSourceComparison {
            status: "source-profile-missing",
            source_profile_id: None,
            source_mask_id: None,
            generated_dominant_angle_degrees: None,
            source_lattice_angle_degrees: None,
            radial_profile_error: None,
            angular_profile_error: None,
            edge_overlap: None,
            active_fraction_error: None,
            edge_density_error: None,
            lattice_angle_error_degrees: None,
        };
    };

    let radial_profile_error = source
        .radial_profile
        .as_ref()
        .and_then(|profile| mean_absolute_profile_error(&metrics.radial_profile, profile));
    let angular_profile_error = source
        .angular_profile
        .as_ref()
        .and_then(|profile| mean_absolute_profile_error(&metrics.angular_profile, profile));
    let active_fraction_error = source
        .active_fraction
        .map(|value| (metrics.active_fraction - value).abs());
    let edge_density_error = source
        .edge_density
        .map(|value| (metrics.edge_density - value).abs());
    let edge_overlap = source
        .edge_density
        .map(|value| edge_overlap_from_densities(metrics.edge_density, value));
    let lattice_angle_error_degrees = source
        .lattice_angle_degrees
        .map(|value| angular_difference_degrees(metrics.dominant_angle_degrees, value, 180.0));

    BressloffSourceComparison {
        status: "compared",
        source_profile_id: source
            .profile_id
            .clone()
            .or_else(|| Some(format!("{preset_id}-source-profile"))),
        source_mask_id: source.mask_id.clone(),
        generated_dominant_angle_degrees: Some(metrics.dominant_angle_degrees),
        source_lattice_angle_degrees: source.lattice_angle_degrees,
        radial_profile_error,
        angular_profile_error,
        edge_overlap,
        active_fraction_error,
        edge_density_error,
        lattice_angle_error_degrees,
    }
}

pub(crate) fn bressloff_geometry_acceptance_policy() -> BressloffGeometryAcceptancePolicy {
    BressloffGeometryAcceptancePolicy {
        policy_id: "bressloff-figure-geometry-diagnostic-v1",
        claim_level: "source-target-diagnostic",
        calibration_claim_allowed: false,
        radial_profile_error_max: 0.12,
        angular_profile_error_max: 0.12,
        edge_overlap_min: 0.70,
        active_fraction_error_max: 0.08,
        edge_density_error_max: 0.04,
        lattice_angle_error_degrees_max: 15.0,
        required_compared_still_fraction: 1.0,
        note: "These thresholds gate future calibration language. Passing them would only mark a source-target diagnostic as threshold-accepted; calibration claims remain disabled until private crop/threshold QA, source-angle review, and figure-level acceptance policy are separately approved.",
    }
}

pub(crate) fn evaluate_bressloff_geometry_acceptance(
    comparison: &BressloffSourceComparison,
) -> BressloffGeometryAcceptanceResult {
    let policy = bressloff_geometry_acceptance_policy();
    if comparison.status != "compared" {
        return BressloffGeometryAcceptanceResult {
            status: "source-profile-missing",
            passes_thresholds: false,
            evaluated_metric_count: 0,
            passed_metric_count: 0,
            failed_metrics: vec!["source_profile"],
            note: "No private source-derived profile was available, so the still cannot pass the diagnostic threshold gate.",
        };
    }

    let mut failed_metrics = Vec::new();
    let mut passed_metric_count = 0usize;
    let mut check_metric = |name: &'static str, passes: bool| {
        if passes {
            passed_metric_count += 1;
        } else {
            failed_metrics.push(name);
        }
    };
    check_metric(
        "radial_profile_error",
        comparison
            .radial_profile_error
            .is_some_and(|value| value <= policy.radial_profile_error_max),
    );
    check_metric(
        "angular_profile_error",
        comparison
            .angular_profile_error
            .is_some_and(|value| value <= policy.angular_profile_error_max),
    );
    check_metric(
        "edge_overlap",
        comparison
            .edge_overlap
            .is_some_and(|value| value >= policy.edge_overlap_min),
    );
    check_metric(
        "active_fraction_error",
        comparison
            .active_fraction_error
            .is_some_and(|value| value <= policy.active_fraction_error_max),
    );
    check_metric(
        "edge_density_error",
        comparison
            .edge_density_error
            .is_some_and(|value| value <= policy.edge_density_error_max),
    );
    check_metric(
        "lattice_angle_error_degrees",
        comparison
            .lattice_angle_error_degrees
            .is_some_and(|value| value <= policy.lattice_angle_error_degrees_max),
    );
    let passes_thresholds = failed_metrics.is_empty();
    BressloffGeometryAcceptanceResult {
        status: if passes_thresholds {
            "threshold-accepted-diagnostic"
        } else {
            "outside-threshold-diagnostic"
        },
        passes_thresholds,
        evaluated_metric_count: 6,
        passed_metric_count,
        failed_metrics,
        note: if passes_thresholds {
            "All current diagnostic thresholds pass, but calibration language remains disabled by policy."
        } else {
            "One or more diagnostic thresholds fail or are missing; use source-target diagnostic language only."
        },
    }
}

pub(crate) fn mean_absolute_profile_error(generated: &[f64], source: &[f64]) -> Option<f64> {
    let len = generated.len().min(source.len());
    if len == 0 {
        return None;
    }
    Some(
        generated
            .iter()
            .take(len)
            .zip(source.iter().take(len))
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
            / len as f64,
    )
}

pub(crate) fn edge_overlap_from_densities(generated: f64, source: f64) -> f64 {
    let denom = generated.max(source).max(1.0e-9);
    1.0 - ((generated - source).abs() / denom).clamp(0.0, 1.0)
}

pub(crate) fn angular_difference_degrees(a: f64, b: f64, period: f64) -> f64 {
    let mut delta = (a - b).abs().rem_euclid(period);
    if delta > period * 0.5 {
        delta = period - delta;
    }
    delta
}

pub(crate) fn bressloff_stability_reports() -> Vec<StabilityCalibrationRun> {
    vec![
        stability_report_for(StabilityReportSpec {
            id: "fig37_even_coefficients",
            label: "Fig 37 even eigen/coefficient sign target",
            source_key: "bressloff-2001",
            source_page: "24",
            paper_figure: "Figure 37",
            target: "even perturbative eigenfunction coefficients and even marginal branch",
            params: apply_paper_preset(FrameParams::default(), PaperPreset::Fig17Even),
            expected_branch: "even",
            expected_family: "any",
            expected_pattern: "any",
        }),
        stability_report_for(StabilityReportSpec {
            id: "fig38_even_hex_bifurcation",
            label: "Fig 38 even hexagonal bifurcation target",
            source_key: "bressloff-2001",
            source_page: "24",
            paper_figure: "Figure 38",
            target: "even hexagonal branch and roll exchange diagnostic",
            params: apply_paper_preset(FrameParams::default(), PaperPreset::Fig35HexZeroEven),
            expected_branch: "even",
            expected_family: "hexagonal",
            expected_pattern: "honeycomb",
        }),
        stability_report_for(StabilityReportSpec {
            id: "fig39_odd_coefficients",
            label: "Fig 39 odd eigen/coefficient sign target",
            source_key: "bressloff-2001",
            source_page: "25",
            paper_figure: "Figure 39",
            target: "odd perturbative eigenfunction coefficients and odd marginal branch",
            params: apply_paper_preset(FrameParams::default(), PaperPreset::Fig16Odd),
            expected_branch: "odd",
            expected_family: "any",
            expected_pattern: "any",
        }),
        stability_report_for(StabilityReportSpec {
            id: "fig40_odd_hex_bifurcation",
            label: "Fig 40 odd hexagonal bifurcation target",
            source_key: "bressloff-2001",
            source_page: "25",
            paper_figure: "Figure 40",
            target: "odd hexagonal/triangular higher-order selection target",
            params: apply_paper_preset(FrameParams::default(), PaperPreset::Fig36TriangleOdd),
            expected_branch: "odd",
            expected_family: "hexagonal",
            expected_pattern: "triangle",
        }),
        stability_report_for(StabilityReportSpec {
            id: "rhombic_stability_angle",
            label: "Rhombic stability angle target",
            source_key: "bressloff-2001",
            source_page: "23",
            paper_figure: "Rhombic stability discussion",
            target: "rhombic branch check at the current representative angle",
            params: apply_paper_preset(FrameParams::default(), PaperPreset::Fig33RhombicEven),
            expected_branch: "even",
            expected_family: "rhombic",
            expected_pattern: "rhombic",
        }),
    ]
}

pub(crate) fn stability_report_for(spec: StabilityReportSpec) -> StabilityCalibrationRun {
    let planform = planform_details(spec.params, cell_mm_for(spec.params));
    let branch = &planform.branch_selection;
    let mut checks = Vec::new();
    checks.push(CalibrationCheck {
        name: "critical-branch",
        expected: spec.expected_branch,
        actual: planform.stability.critical_branch.to_string(),
        passed: planform.stability.critical_branch == spec.expected_branch,
    });
    if spec.expected_family != "any" {
        checks.push(CalibrationCheck {
            name: "selected-family",
            expected: spec.expected_family,
            actual: branch.selected_family.to_string(),
            passed: branch.selected_family == spec.expected_family,
        });
    }
    if spec.expected_pattern != "any" {
        checks.push(CalibrationCheck {
            name: "selected-pattern",
            expected: spec.expected_pattern,
            actual: branch.selected_pattern.to_string(),
            passed: branch.selected_pattern == spec.expected_pattern,
        });
    }
    let status = if checks.iter().all(|check| check.passed) {
        "pass"
    } else {
        "review"
    };
    StabilityCalibrationRun {
        id: spec.id,
        label: spec.label,
        source_key: spec.source_key,
        source_page: spec.source_page,
        paper_figure: spec.paper_figure,
        target: spec.target,
        status,
        rendered_parity: planform.parity,
        critical_q: planform.stability.critical_q,
        critical_branch: planform.stability.critical_branch,
        selected_family: branch.selected_family,
        selected_pattern: branch.selected_pattern,
        global_selected_family: branch.global_selected_family,
        global_selected_pattern: branch.global_selected_pattern,
        eta_hex: branch.eta_hex,
        gamma0: branch.gamma0,
        gamma_square: branch.gamma_square,
        gamma_rhombic: branch.gamma_rhombic,
        gamma_hex: branch.gamma_hex,
        checks,
    }
}
