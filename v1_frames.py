from __future__ import annotations

import base64
import time
from dataclasses import dataclass
from typing import Any

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

from v1_model import V1System, inverse_retino_cortical_map


@dataclass(frozen=True)
class FrameParams:
    n: int = 64
    m: int = 12
    t: float = 60.0
    frames: int = 120
    seed: int = 20260522
    alpha: float = 1.0
    beta: float = 3.0
    mu: float = 17.0
    r0: float = 3.2 / 50
    rtol: float = 1e-6
    atol: float = 1e-10
    low_percentile: float = 1.0
    high_percentile: float = 99.0
    cmap: str = "twilight"
    trim_warmup: bool = True
    trim_threshold: float = 0.08
    solver: str = "preview"
    preview_step: float = 0.25


PARAM_LIMITS: dict[str, tuple[float, float]] = {
    "n": (32, 96),
    "m": (4, 24),
    "t": (5.0, 140.0),
    "frames": (8, 240),
    "seed": (0, 2**32 - 1),
    "alpha": (0.1, 4.0),
    "beta": (0.1, 10.0),
    "mu": (1.0, 40.0),
    "r0": (0.02, 0.14),
    "low_percentile": (0.0, 20.0),
    "high_percentile": (80.0, 100.0),
    "trim_threshold": (0.0, 0.5),
    "preview_step": (0.05, 1.0),
}


def _clamp_float(value: Any, key: str, default: float) -> float:
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        parsed = default
    lo, hi = PARAM_LIMITS[key]
    return min(hi, max(lo, parsed))


def _clamp_int(value: Any, key: str, default: int) -> int:
    try:
        parsed = int(value)
    except (TypeError, ValueError):
        parsed = default
    lo, hi = PARAM_LIMITS[key]
    return int(min(hi, max(lo, parsed)))


def _coerce_bool(value: Any, default: bool) -> bool:
    if value is None:
        return default
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return bool(value)
    return str(value).lower() in {"1", "true", "yes", "on"}


def coerce_params(raw: dict[str, Any] | None = None) -> FrameParams:
    raw = raw or {}
    defaults = FrameParams()
    low = _clamp_float(raw.get("low_percentile"), "low_percentile", defaults.low_percentile)
    high = _clamp_float(raw.get("high_percentile"), "high_percentile", defaults.high_percentile)
    if high <= low:
        low, high = defaults.low_percentile, defaults.high_percentile
    solver = str(raw.get("solver") or defaults.solver).lower()
    if solver not in {"preview", "accurate"}:
        solver = defaults.solver

    return FrameParams(
        n=_clamp_int(raw.get("n"), "n", defaults.n),
        m=_clamp_int(raw.get("m"), "m", defaults.m),
        t=_clamp_float(raw.get("t"), "t", defaults.t),
        frames=_clamp_int(raw.get("frames"), "frames", defaults.frames),
        seed=_clamp_int(raw.get("seed"), "seed", defaults.seed),
        alpha=_clamp_float(raw.get("alpha"), "alpha", defaults.alpha),
        beta=_clamp_float(raw.get("beta"), "beta", defaults.beta),
        mu=_clamp_float(raw.get("mu"), "mu", defaults.mu),
        r0=_clamp_float(raw.get("r0"), "r0", defaults.r0),
        rtol=defaults.rtol,
        atol=defaults.atol,
        low_percentile=low,
        high_percentile=high,
        cmap=str(raw.get("cmap") or defaults.cmap),
        trim_warmup=_coerce_bool(raw.get("trim_warmup"), defaults.trim_warmup),
        trim_threshold=_clamp_float(raw.get("trim_threshold"), "trim_threshold", defaults.trim_threshold),
        solver=solver,
        preview_step=_clamp_float(raw.get("preview_step"), "preview_step", defaults.preview_step),
    )


def retino_bounds(n: int, cell_mm: float) -> dict[str, float]:
    x_padded = cell_mm * np.arange(n + 1) - n * cell_mm / 2
    xx_padded, yy_padded = np.meshgrid(x_padded, x_padded)
    xx_retina, yy_retina = inverse_retino_cortical_map(xx_padded, yy_padded)
    return {
        "min_x": float(xx_retina.min()),
        "max_x": float(xx_retina.max()),
        "min_y": float(yy_retina.min()),
        "max_y": float(yy_retina.max()),
    }


def palette(cmap_name: str) -> list[list[int]]:
    try:
        cmap = plt.get_cmap(cmap_name)
    except ValueError:
        cmap = plt.get_cmap(FrameParams().cmap)

    colors = []
    for value in np.linspace(0, 1, 256):
        r, g, b, _ = cmap(value)
        colors.append([round(255 * r), round(255 * g), round(255 * b)])
    return colors


def frame_metrics(frames: np.ndarray) -> dict[str, float]:
    final = frames[-1]
    centered = final - final.mean()
    spectrum = np.abs(np.fft.fftshift(np.fft.fft2(centered))) ** 2
    center = np.array(spectrum.shape) // 2
    spectrum[center[0], center[1]] = 0
    peak = np.unravel_index(int(np.argmax(spectrum)), spectrum.shape)
    dy = peak[0] - center[0]
    dx = peak[1] - center[1]
    dominant_cycles = float(np.hypot(dx, dy))
    temporal_delta = float(np.mean(np.abs(np.diff(frames, axis=0)))) if len(frames) > 1 else 0.0

    return {
        "final_mean": float(final.mean()),
        "final_std": float(final.std()),
        "final_range": float(final.max() - final.min()),
        "dominant_cycles": dominant_cycles,
        "temporal_delta": temporal_delta,
    }


def trim_warmup_frames(
    frames: np.ndarray,
    times: np.ndarray,
    *,
    enabled: bool,
    threshold_fraction: float,
    preroll_frames: int = 2,
) -> tuple[np.ndarray, np.ndarray, dict[str, float | int | bool]]:
    if not enabled or len(frames) <= 3:
        return frames, times, {
            "enabled": enabled,
            "dropped_frames": 0,
            "start_time": float(times[0]) if len(times) else 0.0,
            "threshold_fraction": float(threshold_fraction),
            "max_std": 0.0,
        }

    contrast = frames.reshape(len(frames), -1).std(axis=1)
    max_std = float(contrast.max())
    threshold = max_std * threshold_fraction
    active = np.flatnonzero(contrast >= threshold)
    if max_std <= 0 or len(active) == 0:
        start = 0
    else:
        start = max(0, int(active[0]) - preroll_frames)
        min_remaining = min(len(frames), max(16, len(frames) // 3))
        start = min(start, len(frames) - min_remaining)

    return frames[start:], times[start:], {
        "enabled": enabled,
        "dropped_frames": int(start),
        "start_time": float(times[start]) if len(times) else 0.0,
        "threshold_fraction": float(threshold_fraction),
        "threshold_std": float(threshold),
        "max_std": max_std,
    }


def generate_payload(params: FrameParams) -> dict[str, Any]:
    started = time.perf_counter()
    system = V1System(
        alpha=params.alpha,
        beta=params.beta,
        mu=params.mu,
        r0=params.r0,
        N=params.n,
        M=params.m,
    )
    built = time.perf_counter()

    rng = np.random.default_rng(params.seed)
    total_dim = system._N**2 * system._M
    a0 = rng.uniform(-1e-12, 1e-12, total_dim)
    t_eval = np.linspace(0, params.t, params.frames)
    if params.solver == "accurate":
        sol = system.integrate((0, params.t), a0, t_eval=t_eval, rtol=params.rtol, atol=params.atol)
    else:
        sol = system.integrate_preview((0, params.t), a0, t_eval=t_eval, step=params.preview_step)
    solved = time.perf_counter()

    if not sol.success:
        raise RuntimeError(sol.message)

    frames = sol.y.T.reshape(len(sol.t), system._N, system._N, system._M).mean(axis=3).astype(np.float32)
    frames, times, warmup = trim_warmup_frames(
        frames,
        sol.t,
        enabled=params.trim_warmup,
        threshold_fraction=params.trim_threshold,
    )
    lo, hi = np.percentile(frames, [params.low_percentile, params.high_percentile])
    if hi <= lo:
        hi = lo + 1e-9
    normalized = np.clip((frames - lo) / (hi - lo) * 255, 0, 255).astype(np.uint8)

    cell_mm = 0.7
    return {
        "format": "bressloff-v1-u8-frames",
        "width": system._N,
        "height": system._N,
        "frame_count": len(times),
        "orientation_count": system._M,
        "times": [float(t) for t in times],
        "scale_min": float(lo),
        "scale_max": float(hi),
        "raw_min": float(frames.min()),
        "raw_max": float(frames.max()),
        "cell_mm": cell_mm,
        "retino_bounds": retino_bounds(system._N, cell_mm),
        "retino_params": {"eps": 0.051, "w0": 0.087, "alpha": 3 / np.pi, "beta": 1.589 / 2},
        "palette": palette(params.cmap),
        "params": {
            "n": params.n,
            "m": params.m,
            "t": params.t,
            "frames": params.frames,
            "seed": params.seed,
            "alpha": params.alpha,
            "beta": params.beta,
            "mu": params.mu,
            "r0": params.r0,
            "low_percentile": params.low_percentile,
            "high_percentile": params.high_percentile,
            "cmap": params.cmap,
            "trim_warmup": params.trim_warmup,
            "trim_threshold": params.trim_threshold,
            "solver": params.solver,
            "preview_step": params.preview_step,
        },
        "metrics": frame_metrics(frames),
        "warmup": warmup,
        "timing": {
            "matrix_build_sec": built - started,
            "solve_sec": solved - built,
            "total_sec": solved - started,
            "matrix_cache_hit": system._cache_hit,
        },
        "data_base64": base64.b64encode(normalized.tobytes(order="C")).decode("ascii"),
    }
