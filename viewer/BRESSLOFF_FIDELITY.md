# Bressloff Fidelity Tracker

Goal: converge this viewer toward a traceable interactive implementation of the
Bressloff, Cowan, Golubitsky, Thomas, and Wiener geometric hallucination model.

## Implemented

- Retino-cortical map from the notebook/project constants.
- Rust server and generator for fast interactive iteration.
- Neural-field dynamics path derived from the original MIT-licensed
  `karacsm/V1-sim` notebook implementation.
- Analytic cortical planforms for rings, rays, spirals, cobweb/square,
  honeycomb, rhombic, hex-pi, and triangular odd-hex families.
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
- First-class paper-oriented presets for figures 16, 17, and 31-36,
  exposed through the viewer, `/api/defaults`, CLI export, and calibration
  report paths. Figure 36a uses a triangular sine-combination basis represented
  by per-mode phase offsets.
- Non-contoured scalar planform mode for the Figure 29 and Figure 30 single-map
  examples. This path renders activity directly instead of selecting the
  strongest orientation channel.
- Roll subpanel presets for Figures 31-34 and 2002 convenience aliases for
  Figure 5 cortical planforms, Figure 6 visual-field planforms, and the Figure 7
  lattice-tunnel simulation target.
- Optional orientation-channel payload export with
  `frame,row,col,orientation` ordering.
- `calibrate` command that writes a v4 JSON side-by-side report comparing all 24
  named presets against rendered contour mode, parity, rendered planform,
  same-lattice branch selection, and global score winner.
- `bressloff-geometry` command that writes generated-only still targets for
  Figures 29-36, with normalized frame data plus radial, angular, and edge
  metrics for later private source-mask comparison.
- Non-rendering stability reports for the Fig 37-40 coefficient/bifurcation
  targets and the current rhombic-angle diagnostic.
- Separate Rule 2011 scalar E/I flicker track under
  `model_family = rule_flicker_ei`; it reuses the display surface but not the
  Bressloff orientation-contour or branch-selection machinery.

## Approximate

- The stability scan uses the Fourier/Bessel kernel family described in the
  paper, but still needs direct paper-figure calibration.
- The scalar frame for planforms is currently the strongest sampled orientation
  response at each cortical location in contoured mode. Non-contoured mode uses
  the scalar activity sum directly.
- Branch selection now follows the cubic amplitude-equation coefficient structure,
  but the bifurcation distance and nonlinear sigmoid constants are normalized
  rather than fit to a biological parameter set.
- The square/cobweb paper presets intentionally render their target planforms.
  The same-lattice branch readout currently selects a roll/spiral branch, matching
  the known instability issue better than the earlier honeycomb-like global
  comparison.
- The hex-pi preset renders the requested phase variant, while the current
  quadratic-term sign selects the honeycomb phase partner.
- The triangular odd-hex preset renders the requested branch, but the current
  cubic selector reports the honeycomb/0-hexagonal branch because the source
  stability discussion depends on higher-order terms.

## Still Missing

- Private/source-derived masks for exact digitized figure geometry and parameter
  values. Generated comparison stills now exist; source masks are not committed.
- Efficient compact-basis export for orientation channels when the full tensor is
  too large.
- Dynamics-to-contour overlay from simulated orientation channels.
- Public side-by-side figure comparison panels with generated animations and
  DOI/source links. Direct reproduction of paper figures needs permission or a
  confirmed compatible license.

The detailed public work plan is
[`docs/BRESSLOFF_FUTURE_IMPLEMENTATION_PLAN.md`](../docs/BRESSLOFF_FUTURE_IMPLEMENTATION_PLAN.md).
