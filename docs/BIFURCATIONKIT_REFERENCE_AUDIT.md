# BifurcationKit Reference Audit

Updated: 2026-05-26

Local reference clone:

```text
references/BifurcationKit.jl/
```

Remote:

```text
https://github.com/bifurcationkit/BifurcationKit.jl.git
```

Clone snapshot:

```text
commit: 8b49a92
date: 2026-05-21
version: 0.7.3
license: MIT Expat
```

The clone is a local ignored reference cache, not vendored project code and not
a Rust dependency. Use this audit for implementation design, report schema
choices, and possible future Julia-side validation work.

## Why It Matters

BifurcationKit is the public Julia continuation package named in the
Faugeras-Song-Veltz color-hallucination workflow. The upstream project describes
itself as a package for automatic bifurcation analysis of large equations
`F(u, lambda) = 0`, with pseudo-arclength continuation, Newton-Krylov
correction, dense/sparse/matrix-free eigensolvers, periodic-orbit tools,
Floquet analysis, deflated continuation, branch switching, and GPU-capable
large-scale workflows.

For this repository, that makes it useful in two ways:

1. As a design reference for continuation/report abstractions if we ever add
   architecture, pinwheel, color, or nonlinear full-field branches.
2. As an optional Julia sidecar for high-risk validation experiments, not as a
   near-term dependency of the compact Rust simulator.

## Relevant Upstream Structure

| Upstream path | What it contains | Why it matters here |
| --- | --- | --- |
| `src/ContParameters.jl` | Continuation parameter struct: `ds`, `dsmin`, `dsmax`, `p_min`, `p_max`, `max_steps`, eigenvalue counts, bifurcation detection mode, bisection tolerances, event detection, and memory controls for eigenvectors. | Good model for future report config structs and explicit validation thresholds. |
| `src/continuation/Palc.jl` | Pseudo-arclength continuation, weighted state/parameter dot product, secant and bordered tangent predictors, predictor-corrector loop. | Useful if we implement branch continuation for Bressloff planforms, pinwheel fields, or color states. |
| `src/Newton.jl` | `NewtonPar`, residual history, convergence flags, callback hooks, linear-iteration accounting. | Good shape for Rust report schemas: preserve residual histories, convergence flags, and solver diagnostics. |
| `src/EigSolver.jl` and `src/LinearSolver.jl` | Direct, sparse, Arnoldi, Krylov, and GMRES-style solver abstractions. | A reminder to hide solver choice behind small config structs before any continuation feature lands. |
| `src/LinearBorderSolver.jl` and `src/BorderedProblem.jl` | Bordered linear systems for continuation and bifurcation corrections. | Relevant for future pseudo-arclength branch tracking; not needed for current MacKay/Bolelli/Nicks reports. |
| `src/Bifurcations.jl`, `src/BifurcationPoints.jl`, `src/NormalForms.jl` | Detection/refinement of special points and normal-form calculations. | Useful reference for not over-claiming branch labels without eigenvalue and normal-form evidence. |
| `src/periodicorbit/Floquet.jl` | Floquet multiplier handling, monodromy operators, and the explicit rule that periodic-orbit eigensolvers must target largest-modulus multipliers. | Directly relevant to hardening Rule-style Floquet diagnostics and future periodic-orbit validation language. |
| `src/periodicorbit/` | Shooting, Poincare shooting, trapezoid, collocation, periodic-orbit continuation, and period-doubling/Neimark-Sacker codim-2 tools. | Future reference if Rule or Bolelli ever moves from custom periodic-state checks to general periodic-orbit continuation. |
| `src/wave/` | Traveling-wave model and eigen solver utilities. | Probably reference-only for this repo unless future visual-cortex wave examples are added. |
| `examples/SH2d-fronts-cuda.jl` | Matrix-free FFT/GPU Swift-Hohenberg example, KrylovKit eigensolver, deflation, PALC, branch switching, and normal-form calls. | Closest upstream example to Veltz/Faugeras-scale architecture/color work. Use as sidecar inspiration, not as Rust-port source. |
| `examples/pd-1d.jl` | Reaction-diffusion continuation, Hopf branch switching, periodic orbits, period doubling, shooting, and Floquet workflow. | Best reference for period-doubling report structure if Rule/Floquet work grows beyond the current custom monodromy grid. |
| `examples/SHpde_snaking.jl`, `SH2d-fronts.jl`, `SH3d.jl` | Snaking, fronts, and high-dimensional PDE examples. | Relevant only to deferred localized color/pinwheel architecture work. |

## Fit To Current Project

### Good Near-Term Uses

- Borrow report vocabulary, not code: `continuation_status`, `newton_residuals`,
  `linear_iterations`, `eigen_solver`, `detect_bifurcation`,
  `bisection_tolerance`, `special_points`, `branch_id`, and
  `source_claim_level`.
- Use `ContinuationPar` as a checklist when future Rust commands need many
  solver knobs. Add config structs early instead of high-argument helpers.
- Use `NewtonPar` and `NonLinearSolution` as design references for solver
  reports that preserve residual histories and convergence status.
- Use the Floquet audit point now: if a future Rule report computes multipliers
  by a general eigensolver, the selection target must be largest modulus, not
  largest real part.
- Use the periodic-orbit examples as comparison language for Rule and Bolelli:
  "custom periodic-state diagnostic" versus "general periodic-orbit
  continuation" should be explicit.

### Possible Future Sidecar

A Julia sidecar is plausible only for deferred, high-risk work:

- Veltz-style pinwheel architecture with large grids, symmetry branches, and
  continuation.
- Faugeras-Song-Veltz color hallucinations with spatio-chromatic state vectors,
  pseudo-arclength continuation, branch switching, and GPU FFTs.
- Full nonlinear Billock-Tsou/Nicks half-space fields if reduced diagnostics
  stop being enough.

If added, keep it as an optional `tools/` or `experiments/` workflow that emits
public-safe JSON reports. Do not make the Rust viewer depend on Julia for normal
report generation.

### Poor Near-Term Uses

- Do not port BifurcationKit wholesale into Rust.
- Do not add Julia as a required dependency for the existing Bressloff, Rule,
  MacKay, Bolelli, or Nicks reports.
- Do not use BifurcationKit to strengthen public claims until its sidecar output
  is connected to the same report validation and source-target threshold policy
  as the Rust diagnostics.
- Do not copy upstream example code into this repo without preserving the MIT
  notice and making the adaptation boundary explicit.

## Model-Family Implications

| Repo track | BifurcationKit relevance | Action |
| --- | --- | --- |
| `bressloff_orientation_hypercolumn` | Possible future branch continuation and normal-form comparison for planforms. | Reference only until source-target Bressloff geometry/fidelity metrics are stronger. |
| `rule_flicker_ei` | Floquet multiplier conventions and periodic-orbit workflow vocabulary. | Use as a design reference when hardening Rule Floquet reports; no dependency now. |
| `mackay_localized_input` | Low relevance; fixed-point diagnostics are simpler than continuation. | No action. |
| `localized_time_periodic_input` | Future nonlinear periodic-state continuation could use periodic-orbit patterns. | Keep Bolelli pole-width and generated-width conventions stable first. |
| `spatial_forcing_orthogonal_response` | Reduced amplitude equations could be continued later if region boundaries need robust special-point tracking. | Finish source-derived Nicks residual fields before considering continuation. |
| `pinwheel_architecture_extension` | High relevance: the upstream FFT/GPU/PALC examples resemble the computational scale of deferred Veltz work. | Use as the likely sidecar foundation if pinwheel work is promoted. |
| `spatio_chromatic_field` | Highest relevance: this package is named in the source workflow and supports the continuation/GPU machinery that color hallucination work requires. | Defer, but treat BifurcationKit as the preferred reference if this branch starts. |

## Extraction Candidates

These are ideas to extract into our own Rust/report style, not copied code:

1. `ContinuationReportConfig`
   Fields: step size bounds, parameter bounds, max steps, Newton tolerances,
   eigensolver mode, bifurcation detection mode, bisection tolerance, and memory
   policy for eigenvectors.

2. `ContinuationBranchReport`
   Fields: branch id, source example id, parameter axis, point rows, stability
   counts, detected special points, residual histories, and termination reason.

3. `NewtonSolveReport`
   Fields: initial residual, residual history, convergence flag, Newton
   iterations, linear iterations, norm function, and rejected-step callback
   reason.

4. `FloquetReportPolicy`
   Fields: monodromy construction, multiplier sort mode, exponent conversion,
   tolerance for unit multiplier, and whether the computation is direct,
   matrix-free, shooting, collocation, or custom.

5. `SidecarProvenance`
   Fields: tool name, tool version, upstream commit, command/script path,
   dependency manifest, source model family, and generated output hash.

## Recommended Plan

1. Keep `references/BifurcationKit.jl/` ignored as a local reference clone.
2. Do not add Julia dependencies to the Rust crate.
3. Add only small report-schema ideas when a concrete model needs them.
4. For the next Rule/Floquet hardening pass, add a note or field documenting
   multiplier selection convention if a general eigensolver path is introduced.
5. For any future Veltz/Faugeras phase, prototype in a Julia sidecar first and
   emit JSON reports before considering Rust implementation.
6. Treat upstream examples as MIT-licensed references; preserve license notices
   if any code is adapted rather than reimplemented.

## Public Sources

- BifurcationKit.jl GitHub repository:
  https://github.com/bifurcationkit/BifurcationKit.jl
- BifurcationKit.jl documentation:
  https://bifurcationkit.github.io/BifurcationKitDocs.jl/stable/
- Veltz, R. "BifurcationKit.jl." HAL-Inria, 2020.
  https://hal.archives-ouvertes.fr/hal-02902346
- Faugeras, O. D., Song, A., and Veltz, R. "Spatial and Color Hallucinations in
  a Mathematical Model of Primary Visual Cortex." Comptes Rendus Mathematique
  360 (2022). https://doi.org/10.5802/crmath.289
