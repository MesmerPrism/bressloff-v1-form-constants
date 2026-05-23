# Notice

This repository is derived from [`karacsm/V1-sim`](https://github.com/karacsm/V1-sim),
an MIT-licensed project by Márton Karácsony for simulating V1 activity and
visualizing it through the retino-cortical map.

## Upstream Attribution

- Original repository: <https://github.com/karacsm/V1-sim>
- Original author: Márton Karácsony
- Original license: MIT License
- Original scope: Jupyter notebook implementation of a numerical V1 simulation
  based on Bressloff et al.'s geometric visual hallucination model.

The upstream notebook `V1-sim.ipynb` is retained for provenance and comparison.
The Python helpers in this repository are part of the exploratory path from the
upstream notebook toward a browser-playable payload format.

## Additions In This Repository

The following additions are by Till Holzapfel and are also distributed under the
MIT License:

- Rust payload generator and local viewer server.
- Browser viewer controls for planforms, stability scans, and contour overlays.
- Bressloff model notes and fidelity tracker.
- Paper-oriented planform presets and interactive tuning surface.

## Scientific Sources

The model implementation and documentation refer primarily to:

- Bressloff, Cowan, Golubitsky, Thomas, and Wiener, "What Geometric Visual
  Hallucinations Tell Us About the Visual Cortex," *Neural Computation* (2002).
- Ermentrout and Cowan, "A Mathematical Theory of Visual Hallucination
  Patterns," *Biological Cybernetics* (1979).
- Rule, Stoffregen, and Ermentrout, "A Model for the Origin and Properties of
  Flicker-Induced Geometric Phosphenes," *PLOS Computational Biology* (2011).

These papers are cited for scientific context. Their text, figures, and
copyrighted publisher content are not redistributed here.
