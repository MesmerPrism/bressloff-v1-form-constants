# Rule 2011 Figure Mapping

Updated: 2026-05-24

This ledger maps Rule, Stoffregen, and Ermentrout 2011 source figures onto the
current `rule_flicker_ei` implementation track. It keeps Rule's scalar E/I
flicker model separate from the Bressloff orientation-hypercolumn registry.

## Figure Targets

| Source target | Current implementation level | Report surface | Remaining work |
| --- | --- | --- | --- |
| Figure 3 frequency response | Partial qualitative coverage through period sweeps | `reports/rule-2011-sweep-dense.json` | Match source parameter set and measure response curves against the paper axes. |
| Figure 4 pattern islands | First qualitative frequency/amplitude map | `reports/rule-2011-sweep-dense.json` | Calibrate island boundaries and add source-figure comparison metrics. |
| Figure 5 period examples | Seeded high-frequency stripe and low-frequency hexagonal examples | `reports/rule-2011-regimes.json`, `reports/rule-2011-sweep.json` | Tighten exact periods/amplitudes and initial-condition sensitivity. |
| Figure 6 frequency-amplitude map | Dense first-pass sweep grid with spatial and temporal classifiers | `reports/rule-2011-sweep-dense.json` | Increase resolution, tune thresholds, and compare to published map. |
| Figure 8 Floquet boundaries | First homogeneous-orbit monodromy multipliers for representative periods | `reports/rule-2011-sweep.json`, `reports/rule-2011-sweep-dense.json` | Convert representative crossings into continuous phase-boundary curves. |
| Figure 9 feed-forward inhibition | CLI supports `--stim-i-fractions` grids, but no dedicated report yet | `rule-sweep --stim-i-fraction-*` | Generate inhibition-specific report and website panel. |
| Figure 10 hexagonal-lattice normal form | Deferred | None | Implement lattice-reduced normal-form analysis after Floquet boundaries stabilize. |
| Figure 11 two-hemifield coupling | Deferred | None | Add a coupled-domain model only after the one-field Rule track is calibrated. |

## Current Classifiers

The sweep reports use `classification_version:
rule-spatial-temporal-diagnostics-v2`.

- Spatial diagnostics score stripe axes, square pairs, and hexagonal triplets.
- The report exports top spatial modes, mode entropy, dominant cycles, and a
  spatial confidence value.
- Temporal diagnostics export `C(T)`, `C(2T)`, and `C(3T)` plus a one-to-one,
  period-doubled, or mixed response estimate.
- Weak spatial contrast is explicitly marked in `classification_note` so a
  strong temporal repeat is not overstated as a strong visible pattern.

These diagnostics are implementation tools. They should not be read as a final
reproduction of the paper's stability boundaries until the figure-level
calibration pass is complete.
