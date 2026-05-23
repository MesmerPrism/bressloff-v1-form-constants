# Bressloff Model Notes

Primary reference: Bressloff, Cowan, Golubitsky, Thomas, and Wiener,
"Geometric visual hallucinations, Euclidean symmetry and the functional
architecture of striate cortex" / "What geometric visual hallucinations tell us
about the visual cortex".

## Implemented Mapping

- Retino-cortical map: the viewer uses the complex-log style inverse map already
  used by the original notebook, with the orientation double-map relation
  `phi_R = phi + theta_R` for contour glyphs.
- Lateral Fourier-Bessel kernel: `lateral_weight_coeff` implements the
  difference-of-Gaussians form for `W_hat_n(q)` with the modified Bessel
  function term from equation 3.19.
- Lateral spread: `lateral_spread_deg` multiplies nonzero harmonics by
  `sin(2 n theta0) / (2 n theta0)`, matching the spread example in equation 3.20.
- Linear stability: `branch_growth` evaluates the perturbative `G+` and `G-`
  expressions through second order in the lateral interaction parameter.
- Orientation eigenfunctions: `orientation_eigen_details` exports the even/odd
  Fourier coefficients from the perturbation formulas around equations 3.14-3.16.
- Cubic amplitude equations: `branch_selection_for` evaluates `Gamma3(theta)` and
  the even hexagonal `Gamma2` integral numerically from the current eigenfunction,
  then reports roll, square/cobweb, rhombic, and hexagonal branches.
- Paper presets: the Rust model exposes starting points for the marginal-stability
  examples and the double-map planform figures. These set the branch family,
  parity, lateral spread, and scan resolution; they are not final calibrated
  reproductions.
- Orientation-channel export: when requested, payloads include a compressed
  `frame,row,col,orientation` tensor with the same angular channels used by the
  model or analytic planform.
- Calibration reports: each named paper preset records expected parity/family
  checks against the rendered planform and the current branch selector.

## Current Normalizations

- The scan compares the dimensionless growth functions `G+` and `G-`; it does
  not yet convert them into an absolute biological threshold `mu_c`.
- The amplitude selector uses `lambda = max(G(q_c), 0)` as a normalized distance
  past bifurcation. This is useful for relative branch inspection, but not yet a
  calibrated biological excitability axis.
- The nonlinear constants multiplying `Gamma2` and `Gamma3` are normalized to one
  for visualization. The signs and relative values come from the eigenfunction,
  but absolute amplitudes should not be interpreted physiologically yet.

## Next Fidelity Targets

- Calibrate named presets against digitized paper geometry for figures 16, 17,
  and 31-36.
- Add a compact basis export so orientation-channel payloads can be smaller than
  the full tensor.
- Add a calibrated `mu_c` readout from `alpha / (sigma1 G(q_c))` once the
  operating point of the sigmoid is made explicit.
