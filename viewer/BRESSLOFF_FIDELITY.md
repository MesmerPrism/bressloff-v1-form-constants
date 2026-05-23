# Bressloff Fidelity Tracker

Goal: converge this viewer toward a traceable interactive implementation of the
Bressloff, Cowan, Golubitsky, Thomas, and Wiener geometric hallucination model.

## Implemented

- Retino-cortical map from the notebook/project constants.
- Rust server and generator for fast interactive iteration.
- Neural-field dynamics path derived from the original MIT-licensed
  `karacsm/V1-sim` notebook implementation.
- Analytic cortical planforms for rings, rays, spirals, cobweb/square,
  honeycomb, rhombic, and hex-pi families.
- Full angular retinal coverage for analytic planforms.
- Double-map contour overlay using `phi_R = phi + theta_R`.
- Orientation-resolved planform payloads:
  - wave-mode list
  - dimensionless `q`
  - even/odd parity switch
  - perturbative orientation-eigenfunction coefficients
  - hypercolumn scale control
- Exposed planform kernel constants:
  - local narrow/wide angular widths and inhibition
  - lateral narrow/wide widths and inhibition
  - lateral angular spread `theta0` used by Bressloff's widened lateral
    connection example
- Linear stability-style scan over `q` with even/odd branch readout.
- Linear scan now uses the second-order perturbative `G+` and `G-` expressions
  implied by equations 3.13 through 3.16.
- Cubic amplitude-equation branch selector:
  - computes `Gamma3(theta)` by integrating the current orientation eigenfunction
  - computes the even hexagonal `Gamma2` term
  - reports roll, square/cobweb, rhombic, and hexagonal candidates with amplitude
    and stability flags
  - separates the lattice-local selection used for named figure presets from the
    global cross-lattice score comparison
- `Auto branch` planform preset that uses the amplitude-equation selector to
  choose the rendered planform family.
- `Auto branch` now also adopts the critical parity selected by the stability
  scan.
- First-class paper-oriented presets for figures 16, 17, 31, 32, 33, and 35,
  exposed through the viewer, `/api/defaults`, CLI export, and calibration
  report paths.
- Optional orientation-channel payload export with
  `frame,row,col,orientation` ordering.
- `calibrate` command that writes a JSON side-by-side report comparing each
  named preset's expected family with the rendered planform, same-lattice branch
  selection, and global score winner.

## Approximate

- The stability scan uses the Fourier/Bessel kernel family described in the
  paper, but still needs direct paper-figure calibration.
- The scalar frame for planforms is currently the strongest sampled orientation
  response at each cortical location, not a direct display of every orientation
  channel.
- Branch selection now follows the cubic amplitude-equation coefficient structure,
  but the bifurcation distance and nonlinear sigmoid constants are normalized
  rather than fit to a biological parameter set.
- The square/cobweb paper presets intentionally render their target planforms.
  The same-lattice branch readout currently selects a roll/spiral branch, matching
  the known instability issue better than the earlier honeycomb-like global
  comparison.
- The hex-pi preset renders the requested phase variant, while the current
  quadratic-term sign selects the honeycomb phase partner.

## Still Missing

- Calibration of the paper presets against exact digitized figure geometry and
  parameter values.
- Efficient compact-basis export for orientation channels when the full tensor is
  too large.
- Dynamics-to-contour overlay from simulated orientation channels.
- Public side-by-side figure comparison panels with generated animations and
  original-paper figure links. Direct reproduction of paper figures needs
  permission or a confirmed compatible license.
