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
| `fig29_square_noncontoured` | Figure 29 square non-contoured planform | Cobweb/square scalar | Diagnostic only | Passes rendered scalar target and contour-mode checks; branch selection is not the figure target. |
| `fig29_roll_noncontoured` | Figure 29 roll non-contoured planform | Rings/roll scalar | Diagnostic only | Passes rendered scalar target and contour-mode checks; use angle controls for ray or spiral variants. |
| `fig30_rhombic_noncontoured` | Figure 30 rhombic non-contoured planform | Rhombic scalar | Diagnostic only | Passes rendered scalar target and contour-mode checks. |
| `fig30_hex_noncontoured` | Figure 30 hexagonal non-contoured planform | Honeycomb/hex scalar | Diagnostic only | Passes rendered scalar target and contour-mode checks. |
| `fig31_square_even` | Figure 31 square/cobweb even planform | Cobweb/square | Roll/spiral | The explicit planform renders the target; the same-lattice stability readout selects a roll branch. |
| `fig31_square_even_roll` | Figure 31 even roll subpanel | Rings/roll | Roll/spiral | Passes current rendered and branch-family checks. |
| `fig32_square_odd` | Figure 32 square/cobweb odd planform | Cobweb/square | Roll/spiral | The explicit planform renders the target; the same-lattice stability readout selects a roll branch. |
| `fig32_square_odd_roll` | Figure 32 odd roll subpanel | Rings/roll | Roll/spiral | Passes current rendered and branch-family checks. |
| `fig33_rhombic_even` | Figure 33 rhombic even planform | Rhombic | Rhombic | Passes current rendered and branch-family checks. |
| `fig33_rhombic_even_roll` | Figure 33 even rhombic-roll subpanel | Spiral/roll | Roll/spiral | Passes current rendered and branch-family checks. |
| `fig34_rhombic_odd` | Figure 34 rhombic odd planform | Rhombic | Rhombic | Passes current rendered and branch-family checks. |
| `fig34_rhombic_odd_roll` | Figure 34 odd rhombic-roll subpanel | Spiral/roll | Roll/spiral | Passes current rendered and branch-family checks. |
| `fig35_hex_even` | Figure 35 pi-hexagonal even phase variant | Hex-pi | Honeycomb | The family matches; the phase selection is still sensitive to the quadratic-term sign convention. |
| `fig35_hex_zero_even` | Figure 35 zero-hexagonal even phase variant | Honeycomb/0-hexagonal | Honeycomb | Passes current rendered and branch-family checks. |
| `fig36_triangle_odd` | Figure 36 triangular odd planform | Triangle | Honeycomb | The explicit triangular target renders; odd hexagonal stability needs higher-order calibration. |
| `fig36_hex_zero_odd` | Figure 36 zero-hexagonal odd planform | Honeycomb/0-hexagonal | Honeycomb | Passes current rendered and branch-family checks. |
| `fig5_roll_cortical` | 2002 Figure 5 cortical roll | Rings/roll in cortical view | Roll/spiral | Convenience alias for the source cortical planform. |
| `fig5_hex_cortical` | 2002 Figure 5 cortical hexagonal planform | Hex-pi in cortical view | Honeycomb | Convenience alias; phase selection remains a calibration target. |
| `fig5_honeycomb_cortical` | 2002 Figure 5 cortical honeycomb planform | Honeycomb in cortical view | Honeycomb | Convenience alias for the source cortical planform. |
| `fig5_square_cortical` | 2002 Figure 5 cortical square planform | Cobweb/square in cortical view | Roll/spiral | Convenience alias; rendered square target is explicit while branch readout selects roll. |
| `fig6_visual_field_planforms` | 2002 Figure 6 visual-field planforms | Cobweb/square representative | Roll/spiral | Representative alias for the inverse-map visual-field examples. |
| `fig7_lattice_tunnel` | 2002 Figure 7 lattice tunnel simulation | Rings/roll tunnel | Roll/spiral | Representative alias for the captioned even-roll lattice-tunnel target. |

## Stability Reports

The v4 calibration report also includes non-rendering checks for
`fig37_even_coefficients`, `fig38_even_hex_bifurcation`,
`fig39_odd_coefficients`, `fig40_odd_hex_bifurcation`, and
`rhombic_stability_angle`. Four currently pass; the Fig 40 odd triangular
selection remains a review target because the paper's odd-hexagonal discussion
depends on higher-order terms not yet fitted in this normalized selector.

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
