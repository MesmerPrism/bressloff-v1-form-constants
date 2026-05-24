# Source Curves

This folder contains public-safe numeric curve data derived from private source
figure extraction.

## Rule 2011 Figure 8C

`rule-2011-fig8-source-curves.json` contains digitized coordinates for the four
visible Figure 8C stability boundaries:

- `rule_fig8c_minus_upper`
- `rule_fig8c_minus_lower`
- `rule_fig8c_plus_upper`
- `rule_fig8c_plus_lower`

The committed data contains only derived axis coordinates:

- forcing period in milliseconds
- source-figure wave-number coordinate `beta`

It does not contain paper scans, crops, or rendered page images. The private
render and QA overlay live under `private/figure-targets/rule-figure8/`.

Regenerate from the private PDF page render with:

```powershell
python tools\digitize_rule_fig8_source_curves.py
```

The private overlay colors identify branch assignments, not left/right sides:
red is the upper `-1` boundary, blue is the lower `-1` boundary, green is the
upper `+1` boundary, and purple is the lower `+1` boundary.
