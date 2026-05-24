# Figure Targets

This folder holds public-safe derived calibration targets for Bressloff
figure-level geometry work.

Generate the current generated-only still report with:

```powershell
.\rust-v1-sim\target\release\bressloff-v1.exe bressloff-geometry --out reports\figure-targets\bressloff-generated-stills.json
```

`bressloff-generated-stills.json` uses
`format: bressloff-generated-figure-stills-v1`. It includes normalized generated
stills for Figures 29-36 plus radial, angular, and edge metrics. It deliberately
does not include scans, crops, or masks from the original papers.

The intended next step is to create private/source-derived masks or profiles and
compare them against these generated still IDs. Commit only derived numeric
targets when the source license state is clear.
