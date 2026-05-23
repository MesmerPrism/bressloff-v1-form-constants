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
  honeycomb, and hexagonal branches.
- Orientation contour overlays using the Bressloff double-map relation
  `phi_R = phi + theta_R`.
- Kernel controls for local and lateral interaction widths and inhibition.
- Linear stability scan over dimensionless wavenumber `q`, including even/odd
  branch readout.
- Cubic amplitude-equation branch readout for roll, square/cobweb, rhombic, and
  hexagonal families.
- Paper-oriented starting presets for several Bressloff figure families.

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

Export a static payload:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe export --generator planform --pattern cobweb --n 96 --m 24 --frames 120 --t 18 --out viewer\frames.json
```

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

## Project Layout

```text
rust-v1-sim/          Rust server, payload generator, and model implementation
viewer/               Browser viewer, controls, and model notes
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
- [`viewer/README.md`](viewer/README.md) documents the local browser viewer.

## Public Article

A public explanation with Bressloff-rendered animation exports is available at:

<https://mesmerprism.com/projects/bressloff-v1-form-constants.html>

Related work such as Brain Candy can use this repository as a model reference,
but this repository is not a Brain Candy codebase. Its direct purpose is to make
Bressloff's V1 form-constant model interactive and inspectable.

## Sources

- Bressloff, P. C., J. D. Cowan, M. Golubitsky, P. J. Thomas, and M. C. Wiener.
  "What Geometric Visual Hallucinations Tell Us About the Visual Cortex."
  *Neural Computation* 14, no. 3 (2002): 473-491.
  <https://doi.org/10.1162/089976602317250861>
- Ermentrout, G. B., and J. D. Cowan. "A Mathematical Theory of Visual
  Hallucination Patterns." *Biological Cybernetics* 34 (1979): 137-150.
  <https://doi.org/10.1007/BF00336965>
- Rule, M., M. Stoffregen, and B. Ermentrout. "A Model for the Origin and
  Properties of Flicker-Induced Geometric Phosphenes." *PLOS Computational
  Biology* 7, no. 9 (2011). <https://doi.org/10.1371/journal.pcbi.1002158>
- karacsm. "V1-sim." GitHub repository.
  <https://github.com/karacsm/V1-sim>

## License And Attribution

MIT licensed. See [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).

The original `V1-sim` notebook and baseline project were created by Márton
Karácsony and released under the MIT License. This repository retains that
license and attribution. Additions in the Rust viewer, browser UI, and
Bressloff fidelity notes are also distributed under MIT.
