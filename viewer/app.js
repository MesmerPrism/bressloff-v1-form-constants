const canvas = document.getElementById("viewer");
const ctx = canvas.getContext("2d", { alpha: false });

const els = {
  paramForm: document.getElementById("paramForm"),
  paperPreset: document.getElementById("paper_preset"),
  runButton: document.getElementById("runButton"),
  resetButton: document.getElementById("resetButton"),
  playToggle: document.getElementById("playToggle"),
  frame: document.getElementById("frame"),
  fps: document.getElementById("fps"),
  fpsValue: document.getElementById("fpsValue"),
  view: document.getElementById("view"),
  smooth: document.getElementById("smooth"),
  contours: document.getElementById("contours"),
  contourStride: document.getElementById("contour_stride"),
  contourLength: document.getElementById("contour_length"),
  contourThreshold: document.getElementById("contour_threshold"),
  stats: document.getElementById("stats"),
  status: document.getElementById("status"),
  time: document.getElementById("time"),
  metrics: document.getElementById("metrics"),
  stability: document.getElementById("stability"),
};

const DEFAULT_PARAMS = {
  generator: "planform",
  pattern: "auto",
  parity: "even",
  n: 96,
  m: 24,
  t: 18,
  frames: 120,
  seed: 20260522,
  alpha: 1,
  beta: 3,
  mu: 17,
  r0: 0.064,
  low_percentile: 1,
  high_percentile: 99,
  cmap: "twilight",
  trim_warmup: true,
  solver: "preview",
  preview_step: 0.5,
  wave_count: 12,
  drift: 0.45,
  pattern_angle: 45,
  sharpness: 1.8,
  eigen_beta: 0.35,
  hypercolumn_mm: 2,
  local_sigma_deg: 20,
  local_wide_sigma_deg: 60,
  local_inhibition: 1,
  lateral_sigma: 1,
  lateral_wide_sigma: 1.5,
  lateral_inhibition: 1,
  lateral_spread_deg: 0,
  stability_q_min: 0.05,
  stability_q_max: 3.5,
  stability_samples: 80,
};

const PAPER_PRESETS = {
  fig16_odd: {
    generator: "planform",
    pattern: "auto",
    parity: "even",
    eigen_beta: 0.4,
    lateral_sigma: 1,
    lateral_wide_sigma: 3,
    lateral_inhibition: 1,
    lateral_spread_deg: 0,
    stability_q_min: 0.05,
    stability_q_max: 3.5,
    stability_samples: 128,
    wave_count: 12,
    sharpness: 1.8,
  },
  fig17_even: {
    generator: "planform",
    pattern: "auto",
    parity: "even",
    eigen_beta: 0.4,
    lateral_sigma: 1,
    lateral_wide_sigma: 3,
    lateral_inhibition: 1,
    lateral_spread_deg: 60,
    stability_q_min: 0.05,
    stability_q_max: 3.5,
    stability_samples: 128,
    wave_count: 12,
    sharpness: 1.8,
  },
  fig31_square_even: {
    generator: "planform",
    pattern: "cobweb",
    parity: "even",
    lateral_spread_deg: 60,
    wave_count: 12,
    pattern_angle: 90,
    sharpness: 2.1,
  },
  fig32_square_odd: {
    generator: "planform",
    pattern: "cobweb",
    parity: "odd",
    lateral_spread_deg: 0,
    wave_count: 12,
    pattern_angle: 90,
    sharpness: 2.1,
  },
  fig33_rhombic_even: {
    generator: "planform",
    pattern: "rhombic",
    parity: "even",
    lateral_spread_deg: 60,
    wave_count: 12,
    pattern_angle: 45,
    sharpness: 2.1,
  },
  fig35_hex_even: {
    generator: "planform",
    pattern: "hex_pi",
    parity: "even",
    lateral_spread_deg: 60,
    wave_count: 12,
    pattern_angle: 60,
    sharpness: 2.1,
  },
};

const PARAM_IDS = [
  "generator",
  "pattern",
  "parity",
  "n",
  "m",
  "solver",
  "preview_step",
  "wave_count",
  "drift",
  "pattern_angle",
  "sharpness",
  "eigen_beta",
  "hypercolumn_mm",
  "local_sigma_deg",
  "local_wide_sigma_deg",
  "local_inhibition",
  "lateral_sigma",
  "lateral_wide_sigma",
  "lateral_inhibition",
  "lateral_spread_deg",
  "stability_q_min",
  "stability_q_max",
  "stability_samples",
  "alpha",
  "beta",
  "mu",
  "r0",
  "t",
  "frames",
  "seed",
  "cmap",
  "low_percentile",
  "high_percentile",
  "trim_warmup",
];

const FORMATTERS = {
  alpha: (value) => Number(value).toFixed(1),
  beta: (value) => Number(value).toFixed(1),
  mu: (value) => Number(value).toFixed(1),
  r0: (value) => Number(value).toFixed(3),
  t: (value) => Number(value).toFixed(0),
  frames: (value) => Number(value).toFixed(0),
  low_percentile: (value) => Number(value).toFixed(1),
  high_percentile: (value) => Number(value).toFixed(1),
  preview_step: (value) => Number(value).toFixed(2),
  wave_count: (value) => Number(value).toFixed(0),
  drift: (value) => Number(value).toFixed(2),
  pattern_angle: (value) => Number(value).toFixed(0),
  sharpness: (value) => Number(value).toFixed(1),
  eigen_beta: (value) => Number(value).toFixed(2),
  hypercolumn_mm: (value) => Number(value).toFixed(2),
  local_sigma_deg: (value) => Number(value).toFixed(0),
  local_wide_sigma_deg: (value) => Number(value).toFixed(0),
  local_inhibition: (value) => Number(value).toFixed(2),
  lateral_sigma: (value) => Number(value).toFixed(2),
  lateral_wide_sigma: (value) => Number(value).toFixed(2),
  lateral_inhibition: (value) => Number(value).toFixed(2),
  lateral_spread_deg: (value) => Number(value).toFixed(0),
  stability_q_min: (value) => Number(value).toFixed(2),
  stability_q_max: (value) => Number(value).toFixed(2),
  stability_samples: (value) => Number(value).toFixed(0),
  contour_stride: (value) => Number(value).toFixed(0),
  contour_length: (value) => Number(value).toFixed(0),
  contour_threshold: (value) => Number(value).toFixed(2),
};

const state = {
  meta: null,
  bytes: null,
  palette: null,
  frameSize: 0,
  currentFrame: 0,
  playing: true,
  fps: 24,
  lastFrameAt: 0,
  lastPaintAt: 0,
  paintCount: 0,
  measuredFps: 0,
  corticalCanvas: document.createElement("canvas"),
  corticalImage: null,
  retinalImage: null,
  retinalMap: null,
  retinalMapWidth: 0,
  retinalMapHeight: 0,
  requestId: 0,
};

function decodeBase64(value) {
  const binary = atob(value);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    out[i] = binary.charCodeAt(i);
  }
  return out;
}

function packPalette(colors) {
  const out = new Uint8ClampedArray(256 * 4);
  for (let i = 0; i < 256; i += 1) {
    out[i * 4] = colors[i][0];
    out[i * 4 + 1] = colors[i][1];
    out[i * 4 + 2] = colors[i][2];
    out[i * 4 + 3] = 255;
  }
  return out;
}

function resizeCanvas() {
  const rect = canvas.getBoundingClientRect();
  const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
  const size = Math.max(280, Math.round(Math.min(rect.width, rect.height) * dpr));
  if (canvas.width === size && canvas.height === size) {
    return;
  }
  canvas.width = size;
  canvas.height = size;
  state.retinalImage = ctx.createImageData(canvas.width, canvas.height);
  state.retinalMap = null;
}

function frameOffset(index) {
  return index * state.frameSize;
}

function writePixel(target, pixelIndex, value) {
  const p = value * 4;
  const t = pixelIndex * 4;
  target[t] = state.palette[p];
  target[t + 1] = state.palette[p + 1];
  target[t + 2] = state.palette[p + 2];
  target[t + 3] = 255;
}

function renderCortical(index) {
  const meta = state.meta;
  const image = state.corticalImage;
  const imageBytes = image.data;
  const sourceOffset = frameOffset(index);
  for (let i = 0; i < state.frameSize; i += 1) {
    writePixel(imageBytes, i, state.bytes[sourceOffset + i]);
  }

  const offscreen = state.corticalCanvas;
  const offscreenCtx = offscreen.getContext("2d", { alpha: false });
  offscreenCtx.putImageData(image, 0, 0);

  ctx.imageSmoothingEnabled = els.smooth.checked;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(offscreen, 0, 0, meta.width, meta.height, 0, 0, canvas.width, canvas.height);
}

function buildRetinalMap() {
  const meta = state.meta;
  const width = canvas.width;
  const height = canvas.height;
  const map = new Int32Array(width * height);
  const bounds = meta.retino_bounds;
  const params = meta.retino_params;
  const xRange = bounds.max_x - bounds.min_x;
  const yRange = bounds.max_y - bounds.min_y;
  const half = (meta.width * meta.cell_mm) / 2;
  const eps = params.eps;
  const w0 = params.w0;
  const alpha = params.alpha;
  const beta = params.beta;

  for (let y = 0; y < height; y += 1) {
    const retinalY = bounds.max_y - ((y + 0.5) / height) * yRange;
    for (let x = 0; x < width; x += 1) {
      const retinalX = bounds.min_x + ((x + 0.5) / width) * xRange;
      const r = Math.hypot(retinalX, retinalY);
      if (r <= 0) {
        map[y * width + x] = -1;
        continue;
      }

      const theta = Math.atan2(retinalY, retinalX);
      const corticalX = (alpha / eps) * Math.log((eps / w0) * r);
      const corticalY = (beta * theta) / eps;
      const col = Math.floor((corticalX + half) / meta.cell_mm);
      const row = Math.floor((corticalY + half) / meta.cell_mm);

      if (row >= 0 && row < meta.height && col >= 0 && col < meta.width) {
        map[y * width + x] = row * meta.width + col;
      } else {
        map[y * width + x] = -1;
      }
    }
  }

  state.retinalMap = map;
  state.retinalMapWidth = width;
  state.retinalMapHeight = height;
}

function renderRetinal(index) {
  if (
    !state.retinalMap ||
    state.retinalMapWidth !== canvas.width ||
    state.retinalMapHeight !== canvas.height
  ) {
    buildRetinalMap();
  }

  const target = state.retinalImage.data;
  const sourceOffset = frameOffset(index);
  const total = canvas.width * canvas.height;
  for (let i = 0; i < total; i += 1) {
    const sourceIndex = state.retinalMap[i];
    if (sourceIndex < 0) {
      const t = i * 4;
      target[t] = 7;
      target[t + 1] = 7;
      target[t + 2] = 7;
      target[t + 3] = 255;
    } else {
      writePixel(target, i, state.bytes[sourceOffset + sourceIndex]);
    }
  }

  ctx.imageSmoothingEnabled = false;
  ctx.putImageData(state.retinalImage, 0, 0);
}

function retinalToCortical(retinalX, retinalY) {
  const meta = state.meta;
  const bounds = meta.retino_bounds;
  const params = meta.retino_params;
  const r = Math.hypot(retinalX, retinalY);
  if (r <= 0) {
    return null;
  }

  const theta = Math.atan2(retinalY, retinalX);
  const corticalX = (params.alpha / params.eps) * Math.log((params.eps / params.w0) * r);
  const corticalY = (params.beta * theta) / params.eps;
  const half = (meta.width * meta.cell_mm) / 2;
  const col = Math.floor((corticalX + half) / meta.cell_mm);
  const row = Math.floor((corticalY + half) / meta.cell_mm);

  if (
    retinalX < bounds.min_x ||
    retinalX > bounds.max_x ||
    retinalY < bounds.min_y ||
    retinalY > bounds.max_y ||
    row < 0 ||
    row >= meta.height ||
    col < 0 ||
    col >= meta.width
  ) {
    return null;
  }

  return { row, col, corticalX, corticalY, theta };
}

function planformInfo(index) {
  const meta = state.meta;
  const params = meta.params || {};
  const duration = Math.max(1e-9, Number(params.t || 1));
  const time = meta.times[index] || 0;
  const progress = time / duration;
  const planform = meta.planform || {};
  const extent = meta.width * meta.cell_mm;
  return {
    waveNumber:
      Number(planform.wave_number) ||
      (2 * Math.PI * Number(params.wave_count || 1)) / Math.max(1e-9, extent),
    phase: Number(planform.phase_base || 2 * Math.PI * Number(params.drift || 0)) * progress,
    modes: planform.modes || [],
    eigen: planform.eigen || null,
    samples: Math.max(16, Number(params.m || meta.orientation_count || 16)),
  };
}

function orientationEigenValue(delta, eigen) {
  if (!eigen) {
    return Math.cos(2 * delta);
  }
  let value = 0;
  for (const [harmonic, coefficient] of eigen.cos_coefficients || []) {
    value += coefficient * Math.cos(2 * harmonic * delta);
  }
  for (const [harmonic, coefficient] of eigen.sin_coefficients || []) {
    value += coefficient * Math.sin(2 * harmonic * delta);
  }
  return value;
}

function orientationPlanformActivity(corticalX, corticalY, phi, info) {
  let value = 0;
  for (const mode of info.modes) {
    const normal = Number(mode.normal_angle || 0);
    const projection = corticalX * Math.cos(normal) + corticalY * Math.sin(normal);
    const spatial =
      Number(mode.amplitude || 1) *
      Math.cos(info.waveNumber * projection + info.phase * Number(mode.phase_scale || 1));
    const tangentCenter = normal + Math.PI / 2;
    value += spatial * orientationEigenValue(phi - tangentCenter, info.eigen);
  }
  return value;
}

function dominantOrientations(corticalX, corticalY, info, threshold) {
  if (!info.modes.length) {
    return [];
  }
  const values = [];
  const normalizer = Math.max(1, info.modes.length);
  for (let i = 0; i < info.samples; i += 1) {
    const phi = (Math.PI * i) / info.samples;
    const value = orientationPlanformActivity(corticalX, corticalY, phi, info);
    values.push({ phi, value, strength: Math.min(1, Math.abs(value) / normalizer) });
  }

  const candidates = [];
  for (let i = 0; i < values.length; i += 1) {
    const prev = values[(i + values.length - 1) % values.length].strength;
    const next = values[(i + 1) % values.length].strength;
    const current = values[i];
    if (current.strength >= threshold && current.strength >= prev && current.strength >= next) {
      candidates.push(current);
    }
  }

  candidates.sort((a, b) => b.strength - a.strength);
  const selected = [];
  for (const candidate of candidates) {
    const separated = selected.every((item) => {
      const direct = Math.abs(candidate.phi - item.phi);
      return Math.min(direct, Math.PI - direct) > Math.PI / 8;
    });
    if (separated) {
      selected.push(candidate);
    }
    if (selected.length >= 2) {
      break;
    }
  }
  return selected;
}

function drawGlyph(x, y, angle, strength, length) {
  const alpha = 0.18 + 0.62 * strength;
  const dx = Math.cos(angle) * length * (0.55 + 0.45 * strength);
  const dy = Math.sin(angle) * length * (0.55 + 0.45 * strength);
  ctx.beginPath();
  ctx.moveTo(x - dx, y - dy);
  ctx.lineTo(x + dx, y + dy);
  ctx.lineWidth = 3;
  ctx.strokeStyle = `rgba(0, 0, 0, ${alpha * 0.75})`;
  ctx.stroke();
  ctx.lineWidth = 1.15;
  ctx.strokeStyle = `rgba(248, 245, 226, ${alpha})`;
  ctx.stroke();
}

function renderContourOverlay(index) {
  const meta = state.meta;
  if (!els.contours.checked || meta?.params?.generator !== "planform") {
    return;
  }

  const stride = Number(els.contourStride.value);
  const length = Number(els.contourLength.value);
  const threshold = Number(els.contourThreshold.value);
  const info = planformInfo(index);
  const bounds = meta.retino_bounds;
  const xRange = bounds.max_x - bounds.min_x;
  const yRange = bounds.max_y - bounds.min_y;

  ctx.save();
  ctx.lineCap = "round";
  ctx.lineJoin = "round";

  if (els.view.value === "retinal") {
    for (let y = stride / 2; y < canvas.height; y += stride) {
      const retinalY = bounds.max_y - ((y + 0.5) / canvas.height) * yRange;
      for (let x = stride / 2; x < canvas.width; x += stride) {
        const retinalX = bounds.min_x + ((x + 0.5) / canvas.width) * xRange;
        const mapped = retinalToCortical(retinalX, retinalY);
        if (!mapped) {
          continue;
        }
        for (const orientation of dominantOrientations(
          mapped.corticalX,
          mapped.corticalY,
          info,
          threshold,
        )) {
          const retinalPhi = orientation.phi + mapped.theta;
          drawGlyph(x, y, -retinalPhi, orientation.strength, length);
        }
      }
    }
  } else {
    for (let y = stride / 2; y < canvas.height; y += stride) {
      const row = Math.floor((y / canvas.height) * meta.height);
      const corticalY = (row + 0.5) * meta.cell_mm - (meta.height * meta.cell_mm) / 2;
      for (let x = stride / 2; x < canvas.width; x += stride) {
        const col = Math.floor((x / canvas.width) * meta.width);
        const corticalX = (col + 0.5) * meta.cell_mm - (meta.width * meta.cell_mm) / 2;
        for (const orientation of dominantOrientations(corticalX, corticalY, info, threshold)) {
          drawGlyph(x, y, orientation.phi, orientation.strength, length);
        }
      }
    }
  }

  ctx.restore();
}

function paint(index) {
  if (!state.meta) {
    return;
  }
  resizeCanvas();
  if (els.view.value === "retinal") {
    renderRetinal(index);
  } else {
    renderCortical(index);
  }
  renderContourOverlay(index);
  const t = state.meta.times[index] || 0;
  els.time.textContent = `t = ${t.toFixed(2)}`;
  els.frame.value = String(index);

  const now = performance.now();
  state.paintCount += 1;
  if (now - state.lastPaintAt > 500) {
    state.measuredFps = (state.paintCount * 1000) / Math.max(1, now - state.lastPaintAt);
    state.lastPaintAt = now;
    state.paintCount = 0;
    updateStats();
  }
}

function shortNumber(value, digits = 2) {
  if (!Number.isFinite(value)) {
    return "-";
  }
  return Number(value).toFixed(digits);
}

function updateStats() {
  const meta = state.meta;
  if (!meta) {
    return;
  }
  const timing = meta.timing;
  const trimText = meta.warmup?.dropped_frames ? `, trim ${meta.warmup.dropped_frames}` : "";
  const backendText = timing?.backend ? `, ${timing.backend}` : "";
  const generatorText = meta.params?.generator ? `, ${meta.params.generator}` : "";
  const patternText =
    meta.params?.generator === "planform" && meta.params?.pattern ? `, ${meta.params.pattern}` : "";
  const effectiveParity = meta.planform?.parity || meta.params?.parity;
  const parityText =
    meta.params?.generator === "planform" && effectiveParity ? `, ${effectiveParity}` : "";
  const solverText =
    meta.params?.generator !== "planform" && meta.params?.solver ? `, ${meta.params.solver}` : "";
  const cacheText = timing?.matrix_cache_hit ? ", cache" : "";
  const timingText = timing
    ? `build ${shortNumber(timing.matrix_build_sec, 1)}s, solve ${shortNumber(timing.solve_sec, 1)}s`
    : "precomputed";
  els.stats.textContent = `${meta.width}x${meta.height}, ${meta.orientation_count} orientations, ${meta.frame_count} frames${trimText}${backendText}${generatorText}${patternText}${parityText}${solverText}${cacheText}, ${state.measuredFps.toFixed(1)} fps render, ${timingText}`;

  if (meta.metrics) {
    els.metrics.textContent = `std ${shortNumber(meta.metrics.final_std, 3)}, cycles ${shortNumber(meta.metrics.dominant_cycles, 1)}, delta ${shortNumber(meta.metrics.temporal_delta, 4)}`;
  }

  if (meta.planform?.stability) {
    const stability = meta.planform.stability;
    const branch = meta.planform.branch_selection;
    const candidate = branch?.candidates?.[0];
    const amplitude = candidate ? `, A ${shortNumber(candidate.amplitude, 2)}` : "";
    els.stability.textContent = `q* ${shortNumber(stability.critical_q, 2)}, ${stability.critical_branch}, G ${shortNumber(stability.critical_growth, 3)}, ${branch?.selected_family || "-"}${amplitude}`;
  } else {
    els.stability.textContent = "No stability scan";
  }
}

function tick(now) {
  if (state.meta && state.playing && now - state.lastFrameAt >= 1000 / state.fps) {
    state.currentFrame = (state.currentFrame + 1) % state.meta.frame_count;
    state.lastFrameAt = now;
    paint(state.currentFrame);
  }
  requestAnimationFrame(tick);
}

function setPlaying(value) {
  state.playing = value;
  els.playToggle.textContent = value ? "Pause" : "Play";
}

function updateOutput(id) {
  const input = document.getElementById(id);
  const output = document.getElementById(`${id}Value`);
  if (!input || !output) {
    return;
  }
  const format = FORMATTERS[id] || ((value) => value);
  output.textContent = format(input.value);
}

function bindOutputs() {
  Object.keys(FORMATTERS).forEach((id) => {
    const input = document.getElementById(id);
    if (!input) {
      return;
    }
    input.addEventListener("input", () => updateOutput(id));
    updateOutput(id);
  });
}

function updateGeneratorVisibility() {
  const isPlanform = document.getElementById("generator")?.value === "planform";
  document.querySelectorAll(".planform-control").forEach((element) => {
    element.classList.toggle("is-muted", !isPlanform);
  });
}

function currentParams() {
  const data = new FormData(els.paramForm);
  const params = Object.fromEntries(data.entries());
  const trimWarmup = document.getElementById("trim_warmup");
  if (trimWarmup) {
    params.trim_warmup = trimWarmup.checked ? "true" : "false";
  }
  return params;
}

function paramsToQuery(params) {
  const query = new URLSearchParams();
  PARAM_IDS.forEach((id) => {
    if (params[id] !== undefined && params[id] !== "") {
      query.set(id, params[id]);
    }
  });
  return query.toString();
}

function applyParams(params) {
  PARAM_IDS.forEach((id) => {
    const input = document.getElementById(id);
    if (!input || params[id] === undefined || params[id] === null) {
      return;
    }
    if (input.type === "checkbox") {
      input.checked = params[id] === true || params[id] === "true" || params[id] === "on" || params[id] === "1";
      return;
    }
    input.value = String(params[id]);
    updateOutput(id);
  });
  updateGeneratorVisibility();
}

function installPayload(payload) {
  state.meta = payload;
  state.bytes = decodeBase64(payload.data_base64);
  state.palette = packPalette(payload.palette);
  state.frameSize = payload.width * payload.height;
  state.currentFrame = 0;
  state.measuredFps = 0;
  state.paintCount = 0;
  state.lastPaintAt = performance.now();
  state.retinalMap = null;
  state.corticalCanvas.width = payload.width;
  state.corticalCanvas.height = payload.height;
  state.corticalImage = ctx.createImageData(payload.width, payload.height);
  state.retinalImage = ctx.createImageData(canvas.width, canvas.height);
  els.frame.max = String(payload.frame_count - 1);
  els.frame.value = String(state.currentFrame);
  if (payload.params) {
    applyParams(payload.params);
  }
  updateStats();
  paint(state.currentFrame);
}

async function fetchPayload(url) {
  const response = await fetch(url, { cache: "no-store" });
  if (!response.ok) {
    let message = `Request failed (${response.status})`;
    try {
      const errorPayload = await response.json();
      message = errorPayload.error || message;
    } catch {
      // The server may return a plain error page for static file failures.
    }
    throw new Error(message);
  }
  return response.json();
}

async function loadDefaultPayload() {
  try {
    return await fetchPayload(`/api/generate?${paramsToQuery(currentParams())}`);
  } catch (serverError) {
    return fetchPayload("frames.json");
  }
}

async function runModel(event) {
  event?.preventDefault();
  const requestId = state.requestId + 1;
  state.requestId = requestId;
  const wasPlaying = state.playing;
  setPlaying(false);
  els.runButton.disabled = true;
  els.status.textContent = "Running model...";

  try {
    const payload = await fetchPayload(`/api/generate?${paramsToQuery(currentParams())}`);
    if (state.requestId !== requestId) {
      return;
    }
    installPayload(payload);
    els.status.textContent = "Ready";
    setPlaying(wasPlaying);
  } catch (error) {
    els.status.textContent = error.message;
  } finally {
    if (state.requestId === requestId) {
      els.runButton.disabled = false;
    }
  }
}

function applyDefaultControls() {
  applyParams(DEFAULT_PARAMS);
  if (els.paperPreset) {
    els.paperPreset.value = "manual";
  }
}

function applyPaperPreset(name) {
  const preset = PAPER_PRESETS[name];
  if (!preset) {
    return false;
  }
  applyParams({ ...DEFAULT_PARAMS, ...preset });
  return true;
}

async function resetToDefault() {
  setPlaying(false);
  els.runButton.disabled = true;
  els.status.textContent = "Loading default...";
  applyDefaultControls();
  try {
    const payload = await loadDefaultPayload();
    installPayload(payload);
    els.status.textContent = "Ready";
    setPlaying(true);
  } catch (error) {
    els.status.textContent = error.message;
  } finally {
    els.runButton.disabled = false;
  }
}

async function init() {
  bindOutputs();
  applyDefaultControls();

  els.frame.addEventListener("input", () => {
    state.currentFrame = Number(els.frame.value);
    paint(state.currentFrame);
  });
  els.fps.addEventListener("input", () => {
    state.fps = Number(els.fps.value);
    els.fpsValue.textContent = String(state.fps);
  });
  els.playToggle.addEventListener("click", () => setPlaying(!state.playing));
  els.view.addEventListener("change", () => paint(state.currentFrame));
  els.smooth.addEventListener("change", () => paint(state.currentFrame));
  els.contours.addEventListener("change", () => paint(state.currentFrame));
  [els.contourStride, els.contourLength, els.contourThreshold].forEach((input) => {
    input.addEventListener("input", () => paint(state.currentFrame));
  });
  document.getElementById("generator")?.addEventListener("change", updateGeneratorVisibility);
  els.paperPreset?.addEventListener("change", () => {
    if (applyPaperPreset(els.paperPreset.value)) {
      runModel();
    }
  });
  els.paramForm.addEventListener("submit", runModel);
  els.resetButton.addEventListener("click", resetToDefault);
  window.addEventListener("resize", () => paint(state.currentFrame));

  try {
    const payload = await loadDefaultPayload();
    installPayload(payload);
    els.status.textContent = "Ready";
  } catch (error) {
    els.status.textContent = error.message;
    els.stats.textContent = error.message;
  }

  setPlaying(true);
  requestAnimationFrame(tick);
}

init();
