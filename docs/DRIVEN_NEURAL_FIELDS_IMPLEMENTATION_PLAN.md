# Driven Neural Fields Implementation Plan

Updated: 2026-05-26

This plan covers the seven driven-input, architecture, color, and perceptual
function papers added after the Bressloff and Rule extraction passes. It keeps
three existing model tracks distinct:

- `bressloff_orientation_hypercolumn`: spontaneous orientation-hypercolumn
  planforms, contour overlays, stability, and Bressloff figure targets.
- `rule_flicker_ei`: diffuse time-periodic E/I flicker, frequency/amplitude
  sweeps, and Floquet diagnostics.
- Driven-input families: localized or structured inputs, MacKay and
  Billock-Tsou-style targets, pinwheel architecture, color extensions, contrast
  gradients, and perceptual grouping.

The public repo should contain only generated model outputs, abstractions,
implementation plans, report schemas, and source citations. Original PDFs,
figure crops, page renders, and extraction notes stay under ignored
`private/papers/`.

Current tracked outputs:

- `reports/driven-neural-fields-registry.json` uses
  `format: driven-neural-fields-registry-v1`.
- `reports/mackay-localized-input.json` uses
  `format: mackay-localized-input-report-v2`.
- `reports/bolelli-time-periodic-input.json` uses
  `format: bolelli-time-periodic-input-report-v6`.
- `reports/nicks-orthogonal-response.json` uses
  `format: nicks-orthogonal-response-report-v6`.
- The MacKay report is a generated first-pass diagnostic. Bolelli and Nicks now
  include equation-derived source-target comparison layers while still reporting
  `calibrated=false`.

Related provenance note:

- [`docs/ORIGINAL_AUTHOR_SOFTWARE_METHODS.md`](ORIGINAL_AUTHOR_SOFTWARE_METHODS.md)
  records what the source papers say about AUTO/XPPAUT, MATLAB, Julia,
  Mathematica, PETSc/Trilinos, BifurcationKit, and other original-author
  computational workflows. It should guide implementation decisions, but it does
  not change any report's calibration status.

## Audit Verdict

The seven papers contain enough mathematics for implementation planning, but
not all should become simulator code at the same depth.

| Paper | Implementation depth | Math completeness | Main gap |
| --- | --- | --- | --- |
| Nicks et al. 2021, sensory-induced hallucinations | High | Strong: driven neural field, kernels, 1D and 2D amplitude equations, resonance tongues, orthogonal-response figures, and numerical method are available. | Exact figure-level source curves/images would need private digitization before calibration claims. |
| Tamekue, Prandi, Chitour 2024, MacKay effect | High | Strong: Amari field, DoG kernel, retinocortical map, MacKay inputs, fixed-point map, parameter examples, and Gaussian-kernel negative control are available. | Public validation should be generated zero-level metrics, not copied SIAM figures. |
| Bolelli and Prandi 2025, time-periodic inputs | High | Strong: periodic input theorem, linear periodic-state formula, DoG kernel, localized flicker inputs, contour-width pole diagnostics, and example parameters are available. | A robust pole solver and nonlinear Billock-Tsou calibration are nontrivial. |
| Veltz, Chossat, Faugeras 2015, pinwheels | Medium | Good: neural field, local/long-range kernels, pinwheel lattices, symmetry groups, torus dynamics, and simulation parameters are available. | Requires pinwheel maps, wallpaper-group machinery, and large-domain FFT/continuation work. |
| Faugeras, Song, Veltz 2022, spatial-color hallucinations | Medium | Good: 4D spatio-chromatic field, separable kernels, planform equations, branch examples, and continuation workflow are available. | Full implementation is high cost: 3D/4D state, continuation, and likely GPU-scale workloads. |
| Carroll and Bressloff 2018, contrast gradients | Low/medium | Good: R2 x S1 field, gradient/tangent solution interpretation, symmetry analysis, and laminar variant are available. | It is an adjacent perceptual-function model, not a hallucination preset. |
| Sarti and Citti 2015, perceptual units | Low/medium | Good: SE(2) mean-field model, Fokker-Planck connectivity, discrete affinity matrix, and eigenmode grouping examples are available. | It is a separate image/grouping module and needs public input stimuli or generated datasets. |

## Ranked Examples

1. `mackay_rays_linear_stationary`
   Source: Tamekue-Prandi-Chitour 2024. Implemented as a scalar Amari
   fixed-point report with DoG kernel, finite cortical diagnostic grid, MacKay
   rays input, generated thumbnails/metrics, and Gaussian-only negative
   control. This is the easiest high-value first example because it is static,
   generated-only, and compatible with the current scalar report style.

2. `mackay_target_linear_stationary`
   Implemented in the same report, with the MacKay target/ray-control input.
   It validates that the input generator is not hardcoded to one orientation.

3. `bolelli_heaviside_flicker_periodic_state`
   Source: Bolelli-Prandi 2025. Implemented with time-periodic localized input,
   transient trimming, periodic-state residual, and response phase metrics.

4. `bolelli_contour_width_frequency_sweep`
   Implemented as generated stripe-width rows versus flicker frequency plus an
   equation-derived principal-pole width comparison. The source-side pole-width
   convention is now accepted by residual threshold, and the report contains
   public-safe Figure 5 source-equation curve samples for all three source DoG
   pairs. A generated decay-width estimate uses the same pole convention only
   when its envelope fit passes the diagnostic quality gate; generated half-max
   width remains an auxiliary renderer metric.

5. `nicks_2d_orthogonal_response_amplitude`
   Source: Nicks et al. 2021. Implemented as a reduced 2:1 resonance amplitude
   diagnostic with forcing wavevector, response wavevector,
   rectangle/oblique/orthogonal response family, orthogonality error,
   Appendix-B kernel-derived coefficient diagnostics, a source-equation Figure
   8 boundary curve, a public residual field over the source grid, and
   parameter-grid/region residual checks.

6. `nicks_billock_tsou_generated_map`
   Partially implemented as a source-safe inverse-log-polar generated response
   frame in the Nicks report. It remains a website/report target, not a
   reproduction claim.

7. `nicks_halfspace_forcing_full_field`
   Full scalar driven neural-field simulation with half-space forcing. Higher
   value than a diagram, but more sensitive to grid, boundary, initial
   condition, and convolution performance.

8. `bolelli_billock_tsou_nonlinear_flicker`
   Nonlinear localized static-plus-flicker input. Valuable, but it should wait
   until the linear periodic-state machinery and contour-width metrics are
   stable.

9. `carroll_contrast_gradient_phase_overlay`
   A sidebar/report that overlays generated phase vectors aligned with a
   scalar field gradient or level-set tangent. This is public-safe and useful
   for honesty about other V1 computations, but not a core hallucination model.

10. `sarti_citti_perceptual_grouping_affinity`
    A separate generated-stimulus eigenmode report. Useful if the project
    becomes a broader V1 computation atlas.

11. `pinwheel_lattice_architecture_caveat`
    Start as generated architecture schematics and report metadata only. Full
    torus dynamics and long-range pinwheel connectivity are deferred.

12. `faugeras_color_planforms`
    Start, if ever needed, as analytic color-planform stills. Full
    spatio-chromatic continuation and localized snaking are deferred high-risk
    work.

## Model Example Registry

Use a separate driven-example registry rather than expanding the Bressloff or
Rule preset enums directly. The registry emits
`reports/driven-neural-fields-registry.json` and is exposed through the local
metadata API as `driven_examples`.

Suggested entry fields:

| Field | Meaning |
| --- | --- |
| `id` | Stable example id, such as `mackay_rays_linear_stationary`. |
| `source_key` | Bibliographic key, not a private file path. |
| `model_family` | One of the driven families listed below. |
| `implementation_status` | `implemented`, `partial`, `future`, or `reference_only`. |
| `public_claim_level` | `diagnostic`, `calibration_target`, `first_pass`, or `source_target_comparison`. |
| `rights_status` | `generated_only`, `private_source_required`, `cc_by_with_caveat`, or `permission_required`. |
| `mathematical_object` | Neural-field equation, fixed-point map, reduced amplitude equation, or continuation target. |
| `domain` | 1D cortical line, 2D cortical plane, periodic cell, log-polar retinal view, SE(2), pinwheel lattice, or spatio-chromatic domain. |
| `kernel_family` | DoG, Wizard-hat, separable spatial-color, FP/SE(2), pinwheel local/long-range, or other. |
| `input_type` | None, spatial stripe, localized half-space, MacKay rays/target, time-periodic flicker, static-plus-flicker, generated image fragments. |
| `method` | Fixed-point iteration, time stepping, periodic-state residual, amplitude-equation scan, Floquet/continuation, eigenmode extraction. |
| `parameters` | Named source-like constants and repo defaults. |
| `report_target` | JSON report path and format version. |
| `expected_behavior` | Qualitative or quantitative behavior to check. |
| `missing_evidence` | What prevents a stronger reproduction claim. |

Initial implemented/planned model-family names:

| Model family | Scope |
| --- | --- |
| `spatial_forcing_orthogonal_response` | Nicks/Billock-Tsou-style spatial forcing and 2:1 response diagnostics. |
| `mackay_localized_input` | Static localized MacKay input-output examples. |
| `localized_time_periodic_input` | Bolelli-Prandi-style localized input plus periodic forcing. |
| `pinwheel_architecture_extension` | Pinwheel lattice and long-range architecture caveats. |
| `spatio_chromatic_field` | Faugeras-Song-Veltz color planforms and localized states. |
| `contrast_gradient_field` | Carroll-Bressloff contrast-gradient direction encoding. |
| `perceptual_grouping_field` | Sarti-Citti SE(2) grouping and affinity eigenmodes. |

## Per-Paper Implementation Units

| Example id | Equation or reduction | Domain and symmetry | Kernel/connectivity | Input/forcing | Source parameters | Method | Output target | Difficulty | Missing evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `nicks_1d_resonance_tongues` | 1D forced scalar neural field and n:1 amplitude equation | Cortical line, weak forcing near Turing instability | Balanced Wizard-hat/Mexican-hat kernel | `cos(kf x)` multiplicative drive | Example uses `sigma = 0.8`, threshold `h = 0`, small detuning/distance-from-threshold constants | Amplitude-equation scan | Resonance tongue report; dominant 2:1 band | Medium | Needs source curve digitization for quantitative tongue comparison. |
| `nicks_2d_orthogonal_response_amplitude` | 2D 2:1 reduced amplitude equations | 2D cortical plane; forcing along x; response allowed along y | Balanced isotropic source kernel with Appendix-B mode coefficients | Spatial stripe forcing | Generated default uses source Figure 8-style `sigma = 0.5`, `h = 0`, `epsilon^2 delta = 0.3`, `gamma = {0.1,0.4,0.65,1.1}`, `v2/k0 = {0,0.05,0.25,0.75,1}`, and kernel-derived `Phi` coefficients | Symmetric-branch amplitude solve, generated fields, source-grid checks, coefficient table, and diagnostic classification | `reports/nicks-orthogonal-response.json` | Medium | Implemented as source-equation and Figure 8-style diagnostics; digitized region curves and calibrated thresholds are still missing. |
| `nicks_billock_tsou_generated_map` | Reduced-amplitude response mapped through an inverse log-polar visual-field frame | 2D cortical diagnostic frame plus generated retinal/log-polar frame | Same normalized mode-gain abstraction | Stripe forcing with orthogonal response mode | Uses the same generated Nicks defaults | Generated map diagnostic | `reports/nicks-orthogonal-response.json` | Medium/hard | Partially implemented; localized source-target geometry and source metrics remain missing. |
| `nicks_halfspace_billock_tsou_full_field` | Full neural field with adaptation | 2D cortical half-space plus inverse log-polar retinal view | Isotropic local kernel plus adaptation | Half-space stripe/ring/fan forcing, optional flicker | Numerical method uses regular square mesh and FFT in source; repo can start smaller | Time stepping | Future Billock-Tsou-style ring/fan response report | Hard | Needs grid/boundary choices and private source-target metrics. |
| `tamekue_plain_funnel_tunnel_negative_control` | Stationary Amari fixed point `a = I + mu omega * f(a)` | 2D cortical plane; funnel/tunnel remain symmetric | 2D DoG | Pure `PF` or `PT` cosine input | DoG constraints and `mu < mu0`; generated defaults can use source-like DoG | Fixed-point iteration | Negative-control report showing no induced MacKay contours | Easy | Needs acceptance metric for "same zero-level structure." |
| `mackay_rays_linear_stationary` | Linear stationary input-output map | 2D finite cortical diagnostic grid; retinal view deferred | 2D DoG, balanced case supported | `cos(5 pi x2) + epsilon H(2 - x1)` | Source examples use `L = 10`, `epsilon = 0.025`, `mu = 1`, `kappa = 1`, `2 pi^2 sigma1^2 = 1`, `2 pi^2 sigma2^2 = 2`; generated report uses `n = 112`, finite zero-padded convolution | Fixed-point iteration | `reports/mackay-localized-input.json` | Easy | Implemented as first-pass generated diagnostic; source-target numeric comparison is still missing. |
| `mackay_target_linear_stationary` | Same fixed-point map | 2D finite cortical diagnostic grid; retinal view deferred | 2D DoG | Target cosine plus localized symmetric ray controls | Same source-like kernel and epsilon constants | Fixed-point iteration | Same report, second example row | Easy/medium | Implemented as first-pass generated diagnostic; input geometry needs source-target calibration. |
| `tamekue_nonlinear_mackay_stationary` | Nonlinear Amari fixed point | 2D cortex and retinal view | 2D DoG | Same MacKay inputs | Source examples include odd saturating functions such as `s/(1 + |s|)` | Fixed-point iteration | Linear-vs-nonlinear comparison | Medium | Convergence and threshold choices need report fields. |
| `mackay_gaussian_kernel_negative_check` | Same fixed point with excitatory Gaussian-only kernel | 2D finite cortical diagnostic grid | Gaussian-only comparator | Same localized input | No source reproduction expected | Negative-control report | Kernel-family diagnostic in `reports/mackay-localized-input.json` | Easy | Implemented as a diagnostic control, not a figure match. |
| `bolelli_periodic_attractor` | Neural field with T-periodic input and unique periodic state under contraction assumptions | 1D/2D cortical domain, depending on input | Even kernel with `||omega||_1 < 1`; linear case explicit | Generic T-periodic input | Repo defines period/frequency grid | Time stepping plus period residual | `reports/bolelli-time-periodic-input.json` | Easy/medium | The theorem is broad; example-specific metrics must be defined. |
| `bolelli_heaviside_flicker_stripes` | Linear periodic solution with kernel expansion | 1D cortical coordinate lifted to 2D display | 1D DoG | `H(x1) cos(lambda t)` or center/periphery variant | Source examples use `k = 1` and DoG pairs including `(0.4/sqrt(2 pi), 0.8/sqrt(2 pi))` | Periodic-state solver plus source-convention pole diagnostic | Moving-stripe/contour-width diagnostic | Medium | Implemented as source-target diagnostic; generated decay-width estimates share the source pole convention only when the fit passes and do not support calibration language yet. |
| `bolelli_contour_width_frequency_sweep` | Principal-pole/stripe-width dependence | 1D cortical coordinate | DoG inhibitory scale | Frequency sweep `lambda` | Source sweep range includes `lambda` from about `2` to `100` in examples | Numeric pole/root search, source-equation Figure 5 curve samples, and generated simulation metric | `reports/bolelli-time-periodic-input.json` | Medium/hard | Implemented as an equation-derived source-target diagnostic; source-panel digitization thresholds and nonlinear examples remain missing. |
| `bolelli_mackay_flicker_persistence` | Static MacKay input plus localized flicker | 1D cortical effective input, retinal display | DoG | Center localized flicker replacing static localized information | Frequency examples include `lambda = 0` and high-frequency comparison | Time stepping and width metric | MacKay flicker report row | Medium | Source examples are visual, not table-calibrated. |
| `bolelli_billock_tsou_nonlinear_flicker` | Nonlinear periodic field | 2D display with effective 1D input | DoG | Static radial/fan term plus peripheral flicker | Source example uses clipped linear nonlinearity, `lambda = 60`, `k = 1`, source-like DoG pair | Time stepping over one period after convergence | Animated generated report frames | Hard | Nonlinear parameter calibration and source comparison are deferred. |
| `pinwheel_lattice_bifurcation_baseline` | Neural field on periodic square/hexagonal domain | Periodic square or hexagonal cortex | Local DoG plus optional long-range pinwheel term | None; spontaneous activity | Source examples specify sigmoid threshold, local width, large meshes | Bifurcation/continuation or generated schematic first | Architecture caveat report | Hard | Requires pinwheel map generation and continuation machinery. |
| `pinwheel_torus_dynamics` | Perturbation of translation torus under long-range connections | Pinwheel wallpaper groups such as pmm and p3m1 | Orientation-selective long-range connectivity | None | Source examples use long-range strength, anisotropy, and large domains | Phase portrait/torus diagnostics | Reference-only until architecture module exists | Very hard | Not needed for first driven-input implementation. |
| `color_planforms_analytic` | Spatio-chromatic neural-field planforms | 2D periodic cortex x chromaticity | Separable spatial Mexican-hat and color kernels | None | Analytic planforms SR, S2, S2uv, S4uv; source examples use color harmonic choices | Direct analytic renderer | Optional generated color stills | Hard | Color-space display and stability claims need care. |
| `color_continuation_snaking` | Full spatio-chromatic continuation | 2D cortex x 1D/2D color | Same separable kernels | None | Source continuation uses large grids and GPU workflow | Pseudo-arclength continuation | Deferred localized color-state report | Very hard | Outside current Rust dependencies and report scope. |
| `contrast_gradient_phase_overlay` | R2 x S1 contrast-polarity/orientation field near a scalar solution | 2D scalar field plus phase variable | Isotropic spatial kernel times angular kernel | Optional generated scalar pattern | Phase aligns with gradient or tangent of scalar field | Derived vector overlay | Website/sidebar diagnostic | Medium | Adjacent perceptual function; do not label as hallucination reproduction. |
| `contrast_gradient_laminar_model` | Deep scalar layer feeding superficial phase layer | Two-layer cortical field | Deep and superficial kernels plus vertical feedforward | None or generated scalar state | Source gives simplified laminar equations | Reference implementation later | Future contrast report | Hard | Requires new model family and validation data. |
| `sarti_citti_grouping_affinity` | Discrete SE(2) mean-field eigenproblem | Position-orientation samples in SE(2) | Symmetrized Fokker-Planck/association-field kernel | Generated oriented segments | Examples use discrete oriented patches and eigenvectors | Affinity matrix eigenmodes | Perceptual grouping report | Medium | Needs public generated stimuli; separate from driven hallucination track. |

## Implementation Phases

### Phase D0 - Registry and Schema

Status: implemented for the first high-priority driven examples.

Add a small shared registry layer for cross-paper examples. Keep it separate from
`PaperPreset` and `RulePreset`; those enums should not absorb every later paper.

Delivered:

- `rust-v1-sim/src/models/driven/registry.rs` with `DrivenExampleDetails`
  and the driven example registry.
- CLI command: `driven-registry --out reports/driven-neural-fields-registry.json`.
- Report format: `driven-neural-fields-registry-v1`.
- Tests that all driven entries have `model_family`, `source_key`,
  `public_claim_level`, `rights_status`, and `implementation_status`.

### Phase D1 - Minimal Shared Driven-Field Abstraction

Status: partial, with the module-boundary prerequisite now complete. The Rust
binary remains one crate, but its model-family code is split under
`rust-v1-sim/src/models/`:

- `models/bressloff/presets.rs`, `planform.rs`, and `reports.rs` own the
  Bressloff preset registry, orientation-hypercolumn planforms, stability
  diagnostics, and public calibration reports.
- `models/rule/presets.rs`, `mod.rs`, `sweep.rs`, `floquet.rs`, `fit.rs`, and
  `reports.rs` own the Rule preset registry, E/I flicker simulator, sweeps,
  Floquet boundary curves, Figure 8 fit diagnostics, and Rule reports.
- `models/driven/registry.rs`, `mackay.rs`, `bolelli.rs`, `nicks.rs`, and
  `reports.rs` own the driven-field registry and first generated MacKay,
  Bolelli, and Nicks diagnostic reports.
- `numeric/convolution.rs` and `numeric/metrics.rs` hold shared numeric helpers
  that Bolelli and Nicks should reuse or extend before introducing new solver
  machinery.

The first MacKay path has a finite-grid input generator, separable Gaussian/DoG
convolution, fixed-point iteration, generated report schema, and generated
public-safe output report. The Bolelli path adds localized time-periodic
diagnostics, and the Nicks path adds reduced amplitude-equation orthogonal
response diagnostics. The next implementation should focus on source-derived
numeric targets and deferred half-space/nonlinear extensions without
re-expanding `main.rs`.

Keep the scalar driven-field abstraction smaller than the Bressloff
orientation-hypercolumn and Rule E/I implementations:

- Domains: 1D cortical line, 2D rectangular cortical grid, generated retinal view.
- Kernels: 1D/2D DoG, Wizard-hat comparator, Gaussian-only negative control.
- Transfer functions: linear, odd sigmoid, clipped linear.
- Input primitives: cosine stripe, Heaviside half-space, MacKay rays, MacKay
  target, localized center/periphery mask, time-periodic multiplier.
- Solvers: fixed-point stationary iteration, explicit/RK time stepping,
  periodic-state residual after warmup.
- Metrics: zero-level count, contour/stripe width, input-output correlation,
  response orientation, orthogonality error, period residual, kernel-family flag.

Do not add color, pinwheel, SE(2), or continuation dependencies in this phase.

### Phase D2 - First Implementable Example

Status: implemented as a generated first-pass diagnostic.

`mackay_rays_linear_stationary`, `mackay_target_linear_stationary`, and
`mackay_gaussian_kernel_negative_check` are generated by one report command.

Command:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe mackay-report --out reports\mackay-localized-input.json --n 112 --iterations 48
```

Report format:

```text
mackay-localized-input-report-v2
```

The report includes generated input/output thumbnails, DoG parameters,
fixed-point residuals, zero-crossing metrics, input-output correlation,
validation flags, and a Gaussian-only negative-control row. Status language
remains first-pass diagnostic until source-derived numeric targets exist.

### Phase D3 - Localized Time-Periodic Input

Status: implemented as a generated source-target diagnostic for the
periodic-state, Heaviside-flicker, and frequency-sweep examples. The report now
applies an accepted source-side principal-pole width convention under the paper
Fourier convention, adds public-safe Figure 5 source-equation curve samples for
the three source DoG pairs, and adds a generated decay-width estimate that
shares that pole convention when the envelope fit passes the diagnostic gate.
Generated half-max support remains an auxiliary renderer metric rather than a
width residual.

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe bolelli-report --out reports\bolelli-time-periodic-input.json
```

Report format:

```text
bolelli-time-periodic-input-report-v6
```

Delivered fields:

- period/frequency and time-step settings,
- transient/warmup duration,
- periodic-state residual over one period,
- response phase relative to input,
- contour or stripe width,
- equation-derived principal-pole width comparison with source-parameter,
  lambda-range, accepted source-side pole residual, and asymptotic-width quality
  flags,
- top-level `figure5_source_curves` with source-equation curve samples for all
  three Figure 5 DoG pairs, pole residuals, principal-pole stripe widths, and
  Proposition 4.17 asymptotic-width residuals,
- generated decay-width estimate, decay-rate fit, fit R-squared, fit-point
  count, and a documented minimum R-squared acceptance gate,
- frequency sweep rows,
- generated thumbnails or row-major compact frames.

Implemented code shape:

- `rust-v1-sim/src/models/driven/bolelli.rs`.
- Bolelli report structs in `models/driven/reports.rs`.
- `BolelliReportConfig` and `bolelli_time_periodic_report(config)`.
- `bolelli-report` CLI command writes only generated numeric data and generated
  thumbnails to `reports/bolelli-time-periodic-input.json`.
- Keep the comparison diagnostic: use generated contour/stripe-width metrics,
  period-lock residuals, source-like parameter notes, and the principal-pole
  target relation, but do not claim source-figure reproduction or generated
  width calibration.

Implemented numerical target:

- 1D finite cortical coordinate with zero or periodic boundary option recorded
  in the report.
- Linear DoG kernel using source Fig. 5 pairs such as
  `k=1`, `(0.4/sqrt(2*pi), 0.8/sqrt(2*pi))`.
- Localized Heaviside or center/periphery periodic input with frequency rows.
- Warmup over several periods, then compare the last two periods by residual and
  phase.
- Stripe/contour width measured from generated threshold crossings or
  half-maximum support and kept as an auxiliary renderer metric.
- Generated decay-width fitted from the unforced-side amplitude envelope and
  converted to the same `1/(2*alpha)` pole-width convention only when the fit
  passes the recorded R-squared and sample-count gate.
- Principal-pole target width from the public-safe equation relation
  `1 +/- i*lambda = omega_hat(z)` with the source Fourier convention
  `omega_hat(z)=exp(-2*pi^2*sigma1^2*z^2)-k exp(-2*pi^2*sigma2^2*z^2)`.
- Accepted source-side width convention
  `1/(2*Re z0(lambda))` with pole-equation residual tolerance recorded in the
  report.
- Source-equation Figure 5 curve table generated from the same public equations
  over repo-selected lambda samples in `[2, 100]`; it is not digitized
  source-panel data and does not permit calibration language.

Remaining work:

- Add independently digitized source-panel targets only if the project needs to
  compare against the published plot image itself; current Figure 5 data are
  source-equation diagnostics.
- Tighten or replace the generated decay-width fit after source-panel numeric
  targets exist; current residuals are diagnostic and not calibration evidence.
- Add nonlinear Billock-Tsou-style flicker only after the linear periodic-state
  report has stable validation thresholds.

### Phase D4 - Nicks Orthogonal-Response Diagnostics

Status: implemented as a generated source-target diagnostic for the 2D
orthogonal-response amplitude example. The report compares generated wavevector
geometry with the equation-derived 2:1 response target and now records
source-equation coefficient tables, a Figure 8 source-equation boundary curve,
public residual field over the source grid, curve residual thresholds, and
source-grid region-side margin checks. Full half-space simulations and
source-panel calibration remain deferred.

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe nicks-report --out reports\nicks-orthogonal-response.json
```

Report format:

```text
nicks-orthogonal-response-report-v6
```

Delivered fields:

- forcing wavevector and response wavevector,
- 2:1 resonance ratio, Turing wavenumber, and detuning,
- rectangle/oblique/orthogonal-response classification,
- orthogonality error in degrees,
- generated cortical forcing and response frames,
- generated inverse-log-polar visual-field frame,
- symmetric-branch amplitude-equation residual,
- equation-derived 2:1 wavevector target comparison,
- source Figure 8-style parameter grid checks for
  `sigma=0.5`, `h=0`, `epsilon^2 delta=0.3`,
  `gamma={0.1,0.4,0.65,1.1}`, and
  `v2/k0={0,0.05,0.25,0.75,1}`,
- source-equation Figure 8 boundary curve
  `gamma_p(v2/k0)=(Phi4-Phi1)*epsilon^2*delta/(Phi1*beta_c)`,
- public Figure 8 residual field over the source gamma/detuning grid,
- curve residual tolerance in gamma units and a source-grid region-margin
  threshold derived from the published gamma grid,
- kernel-derived `beta_c`, `mu`, `beta2`, `beta3`, `zeta1`, `zeta4`,
  `zeta6`, `Phi1`, `Phi4`, `gamma_c`, and `gamma_p` coefficient
  diagnostics from equations 4.11-4.18 and Appendix B,
- validation flags for rendered target coverage, diagnostic metric availability,
  source-target comparison, and calibration.

Implemented code shape:

- `rust-v1-sim/src/models/driven/nicks.rs`.
- Nicks report structs in `models/driven/reports.rs`.
- `NicksReportConfig` and `nicks_orthogonal_response_report(config)`.
- `nicks-report` CLI command writes only generated numeric data and generated
  thumbnails to `reports/nicks-orthogonal-response.json`.

Remaining work:

Only after these reduced examples are stable should the repo add a full
half-space driven neural-field simulation.

- Add source-panel or independent full-field residuals before any calibrated
  Figure 8 source-figure match is reported.
- Add localized half-space Billock-Tsou geometry and adaptation only after the
  reduced diagnostic source-equation thresholds remain stable.

### Phase D5 - Public Website Abstraction

Status: partial. The local metadata endpoint now exposes `driven_examples`, but
the public viewer should not present private-source details or claim calibrated
matches.

The website should use abstract, source-safe language:

- "driven neural fields after Rule"
- "localized input"
- "time-periodic input"
- "MacKay-style target"
- "Billock-Tsou-style orthogonal response"
- "diagnostic generated output"

It should not publish private extraction details, source-page references,
unlicensed figures, or language suggesting perceptual prediction. Generated
still frames and report metrics are preferred over paper-figure reuse.

### Phase D6 - Validation Criteria

For every driven example, the report should distinguish:

- `rendered_target_coverage`: the generator can draw the expected family.
- `diagnostic_metric_available`: a numeric metric exists.
- `source_target_comparison`: private/source-derived public-safe numeric target
  is available and compared.
- `calibrated`: metric residuals meet a documented threshold.

Do not claim reproduction from visual resemblance alone. Until a report-backed
match exists, use `diagnostic`, `first_pass`, `calibration_target`, or
`source_target_comparison`.

Current report status:

- MacKay: rendered target coverage and diagnostic metrics are present;
  source-target comparison is still false.
- Bolelli: principal-pole width source-target comparisons and an accepted
  source-side pole-width convention are present. The report now includes
  public-safe Figure 5 source-equation curves for the three source DoG pairs. A
  generated decay-width estimate shares the pole convention when its envelope
  fit passes, but calibration remains false because source-panel digitization
  and stable generated residual thresholds are still missing.
- Nicks: 2:1 wavevector geometry source-target comparisons, Appendix-B
  kernel-derived coefficient tables, source-equation Figure 8 boundary curves,
  a public residual field over the source grid, and residual thresholds are
  present; calibration remains false because these are equation diagnostics,
  not source-panel or full-field reproduction residuals.

### Phase D7 - Deferred High-Risk Work

Defer these until the three core driven-input reports are implemented:

- Full Nicks half-space simulations with adaptation and optional flicker.
- Veltz pinwheel lattice dynamics and wallpaper-group torus reports.
- Faugeras spatio-chromatic continuation and localized snaking.
- Carroll laminar contrast-gradient neural field.
- Sarti-Citti SE(2) grouping over generated image fragments.
- Any source-figure reuse path that requires explicit license verification.

The software-methods audit reinforces this deferral: Veltz/Faugeras-style
architecture and color work was computed with continuation, Krylov, PETSc,
Trilinos, Julia, BifurcationKit, CUDA, and large-grid workflows that are outside
the current compact Rust report layer.
The local reference audit in
[`docs/BIFURCATIONKIT_REFERENCE_AUDIT.md`](BIFURCATIONKIT_REFERENCE_AUDIT.md)
records which BifurcationKit continuation, Newton-Krylov, Floquet, and sidecar
ideas are useful if those deferred branches are promoted.

## Public Sources

- Nicks, R., Cocks, A., Avitabile, D., Johnston, A., and Coombes, S.
  "Understanding Sensory Induced Hallucinations: From Neural Fields to
  Amplitude Equations." SIAM Journal on Applied Dynamical Systems 20, no. 4
  (2021): 1683-1714. https://doi.org/10.1137/20M1366885
- Tamekue, C., Prandi, D., and Chitour, Y. "A Mathematical Model of the Visual
  MacKay Effect." SIAM Journal on Applied Dynamical Systems 23, no. 3 (2024):
  2138-2178. https://doi.org/10.1137/23M1616686
- Bolelli, M. V., and Prandi, D. "Neural Field Equations with Time-Periodic
  External Inputs and Some Applications to Visual Processing." Journal of
  Mathematical Imaging and Vision 67, article 47 (2025).
  https://doi.org/10.1007/s10851-025-01257-7
- Veltz, R., Chossat, P., and Faugeras, O. "On the Effects on Cortical
  Spontaneous Activity of the Symmetries of the Network of Pinwheels in Visual
  Area V1." Journal of Mathematical Neuroscience 5, article 11 (2015).
  https://doi.org/10.1186/s13408-015-0023-8
- Faugeras, O. D., Song, A., and Veltz, R. "Spatial and Color Hallucinations in
  a Mathematical Model of Primary Visual Cortex." Comptes Rendus Mathematique
  360 (2022): 59-87. https://doi.org/10.5802/crmath.289
- Carroll, S. R., and Bressloff, P. C. "Symmetric Bifurcations in a Neural
  Field Model for Encoding the Direction of Spatial Contrast Gradients." SIAM
  Journal on Applied Dynamical Systems 17, no. 1 (2018): 1-51.
  https://doi.org/10.1137/16M1076125
- Sarti, A., and Citti, G. "The Constitution of Visual Perceptual Units in the
  Functional Architecture of V1." Journal of Computational Neuroscience 38, no.
  2 (2015): 285-300. https://doi.org/10.1007/s10827-014-0540-6
