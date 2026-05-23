from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from v1_frames import FrameParams, generate_payload


def parse_args():
    parser = argparse.ArgumentParser(description="Export Bressloff V1 frames for the browser viewer.")
    parser.add_argument("--out", type=Path, default=ROOT / "viewer" / "frames.json")
    parser.add_argument("--n", type=int, default=64, help="Spatial grid size.")
    parser.add_argument("--m", type=int, default=12, help="Orientation bins.")
    parser.add_argument("--t", type=float, default=60, help="Simulation end time.")
    parser.add_argument("--frames", type=int, default=120, help="Number of exported playback frames.")
    parser.add_argument("--seed", type=int, default=20260522)
    parser.add_argument("--alpha", type=float, default=1)
    parser.add_argument("--beta", type=float, default=3)
    parser.add_argument("--mu", type=float, default=17)
    parser.add_argument("--r0", type=float, default=3.2 / 50, help="Lateral kernel radius scale.")
    parser.add_argument("--rtol", type=float, default=1e-6)
    parser.add_argument("--atol", type=float, default=1e-10)
    parser.add_argument("--low-percentile", type=float, default=1)
    parser.add_argument("--high-percentile", type=float, default=99)
    parser.add_argument("--cmap", default="twilight")
    parser.add_argument("--no-trim-warmup", action="store_true", help="Keep low-contrast onset frames in the payload.")
    parser.add_argument("--trim-threshold", type=float, default=0.08, help="Fraction of max frame contrast used to trim warmup.")
    parser.add_argument("--solver", choices=["preview", "accurate"], default="preview")
    parser.add_argument("--preview-step", type=float, default=0.25)
    return parser.parse_args()


def main():
    args = parse_args()
    args.out.parent.mkdir(parents=True, exist_ok=True)

    params = FrameParams(
        n=args.n,
        m=args.m,
        t=args.t,
        frames=args.frames,
        seed=args.seed,
        alpha=args.alpha,
        beta=args.beta,
        mu=args.mu,
        r0=args.r0,
        rtol=args.rtol,
        atol=args.atol,
        low_percentile=args.low_percentile,
        high_percentile=args.high_percentile,
        cmap=args.cmap,
        trim_warmup=not args.no_trim_warmup,
        trim_threshold=args.trim_threshold,
        solver=args.solver,
        preview_step=args.preview_step,
    )
    payload = generate_payload(params)
    args.out.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")

    print(f"wrote {args.out}")
    print(
        f"grid={payload['width']}x{payload['height']} "
        f"orientations={payload['orientation_count']} frames={payload['frame_count']}"
    )
    print(
        f"matrix_build_sec={payload['timing']['matrix_build_sec']:.2f} "
        f"solve_sec={payload['timing']['solve_sec']:.2f} "
        f"solver={payload['params']['solver']}"
    )
    print(
        f"trimmed_frames={payload['warmup']['dropped_frames']} "
        f"start_time={payload['warmup']['start_time']:.2f}"
    )
    print(
        f"scale=[{payload['scale_min']:.6g}, {payload['scale_max']:.6g}] "
        f"raw=[{payload['raw_min']:.6g}, {payload['raw_max']:.6g}]"
    )


if __name__ == "__main__":
    main()
