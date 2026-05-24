# Rule 2011 Implementation Status

Updated: 2026-05-24

Rule, Stoffregen, and Ermentrout 2011 is implemented as a separate model
family:

```text
model_family = rule_flicker_ei
generator = rule_flicker
```

It is not represented as a Bressloff orientation-hypercolumn preset. The current
code keeps Bressloff's registry under
`model_family = bressloff_orientation_hypercolumn` and exposes Rule examples
through a separate Rule preset registry.

## Implemented First Pass

- Two-population scalar Wilson-Cowan E/I field from Rule equations 1-2.
- Periodic 2D domain with separable normalized Gaussian E and I kernels.
- Flicker stimulus `S(t) = A H(sin(2 pi t / T) - threshold)` with optional
  smooth-step approximation.
- Rule parameter names and millisecond time units.
- Rule presets:
  - `rule_fig4_high_freq_stripes`
  - `rule_fig4_low_freq_hexagons`
  - `rule_fig5_period_doubled_stripes`
  - `rule_fig5_one_to_one_hexagons`
- Qualitative regime report fields:
  - spatial family: `stripe`, `hexagonal`, or `homogeneous`
  - temporal response: `period_doubled`, `one_to_one`, or `mixed`
  - pattern strength
  - `T` and `2T` temporal correlations
- Simulator-backed `rule-sweep` report with:
  - period/amplitude grid points
  - row-major base64 thumbnail frames for website sweep strips
  - peak activity, dominant cycles, spatial family, and temporal correlations
  - stripe, square, and hexagonal Fourier-family scores
  - top spatial modes, mode entropy, and spatial confidence
  - `T`, `2T`, and `3T` temporal correlations with response-period confidence
  - homogeneous periodic-orbit summaries
  - first-pass 2x2 monodromy multipliers for representative spatial modes
- Dedicated `rule-floquet` report with:
  - dense homogeneous-orbit monodromy grid points
  - per-mode 2x2 monodromy trace and determinant
  - source-style +1, -1, and determinant threshold conditions
  - first-pass `sign_change` boundary candidates along beta, period, and
    amplitude grid edges
  - refined beta-axis `boundary_curves` grouped as wave-number-versus-period
    curves for each amplitude and inhibitory-drive setting
  - source-curve comparison fields with configured domain-normalized, raw,
    scale-only, and affine beta-axis normalization diagnostics
  - a report-level Figure 8 fit objective that combines beta residuals, branch
    coverage, continuity, underresolved branches, and +1/-1 period ordering
  - `nearest_margin` candidates that mark closest-to-threshold points for the
    current coarse calibration
- Compact `rule-fit` report with one-parameter-at-a-time calibration trials
  around `rule_fig8_source_like`, kept separate from named preset promotion.

The low-frequency preset currently uses a 120 ms qualitative representative
rather than claiming exact reproduction of Rule Figure 5B's 110 ms panel. This
keeps the first pass honest: it demonstrates the intended one-to-one hexagonal
regime while leaving figure-level period/amplitude calibration to later exact
parameter tuning.

## Report Command

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-report --out reports\rule-2011-regimes.json
```

The report format is:

```text
rule-2011-regime-report-v1
```

Generate the first sweep and monodromy report with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-sweep --out reports\rule-2011-sweep.json
```

Generate the denser frequency/amplitude map with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-sweep --preset-grid dense --out reports\rule-2011-sweep-dense.json
```

Generate the dedicated Floquet calibration surface with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-floquet --out reports\rule-2011-floquet.json
```

The default Floquet scan now includes 0.5-4.0 spatial cycles, which produces
first-pass +1 and -1 sign-change boundary crossings and refined beta-axis curve
points. Use `--modes` for an explicit wave-number list, or `--mode-min`,
`--mode-max`, and `--mode-steps` for regular refinement around an observed
crossing.

The Figure 8 calibration command now uses the named
`rule_fig8_source_like` parameter set by default and reports source-style axes:
forcing period in milliseconds, secondary stimulus frequency in Hz, and wave
number in radians across the modeled domain. The refined `boundary_curves`
include branch labels, beta bracket widths, Floquet-condition residuals, curve
continuity fields, and a polynomial period-to-wave-number fit. Use
`--curve-refine-steps` and `--curve-refine-tolerance` to densify the targeted
beta-root refinement inside the current sign-change bands without expanding the
whole grid.

The Figure 8 domain/wave-number normalization is now an explicit report and CLI
parameter instead of only a post-hoc affine display fit. The default v5
decision is zero-offset:

```text
source_beta = 0.42868451880191133 * model_beta_cycles
```

This scale is used by source-curve errors and `fit_objective`; raw identity,
best scale-only, and affine beta mappings remain in the report as diagnostics.
Use `--figure8-beta-scale`, `--source-beta-per-model-cycle`, or
`--model-cycles-per-source-beta` to change the domain scale. Use
`--source-beta-modes` or `--source-beta-min/--source-beta-max/--source-beta-steps`
to define the scan in the source Figure 8 beta coordinate.

The Floquet report also reads
`reports/source-curves/rule-2011-fig8-source-curves.json` by default when the
report is written to `reports/rule-2011-floquet.json`. That file contains
public-safe numeric coordinates digitized from the private Rule Figure 8C page
render. The report-level `source_curve_comparison` summary and per-curve
`source_comparison` fields are now the first quantitative validation surface
against the published Figure 8C boundary geometry. The current comparison is a
calibration diagnostic, not a claim of reproduction; remaining error is
expected until the Rule constants, branch coverage, and domain scale are tuned.

Generate the compact Figure 8 fit-search report with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-fit --out reports\rule-2011-fit-search.json --max-trials 25
```

The fit report keeps the baseline preset intact and ranks small perturbations
of `tau_e`, `tau_i`, coupling strengths, kernel widths, thresholds, and stimulus
threshold against the same source-curve objective.

The sweep command also accepts explicit lists and regular grids:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-sweep --period-min 40 --period-max 160 --period-steps 13 --amplitude-min 0.4 --amplitude-max 1.2 --amplitude-steps 5
```

The sweep and Floquet report formats are:

```text
rule-2011-sweep-report-v1
rule-2011-floquet-calibration-v5
rule-2011-figure8-fit-search-v2
```

This is intentionally separate from:

```text
bressloff-paper-calibration-v4
```

## Deferred

- Paper-calibrated dense sweeps for Rule Figures 3 and 6.
- Further Figure 8 calibration: reduce the current source-curve residuals by
  tuning model constants and domain/wave-number normalization.
- Feed-forward inhibition sweep for Rule Figure 9.
- Hexagonal-lattice normal-form report for Rule Figure 10.
- Two-hemifield coupling for Rule Figure 11.
- Exact figure-level calibration of periods, amplitudes, domain size, time step,
  and initial-condition sensitivity.
