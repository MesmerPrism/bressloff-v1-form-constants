# Paper Figure Comparison Plan

This file tracks the public-facing comparison between named Bressloff-style
paper figures and the generated animations in this repository.

The public website should show generated animations next to a cited original
figure reference. Do not embed scans or crops of the original paper figures
unless reuse permission or a compatible license has been confirmed.

## Named Presets

| Preset | Paper target | Rendered target | Same-lattice branch readout | Current interpretation |
| --- | --- | --- | --- | --- |
| `fig16_odd` | Figure 16 stability example | Branch-selected rhombic | Rhombic | Passes current parity and branch-family checks. |
| `fig17_even` | Figure 17 widened-spread stability example | Branch-selected rhombic | Rhombic | Passes current parity and branch-family checks. |
| `fig31_square_even` | Figure 31 square/cobweb even planform | Cobweb/square | Roll/spiral | The explicit planform renders the target; the same-lattice stability readout selects a roll branch. |
| `fig32_square_odd` | Figure 32 square/cobweb odd planform | Cobweb/square | Roll/spiral | The explicit planform renders the target; the same-lattice stability readout selects a roll branch. |
| `fig33_rhombic_even` | Figure 33 rhombic even planform | Rhombic | Rhombic | Passes current rendered and branch-family checks. |
| `fig35_hex_even` | Figure 35 hexagonal phase variant | Hex-pi | Honeycomb | The family matches; the phase selection is still sensitive to the quadratic-term sign convention. |

## Presentation Rules

- Label generated panels as generated implementation output, not reproduced paper
  figures.
- Link original references through the DOI or PubMed/PMC landing page.
- Keep the implementation note short: preset, formula path, rendered target, and
  branch-readout result.
- Keep copyright-sensitive material out of the public repo until the license
  state is clear.

## Source Anchors

- Bressloff et al. 2001, detailed Royal Society treatment:
  <https://doi.org/10.1098/rstb.2000.0769>
- Bressloff et al. 2002, shorter Neural Computation summary:
  <https://doi.org/10.1162/089976602317250861>
