use std::{collections::HashMap, fs, path::PathBuf};

use crate::{coerce_params, generate_payload, ServerState};

pub(crate) fn export_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("viewer/frames.json");
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
            "--no-trim-warmup" => {
                raw.insert("trim_warmup".to_string(), "false".to_string());
            }
            flag if flag.starts_with("--") => {
                let key = flag.trim_start_matches("--").replace('-', "_");
                let value = iter.next().ok_or("flag requires a value")?;
                raw.insert(key, value.clone());
            }
            _ => {}
        }
    }

    let state = ServerState::default();
    let params = coerce_params(&raw);
    let payload = generate_payload(params, &state)?;
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out, serde_json::to_vec(&payload)?)?;
    println!(
        "wrote {} grid={}x{} orientations={} frames={} backend={} build={:.3}s solve={:.3}s trim={}",
        out.display(),
        payload.width,
        payload.height,
        payload.orientation_count,
        payload.frame_count,
        payload.timing.backend,
        payload.timing.matrix_build_sec,
        payload.timing.solve_sec,
        payload.warmup.dropped_frames
    );
    if let Some(channels) = &payload.orientation_channels {
        println!(
            "orientation_channels={}x{}x{}x{} raw=[{:.4},{:.4}]",
            channels.frame_count,
            channels.width,
            channels.height,
            channels.orientation_count,
            channels.raw_min,
            channels.raw_max
        );
    }
    if let Some(calibration) = &payload.calibration {
        println!(
            "calibration={} status={} rendered={} selected={}",
            calibration.preset.id,
            calibration.status,
            calibration.rendered_pattern,
            calibration.selected_family
        );
    }
    if let Some(rule) = &payload.rule {
        println!(
            "rule={} status={} family={} response={} freq={:.2}Hz",
            rule.preset.map(|preset| preset.id).unwrap_or("manual"),
            rule.status,
            rule.spatial_family,
            rule.response_mode,
            rule.stimulus_frequency_hz
        );
    }
    Ok(())
}
