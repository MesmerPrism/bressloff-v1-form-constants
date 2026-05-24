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
  - homogeneous periodic-orbit summaries
  - first-pass 2x2 monodromy multipliers for representative spatial modes

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

The sweep report format is:

```text
rule-2011-sweep-report-v1
```

This is intentionally separate from:

```text
bressloff-paper-calibration-v4
```

## Deferred

- Dense 1D and 2D frequency/amplitude sweeps for Rule Figures 3 and 6.
- Figure-level Floquet phase-boundary calibration for Rule Figure 8.
- Feed-forward inhibition sweep for Rule Figure 9.
- Hexagonal-lattice normal-form report for Rule Figure 10.
- Two-hemifield coupling for Rule Figure 11.
- Exact figure-level calibration of periods, amplitudes, domain size, time step,
  and initial-condition sensitivity.
