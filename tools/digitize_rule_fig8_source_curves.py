#!/usr/bin/env python3
"""Digitize Rule et al. 2011 Figure 8C into numeric source curves.

The input page image is private/local-only. The output is a public-safe JSON
file containing derived axis coordinates for the Figure 8C boundary curves.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Iterable

import numpy as np
from PIL import Image, ImageDraw


PAGE_CROP = (335, 835, 930, 1295)
PLOT_LEFT = 44
PLOT_RIGHT = 587
PLOT_TOP = 27
PLOT_BOTTOM = 434
PERIOD_MIN_MS = 20.0
PERIOD_MAX_MS = 150.0
WAVE_MIN_BETA = 0.0
WAVE_MAX_BETA = 1.0
PLUS_LABEL_MASK = {
    "label": "+1 annotation",
    "crop_pixel_box": [465, 245, 515, 290],
    "reason": "Suppress printed panel annotation pixels near the lower +1 branch.",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Digitize private Rule Figure 8C page render into source-curve JSON."
    )
    parser.add_argument(
        "--page-png",
        type=Path,
        default=Path("private/figure-targets/rule-figure8/rule-page-10.png"),
        help="Private rendered PDF page image containing Rule Figure 8.",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("reports/source-curves/rule-2011-fig8-source-curves.json"),
        help="Public-safe derived numeric source-curve JSON.",
    )
    parser.add_argument(
        "--overlay",
        type=Path,
        default=Path("private/figure-targets/rule-figure8/rule-fig8c-digitized-overlay.png"),
        help="Private visual QA overlay for the extracted points.",
    )
    return parser.parse_args()


def cluster_runs(values: Iterable[int]) -> list[tuple[float, int]]:
    ordered = sorted(values)
    if not ordered:
        return []
    runs: list[list[int]] = [[ordered[0]]]
    for value in ordered[1:]:
        if value <= runs[-1][-1] + 2:
            runs[-1].append(value)
        else:
            runs.append([value])
    return [(sum(run) / len(run), len(run)) for run in runs]


def to_source_axes(px: float, py: float) -> tuple[float, float]:
    period = PERIOD_MIN_MS + (px - PLOT_LEFT) / (PLOT_RIGHT - PLOT_LEFT) * (
        PERIOD_MAX_MS - PERIOD_MIN_MS
    )
    wave = WAVE_MAX_BETA - (py - PLOT_TOP) / (PLOT_BOTTOM - PLOT_TOP) * (
        WAVE_MAX_BETA - WAVE_MIN_BETA
    )
    return period, wave


def extract_branch_pair(
    crop: Image.Image,
    x_start: int,
    x_end: int,
    y_start: int,
    y_end: int,
    sample_step: int,
    mask_boxes: list[tuple[int, int, int, int]] | None = None,
    singleton_policy: str = "duplicate",
) -> tuple[list[dict[str, float]], list[dict[str, float]]]:
    arr = np.array(crop.convert("L"))
    mask = arr < 170
    if mask_boxes:
        for x1, y1, x2, y2 in mask_boxes:
            mask[y1:y2, x1:x2] = False

    upper: list[tuple[int, float]] = []
    lower: list[tuple[int, float]] = []
    for px in range(x_start, x_end + 1):
        ys = np.where(mask[y_start : y_end + 1, px])[0] + y_start
        clusters = cluster_runs(int(y) for y in ys)
        if not clusters:
            continue
        clusters.sort(key=lambda item: item[0])
        if len(clusters) == 1:
            upper.append((px, clusters[0][0]))
            if singleton_policy == "duplicate":
                lower.append((px, clusters[0][0]))
            elif singleton_policy == "skip":
                upper.pop()
            elif singleton_policy != "upper_only":
                raise ValueError(f"unknown singleton_policy: {singleton_policy}")
        else:
            upper.append((px, clusters[0][0]))
            lower.append((px, clusters[-1][0]))

    return bin_points(upper, sample_step), bin_points(lower, sample_step)


def bin_points(samples: list[tuple[int, float]], sample_step: int) -> list[dict[str, float]]:
    if not samples:
        return []
    points: list[dict[str, float]] = []
    first = samples[0][0]
    last = samples[-1][0]
    for bin_x in range(first, last + 1, sample_step):
        vals = [(x, y) for x, y in samples if bin_x <= x < bin_x + sample_step]
        if not vals:
            continue
        px = sum(x for x, _ in vals) / len(vals)
        py = sum(y for _, y in vals) / len(vals)
        period, wave = to_source_axes(px, py)
        points.append(
            {
                "period_ms": round(period, 3),
                "wave_number_beta": round(wave, 4),
                "pixel_x": round(px, 1),
                "pixel_y": round(py, 1),
            }
        )
    return points


def without_pixel_fields(points: list[dict[str, float]]) -> list[dict[str, float]]:
    return [
        {
            "period_ms": point["period_ms"],
            "wave_number_beta": point["wave_number_beta"],
        }
        for point in points
    ]


def monotonic_unique(points: list[dict[str, float]]) -> list[dict[str, float]]:
    out: list[dict[str, float]] = []
    for point in points:
        if out and abs(out[-1]["period_ms"] - point["period_ms"]) < 0.001:
            out[-1] = point
        else:
            out.append(point)
    return out


def prepend_endpoint(
    points: list[dict[str, float]], period_ms: float, wave_number_beta: float
) -> list[dict[str, float]]:
    return [{"period_ms": period_ms, "wave_number_beta": wave_number_beta}, *points]


def draw_overlay(
    crop: Image.Image,
    overlay_path: Path,
    branches: list[tuple[list[dict[str, float]], str]],
) -> None:
    rgb = crop.convert("RGB")
    draw = ImageDraw.Draw(rgb)
    for points, color in branches:
        screen_points: list[tuple[float, float]] = []
        for point in points:
            if "pixel_x" in point and "pixel_y" in point:
                x = point["pixel_x"]
                y = point["pixel_y"]
            else:
                x = PLOT_LEFT + (point["period_ms"] - PERIOD_MIN_MS) / (
                    PERIOD_MAX_MS - PERIOD_MIN_MS
                ) * (PLOT_RIGHT - PLOT_LEFT)
                y = PLOT_TOP + (WAVE_MAX_BETA - point["wave_number_beta"]) / (
                    WAVE_MAX_BETA - WAVE_MIN_BETA
                ) * (PLOT_BOTTOM - PLOT_TOP)
            screen_points.append((x, y))
        if len(screen_points) >= 2:
            draw.line(screen_points, fill=color, width=2)
        for x, y in screen_points:
            draw.ellipse((x - 1.5, y - 1.5, x + 1.5, y + 1.5), fill=color)
    overlay_path.parent.mkdir(parents=True, exist_ok=True)
    rgb.save(overlay_path)


def main() -> None:
    args = parse_args()
    page = Image.open(args.page_png).convert("RGB")
    crop = page.crop(PAGE_CROP)

    minus_upper, minus_lower = extract_branch_pair(
        crop,
        90,
        236,
        130,
        405,
        7,
        singleton_policy="upper_only",
    )
    plus_upper, plus_lower = extract_branch_pair(
        crop,
        388,
        550,
        145,
        260,
        7,
        mask_boxes=[tuple(PLUS_LABEL_MASK["crop_pixel_box"])],
        singleton_policy="duplicate",
    )

    minus_upper = prepend_endpoint(monotonic_unique(minus_upper), 31.0, 0.0)
    minus_lower = prepend_endpoint(
        [point for point in monotonic_unique(minus_lower) if point["period_ms"] >= 39.0],
        39.5,
        0.0,
    )
    plus_upper = monotonic_unique(plus_upper)
    plus_lower = monotonic_unique(plus_lower)

    curves = [
        (
            "rule_fig8c_minus_upper",
            "minus_period_doubling",
            "-1 upper boundary",
            minus_upper,
        ),
        (
            "rule_fig8c_minus_lower",
            "minus_period_doubling",
            "-1 lower boundary",
            minus_lower,
        ),
        (
            "rule_fig8c_plus_upper",
            "plus_one_to_one",
            "+1 upper boundary",
            plus_upper,
        ),
        (
            "rule_fig8c_plus_lower",
            "plus_one_to_one",
            "+1 lower boundary",
            plus_lower,
        ),
    ]

    body = {
        "format": "rule-2011-figure8-source-curves-v1",
        "source_key": "rule-2011",
        "figure": "Figure 8C",
        "source_axes": {
            "x_axis": "forcing_period",
            "x_units": "ms",
            "x_min": PERIOD_MIN_MS,
            "x_max": PERIOD_MAX_MS,
            "y_axis": "wave_number_beta",
            "y_units": "dimensionless",
            "y_min": WAVE_MIN_BETA,
            "y_max": WAVE_MAX_BETA,
        },
        "digitization": {
            "method": "thresholded_private_pdf_page_render_with_manual_axis_calibration",
            "pdf_page": 10,
            "panel": "C",
            "page_render_dpi": 300,
            "crop_pixels": list(PAGE_CROP),
            "plot_pixels": {
                "left": PLOT_LEFT,
                "right": PLOT_RIGHT,
                "top": PLOT_TOP,
                "bottom": PLOT_BOTTOM,
            },
            "annotation_masks": [PLUS_LABEL_MASK],
            "overlay_legend": {
                "red": "-1 upper boundary",
                "blue": "-1 lower boundary",
                "green": "+1 upper boundary",
                "purple": "+1 lower boundary",
            },
            "qa_note": (
                "Overlay colors identify upper/lower branch assignments, not left/right sides. "
                "Connected line segments are drawn over sampled points so isolated-looking dots "
                "can be checked against the branch trace."
            ),
            "note": "Derived numeric coordinates only; source PDF render and QA overlay stay under private/.",
        },
        "curves": [
            {
                "curve_id": curve_id,
                "kind": kind,
                "branch_label": label,
                "point_count": len(points),
                "points": without_pixel_fields(points),
            }
            for curve_id, kind, label, points in curves
        ],
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(body, indent=2) + "\n", encoding="utf-8")
    draw_overlay(
        crop,
        args.overlay,
        [
            (minus_upper, "red"),
            (minus_lower, "blue"),
            (plus_upper, "green"),
            (plus_lower, "purple"),
        ],
    )
    print(f"wrote {args.out}")
    print(f"wrote private overlay {args.overlay}")


if __name__ == "__main__":
    main()
