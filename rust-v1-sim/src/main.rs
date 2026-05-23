use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;
use serde::Serialize;

const PI: f64 = std::f64::consts::PI;
const DYNAMIC_CELL_MM: f64 = 0.7;
const RETINO_EPS: f64 = 0.051;
const RETINO_W0: f64 = 0.087;
const RETINO_ALPHA: f64 = 3.0 / PI;
const RETINO_BETA: f64 = 1.589 / 2.0;

#[derive(Clone, Copy, Debug)]
struct FrameParams {
    generator: Generator,
    pattern: PatternPreset,
    parity: Parity,
    n: usize,
    m: usize,
    t: f64,
    frames: usize,
    seed: u64,
    alpha: f64,
    beta: f64,
    mu: f64,
    r0: f64,
    low_percentile: f64,
    high_percentile: f64,
    cmap: &'static str,
    trim_warmup: bool,
    trim_threshold: f64,
    solver: Solver,
    preview_step: f64,
    wave_count: f64,
    drift: f64,
    pattern_angle: f64,
    sharpness: f64,
    eigen_beta: f64,
    hypercolumn_mm: f64,
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
    stability_q_min: f64,
    stability_q_max: f64,
    stability_samples: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Generator {
    Dynamics,
    Planform,
}

impl Generator {
    fn as_str(self) -> &'static str {
        match self {
            Generator::Dynamics => "dynamics",
            Generator::Planform => "planform",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PatternPreset {
    Auto,
    Rings,
    Rays,
    Spiral,
    Cobweb,
    Honeycomb,
    Rhombic,
    HexPi,
}

impl PatternPreset {
    fn as_str(self) -> &'static str {
        match self {
            PatternPreset::Auto => "auto",
            PatternPreset::Rings => "rings",
            PatternPreset::Rays => "rays",
            PatternPreset::Spiral => "spiral",
            PatternPreset::Cobweb => "cobweb",
            PatternPreset::Honeycomb => "honeycomb",
            PatternPreset::Rhombic => "rhombic",
            PatternPreset::HexPi => "hex_pi",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parity {
    Even,
    Odd,
}

impl Parity {
    fn as_str(self) -> &'static str {
        match self {
            Parity::Even => "even",
            Parity::Odd => "odd",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Solver {
    Preview,
    Accurate,
}

impl Solver {
    fn as_str(self) -> &'static str {
        match self {
            Solver::Preview => "preview",
            Solver::Accurate => "accurate",
        }
    }
}

impl Default for FrameParams {
    fn default() -> Self {
        Self {
            generator: Generator::Dynamics,
            pattern: PatternPreset::Cobweb,
            parity: Parity::Even,
            n: 64,
            m: 12,
            t: 60.0,
            frames: 120,
            seed: 20_260_522,
            alpha: 1.0,
            beta: 3.0,
            mu: 17.0,
            r0: 3.2 / 50.0,
            low_percentile: 1.0,
            high_percentile: 99.0,
            cmap: "twilight",
            trim_warmup: true,
            trim_threshold: 0.08,
            solver: Solver::Preview,
            preview_step: 0.5,
            wave_count: 12.0,
            drift: 0.35,
            pattern_angle: 45.0,
            sharpness: 1.0,
            eigen_beta: 0.35,
            hypercolumn_mm: 2.0,
            local_sigma_deg: 20.0,
            local_wide_sigma_deg: 60.0,
            local_inhibition: 1.0,
            lateral_sigma: 1.0,
            lateral_wide_sigma: 1.5,
            lateral_inhibition: 1.0,
            lateral_spread_deg: 0.0,
            stability_q_min: 0.05,
            stability_q_max: 3.5,
            stability_samples: 80,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StructureKey {
    n: usize,
    m: usize,
    r0_key: i64,
}

impl StructureKey {
    fn new(params: FrameParams) -> Self {
        Self {
            n: params.n,
            m: params.m,
            r0_key: (params.r0 * 100_000_000.0).round() as i64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Offset {
    dr: isize,
    dc: isize,
    weight: f64,
}

#[derive(Clone, Copy, Debug)]
struct SourceWeight {
    source_index: usize,
    weight: f64,
}

#[derive(Debug)]
struct SectorSources {
    per_cell: usize,
    entries: Vec<SourceWeight>,
}

#[derive(Debug)]
struct Structure {
    m: usize,
    angle_weights: Vec<f64>,
    sector_sources: Vec<SectorSources>,
}

#[derive(Default)]
struct ServerState {
    structures: Mutex<HashMap<StructureKey, Arc<Structure>>>,
    payloads: Mutex<HashMap<String, Arc<Payload>>>,
}

#[derive(Serialize)]
struct Payload {
    format: &'static str,
    width: usize,
    height: usize,
    frame_count: usize,
    orientation_count: usize,
    times: Vec<f64>,
    scale_min: f64,
    scale_max: f64,
    raw_min: f32,
    raw_max: f32,
    cell_mm: f64,
    retino_bounds: RetinoBounds,
    retino_params: RetinoParams,
    palette: Vec<[u8; 3]>,
    planform: Option<PlanformDetails>,
    params: PayloadParams,
    metrics: Metrics,
    warmup: Warmup,
    timing: Timing,
    data_base64: String,
}

#[derive(Serialize)]
struct PayloadParams {
    generator: &'static str,
    pattern: &'static str,
    parity: &'static str,
    n: usize,
    m: usize,
    t: f64,
    frames: usize,
    seed: u64,
    alpha: f64,
    beta: f64,
    mu: f64,
    r0: f64,
    low_percentile: f64,
    high_percentile: f64,
    cmap: &'static str,
    trim_warmup: bool,
    trim_threshold: f64,
    solver: &'static str,
    preview_step: f64,
    wave_count: f64,
    drift: f64,
    pattern_angle: f64,
    sharpness: f64,
    eigen_beta: f64,
    hypercolumn_mm: f64,
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
    stability_q_min: f64,
    stability_q_max: f64,
    stability_samples: usize,
}

#[derive(Serialize)]
struct PlanformDetails {
    parity: &'static str,
    q: f64,
    wave_number: f64,
    phase_base: f64,
    modes: Vec<PlanformModeDetails>,
    eigen: OrientationEigenDetails,
    stability: StabilityDetails,
    branch_selection: BranchSelectionDetails,
    kernel: KernelDetails,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct PlanformModeDetails {
    normal_angle: f64,
    phase_scale: f64,
    amplitude: f64,
}

#[derive(Clone, Debug, Serialize)]
struct OrientationEigenDetails {
    parity: &'static str,
    beta: f64,
    cos_coefficients: Vec<[f64; 2]>,
    sin_coefficients: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize)]
struct KernelDetails {
    local_sigma_deg: f64,
    local_wide_sigma_deg: f64,
    local_inhibition: f64,
    lateral_sigma: f64,
    lateral_wide_sigma: f64,
    lateral_inhibition: f64,
    lateral_spread_deg: f64,
}

#[derive(Clone, Debug, Serialize)]
struct StabilityDetails {
    q_min: f64,
    q_max: f64,
    samples: usize,
    critical_q: f64,
    critical_branch: &'static str,
    critical_growth: f64,
    selected_pattern: &'static str,
    points: Vec<StabilityPoint>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct StabilityPoint {
    q: f64,
    even_growth: f64,
    odd_growth: f64,
}

#[derive(Clone, Debug, Serialize)]
struct BranchSelectionDetails {
    model: &'static str,
    lambda: f64,
    gamma0: f64,
    gamma_square: f64,
    gamma_rhombic: f64,
    gamma_hex: f64,
    eta_hex: f64,
    selected_family: &'static str,
    selected_pattern: &'static str,
    candidates: Vec<BranchCandidate>,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct BranchCandidate {
    family: &'static str,
    pattern: &'static str,
    mode_count: usize,
    theta_rad: f64,
    gamma_cross: f64,
    eta: f64,
    amplitude: f64,
    score: f64,
    stable: bool,
    note: &'static str,
}

#[derive(Serialize)]
struct Timing {
    matrix_build_sec: f64,
    solve_sec: f64,
    total_sec: f64,
    matrix_cache_hit: bool,
    backend: &'static str,
}

#[derive(Serialize)]
struct Metrics {
    final_mean: f32,
    final_std: f32,
    final_range: f32,
    dominant_cycles: f32,
    temporal_delta: f32,
}

#[derive(Serialize)]
struct Warmup {
    enabled: bool,
    dropped_frames: usize,
    start_time: f64,
    threshold_fraction: f64,
    threshold_std: f32,
    max_std: f32,
}

#[derive(Serialize)]
struct RetinoBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

#[derive(Serialize)]
struct RetinoParams {
    eps: f64,
    w0: f64,
    alpha: f64,
    beta: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("serve");
    match command {
        "export" => export_command(&args[2..])?,
        "serve" => serve_command(&args[2..])?,
        "--help" | "-h" => print_usage(),
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
        }
    }
    Ok(())
}

fn print_usage() {
    println!(
        "usage:\n  bressloff-v1 serve [--host 127.0.0.1] [--port 8892] [--root .]\n  bressloff-v1 export [--out viewer/frames.json] [model params]"
    );
}

fn export_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("viewer/frames.json");
    let mut raw = HashMap::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(iter.next().ok_or("--out requires a value")?),
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
    Ok(())
}

fn serve_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
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
                    "t": [5.0, 140.0],
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
                    "stability_samples": [16, 256]
                },
                "generator_options": ["dynamics", "planform"],
                "pattern_options": ["auto", "rings", "rays", "spiral", "cobweb", "honeycomb", "rhombic", "hex_pi"],
                "parity_options": ["even", "odd"],
                "resolution_options": [32, 48, 64, 80, 96],
                "orientation_options": [4, 8, 12, 16, 24],
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
    let query = pieces.next().map(parse_query).unwrap_or_else(HashMap::new);
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

fn default_params_json() -> serde_json::Value {
    let defaults = FrameParams::default();
    serde_json::json!({
        "generator": defaults.generator.as_str(),
        "pattern": defaults.pattern.as_str(),
        "parity": defaults.parity.as_str(),
        "n": defaults.n,
        "m": defaults.m,
        "t": defaults.t,
        "frames": defaults.frames,
        "seed": defaults.seed,
        "alpha": defaults.alpha,
        "beta": defaults.beta,
        "mu": defaults.mu,
        "r0": defaults.r0,
        "low_percentile": defaults.low_percentile,
        "high_percentile": defaults.high_percentile,
        "cmap": defaults.cmap,
        "trim_warmup": defaults.trim_warmup,
        "trim_threshold": defaults.trim_threshold,
        "solver": defaults.solver.as_str(),
        "preview_step": defaults.preview_step,
        "wave_count": defaults.wave_count,
        "drift": defaults.drift,
        "pattern_angle": defaults.pattern_angle,
        "sharpness": defaults.sharpness,
        "eigen_beta": defaults.eigen_beta,
        "hypercolumn_mm": defaults.hypercolumn_mm,
        "local_sigma_deg": defaults.local_sigma_deg,
        "local_wide_sigma_deg": defaults.local_wide_sigma_deg,
        "local_inhibition": defaults.local_inhibition,
        "lateral_sigma": defaults.lateral_sigma,
        "lateral_wide_sigma": defaults.lateral_wide_sigma,
        "lateral_inhibition": defaults.lateral_inhibition,
        "lateral_spread_deg": defaults.lateral_spread_deg,
        "stability_q_min": defaults.stability_q_min,
        "stability_q_max": defaults.stability_q_max,
        "stability_samples": defaults.stability_samples
    })
}

fn payload_cache_key(params: FrameParams) -> String {
    format!(
        "{}:{}:{}:{}:{}:{:.6}:{}:{}:{:.4}:{:.4}:{:.4}:{:.6}:{:.2}:{:.2}:{}:{}:{:.3}:{}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{}",
        params.generator.as_str(),
        params.pattern.as_str(),
        params.parity.as_str(),
        params.n,
        params.m,
        params.t,
        params.frames,
        params.seed,
        params.alpha,
        params.beta,
        params.mu,
        params.r0,
        params.low_percentile,
        params.high_percentile,
        params.cmap,
        params.trim_warmup,
        params.trim_threshold,
        params.solver.as_str(),
        params.preview_step,
        params.wave_count,
        params.drift,
        params.pattern_angle,
        params.sharpness,
        params.eigen_beta,
        params.hypercolumn_mm,
        params.local_sigma_deg,
        params.local_wide_sigma_deg,
        params.local_inhibition,
        params.lateral_sigma,
        params.lateral_wide_sigma,
        params.lateral_inhibition,
        params.lateral_spread_deg,
        params.stability_q_min,
        params.stability_q_max,
        params.stability_samples
    )
}

fn coerce_params(raw: &HashMap<String, String>) -> FrameParams {
    let defaults = FrameParams::default();
    let mut low = get_f64(raw, "low_percentile", defaults.low_percentile, 0.0, 20.0);
    let mut high = get_f64(
        raw,
        "high_percentile",
        defaults.high_percentile,
        80.0,
        100.0,
    );
    if high <= low {
        low = defaults.low_percentile;
        high = defaults.high_percentile;
    }

    FrameParams {
        generator: match raw.get("generator").map(String::as_str) {
            Some("planform") => Generator::Planform,
            _ => Generator::Dynamics,
        },
        pattern: match raw.get("pattern").map(String::as_str) {
            Some("auto") => PatternPreset::Auto,
            Some("rings") => PatternPreset::Rings,
            Some("rays") => PatternPreset::Rays,
            Some("spiral") => PatternPreset::Spiral,
            Some("honeycomb") => PatternPreset::Honeycomb,
            Some("rhombic") => PatternPreset::Rhombic,
            Some("hex_pi") => PatternPreset::HexPi,
            _ => PatternPreset::Cobweb,
        },
        parity: match raw.get("parity").map(String::as_str) {
            Some("odd") => Parity::Odd,
            _ => Parity::Even,
        },
        n: get_usize(raw, "n", defaults.n, 32, 96),
        m: get_usize(raw, "m", defaults.m, 4, 24),
        t: get_f64(raw, "t", defaults.t, 5.0, 140.0),
        frames: get_usize(raw, "frames", defaults.frames, 8, 240),
        seed: get_u64(raw, "seed", defaults.seed),
        alpha: get_f64(raw, "alpha", defaults.alpha, 0.1, 4.0),
        beta: get_f64(raw, "beta", defaults.beta, 0.1, 10.0),
        mu: get_f64(raw, "mu", defaults.mu, 1.0, 40.0),
        r0: get_f64(raw, "r0", defaults.r0, 0.02, 0.14),
        low_percentile: low,
        high_percentile: high,
        cmap: colormap_name(raw.get("cmap").map(String::as_str).unwrap_or(defaults.cmap)),
        trim_warmup: get_bool(raw, "trim_warmup", defaults.trim_warmup),
        trim_threshold: get_f64(raw, "trim_threshold", defaults.trim_threshold, 0.0, 0.5),
        solver: match raw.get("solver").map(String::as_str) {
            Some("accurate") => Solver::Accurate,
            _ => Solver::Preview,
        },
        preview_step: get_f64(raw, "preview_step", defaults.preview_step, 0.02, 1.0),
        wave_count: get_f64(raw, "wave_count", defaults.wave_count, 1.0, 40.0),
        drift: get_f64(raw, "drift", defaults.drift, -4.0, 4.0),
        pattern_angle: get_f64(raw, "pattern_angle", defaults.pattern_angle, 0.0, 90.0),
        sharpness: get_f64(raw, "sharpness", defaults.sharpness, 0.25, 8.0),
        eigen_beta: get_f64(raw, "eigen_beta", defaults.eigen_beta, 0.0, 1.5),
        hypercolumn_mm: get_f64(raw, "hypercolumn_mm", defaults.hypercolumn_mm, 0.1, 4.0),
        local_sigma_deg: get_f64(raw, "local_sigma_deg", defaults.local_sigma_deg, 1.0, 80.0),
        local_wide_sigma_deg: get_f64(
            raw,
            "local_wide_sigma_deg",
            defaults.local_wide_sigma_deg,
            5.0,
            120.0,
        ),
        local_inhibition: get_f64(raw, "local_inhibition", defaults.local_inhibition, 0.0, 3.0),
        lateral_sigma: get_f64(raw, "lateral_sigma", defaults.lateral_sigma, 0.1, 4.0),
        lateral_wide_sigma: get_f64(
            raw,
            "lateral_wide_sigma",
            defaults.lateral_wide_sigma,
            0.1,
            6.0,
        ),
        lateral_inhibition: get_f64(
            raw,
            "lateral_inhibition",
            defaults.lateral_inhibition,
            0.0,
            3.0,
        ),
        lateral_spread_deg: get_f64(
            raw,
            "lateral_spread_deg",
            defaults.lateral_spread_deg,
            0.0,
            90.0,
        ),
        stability_q_min: get_f64(raw, "stability_q_min", defaults.stability_q_min, 0.0, 2.0),
        stability_q_max: get_f64(raw, "stability_q_max", defaults.stability_q_max, 0.2, 8.0),
        stability_samples: get_usize(
            raw,
            "stability_samples",
            defaults.stability_samples,
            16,
            256,
        ),
    }
}

fn get_usize(
    raw: &HashMap<String, String>,
    key: &str,
    default: usize,
    min: usize,
    max: usize,
) -> usize {
    raw.get(key)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

fn get_u64(raw: &HashMap<String, String>, key: &str, default: u64) -> u64 {
    raw.get(key)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn get_f64(raw: &HashMap<String, String>, key: &str, default: f64, min: f64, max: f64) -> f64 {
    raw.get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
        .clamp(min, max)
}

fn get_bool(raw: &HashMap<String, String>, key: &str, default: bool) -> bool {
    raw.get(key)
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(default)
}

fn generate_payload(
    params: FrameParams,
    state: &ServerState,
) -> Result<Payload, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let (mut frames, mut times, matrix_cache_hit, matrix_build_sec, solve_sec) =
        match params.generator {
            Generator::Dynamics => {
                let (structure, cache_hit) = get_structure(params, state);
                let built = Instant::now();
                let (frames, times) = simulate_frames(params, &structure);
                let solved = Instant::now();
                (
                    frames,
                    times,
                    cache_hit,
                    built.duration_since(started).as_secs_f64(),
                    solved.duration_since(built).as_secs_f64(),
                )
            }
            Generator::Planform => {
                let built = Instant::now();
                let (frames, times) = generate_planform_frames(params);
                let solved = Instant::now();
                (
                    frames,
                    times,
                    false,
                    built.duration_since(started).as_secs_f64(),
                    solved.duration_since(built).as_secs_f64(),
                )
            }
        };
    let warmup = match params.generator {
        Generator::Dynamics => trim_warmup(&mut frames, &mut times, params),
        Generator::Planform => Warmup {
            enabled: false,
            dropped_frames: 0,
            start_time: times.first().copied().unwrap_or(0.0),
            threshold_fraction: params.trim_threshold,
            threshold_std: 0.0,
            max_std: 0.0,
        },
    };
    let (scale_min, scale_max) =
        percentile_range(&frames, params.low_percentile, params.high_percentile);
    let (raw_min, raw_max) = raw_range(&frames);
    let normalized = normalize_u8(&frames, scale_min, scale_max);
    let metrics = frame_metrics(&frames, params.n);
    let cell_mm = cell_mm_for(params);
    let planform = match params.generator {
        Generator::Planform => Some(planform_details(params, cell_mm)),
        Generator::Dynamics => None,
    };

    Ok(Payload {
        format: "bressloff-v1-u8-frames",
        width: params.n,
        height: params.n,
        frame_count: times.len(),
        orientation_count: params.m,
        times,
        scale_min,
        scale_max,
        raw_min,
        raw_max,
        cell_mm,
        retino_bounds: retino_bounds(params.n, cell_mm),
        retino_params: RetinoParams {
            eps: RETINO_EPS,
            w0: RETINO_W0,
            alpha: RETINO_ALPHA,
            beta: RETINO_BETA,
        },
        palette: palette(params.cmap),
        planform,
        params: PayloadParams {
            generator: params.generator.as_str(),
            pattern: params.pattern.as_str(),
            parity: params.parity.as_str(),
            n: params.n,
            m: params.m,
            t: params.t,
            frames: params.frames,
            seed: params.seed,
            alpha: params.alpha,
            beta: params.beta,
            mu: params.mu,
            r0: params.r0,
            low_percentile: params.low_percentile,
            high_percentile: params.high_percentile,
            cmap: params.cmap,
            trim_warmup: params.trim_warmup,
            trim_threshold: params.trim_threshold,
            solver: params.solver.as_str(),
            preview_step: params.preview_step,
            wave_count: params.wave_count,
            drift: params.drift,
            pattern_angle: params.pattern_angle,
            sharpness: params.sharpness,
            eigen_beta: params.eigen_beta,
            hypercolumn_mm: params.hypercolumn_mm,
            local_sigma_deg: params.local_sigma_deg,
            local_wide_sigma_deg: params.local_wide_sigma_deg,
            local_inhibition: params.local_inhibition,
            lateral_sigma: params.lateral_sigma,
            lateral_wide_sigma: params.lateral_wide_sigma,
            lateral_inhibition: params.lateral_inhibition,
            lateral_spread_deg: params.lateral_spread_deg,
            stability_q_min: params.stability_q_min,
            stability_q_max: params.stability_q_max,
            stability_samples: params.stability_samples,
        },
        metrics,
        warmup,
        timing: Timing {
            matrix_build_sec,
            solve_sec,
            total_sec: started.elapsed().as_secs_f64(),
            matrix_cache_hit,
            backend: "rust",
        },
        data_base64: general_purpose::STANDARD.encode(normalized),
    })
}

fn generate_planform_frames(params: FrameParams) -> (Vec<f32>, Vec<f64>) {
    let frame_count = params.frames.max(1);
    let frame_size = params.n * params.n;
    let mut frames = vec![0.0_f32; frame_count * frame_size];
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

    frames
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
                    let value =
                        planform_value(planform_params, x, y, wave_number, phase, &modes, &eigen);
                    frame[row * params.n + col] = (value * params.sharpness).tanh() as f32;
                }
            }
        });

    (frames, times)
}

fn cell_mm_for(params: FrameParams) -> f64 {
    match params.generator {
        Generator::Dynamics => DYNAMIC_CELL_MM,
        Generator::Planform => (2.0 * PI * RETINO_BETA / RETINO_EPS) / params.n as f64,
    }
}

fn planform_details(params: FrameParams, cell_mm: f64) -> PlanformDetails {
    let stability = stability_scan(params);
    let planform_params = effective_planform_params(params, &stability);
    let wave_number = planform_wave_number(params, cell_mm, Some(&stability));
    let q = wave_number * params.hypercolumn_mm;
    let branch_selection = branch_selection(planform_params, &stability);
    let effective_pattern = effective_pattern(params, &branch_selection);
    PlanformDetails {
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

fn planform_wave_number(
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
    let roll = branch_candidate(
        "roll",
        "spiral",
        1,
        0.0,
        gamma0,
        0.0,
        lambda,
        gamma0,
        0.0,
        roll_stable,
        "single active wavevector",
    );
    let square = branch_candidate(
        "square",
        "cobweb",
        2,
        square_theta,
        gamma0,
        gamma_square,
        lambda,
        gamma0 + 2.0 * gamma_square,
        0.0,
        gamma_square > 0.0 && 2.0 * gamma_square < gamma0,
        "two equal amplitudes on a square lattice",
    );
    let rhombic = branch_candidate(
        "rhombic",
        "rhombic",
        2,
        rhombic_theta,
        gamma0,
        gamma_rhombic,
        lambda,
        gamma0 + 2.0 * gamma_rhombic,
        0.0,
        gamma_rhombic > 0.0 && 2.0 * gamma_rhombic < gamma0,
        "two equal amplitudes on an oblique lattice",
    );
    let hex_pattern = if eta_hex < 0.0 { "hex_pi" } else { "honeycomb" };
    let hex_note = match params.parity {
        Parity::Even => "three-wave hexagonal branch with quadratic term",
        Parity::Odd => "odd hexagonal branch has zero quadratic term at cubic order",
    };
    let hex = branch_candidate(
        "hexagonal",
        hex_pattern,
        3,
        hex_theta,
        gamma0,
        gamma_hex,
        lambda,
        gamma0 + 4.0 * gamma_hex,
        eta_hex,
        gamma_hex > 0.0
            && (params.parity == Parity::Even || 2.0 * gamma_hex < gamma0)
            && gamma0 + 4.0 * gamma_hex > 0.0,
        hex_note,
    );
    let mut candidates = vec![roll, square, rhombic, hex];
    candidates.sort_by(|a, b| {
        b.stable
            .cmp(&a.stable)
            .then_with(|| b.score.total_cmp(&a.score))
    });
    let selected = candidates.first().copied().unwrap_or(roll);

    BranchSelectionDetails {
        model: "cubic-amplitude-equation",
        lambda,
        gamma0,
        gamma_square,
        gamma_rhombic,
        gamma_hex,
        eta_hex,
        selected_family: selected.family,
        selected_pattern: selected.pattern,
        candidates,
    }
}

fn branch_candidate(
    family: &'static str,
    pattern: &'static str,
    mode_count: usize,
    theta_rad: f64,
    gamma0: f64,
    gamma_cross: f64,
    lambda: f64,
    denominator: f64,
    eta: f64,
    stable: bool,
    note: &'static str,
) -> BranchCandidate {
    let denominator = denominator.max(1.0e-9);
    let lambda = lambda.max(0.0);
    let amplitude = if lambda <= 0.0 {
        0.0
    } else if eta.abs() > 1.0e-9 {
        ((eta.abs() + (eta * eta + 4.0 * denominator * lambda).sqrt()) / (2.0 * denominator))
            .max(0.0)
    } else {
        (lambda / denominator).sqrt()
    };
    let score = if lambda <= 0.0 {
        f64::NEG_INFINITY
    } else if mode_count == 1 {
        lambda * lambda / (4.0 * gamma0.max(1.0e-9))
    } else {
        mode_count as f64
            * (0.5 * lambda * amplitude * amplitude + eta.abs() * amplitude.powi(3) / 3.0
                - 0.25 * denominator * amplitude.powi(4))
    };
    BranchCandidate {
        family,
        pattern,
        mode_count,
        theta_rad,
        gamma_cross,
        eta,
        amplitude,
        score,
        stable: stable && lambda > 0.0 && amplitude.is_finite(),
        note,
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

fn planform_modes(params: FrameParams, pattern: PatternPreset) -> Vec<PlanformModeDetails> {
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
    }
}

fn planform_mode(normal_angle: f64, phase_scale: f64, amplitude: f64) -> PlanformModeDetails {
    PlanformModeDetails {
        normal_angle,
        phase_scale,
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
            let spatial =
                mode.amplitude * (wave_number * projection + phase * mode.phase_scale).cos();
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
    let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
    lateral_spread_factor(params, n)
        * 0.5
        * sign
        * ((-narrow).exp() * modified_bessel_i(n, narrow)
            - inhibition * (-broad).exp() * modified_bessel_i(n, broad))
}

fn lateral_spread_factor(params: FrameParams, n: usize) -> f64 {
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

fn get_structure(params: FrameParams, state: &ServerState) -> (Arc<Structure>, bool) {
    let key = StructureKey::new(params);
    if let Some(structure) = state.structures.lock().unwrap().get(&key).cloned() {
        return (structure, true);
    }
    let structure = Arc::new(Structure::new(params.n, params.m, params.r0));
    state
        .structures
        .lock()
        .unwrap()
        .insert(key, Arc::clone(&structure));
    (structure, false)
}

impl Structure {
    fn new(n: usize, m: usize, r0: f64) -> Self {
        let (sigma1, sigma2) = get_lateral_sigmas(r0);
        let step_size = 1.0 / n as f64;
        let cell_area = step_size * step_size;
        let delta_phi = PI / m as f64;
        let cutoff_radius = 3.5 * sigma2;
        let kernel_half_width = (cutoff_radius / step_size).floor() as isize;
        let mut offsets_by_sector = vec![Vec::new(); m];

        for dr in -kernel_half_width..=kernel_half_width {
            for dc in -kernel_half_width..=kernel_half_width {
                let x = step_size * dr as f64;
                let y = step_size * dc as f64;
                let dist = (x * x + y * y).sqrt();
                if dist > cutoff_radius || dist <= step_size / 2.0 {
                    continue;
                }
                let kernel_weight = weight_func(dist, sigma1, sigma2) / dist;
                let angle = y.atan2(x).rem_euclid(2.0 * PI);
                let sector =
                    (((angle + delta_phi / 2.0).rem_euclid(PI)) / delta_phi).floor() as usize;
                offsets_by_sector[sector.min(m - 1)].push(Offset {
                    dr,
                    dc,
                    weight: kernel_weight * cell_area / delta_phi,
                });
            }
        }

        let mut angle_weights = vec![0.0; m * m];
        for k in 0..m {
            for l in 0..m {
                let angle_k = PI * k as f64 / m as f64;
                let angle_l = PI * l as f64 / m as f64;
                angle_weights[k * m + l] = weight_func(
                    angle_dist(angle_k, angle_l),
                    0.6060482974023431,
                    1.538382226567759,
                );
            }
        }

        let cell_count = n * n;
        let mut sector_sources = Vec::with_capacity(m);
        for (sector, offsets) in offsets_by_sector.iter().enumerate() {
            let mut entries = Vec::with_capacity(cell_count * offsets.len());
            for row in 0..n {
                for col in 0..n {
                    for offset in offsets {
                        let source_row = wrap_index(row, offset.dr, n);
                        let source_col = wrap_index(col, offset.dc, n);
                        entries.push(SourceWeight {
                            source_index: index(source_row, source_col, sector, n, m),
                            weight: offset.weight,
                        });
                    }
                }
            }
            sector_sources.push(SectorSources {
                per_cell: offsets.len(),
                entries,
            });
        }

        Self {
            m,
            angle_weights,
            sector_sources,
        }
    }
}

fn simulate_frames(params: FrameParams, structure: &Structure) -> (Vec<f32>, Vec<f64>) {
    let total_dim = params.n * params.n * params.m;
    let mut rng = SplitMix64::new(params.seed);
    let mut state: Vec<f64> = (0..total_dim)
        .map(|_| (rng.next_f64() * 2.0 - 1.0) * 1.0e-12)
        .collect();
    let mut times = Vec::with_capacity(params.frames);
    let mut frames = Vec::with_capacity(params.frames * params.n * params.n);
    let mut sigmoid_buffer = vec![0.0; total_dim];
    let mut coupling_buffer = vec![0.0; total_dim];
    let step = match params.solver {
        Solver::Preview => params.preview_step,
        Solver::Accurate => params.preview_step.min(0.08),
    };
    let mut current_t = 0.0;

    for frame_index in 0..params.frames {
        let target_t = if params.frames <= 1 {
            0.0
        } else {
            params.t * frame_index as f64 / (params.frames - 1) as f64
        };

        while current_t + 1.0e-12 < target_t {
            let dt = step.min(target_t - current_t);
            match params.solver {
                Solver::Preview => step_preview(
                    &mut state,
                    structure,
                    params,
                    dt,
                    &mut sigmoid_buffer,
                    &mut coupling_buffer,
                ),
                Solver::Accurate => step_rk4(&mut state, structure, params, dt),
            }
            current_t += dt;
        }

        times.push(target_t);
        append_scalar_frame(&state, params, &mut frames);
    }

    (frames, times)
}

fn append_scalar_frame(state: &[f64], params: FrameParams, frames: &mut Vec<f32>) {
    for cell in 0..params.n * params.n {
        let base = cell * params.m;
        let mut sum = 0.0;
        for k in 0..params.m {
            sum += state[base + k];
        }
        frames.push((sum / params.m as f64) as f32);
    }
}

fn step_preview(
    state: &mut [f64],
    structure: &Structure,
    params: FrameParams,
    dt: f64,
    sigmoid_buffer: &mut [f64],
    coupling_buffer: &mut [f64],
) {
    connectivity_into(state, structure, params, sigmoid_buffer, coupling_buffer);
    let decay = 1.0 + params.alpha * dt;
    state
        .par_iter_mut()
        .zip(coupling_buffer.par_iter())
        .for_each(|(value, coupling)| {
            *value = (*value + dt * coupling) / decay;
        });
}

fn step_rk4(state: &mut [f64], structure: &Structure, params: FrameParams, dt: f64) {
    let k1 = derivative(state, structure, params);
    let tmp2: Vec<f64> = state
        .iter()
        .zip(k1.iter())
        .map(|(a, k)| a + 0.5 * dt * k)
        .collect();
    let k2 = derivative(&tmp2, structure, params);
    let tmp3: Vec<f64> = state
        .iter()
        .zip(k2.iter())
        .map(|(a, k)| a + 0.5 * dt * k)
        .collect();
    let k3 = derivative(&tmp3, structure, params);
    let tmp4: Vec<f64> = state
        .iter()
        .zip(k3.iter())
        .map(|(a, k)| a + dt * k)
        .collect();
    let k4 = derivative(&tmp4, structure, params);
    state.par_iter_mut().enumerate().for_each(|(i, value)| {
        *value += dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;
    });
}

fn derivative(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut conn = connectivity(state, structure, params);
    conn.par_iter_mut()
        .zip(state.par_iter())
        .for_each(|(value, a)| *value -= params.alpha * a);
    conn
}

fn connectivity(state: &[f64], structure: &Structure, params: FrameParams) -> Vec<f64> {
    let mut sigmoid_state = vec![0.0; state.len()];
    let mut out = vec![0.0; state.len()];
    connectivity_into(state, structure, params, &mut sigmoid_state, &mut out);
    out
}

fn connectivity_into(
    state: &[f64],
    structure: &Structure,
    params: FrameParams,
    sigmoid_state: &mut [f64],
    out: &mut [f64],
) {
    let m = structure.m;
    sigmoid_state
        .par_iter_mut()
        .zip(state.par_iter())
        .for_each(|(target, value)| *target = sigmoid(*value));

    out.par_chunks_mut(m).enumerate().for_each(|(cell, chunk)| {
        let base = cell * m;
        for k in 0..m {
            let mut angular_sum = 0.0;
            for l in 0..m {
                if l != k {
                    angular_sum +=
                        structure.angle_weights[k * m + l] * sigmoid_state[base + l] / m as f64;
                }
            }

            let mut lateral_sum = 0.0;
            let sector = &structure.sector_sources[k];
            let start = cell * sector.per_cell;
            let end = start + sector.per_cell;
            for source in &sector.entries[start..end] {
                lateral_sum += source.weight * sigmoid_state[source.source_index];
            }
            chunk[k] = params.mu * (angular_sum + params.beta * lateral_sum);
        }
    });
}

fn wrap_index(value: usize, delta: isize, size: usize) -> usize {
    (value as isize + delta).rem_euclid(size as isize) as usize
}

fn index(row: usize, col: usize, k: usize, n: usize, m: usize) -> usize {
    m * n * row + m * col + k
}

fn trim_warmup(frames: &mut Vec<f32>, times: &mut Vec<f64>, params: FrameParams) -> Warmup {
    if !params.trim_warmup || times.len() <= 3 {
        return Warmup {
            enabled: params.trim_warmup,
            dropped_frames: 0,
            start_time: times.first().copied().unwrap_or(0.0),
            threshold_fraction: params.trim_threshold,
            threshold_std: 0.0,
            max_std: 0.0,
        };
    }

    let frame_size = params.n * params.n;
    let contrast: Vec<f32> = frames.chunks(frame_size).map(stddev).collect();
    let max_std = contrast.iter().copied().fold(0.0_f32, f32::max);
    let threshold_std = max_std * params.trim_threshold as f32;
    let mut start = contrast
        .iter()
        .position(|value| *value >= threshold_std)
        .unwrap_or(0)
        .saturating_sub(2);
    let min_remaining = times.len().min(16.max(times.len() / 3));
    start = start.min(times.len().saturating_sub(min_remaining));

    if start > 0 {
        frames.drain(0..start * frame_size);
        times.drain(0..start);
    }

    Warmup {
        enabled: true,
        dropped_frames: start,
        start_time: times.first().copied().unwrap_or(0.0),
        threshold_fraction: params.trim_threshold,
        threshold_std,
        max_std,
    }
}

fn percentile_range(frames: &[f32], low_percentile: f64, high_percentile: f64) -> (f64, f64) {
    let mut sorted = frames.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let low = percentile_sorted(&sorted, low_percentile);
    let mut high = percentile_sorted(&sorted, high_percentile);
    if high <= low {
        high = low + 1.0e-9;
    }
    (low as f64, high as f64)
}

fn percentile_sorted(sorted: &[f32], percentile: f64) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (percentile.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let t = (rank - lo as f64) as f32;
        sorted[lo] * (1.0 - t) + sorted[hi] * t
    }
}

fn normalize_u8(frames: &[f32], low: f64, high: f64) -> Vec<u8> {
    let denom = (high - low).max(1.0e-9);
    frames
        .iter()
        .map(|value| (((*value as f64 - low) / denom) * 255.0).clamp(0.0, 255.0) as u8)
        .collect()
}

fn raw_range(frames: &[f32]) -> (f32, f32) {
    frames
        .iter()
        .copied()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        })
}

fn frame_metrics(frames: &[f32], n: usize) -> Metrics {
    let frame_size = n * n;
    let Some(final_frame) = frames.chunks(frame_size).last() else {
        return Metrics {
            final_mean: 0.0,
            final_std: 0.0,
            final_range: 0.0,
            dominant_cycles: 0.0,
            temporal_delta: 0.0,
        };
    };
    let mean = final_frame.iter().sum::<f32>() / final_frame.len() as f32;
    let std = stddev(final_frame);
    let (lo, hi) = raw_range(final_frame);
    let dominant_cycles = projected_dominant_cycles(final_frame, n);
    let temporal_delta = if frames.len() > frame_size {
        frames
            .windows(frame_size * 2)
            .step_by(frame_size)
            .map(|pair| {
                pair[..frame_size]
                    .iter()
                    .zip(pair[frame_size..].iter())
                    .map(|(a, b)| (b - a).abs())
                    .sum::<f32>()
                    / frame_size as f32
            })
            .sum::<f32>()
            / (frames.len() / frame_size - 1) as f32
    } else {
        0.0
    };

    Metrics {
        final_mean: mean,
        final_std: std,
        final_range: hi - lo,
        dominant_cycles,
        temporal_delta,
    }
}

fn stddev(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values
        .iter()
        .map(|value| {
            let delta = *value - mean;
            delta * delta
        })
        .sum::<f32>()
        / values.len() as f32;
    variance.sqrt()
}

fn projected_dominant_cycles(frame: &[f32], n: usize) -> f32 {
    let mut row_profile = vec![0.0_f32; n];
    let mut col_profile = vec![0.0_f32; n];
    for row in 0..n {
        for col in 0..n {
            let value = frame[row * n + col];
            row_profile[row] += value;
            col_profile[col] += value;
        }
    }
    let row_k = dominant_1d_frequency(&row_profile);
    let col_k = dominant_1d_frequency(&col_profile);
    ((row_k * row_k + col_k * col_k) as f32).sqrt()
}

fn dominant_1d_frequency(values: &[f32]) -> usize {
    let n = values.len();
    let mean = values.iter().sum::<f32>() / n as f32;
    let mut best_k = 0;
    let mut best_power = 0.0_f64;
    for k in 1..n / 2 {
        let mut re = 0.0;
        let mut im = 0.0;
        for (i, value) in values.iter().enumerate() {
            let angle = -2.0 * PI * k as f64 * i as f64 / n as f64;
            let centered = (*value - mean) as f64;
            re += centered * angle.cos();
            im += centered * angle.sin();
        }
        let power = re * re + im * im;
        if power > best_power {
            best_power = power;
            best_k = k;
        }
    }
    best_k
}

fn retino_bounds(n: usize, cell_mm: f64) -> RetinoBounds {
    let mut bounds = RetinoBounds {
        min_x: f64::INFINITY,
        max_x: f64::NEG_INFINITY,
        min_y: f64::INFINITY,
        max_y: f64::NEG_INFINITY,
    };
    for row in 0..=n {
        for col in 0..=n {
            let x = cell_mm * col as f64 - n as f64 * cell_mm / 2.0;
            let y = cell_mm * row as f64 - n as f64 * cell_mm / 2.0;
            let (rx, ry) = inverse_retino_cortical_map(x, y);
            bounds.min_x = bounds.min_x.min(rx);
            bounds.max_x = bounds.max_x.max(rx);
            bounds.min_y = bounds.min_y.min(ry);
            bounds.max_y = bounds.max_y.max(ry);
        }
    }
    bounds
}

fn inverse_retino_cortical_map(x: f64, y: f64) -> (f64, f64) {
    let r = RETINO_W0 / RETINO_EPS * (RETINO_EPS * x / RETINO_ALPHA).exp();
    let theta = RETINO_EPS * y / RETINO_BETA;
    (r * theta.cos(), r * theta.sin())
}

fn palette(name: &str) -> Vec<[u8; 3]> {
    (0..256)
        .map(|i| {
            let t = i as f64 / 255.0;
            match name {
                "gray" => {
                    let v = (255.0 * t).round() as u8;
                    [v, v, v]
                }
                "viridis" => interpolate_stops(t, VIRIDIS),
                "magma" => interpolate_stops(t, MAGMA),
                "inferno" => interpolate_stops(t, INFERNO),
                "turbo" => turbo(t),
                _ => interpolate_stops(t, TWILIGHT),
            }
        })
        .collect()
}

fn colormap_name(name: &str) -> &'static str {
    match name {
        "viridis" => "viridis",
        "magma" => "magma",
        "inferno" => "inferno",
        "turbo" => "turbo",
        "gray" => "gray",
        _ => "twilight",
    }
}

type Stop = (f64, u8, u8, u8);

const TWILIGHT: &[Stop] = &[
    (0.0, 34, 25, 74),
    (0.18, 68, 56, 130),
    (0.36, 64, 125, 177),
    (0.50, 222, 219, 221),
    (0.66, 190, 91, 81),
    (0.84, 93, 35, 95),
    (1.0, 34, 25, 74),
];
const VIRIDIS: &[Stop] = &[
    (0.0, 68, 1, 84),
    (0.25, 59, 82, 139),
    (0.5, 33, 145, 140),
    (0.75, 94, 201, 98),
    (1.0, 253, 231, 37),
];
const MAGMA: &[Stop] = &[
    (0.0, 0, 0, 4),
    (0.25, 80, 18, 123),
    (0.5, 182, 54, 121),
    (0.75, 251, 136, 97),
    (1.0, 252, 253, 191),
];
const INFERNO: &[Stop] = &[
    (0.0, 0, 0, 4),
    (0.25, 87, 15, 109),
    (0.5, 188, 55, 84),
    (0.75, 249, 142, 8),
    (1.0, 252, 255, 164),
];

fn interpolate_stops(t: f64, stops: &[Stop]) -> [u8; 3] {
    for window in stops.windows(2) {
        let (a_t, ar, ag, ab) = window[0];
        let (b_t, br, bg, bb) = window[1];
        if t <= b_t {
            let local = ((t - a_t) / (b_t - a_t)).clamp(0.0, 1.0);
            return [
                lerp_u8(ar, br, local),
                lerp_u8(ag, bg, local),
                lerp_u8(ab, bb, local),
            ];
        }
    }
    let (_, r, g, b) = stops[stops.len() - 1];
    [r, g, b]
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn turbo(t: f64) -> [u8; 3] {
    interpolate_stops(
        t,
        &[
            (0.0, 48, 18, 59),
            (0.18, 34, 113, 186),
            (0.36, 29, 185, 206),
            (0.54, 112, 218, 87),
            (0.72, 249, 206, 57),
            (0.88, 238, 98, 38),
            (1.0, 122, 4, 3),
        ],
    )
}

fn weight_func(x: f64, sigma1: f64, sigma2: f64) -> f64 {
    (-(x * x) / (2.0 * sigma1 * sigma1)).exp() / sigma1
        - (-(x * x) / (2.0 * sigma2 * sigma2)).exp() / sigma2
}

fn sigmoid(x: f64) -> f64 {
    if x < -4.0 {
        0.0
    } else if x > 4.0 {
        1.0
    } else {
        1.0 / (1.0 + (-2.0 * x).exp())
    }
}

fn angle_dist(angle1: f64, angle2: f64) -> f64 {
    PI / 2.0 - (PI / 2.0 - (angle1 - angle2).abs().rem_euclid(PI)).abs()
}

fn get_lateral_sigmas(r0: f64) -> (f64, f64) {
    let mut sigma2 = r0;
    for _ in 0..100 {
        let diff = r0 - ((2.0 * sigma2 * (sigma2 + 1.0).ln()) / (sigma2 + 2.0)).sqrt();
        if diff.abs() < 1.0e-7 {
            return (sigma2 / (1.0 + sigma2), sigma2);
        }
        sigma2 += diff;
    }
    (sigma2 / (1.0 + sigma2), sigma2)
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_generates_payload() {
        let state = ServerState::default();
        let params = FrameParams {
            n: 32,
            m: 4,
            frames: 16,
            t: 10.0,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        assert_eq!(payload.width, 32);
        assert_eq!(payload.orientation_count, 4);
        assert!(!payload.data_base64.is_empty());
    }

    #[test]
    fn cache_marks_second_structure_hit() {
        let state = ServerState::default();
        let params = FrameParams {
            n: 32,
            m: 4,
            frames: 8,
            t: 5.0,
            ..FrameParams::default()
        };
        let first = generate_payload(params, &state).unwrap();
        let second = generate_payload(
            FrameParams {
                alpha: 1.2,
                ..params
            },
            &state,
        )
        .unwrap();
        assert!(!first.timing.matrix_cache_hit);
        assert!(second.timing.matrix_cache_hit);
    }

    #[test]
    fn lateral_spread_can_flip_even_odd_gap() {
        let base = FrameParams::default();
        assert!((lateral_spread_factor(base, 2) - 1.0).abs() < 1.0e-12);

        let spread = FrameParams {
            lateral_spread_deg: 60.0,
            ..base
        };
        assert!(lateral_spread_factor(spread, 2) < 0.0);
    }

    #[test]
    fn auto_planform_metadata_uses_critical_parity() {
        let state = ServerState::default();
        let params = FrameParams {
            generator: Generator::Planform,
            pattern: PatternPreset::Auto,
            n: 32,
            m: 8,
            frames: 2,
            t: 1.0,
            ..FrameParams::default()
        };
        let payload = generate_payload(params, &state).unwrap();
        let planform = payload.planform.as_ref().unwrap();
        assert_eq!(planform.parity, planform.stability.critical_branch);
        assert_eq!(planform.branch_selection.model, "cubic-amplitude-equation");
        assert!(!planform.branch_selection.candidates.is_empty());
        assert!(planform.branch_selection.gamma0.is_finite());
    }
}
