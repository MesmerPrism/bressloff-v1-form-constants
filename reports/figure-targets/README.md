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
dominant-angle, source-angle, source-comparison, and diagnostic-threshold
fields. When private source profiles are present, the report carries residuals
and comparison status only; it deliberately does not include scans, crops, or
masks from the original papers.

Private extraction is intentionally local-only:

```powershell
python tools\extract_bressloff_source_profiles.py --config private\figure-targets\source-mask-config.json --out-dir private\figure-targets\derived
```

The private config should list local source image paths and preset IDs. It may
also list `crop_box`, `invert`, `threshold`, and `autocrop` values so private
page renders can be cropped without moving crops into tracked folders. The
script writes one derived numeric profile per preset under the ignored
`private/figure-targets/derived/` path. Re-run `bressloff-geometry` with
`--source-profile-dir private\figure-targets\derived` to populate radial profile
error, angular profile error, edge overlap, active-fraction error, edge-density
error, and lattice-angle error. The extractor also records a first-pass source
angle as the dominant angular-profile bin center. Keep private source profiles
ignored unless a separate rights review explicitly approves publishing derived
numeric targets.

The current acceptance policy is deliberately conservative:
`calibration_claim_allowed=false`, `threshold_accepted_still_count` must be read
as a diagnostic gate, and no Bressloff figure should be described as calibrated
until the threshold checks, crop QA, and source-angle review are all explicitly
approved.
