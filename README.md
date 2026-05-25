# Bressloff V1 Form Constants Lab

Interactive visualizations for the geometric hallucination model described by
Bressloff, Cowan, Golubitsky, Thomas, and Wiener. The lab focuses on making the
model's cortical planforms, retino-cortical mapping, orientation contours, and
linear-stability / branch-selection machinery inspectable in a browser.

This repository is a public, MIT-licensed derivative of
[`karacsm/V1-sim`](https://github.com/karacsm/V1-sim), which implemented a
notebook simulation of V1 activity and retino-cortical visualization. The Rust
viewer added here is meant for fast parameter exploration and paper-figure
calibration, not for claiming a calibrated flicker-frequency prediction.

## What It Shows

- Retino-cortical mapping from cortical activity to visual-field coordinates.
- Neural-field dynamics based on equation 2.1 from Bressloff et al.
- Direct analytic planforms for rings, rays, spirals, cobweb/square, rhombic,
  honeycomb, pi-hexagonal, and triangular hexagonal branches.
- Orientation contour overlays using the Bressloff double-map relation
  `phi_R = phi + theta_R`.
- Kernel controls for local and lateral interaction widths and inhibition.
- Linear stability scan over dimensionless wavenumber `q`, including even/odd
  branch readout.
- Cubic amplitude-equation branch readout for roll, square/cobweb, rhombic, and
  hexagonal families.
- First-class paper-oriented presets for the figure 16/17 stability examples,
  the figure 29/30 single-map non-contoured examples, the figure 31-36
  double-map contoured planform families including roll subpanels, and the 2002
  figure 5-7 convenience examples.
- Optional orientation-channel export payloads for planform or dynamics runs.
- JSON calibration reports that compare each named preset's expected target with
  the rendered planform, same-lattice branch readout, and source metadata.
- A separate Rule-Stoffregen-Ermentrout 2011 scalar E/I flicker generator with
  qualitative high-frequency stripe and low-frequency hexagonal presets.
- A separate driven-input neural-field registry plus first-pass generated
  MacKay localized-input diagnostics. These stay distinct from both the
  Bressloff orientation-hypercolumn and Rule flicker model families.

## What It Does Not Claim

- It is not a calibrated simulator of specific strobe or flicker frequencies.
- It does not reproduce an entire psychedelic or altered state.
- It does not make therapeutic, clinical, or safety claims.
- It should not be used as a substitute for participant-report or empirical
  reconstruction tools.

The honest use is exploratory: tune the model, compare it with the paper's
descriptions, and make the mathematical claim visually checkable.

## Quick Start

Requirements:

- Rust toolchain with Cargo.
- A modern browser.
- Python is optional and only needed for the legacy helper scripts.

Build the fast Rust viewer:

```powershell
cargo build --manifest-path rust-v1-sim\Cargo.toml --release
```

Serve the interactive viewer:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe serve --port 8892 --host 127.0.0.1 --root .
```

Open:

```text
http://127.0.0.1:8892/viewer/index.html
```

The long-form generated-model walkthrough lives at:

```text
http://127.0.0.1:8892/viewer/deep-dive.html
```

Export a static payload:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe export --generator planform --pattern cobweb --n 96 --m 24 --frames 120 --t 18 --out viewer\frames.json
```

Export one named paper preset with the full orientation-channel tensor:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe export --paper-preset fig31_square_even --export-orientations --out reports\fig31-square-even.json
```

Generate a side-by-side JSON calibration report for the named paper presets:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe calibrate --out reports\paper-calibration.json
```

Generate public-safe Bressloff figure-geometry still targets for Figures 29-36:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe bressloff-geometry --out reports\figure-targets\bressloff-generated-stills.json
```

Generate the first Rule 2011 qualitative regime report:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-report --out reports\rule-2011-regimes.json
```

Generate the first simulator-backed Rule frequency/amplitude sweep report:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-sweep --out reports\rule-2011-sweep.json
```

Generate the denser website/analysis map:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-sweep --preset-grid dense --out reports\rule-2011-sweep-dense.json
```

Generate the first dedicated Rule Floquet calibration surface:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-floquet --out reports\rule-2011-floquet.json --curve-refine-steps 64 --curve-refine-tolerance 0.000001
```

Use `--mode-min`, `--mode-max`, and `--mode-steps` to refine the Figure
8-style wave-number scan. The default mode grid covers 0.5-4.0 cycles and is
dense enough to expose first-pass +1 and -1 sign-change crossings. The report
also refines beta-axis crossings into source-axis `boundary_curves` for Figure
8-style wave-number versus forcing-period calibration. The default
`rule_fig8_source_like` parameter set names the current source-like constants
from the paper extraction; explicit CLI flags still override individual values.
The Figure 8 source beta normalization is now an explicit zero-offset model
decision: `source_beta = 0.42868451880191133 * model_beta_cycles` by default.
Use `--figure8-beta-scale` to change that domain scale, or use
`--source-beta-modes` / `--source-beta-min` / `--source-beta-max` /
`--source-beta-steps` to specify the scan on the source Figure 8 beta axis.

Generate the first Rule Figure 8 fit-search report:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe rule-fit --out reports\rule-2011-fit-search.json --max-trials 25 --curve-refine-steps 48 --curve-refine-tolerance 0.000001
```

This keeps `rule_fig8_source_like` unchanged and records one-parameter
calibration trials against the digitized Figure 8C source curves. The objective
uses the configured domain-normalized beta residual while preserving raw,
scale-only, and affine beta-axis mappings as diagnostics.

Generate the public-safe driven neural-field example registry:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe driven-registry --out reports\driven-neural-fields-registry.json
```

Generate the first MacKay localized-input diagnostic report:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe mackay-report --out reports\mackay-localized-input.json --n 112 --iterations 48
```

The MacKay report contains generated fields and numeric diagnostics only. It is
a first-pass diagnostic, not a source-figure reproduction claim.

Generate Bressloff figure-geometry stills and public-safe comparison slots:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe bressloff-geometry --out reports\figure-targets\bressloff-generated-stills.json
```

Original paper scans/crops stay under `private/`. Use
`tools\extract_bressloff_source_profiles.py` to derive local numeric masks and
profiles, then re-run `bressloff-geometry` with `--source-profile-dir` to fill
radial, angular, edge-overlap, and lattice-angle comparison metrics.

## Viewer Workflow

Use `Generator -> Planform` for direct Bressloff-style pattern families. This is
the fastest way to inspect the retino-cortical map and contour overlays.

Use `Pattern -> Auto branch` to let the current linear-stability scan and branch
selector choose a rendered planform family. The `Stability` controls set the
`q` scan range and resolution. The `Kernel` controls expose the local and
lateral interaction parameters, including the lateral angular spread `theta0`
used by Bressloff's widened lateral-connection example.

Use `Generator -> Dynamics` to run the neural-field solver path. Warmup trimming
is enabled by default for playback so the low-contrast onset transient does not
consume half the animation.

Use `Generator -> Rule flicker E/I` for the Rule, Stoffregen, and Ermentrout
2011 scalar E/I model family. Rule presets are separate from the Bressloff paper
preset list and start with qualitative seeded regimes: high-frequency
period-doubled stripes and low-frequency one-to-one hexagons. These reuse the
viewer and retinocortical display surface, but not Bressloff orientation
contour glyphs or amplitude-equation branch selection.

Use `Bressloff preset` to load a named Bressloff figure starting point. The catalog
currently covers 24 targets: figure 16/17 stability cases, figure 29/30
non-contoured single-map planforms, figure 31-36 contoured double-map planforms
including roll subpanels, and 2002 figure 5-7 aliases. Use `Planform mode ->
Non-contoured` for scalar activity-threshold images where contour orientation is
not part of the paper target. The calibration readout reports whether the
current parity, contour mode, rendered planform, and branch selector agree with
the preset's expected target. Enable `Export orientation channels` only when you
need the heavier `[frame,row,col,orientation]` payload for downstream analysis.

## Project Layout

```text
rust-v1-sim/          Rust server, payload generator, and model implementation
  src/models/         Bressloff, Rule, and driven-field model-family modules
  src/numeric/        Shared numeric helpers for convolution and metrics
viewer/               Browser viewer, controls, and model notes
docs/                 Public implementation roadmaps and future work plans
tools/                Legacy Python helpers for exporting/serving frames
v1_model.py           Python model code retained from the exploratory path
v1_frames.py          Python payload exporter retained for comparison
V1-sim.ipynb          Upstream notebook lineage from karacsm/V1-sim
```

## Model Notes

- [`viewer/BRESSLOFF_MODEL_NOTES.md`](viewer/BRESSLOFF_MODEL_NOTES.md) tracks
  the implemented formulas, normalizations, and remaining fidelity targets.
- [`viewer/BRESSLOFF_FIDELITY.md`](viewer/BRESSLOFF_FIDELITY.md) is the current
  fidelity checklist.
- [`viewer/PAPER_FIGURE_COMPARISON.md`](viewer/PAPER_FIGURE_COMPARISON.md)
  tracks the named paper figure presets and public comparison rules.
- [`docs/BRESSLOFF_FUTURE_IMPLEMENTATION_PLAN.md`](docs/BRESSLOFF_FUTURE_IMPLEMENTATION_PLAN.md)
  turns the remaining Bressloff calibration gaps into concrete implementation
  workstreams.
- [`docs/RULE_2011_IMPLEMENTATION_STATUS.md`](docs/RULE_2011_IMPLEMENTATION_STATUS.md)
  tracks the separate Rule flicker E/I implementation and deferred Floquet/sweep
  work.
- [`docs/DRIVEN_NEURAL_FIELDS_IMPLEMENTATION_PLAN.md`](docs/DRIVEN_NEURAL_FIELDS_IMPLEMENTATION_PLAN.md)
  audits the post-Rule driven-input neural-field papers and turns them into a
  public-safe implementation plan for localized input, time-periodic input,
  MacKay/Billock-Tsou-style targets, and deferred architecture/color extensions.
- [`docs/WEB_HOSTING_PLAN.md`](docs/WEB_HOSTING_PLAN.md) describes the
  server-backed container deployment path for a public interactive version.
- [`viewer/README.md`](viewer/README.md) documents the local browser viewer.
- [`viewer/deep-dive.html`](viewer/deep-dive.html) is the public, notebook-style
  Bressloff and Rule modeling article backed by the same generated animations as
  the interactive lab.

## Public Article

A public explanation with Bressloff-rendered animation exports is available at:

<https://mesmerprism.com/projects/bressloff-v1-form-constants.html>

Related work such as Brain Candy can use this repository as a model reference,
but this repository is not a Brain Candy codebase. Its direct purpose is to make
Bressloff's V1 form-constant model interactive and inspectable.

## Sources

- Bressloff, P. C., J. D. Cowan, M. Golubitsky, P. J. Thomas, and M. C.
  Wiener. "Geometric Visual Hallucinations, Euclidean Symmetry and the
  Functional Architecture of Striate Cortex." *Philosophical Transactions of
  the Royal Society B* 356, no. 1407 (2001): 299-330.
  <https://doi.org/10.1098/rstb.2000.0769>
- Bressloff, P. C., J. D. Cowan, M. Golubitsky, P. J. Thomas, and M. C. Wiener.
  "What Geometric Visual Hallucinations Tell Us About the Visual Cortex."
  *Neural Computation* 14, no. 3 (2002): 473-491.
  <https://doi.org/10.1162/089976602317250861>
  Public PDF mirror: <https://gwern.net/doc/psychology/vision/2002-bressloff.pdf>
- Bressloff, P. C. "Spatiotemporal Dynamics of Continuum Neural Fields."
  *Journal of Physics A: Mathematical and Theoretical* 45, no. 3 (2012):
  033001. <https://doi.org/10.1088/1751-8113/45/3/033001>
  Note: search indexes an old Utah PDF path for this review, but direct requests
  currently return 404, so the DOI/IOP landing page is the stable public link.
- Ermentrout, G. B., and J. D. Cowan. "A Mathematical Theory of Visual
  Hallucination Patterns." *Biological Cybernetics* 34 (1979): 137-150.
  <https://doi.org/10.1007/BF00336965>
- Rule, M., M. Stoffregen, and B. Ermentrout. "A Model for the Origin and
  Properties of Flicker-Induced Geometric Phosphenes." *PLOS Computational
  Biology* 7, no. 9 (2011). <https://doi.org/10.1371/journal.pcbi.1002158>
- Amaya, I. A., N. Behrens, D. J. Schwartzman, T. Hewitt, and T. T. Schmidt.
  "Effect of Frequency and Rhythmicity on Flicker Light-Induced Hallucinatory
  Phenomena." *PLOS ONE* 18, no. 4 (2023).
  <https://doi.org/10.1371/journal.pone.0284271>
- Hewitt, T., I. Amaya, R. Beaute, A. K. Seth, T. T. Schmidt, and D. J.
  Schwartzman. "Stroboscopically Induced Visual Hallucinations: Historical,
  Phenomenological, and Neurobiological Perspectives." *Neuroscience of
  Consciousness* (2025). <https://doi.org/10.1093/nc/niaf020>
- Hewitt, T., E. J. Grove, A. Seth, and D. J. Schwartzman. "Image Recreation
  Methods Enable Quantitative Characterization of Geometric Visual
  Hallucinations." PsyArXiv preprint (2026).
  <https://doi.org/10.31234/osf.io/2gtsy_v1>
  Interactive visualization: <https://imagerecreationdataviz.netlify.app/>
- CountYourCulture. "Form Constants and the Visual Cortex."
  <https://isomerdesign.com/countyourculture/2011/03/13/form-constants-visual-cortex/>
- CountYourCulture. "Form Constant Visualization - Type I."
  <https://isomerdesign.com/countyourculture/2011/03/16/form-constant-visualization-type-1/>
- Qualia Research Institute. "Oscilleditor Reference Manual."
  <https://qri.org/oscilleditor/doc/reference-manual>
- karacsm. "V1-sim." GitHub repository.
  <https://github.com/karacsm/V1-sim>

## License And Attribution

MIT licensed. See [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).

The original `V1-sim` notebook and baseline project were created by Márton
Karácsony and released under the MIT License. This repository retains that
license and attribution. Additions in the Rust viewer, browser UI, and
Bressloff fidelity notes are also distributed under MIT.
