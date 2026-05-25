use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;

use crate::{
    coerce_params, default_params_json, generate_payload, models::driven::driven_example_catalog,
    paper_preset_catalog, payload_cache_key, rule_preset_catalog, ServerState,
};

pub(crate) fn serve_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 8892_u16;
    let mut root = env::current_dir()?;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--host" => host = iter.next().ok_or("--host requires a value")?.clone(),
            "--port" => port = iter.next().ok_or("--port requires a value")?.parse()?,
            "--root" => root = PathBuf::from(iter.next().ok_or("--root requires a value")?),
            _ => {}
        }
    }

    let listener = TcpListener::bind((host.as_str(), port))?;
    let state = Arc::new(ServerState::default());
    println!("Serving Bressloff V1 viewer on http://{host}:{port}/viewer/index.html");
    for stream in listener.incoming() {
        let stream = stream?;
        let root = root.clone();
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            if let Err(error) = handle_connection(stream, &root, &state) {
                eprintln!("request error: {error}");
            }
        });
    }
    Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    root: &Path,
    state: &ServerState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0_u8; 8192];
    let size = stream.read(&mut buffer)?;
    if size == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buffer[..size]);
    let Some(line) = request.lines().next() else {
        return Ok(());
    };
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 || parts[0] != "GET" {
        write_response(&mut stream, 405, "text/plain", b"method not allowed")?;
        return Ok(());
    }

    let (path, query) = split_query(parts[1]);
    match path.as_str() {
        "/api/defaults" => {
            let body = serde_json::json!({
                "defaults": default_params_json(),
                "limits": {
                    "n": [32, 96],
                    "m": [4, 24],
                    "t": [5.0, 800.0],
                    "frames": [8, 240],
                    "seed": [0, u32::MAX],
                    "alpha": [0.1, 4.0],
                    "beta": [0.1, 10.0],
                    "mu": [1.0, 40.0],
                    "r0": [0.02, 0.14],
                    "low_percentile": [0.0, 20.0],
                    "high_percentile": [80.0, 100.0],
                    "trim_threshold": [0.0, 0.5],
                    "preview_step": [0.02, 1.0],
                    "wave_count": [1.0, 40.0],
                    "drift": [-4.0, 4.0],
                    "pattern_angle": [0.0, 90.0],
                    "sharpness": [0.25, 8.0],
                    "eigen_beta": [0.0, 1.5],
                    "hypercolumn_mm": [0.1, 4.0],
                    "local_sigma_deg": [1.0, 80.0],
                    "local_wide_sigma_deg": [5.0, 120.0],
                    "local_inhibition": [0.0, 3.0],
                    "lateral_sigma": [0.1, 4.0],
                    "lateral_wide_sigma": [0.1, 6.0],
                    "lateral_inhibition": [0.0, 3.0],
                    "lateral_spread_deg": [0.0, 90.0],
                    "stability_q_min": [0.0, 2.0],
                    "stability_q_max": [0.2, 8.0],
                    "stability_samples": [16, 256],
                    "export_orientation_channels": [false, true],
                    "rule_tau_e_ms": [1.0, 80.0],
                    "rule_tau_i_ms": [1.0, 120.0],
                    "rule_aee": [0.0, 30.0],
                    "rule_aei": [0.0, 30.0],
                    "rule_aie": [0.0, 30.0],
                    "rule_aii": [0.0, 30.0],
                    "rule_theta_e": [0.0, 8.0],
                    "rule_theta_i": [0.0, 8.0],
                    "rule_sigma_e": [0.4, 10.0],
                    "rule_sigma_i": [0.4, 16.0],
                    "rule_stim_amplitude": [0.0, 1.5],
                    "rule_stim_period_ms": [20.0, 180.0],
                    "rule_stim_threshold": [-1.0, 1.0],
                    "rule_stim_smoothing": [0.0, 100.0],
                    "rule_stim_i_fraction": [0.0, 1.0],
                    "rule_seed_strength": [0.0, 0.2]
                },
                "paper_presets": paper_preset_catalog(),
                "rule_presets": rule_preset_catalog(),
                "driven_examples": driven_example_catalog(),
                "generator_options": ["dynamics", "planform", "rule_flicker"],
                "pattern_options": ["auto", "rings", "rays", "spiral", "cobweb", "honeycomb", "rhombic", "hex_pi", "triangle"],
                "contour_mode_options": ["contoured", "noncontoured"],
                "parity_options": ["even", "odd"],
                "resolution_options": [32, 40, 48, 64, 80, 96],
                "orientation_options": [4, 8, 12, 16, 24],
                "rule_seed_pattern_options": ["random", "stripes", "hexagonal"],
                "solver_options": ["preview", "accurate"],
                "colormaps": ["twilight", "viridis", "magma", "inferno", "turbo", "gray"],
                "backend": "rust"
            });
            write_json(&mut stream, &body)?;
        }
        "/api/generate" => {
            let params = coerce_params(&query);
            let cache_key = payload_cache_key(params);
            let cached = state.payloads.lock().unwrap().get(&cache_key).cloned();
            let payload = if let Some(payload) = cached {
                payload
            } else {
                let payload = Arc::new(generate_payload(params, state)?);
                state
                    .payloads
                    .lock()
                    .unwrap()
                    .insert(cache_key, Arc::clone(&payload));
                payload
            };
            write_json(&mut stream, payload.as_ref())?;
        }
        _ => serve_static(&mut stream, root, &path)?,
    }
    Ok(())
}

fn split_query(target: &str) -> (String, HashMap<String, String>) {
    let mut pieces = target.splitn(2, '?');
    let path = pieces.next().unwrap_or("/").to_string();
    let query = pieces.next().map(parse_query).unwrap_or_default();
    (path, query)
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut pieces = pair.splitn(2, '=');
            let key = pieces.next()?;
            let value = pieces.next().unwrap_or("");
            Some((decode_uri_component(key), decode_uri_component(value)))
        })
        .collect()
}

fn decode_uri_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                    if let Ok(parsed) = u8::from_str_radix(hex, 16) {
                        out.push(parsed);
                        i += 3;
                        continue;
                    }
                }
                out.push(bytes[i]);
                i += 1;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn serve_static(
    stream: &mut TcpStream,
    root: &Path,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let safe_path = path.trim_start_matches('/').replace('\\', "/");
    if safe_path.contains("..") {
        write_response(stream, 400, "text/plain", b"bad path")?;
        return Ok(());
    }
    let path = if safe_path.is_empty() {
        root.join("viewer/index.html")
    } else {
        root.join(safe_path)
    };
    match fs::read(&path) {
        Ok(body) => write_response(stream, 200, content_type(&path), &body)?,
        Err(_) => write_response(stream, 404, "text/plain", b"not found")?,
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

fn write_json<T: Serialize>(
    stream: &mut TcpStream,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec(value)?;
    write_response(stream, 200, "application/json; charset=utf-8", &body)
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    Ok(())
}
