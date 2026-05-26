# Bressloff Future Implementation Plan

Updated: 2026-05-26

This plan starts from the current Bressloff implementation state:

- 24 named paper presets are exposed through the Rust registry, CLI, API, and
  browser viewer.
- `reports/paper-calibration.json` now uses
  `format: bressloff-paper-calibration-v4` with explicit
  `model_family = bressloff_orientation_hypercolumn` metadata.
- The report currently has 17 preset passes, 7 preset review items, 4 passing
  stability reports, and 1 stability review item.
- The remaining work is calibration and model fidelity, not basic source
  incorporation.
- Workstream 1 now has a generated report plus a private source-profile pass:
  `reports/figure-targets/bressloff-generated-stills.json` exports stable
  Figures 29-36 stills, public-safe derived metrics, and source-profile
  residual fields populated from ignored private page renders/profile JSON.
  It also records a conservative `acceptance_policy`, source-angle residuals,
  and per-still threshold checks. This is a source-target comparison layer, not
  figure-level reproduction.

The implementation deliberately separates three claims:

1. **Rendered target coverage**: the requested visual family can be generated.
2. **Branch-selection agreement**: the normalized amplitude/stability machinery
   selects the same family or phase variant as the source discussion.
3. **Figure-level reproduction**: the generated image or curve matches the
   paper figure geometry closely enough to support quantitative comparison.

## Workstream 1 - Digitized Figure Geometry Calibration

**What it reflects**

This work asks whether generated outputs match the paper figures as figures, not
only as named pattern families. It covers the non-contoured single-map figures
29/30 and the contoured double-map figures 31-36.

**Information quality**

- Strong: source figure identity, planform family, single-map versus double-map
  distinction, and even/odd parity.
- Medium: exact scale, phase, threshold, roll angle, rhombic angle, contour
  sampling density, and crop geometry.
- Weak: machine-readable reference data. The figures need manual or scripted
  digitization from the source pages; the papers do not provide numeric image
  masks.

**Concrete tasks**

1. Create a `reports/figure-targets/` convention for public-safe derived
   calibration data. Do not commit original figure scans unless reuse permission
   or a compatible license is confirmed.
   Status: complete for the current v2 report schema.
2. Add a local/private extraction path for source-page screenshots and derived
   binary/edge masks.
   Status: first pass complete for Figures 29-36. Keep source PDFs, page
   renders, crops, private config, and derived source profiles under ignored
   `private/figure-targets/`; commit only generated report fields and
   public-safe documentation.
3. Add a calibration command mode that exports fixed-size rendered stills for
   each preset with stable viewport, scale, phase, and threshold settings.
   Status: started as `bressloff-geometry`.
4. Add image metrics:
   - edge/contour overlap for contoured figures,
   - radial/angular profile error for rings, tunnels, and cobwebs,
   - active-fraction and edge-density residuals for scalar masks,
   - lattice-angle error from private source-angle annotations.
   Status: implemented as first-pass source-target diagnostics. Source angles
   currently use the same dominant angular-profile bin convention as the
   generated still metrics.
5. Add per-preset calibration fields:
   - target image/mask ID,
   - metric values,
   - accepted parameter set,
   - figure-level status.

**Acceptance criteria**

- Each Figure 29-36 preset has a reproducible still export.
- Each still has at least one quantitative comparison metric.
- The report distinguishes categorical pass from geometry-calibrated pass.
- Public docs can say which figures are calibrated without showing copyrighted
  figure crops.
- Calibration language stays disabled until every required metric passes the
  documented threshold gate and private crop/threshold/source-angle QA is
  explicitly reviewed.

Near-term source-profile extraction should prioritize public-safe derived data:
radial profiles for rings/tunnels, angular profiles for rays/spirals/cobwebs,
edge-density masks for contoured examples, and lattice-angle summaries for
square, rhombic, and hexagonal examples. The first private profile pass now
populates profile, active-fraction, edge-density, and angle residuals for all
Figure 29-36 presets. The current public report records zero
threshold-accepted stills, which is the correct state for a first-pass renderer.
The next refinement is improving the renderer/parameters and repeating private
QA of crop/threshold/source-angle choices. These are calibration metrics, not
source-figure republication.

**Effort**

- Practical first pass: 2-4 focused days.
- Polished figure-by-figure calibration: 1-2 weeks.

## Workstream 2 - Source-Fitted Phase And Threshold Values

**What it reflects**

This work asks whether the renderer is drawing the same analytic variant as the
source tables and captions, especially for square/cobweb, `hex_pi`,
honeycomb/0-hexagonal, triangular, and non-contoured threshold figures.

**Information quality**

- Strong: source tables identify lattice families and even/odd branches.
- Medium: phase labels can be reconciled across the 2001 article, the 2002
  summary, and the 2012 review, but the sign conventions need careful checking.
- Medium-low: exact scalar thresholds for non-contoured regions are visual
  rather than directly tabulated.

**Concrete tasks**

1. Add explicit phase metadata to the registry:
   - `branch = roll|square|rhombic|hex_0|hex_pi|triangle|patchwork_quilt`,
   - `lattice = roll|square|rhombic|hexagonal`,
   - `phase_convention = source|renderer|review`.
2. Add renderer-level names for the 0-hexagonal and pi-hexagonal variants rather
   than treating `honeycomb` and `hex_pi` as informal aliases.
3. Add threshold controls to paper presets instead of relying only on global
   defaults.
4. Add a phase-sign audit against the source tables before changing branch
   selector signs.
5. Update calibration checks so phase mismatch and family mismatch are separate
   statuses.

**Acceptance criteria**

- `fig35_hex_even` and `fig5_hex_cortical` report a phase-specific status, not a
  generic hexagonal mismatch.
- Non-contoured Figure 29/30 presets declare their activity threshold and expose
  it in the report.
- The registry has enough explicit taxonomy fields to support Rule 2011 without
  overloading Bressloff-specific labels.

**Effort**

- 2-5 focused days.

## Workstream 3 - Stability Curve Digitization

**What it reflects**

This work asks whether the linear-stability and bifurcation machinery has the
right curve shapes and transition points, not only the right branch labels.

**Information quality**

- Strong: source anchors for Figures 16, 17, 37, 38, 39, and 40.
- Medium: curve shapes are visible in the papers, but exact numeric data is not
  distributed as tables.
- Medium-low: absolute biological threshold values remain tied to normalized
  choices in the current implementation.

**Concrete tasks**

1. Add curve export modes for:
   - even/odd marginal stability scans,
   - eigenfunction coefficient diagnostics,
   - even hexagonal bifurcation diagnostics,
   - odd hexagonal bifurcation diagnostics.
2. Digitize source curves into private calibration data, then commit only
   derived numeric targets when license-safe.
3. Add error metrics:
   - critical `q` error,
   - branch parity agreement,
   - curve-shape correlation,
   - branch-exchange point error.
4. Add `stability_reports[].curve_metrics` to the v4 report schema.
5. Keep normalized and source-fitted reports separate until the sigmoid
   operating point and absolute threshold scale are explicit.

**Acceptance criteria**

- Figures 16/17 have curve-level checks, not just parity checks.
- Figures 37-40 report coefficient/bifurcation curve metrics.
- The report states whether a failure is a normalization issue, a sign issue, or
  a missing higher-order term.

**Effort**

- 3-7 focused days for a useful curve-calibration pass.

## Workstream 4 - Higher-Order Odd-Hexagonal Branch Selection

**What it reflects**

This is the main remaining mathematical fidelity gap. The renderer can draw the
odd triangular branch, but the current normalized cubic selector still selects
the honeycomb/0-hexagonal partner for the odd-hexagonal stability target. The
source discussion says odd hexagonal stability depends on higher-order terms.

**Information quality**

- Strong: the source identifies the odd triangular, 0-hexagonal, roll, and
  patchwork-quilt branches as the relevant branch family.
- Medium: the qualitative stability ordering is available.
- Uncertain: the exact higher-order coefficients may require careful derivation
  from appendices/review formulas or additional source reconciliation.

**Concrete tasks**

1. Write a derivation note for the current cubic selector:
   - implemented terms,
   - normalized constants,
   - missing higher-order terms,
   - branch cases it can and cannot decide.
2. Extract the odd-hexagonal amplitude-equation terms needed for Figure 40.
3. Add a selector mode:
   - `selector = normalized_cubic`,
   - `selector = source_higher_order_odd_hex`.
4. Add tests that keep the current normalized behavior visible while allowing a
   source-fitted selector to pass the Figure 40 target.
5. Update `fig36_triangle_odd` and `fig40_odd_hex_bifurcation` to point to the
   source-fitted selector once validated.

**Acceptance criteria**

- `fig40_odd_hex_bifurcation` no longer fails because triangle selection is
  collapsed into honeycomb by the normalized cubic selector.
- The report names which selector produced each branch result.
- The old normalized selector remains available as a diagnostic, so the change
  is auditable.

**Effort**

- 4-10 focused days if the needed terms are straightforward to extract.
- Longer if the coefficients have to be rederived from multiple source
  sections.

## Workstream 5 - Patchwork Quilt Branch

**What it reflects**

Patchwork quilt is part of the odd-hexagonal branch taxonomy. It is mainly a
branch/stability validation target, not a central public visual-field preset in
the current page.

**Information quality**

- Medium-good for branch identity and source relevance.
- Medium-low for exact visual rendering as a public figure target.

**Concrete tasks**

1. Add `PatternPreset::PatchworkQuilt` only after the branch basis is explicit.
2. Add `fig36_patchwork_quilt_odd` or a non-rendering
   `patchwork_quilt_odd_branch` report entry, depending on whether it should be
   visual or diagnostic.
3. Tie the branch into the higher-order odd-hexagonal selector.
4. Document whether it is unstable, diagnostic-only, or visually useful under
   the source parameter regime.

**Acceptance criteria**

- The registry can represent patchwork quilt without mislabeling it as
  honeycomb, triangle, or generic hexagonal.
- The calibration report explains whether the branch is expected to render,
  remain unstable, or appear only as a bifurcation diagnostic.

**Effort**

- 1-3 days for a basic diagnostic branch.
- More if it is tied to full higher-order odd-hexagonal calibration.

## Workstream 6 - Registry And Cross-Model Generalization

**What it reflects**

The Bressloff registry should become the pattern for adding Rule 2011 and later
models without forcing every paper into Bressloff-specific terms.

**Information quality**

- Strong for engineering design. The current registry already centralizes
  Bressloff paper metadata.
- Medium for cross-model taxonomy. Rule uses a scalar E/I flicker model rather
  than Bressloff's orientation-hypercolumn model, so the shared fields need to
  be chosen carefully.

**Concrete tasks**

1. Split registry fields into:
   - source metadata,
   - model-family metadata,
   - render target metadata,
   - calibration status metadata.
2. Add `model_family`:
   - `bressloff_orientation_hypercolumn`,
   - future `rule_flicker_ei`.
3. Add `render_domain`:
   - `cortical`,
   - `visual_field`,
   - `stability_curve`,
   - `bifurcation_curve`.
4. Add a report schema note for the v4 transition.
   Status: complete; v4 is the current Bressloff report schema.
5. Keep Bressloff and Rule selectors separate, even if both feed a common
   retinocortical rendering stage.

**Acceptance criteria**

- Adding a Rule 2011 preset does not require changing Bressloff-specific
  calibration vocabulary.
- The viewer can list paper examples by source/model family.
- The report can compare Bressloff static planforms and Rule flicker-driven
  regimes without pretending they are the same model.

**Effort**

- 1-2 days for a clean v4 registry/report design.
- Additional work belongs to the Rule implementation itself.

## Recommended Sequence Before Rule

1. Keep the current Bressloff registry refactor as the base.
2. Add `model_family` and `render_domain` fields before adding Rule presets.
   Status: implemented in the v4 Bressloff report metadata.
3. Start Rule 2011 as a separate scalar E/I model path.
   Status: started as `model_family = rule_flicker_ei` with a separate Rule
   preset registry and qualitative regime report.
4. Return to Bressloff digitized calibration after Rule has a minimal working
   simulator and qualitative regime presets.
   Status: unblocked. Rule now has sweep, Floquet, and Figure 8 diagnostic
   reports, so the next Bressloff step is private source-profile extraction for
   Figure 29-36 geometry metrics.

This sequence keeps the project moving while avoiding a false sense that the
remaining Bressloff work is a blocker for Rule. The unresolved Bressloff items
are important, but they are refinement and calibration work rather than missing
source incorporation.
