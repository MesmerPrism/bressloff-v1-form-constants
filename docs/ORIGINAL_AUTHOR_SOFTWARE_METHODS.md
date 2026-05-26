# Original Author Software Methods

Updated: 2026-05-26

This public-safe note records what the source papers and public source records
say about software, numerical methods, and figure-generation workflows. It is
for implementation planning and public methods transparency. It is not a claim
that this repository has the original authors' code, exact figure scripts, or a
calibrated reproduction.

Private PDFs, private page-level notes, figure crops, and local extraction
artifacts remain under ignored `private/papers/`. This note keeps only
source-safe summaries and public citations.

## Summary

- Bressloff's form-constant papers are primarily analytic and symmetry-driven;
  no named figure-generation software or article-linked source repository was
  identified in the public paper record.
- Rule et al. explicitly use AUTO through XPPAUT for continuation and
  stability work, plus custom two-dimensional simulation code attributed in the
  paper record.
- Nicks et al. explicitly name MATLAB, FFT-based pseudo-spectral convolution,
  `ode45`, and XPPAUT for the amplitude-equation stability checks.
- Tamekue, Prandi, and Chitour explicitly use Julia for the MacKay-effect
  visualization/solver workflow and fixed-point iteration for stationary
  states.
- Bolelli and Prandi explicitly use Mathematica for principal-pole
  computations and Julia for nonlinear neural-field simulations.
- Veltz, Chossat, and Faugeras explicitly use high-performance continuation and
  simulation tools: Trilinos, FFTW, PETSc, `petsc4py`, Krylov methods, and BDF
  integration.
- Faugeras, Song, and Veltz explicitly use Julia, BifurcationKit.jl,
  KrylovKit.jl, CUDA.jl, GPU FFTs, and pseudo-arclength continuation.
- Carroll and Bressloff explicitly use Mathematica for symbolic algebraic
  checks in the appendix, while the public source record does not name a
  separate simulation language for the numerical figures.
- Sarti and Citti describe numerical mean-field, affinity-matrix, eigenvector,
  and MCMC-style estimation methods; no named software package or language was
  identified in the public paper record.

## Source Ledger

| Source | Evidence class | Named software or method | Public code status | Repo implication |
| --- | --- | --- | --- | --- |
| Bressloff et al. 2001/2002 | Methods-only in public record | Analytic retinocortical map, Euclidean symmetry, orientation-hypercolumn planforms, stability/bifurcation analysis | No article-linked figure-code repository identified | Keep current generated planform and report code as an independent implementation. Do not imply original-code reuse. |
| Bressloff 2012 review | Review/survey | Continuum neural-field theory and pattern-formation methods | Not a primary figure-code source for this repo | Use as model lineage context, not as implementation provenance. |
| Rule, Stoffregen, and Ermentrout 2011 | Explicit software/methods | AUTO through XPPAUT, monodromy/Floquet stability, two-dimensional periodic-grid simulations | No public source repository identified for the figure code | Our Rule module should keep continuation/Floquet language explicit and treat generated sweep maps as first-pass diagnostics until source-target reports support stronger claims. |
| Nicks et al. 2021 | Explicit software/methods | MATLAB simulations, FFT/inverse-FFT pseudo-spectral convolution, `ode45`, XPPAUT amplitude-equation stability checks | SIAM supplementary material exposes movies, not reusable figure code | Our Nicks implementation should keep reduced amplitude-equation diagnostics separate from deferred full-field MATLAB-style simulations. |
| Tamekue, Prandi, and Chitour 2024 | Explicit software/methods | Julia visualization/solver workflow, fixed-point iteration for stationary Amari fields, dense rectangular grids | Related work points to a Julia toolbox, but no standalone article figure-code repository was identified | Our MacKay implementation can remain a compact Rust fixed-point diagnostic while recording the grid and solver assumptions. |
| Bolelli and Prandi 2025 | Explicit software/methods | Mathematica for principal-pole computations, Julia for nonlinear mean-field simulations | Article states raw data are available from the authors; no public figure-code repository identified | Keep Mathematica-style pole formulas as source-target diagnostics, but keep Rust generated widths separate until conventions match. |
| Veltz, Chossat, and Faugeras 2015 | Explicit software/methods | Trilinos, FFTW, Newton-Krylov/GMRES, Arnoldi eigensolver, PETSc, `petsc4py`, deflated GMRES, BDF integration, large meshes | The article names software stacks; public reusable figure scripts were not identified | Treat pinwheel architecture as deferred high-cost work. It likely needs a different numerical layer before implementation. |
| Faugeras, Song, and Veltz 2022 | Explicit software/methods | Julia 1.4.2, BifurcationKit.jl, KrylovKit.jl, pseudo-arclength continuation, Newton-Krylov/GMRES, Arnoldi, CUDA.jl GPU FFTs | BifurcationKit.jl is public; exact article figure scripts are not part of this repo | Treat color hallucinations and localized snaking as architecture-level deferred work, not as near-term Rust report additions. |
| Carroll and Bressloff 2018 | Partial explicit software/methods | Mathematica for algebraic computations and simplifications; symmetry/bifurcation analysis | No named simulation-code repository identified | Keep contrast-gradient work as an adjacent perceptual-function track unless the repo expands beyond hallucination-style diagnostics. |
| Sarti and Citti 2015 | Methods-only in public record | Discretized mean-field equation, affinity matrix, eigenvectors, Fokker-Planck connectivity, MCMC kernel/fundamental-solution estimation | No named public code package identified | Treat SE(2) perceptual grouping as a separate generated-stimulus/eigenmode report track. |

## Implementation Implications

1. Keep the Rust codebase modular by model family rather than trying to mimic
   every original software stack.
2. Use XPPAUT/AUTO, MATLAB, Julia, Mathematica, PETSc, Trilinos, and
   BifurcationKit references as provenance for what the authors computed, not
   as dependencies unless the current phase requires them.
3. Keep Bolelli pole-width comparisons in equation-derived source-target
   language until the generated-width convention is explicitly comparable.
4. Keep Nicks Figure 8 and orthogonal-response outputs as reduced
   amplitude-equation diagnostics until full-field or source-panel residuals
   exist.
5. Defer Veltz/Faugeras-style continuation, GPU, color, pinwheel, and large
   architecture work until the simpler driven-input report layers are stable.
6. Public website copy should say "named software/methods in the original
   papers" and "generated diagnostics here," not "reproduced with the original
   code."

## Public Claim Boundary

The source-software record helps explain why this project uses generated reports
and staged validation instead of publishing paper-figure copies. It does not
change the calibration status of any report:

- Bressloff and Rule outputs remain generated model diagnostics with explicit
  comparison layers.
- MacKay remains a generated fixed-point diagnostic.
- Bolelli remains a source-target diagnostic for the equation-level pole-width
  convention, not a calibrated visual reproduction.
- Nicks remains a source-target diagnostic for reduced equations and region
  boundaries, not a full-field or paper-panel reproduction.

## Public Sources

- Bressloff, P. C., Cowan, J. D., Golubitsky, M., Thomas, P. J., and Wiener,
  M. C. "Geometric Visual Hallucinations, Euclidean Symmetry and the Functional
  Architecture of Striate Cortex." Philosophical Transactions of the Royal
  Society B 356 (2001). https://doi.org/10.1098/rstb.2000.0769
- Bressloff, P. C., Cowan, J. D., Golubitsky, M., Thomas, P. J., and Wiener,
  M. C. "What Geometric Visual Hallucinations Tell Us About the Visual Cortex."
  Neural Computation 14, no. 3 (2002). https://doi.org/10.1162/089976602317250861
- Bressloff, P. C. "Spatiotemporal Dynamics of Continuum Neural Fields."
  Journal of Physics A: Mathematical and Theoretical 45, no. 3 (2012).
  https://doi.org/10.1088/1751-8113/45/3/033001
- Rule, M., Stoffregen, M., and Ermentrout, B. "A Model for the Origin and
  Properties of Flicker-Induced Geometric Phosphenes." PLOS Computational
  Biology 7, no. 9 (2011). https://doi.org/10.1371/journal.pcbi.1002158
- Nicks, R., Cocks, A., Avitabile, D., Johnston, A., and Coombes, S.
  "Understanding Sensory Induced Hallucinations: From Neural Fields to
  Amplitude Equations." SIAM Journal on Applied Dynamical Systems 20, no. 4
  (2021). https://doi.org/10.1137/20M1366885
- Tamekue, C., Prandi, D., and Chitour, Y. "A Mathematical Model of the Visual
  MacKay Effect." SIAM Journal on Applied Dynamical Systems 23, no. 3 (2024).
  https://doi.org/10.1137/23M1616686
- Bolelli, M. V., and Prandi, D. "Neural Field Equations with Time-Periodic
  External Inputs and Some Applications to Visual Processing." Journal of
  Mathematical Imaging and Vision 67 (2025).
  https://doi.org/10.1007/s10851-025-01257-7
- Veltz, R., Chossat, P., and Faugeras, O. "On the Effects on Cortical
  Spontaneous Activity of the Symmetries of the Network of Pinwheels in Visual
  Area V1." Journal of Mathematical Neuroscience 5 (2015).
  https://doi.org/10.1186/s13408-015-0023-8
- Faugeras, O. D., Song, A., and Veltz, R. "Spatial and Color Hallucinations in
  a Mathematical Model of Primary Visual Cortex." Comptes Rendus Mathematique
  360 (2022). https://doi.org/10.5802/crmath.289
- BifurcationKit contributors. "BifurcationKit.jl." GitHub.
  https://github.com/bifurcationkit/BifurcationKit.jl
  Local project audit:
  [`docs/BIFURCATIONKIT_REFERENCE_AUDIT.md`](BIFURCATIONKIT_REFERENCE_AUDIT.md).
- Carroll, S. R., and Bressloff, P. C. "Symmetric Bifurcations in a Neural
  Field Model for Encoding the Direction of Spatial Contrast Gradients." SIAM
  Journal on Applied Dynamical Systems 17, no. 1 (2018).
  https://doi.org/10.1137/16M1076125
- Sarti, A., and Citti, G. "The Constitution of Visual Perceptual Units in the
  Functional Architecture of V1." Journal of Computational Neuroscience 38,
  no. 2 (2015). https://doi.org/10.1007/s10827-014-0540-6
