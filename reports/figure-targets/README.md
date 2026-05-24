# Figure Targets

This folder holds public-safe derived calibration targets for Bressloff
figure-level geometry work.

Generate the current generated still/comparison report with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe bressloff-geometry --out reports\figure-targets\bressloff-generated-stills.json
```

`bressloff-generated-stills.json` uses
`format: bressloff-generated-figure-stills-v2`. It includes normalized generated
stills for Figures 29-36 plus radial profile, angular profile, edge-density,
dominant-angle, and source-comparison metric slots. It deliberately does not
include scans, crops, or masks from the original papers.

Private extraction is intentionally local-only:

```powershell
python tools\extract_bressloff_source_profiles.py --config private\figure-targets\source-mask-config.json --out-dir private\figure-targets\derived
```

The private config should list local source image paths and preset IDs. The
script writes one derived numeric profile per preset under the ignored
`private/figure-targets/derived/` path. Re-run `bressloff-geometry` with
`--source-profile-dir private\figure-targets\derived` to populate radial profile
error, angular profile error, edge overlap, active-fraction error, edge-density
error, and lattice-angle error. Commit only derived numeric targets when the
source license state is clear.
