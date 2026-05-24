#!/usr/bin/env python3
"""Extract public-safe numeric profiles from private Bressloff source masks.

The script reads local image crops or binary masks listed in a private JSON
config and writes only derived numeric profiles to the ignored private output
directory. It never copies source images into reports.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any

try:
    from PIL import Image, ImageFilter
except ImportError as exc:  # pragma: no cover - runtime dependency guard
    raise SystemExit(
        "Pillow is required: python -m pip install pillow"
    ) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract derived Bressloff figure mask/profile JSON files."
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=Path("private/figure-targets/source-mask-config.json"),
        help="Private JSON config listing preset_id and source_image_path entries.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("private/figure-targets/derived"),
        help="Ignored output directory for derived numeric profile JSON files.",
    )
    parser.add_argument("--radial-bins", type=int, default=16)
    parser.add_argument("--angular-bins", type=int, default=24)
    return parser.parse_args()


def load_luma(path: Path, invert: bool) -> tuple[list[float], int, int]:
    image = Image.open(path).convert("L")
    width, height = image.size
    values = [px / 255.0 for px in image.getdata()]
    if invert:
        values = [1.0 - value for value in values]
    return values, width, height


def threshold_mask(values: list[float], threshold: float) -> list[float]:
    return [1.0 if value >= threshold else 0.0 for value in values]


def edge_density(mask: list[float], width: int, height: int) -> float:
    edge_sum = 0.0
    edge_count = 0
    for y in range(height):
        row = y * width
        for x in range(width):
            here = mask[row + x]
            if x + 1 < width:
                edge_sum += abs(here - mask[row + x + 1])
                edge_count += 1
            if y + 1 < height:
                edge_sum += abs(here - mask[row + width + x])
                edge_count += 1
    return edge_sum / edge_count if edge_count else 0.0


def radial_profile(values: list[float], width: int, height: int, bins: int) -> list[float]:
    sums = [0.0] * bins
    counts = [0] * bins
    cx = (width - 1) * 0.5
    cy = (height - 1) * 0.5
    max_radius = max(math.hypot(cx, cy), 1.0e-9)
    for y in range(height):
        for x in range(width):
            radius = math.hypot(x - cx, y - cy) / max_radius
            index = min(bins - 1, int(math.floor(radius * bins)))
            sums[index] += values[y * width + x]
            counts[index] += 1
    return [sums[i] / counts[i] if counts[i] else 0.0 for i in range(bins)]


def angular_profile(values: list[float], width: int, height: int, bins: int) -> list[float]:
    sums = [0.0] * bins
    counts = [0] * bins
    cx = (width - 1) * 0.5
    cy = (height - 1) * 0.5
    for y in range(height):
        for x in range(width):
            angle = math.atan2(y - cy, x - cx) % (2.0 * math.pi)
            index = min(bins - 1, int(math.floor((angle / (2.0 * math.pi)) * bins)))
            sums[index] += values[y * width + x]
            counts[index] += 1
    return [sums[i] / counts[i] if counts[i] else 0.0 for i in range(bins)]


def normalize_entry(
    config_path: Path, entry: dict[str, Any], radial_bins: int, angular_bins: int
) -> dict[str, Any]:
    preset_id = entry["preset_id"]
    source_path = Path(entry["source_image_path"])
    if not source_path.is_absolute():
        source_path = (config_path.parent / source_path).resolve()
    threshold = float(entry.get("threshold", 0.5))
    invert = bool(entry.get("invert", False))
    values, width, height = load_luma(source_path, invert)
    mask = threshold_mask(values, threshold)
    return {
        "format": "bressloff-source-profile-v1",
        "preset_id": preset_id,
        "profile_id": entry.get("profile_id", f"{preset_id}-private-derived-profile"),
        "mask_id": entry.get("mask_id", f"{preset_id}-private-derived-mask"),
        "source_note": entry.get(
            "source_note",
            "Derived locally from a private source image; original scan/crop is not included.",
        ),
        "width": width,
        "height": height,
        "threshold": threshold,
        "active_fraction": sum(mask) / len(mask) if mask else 0.0,
        "edge_density": edge_density(mask, width, height),
        "lattice_angle_degrees": entry.get("lattice_angle_degrees"),
        "radial_profile": radial_profile(
            mask, width, height, int(entry.get("radial_bins", radial_bins))
        ),
        "angular_profile": angular_profile(
            mask, width, height, int(entry.get("angular_bins", angular_bins))
        ),
    }


def main() -> None:
    args = parse_args()
    config = json.loads(args.config.read_text(encoding="utf-8"))
    entries = config.get("targets", config if isinstance(config, list) else [])
    if not isinstance(entries, list) or not entries:
        raise SystemExit("config must contain a non-empty targets list")

    args.out_dir.mkdir(parents=True, exist_ok=True)
    for entry in entries:
        derived = normalize_entry(
            args.config, entry, args.radial_bins, args.angular_bins
        )
        out_path = args.out_dir / f"{derived['preset_id']}.json"
        out_path.write_text(json.dumps(derived, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
