# Bressloff V1 Form Constants Agent Guide

## Global Machine Coordination

Before using machine-wide exclusive resources on this laptop, check Agent Board:

```powershell
& 'C:\Users\tillh\Agent Bureau\scripts\agent-board.ps1' status
```

Reserve and later release resources such as Quest headsets, `adb-server`, Unity
batchmode builds, long APK/package builds, and local bridge ports:

```powershell
& 'C:\Users\tillh\Agent Bureau\scripts\agent-board.ps1' reserve quest:<serial> --duration 45m --task "Quest validation"
& 'C:\Users\tillh\Agent Bureau\scripts\agent-board.ps1' release <lease-id> --result done
```

For details, use `$bureau-context` and read
`C:\Users\tillh\Agent Bureau\tools\agent-board\README.md`.

## Repo Roles

This repo is the implementation and report source of truth for the Bressloff V1
form-constants work:

```text
S:\Work\repos\active\bressloff-v1-form-constants
```

The paired public website repo is:

```text
S:\Work\repos\active\MesmerPrism.github.io
```

Keep the boundary clear:

- this repo owns Rust model code, generated reports, private paper caches, and
  public-safe implementation plans;
- the MesmerPrism repo owns public HTML/CSS/JS, public generated assets, public
  prose, references, and reader-facing summaries;
- commit implementation/report changes separately from website changes unless
  the user explicitly asks for a single cross-repo history;
- never put original PDFs, paper scans, page renders, crops, or private
  extraction notes into tracked public folders.

## Model Families

Preserve the current separation between model tracks:

- `bressloff_orientation_hypercolumn`: spontaneous orientation-hypercolumn
  planforms, retino-cortical mapping, contour overlays, stability diagnostics,
  and Bressloff source-target reports.
- `rule_flicker_ei`: Rule/Stoffregen/Ermentrout diffuse flicker E/I model,
  frequency-amplitude sweeps, Floquet boundary curves, and Figure 8 fit
  diagnostics.
- driven neural-field families: MacKay localized input, Bolelli localized
  time-periodic input, Nicks orthogonal-response diagnostics, plus deferred
  pinwheel, color, contrast-gradient, and perceptual-grouping extensions.

Do not merge these into one generic hallucination model. Use explicit
`model_family`, `source_key`, `implementation_status`, `public_claim_level`, and
`rights_status` fields when adding reports or registry entries.

## Private Sources

Private source material lives under ignored paths, especially:

```text
private/papers/
private/figure-targets/
```

Use private notes and PDFs for extraction, but tracked outputs must contain only
public-safe summaries, generated model data, report schemas, citations, and
implementation plans. Verify private artifacts remain ignored with
`git check-ignore -v private/papers private/papers/*` when touching that area.

## Public Report and Asset Synchronization

When report JSON changes here, update the matching website asset in
`S:\Work\repos\active\MesmerPrism.github.io`.

Current report-to-asset map:

```text
reports/figure-targets/bressloff-generated-stills.json
  -> assets/bressloff-v1/figure-targets/bressloff-generated-stills.json

reports/rule-2011-sweep.json
  -> assets/bressloff-v1/deep-dive/rule-2011-sweep.json

reports/rule-2011-sweep-dense.json
  -> assets/bressloff-v1/deep-dive/rule-2011-sweep-dense.json

reports/rule-2011-floquet.json
  -> assets/bressloff-v1/deep-dive/rule-2011-floquet.json

reports/source-curves/rule-2011-fig8-source-curves.json
  -> assets/bressloff-v1/deep-dive/rule-2011-fig8-source-curves.json

reports/driven-neural-fields-registry.json
  -> assets/bressloff-v1/driven/driven-neural-fields-registry.json

reports/mackay-localized-input.json
  -> assets/bressloff-v1/driven/mackay-localized-input.json

reports/bolelli-time-periodic-input.json
  -> assets/bressloff-v1/driven/bolelli-time-periodic-input.json

reports/nicks-orthogonal-response.json
  -> assets/bressloff-v1/driven/nicks-orthogonal-response.json
```

After copying, compare hashes from both repos so drift is explicit:

```powershell
Get-FileHash reports\<name>.json
Get-FileHash S:\Work\repos\active\MesmerPrism.github.io\assets\bressloff-v1\<path>\<name>.json
```

## Rust Workflow

Use the existing crate and module boundaries:

```text
rust-v1-sim/src/models/bressloff/
rust-v1-sim/src/models/rule/
rust-v1-sim/src/models/driven/
rust-v1-sim/src/numeric/
rust-v1-sim/src/params.rs
rust-v1-sim/src/payload.rs
rust-v1-sim/src/export.rs
```

Prefer extending the local model-family module over adding logic back into
`main.rs`. Shared numerical helpers belong under `rust-v1-sim/src/numeric/`.
Request/query coercion and payload assembly belong in `params.rs` and
`payload.rs`; export-only CLI glue belongs in `export.rs`.

Common validation commands:

```powershell
cargo fmt --manifest-path rust-v1-sim\Cargo.toml
cargo test --manifest-path rust-v1-sim\Cargo.toml
git diff --check
```

Use the wrapper when a change may affect both reports and website assets:

```powershell
powershell -ExecutionPolicy Bypass -File tools\verify.ps1
```

Report commands documented in `README.md` and `reports/README.md` should
regenerate tracked report JSON. When checking reproducibility, generate into a
temporary directory first and compare hashes before overwriting tracked files.

## Website Workflow

Before editing the MesmerPrism website for this project, read:

```text
S:\Work\repos\active\MesmerPrism.github.io\AGENTS.md
S:\Work\repos\active\MesmerPrism.github.io\docs\WRITING_PROJECT_PAGES.md
S:\Work\writing\AGENTS.md
S:\Work\writing\_registry\STRUCTURE_STANDARD.md
```

Website files currently tied to this repo include:

```text
projects/bressloff-v1-form-constants.html
projects/bressloff-v1-form-constants-deep-dive.html
scripts/rule-dynamics-explorer.js
scripts/driven-field-diagnostics.js
scripts/bressloff-calibration-panel.js
styles.css
assets/bressloff-v1/
```

Public prose rules:

- describe generated outputs as diagnostics or first-pass reports unless a
  report-backed source-target comparison supports a stronger claim;
- do not publish private process language, source-page extraction details, PDF
  page numbers from private notes, or unlicensed figure reuse;
- keep inline citations close to technical, historical, rights, or
  current-status claims;
- external links in public pages should use `target="_blank"` and
  `rel="noopener noreferrer"`;
- generated images and canvases are preferred over source-paper crops.

## Website Tools and Review

Use `rg` for search and `apply_patch` for manual edits. For local website
review, use the Browser plugin/in-app browser rather than an external ad hoc
browser workflow. If a local static server is needed, reserve the port with
Agent Board first, for example:

```powershell
& 'C:\Users\tillh\Agent Bureau\scripts\agent-board.ps1' reserve port:4173 --duration 30m --task "MesmerPrism Bressloff page validation"
python -m http.server 4173 --bind 127.0.0.1
& 'C:\Users\tillh\Agent Bureau\scripts\agent-board.ps1' release <lease-id> --result done
```

Minimum website validation after editing Bressloff/Rule/driven pages:

- local HTTP 200 for the changed page or pages;
- no browser console errors;
- no stale placeholders such as `loading`, `pending`, or
  `available after report load` after report assets load;
- no visible broken images or blank report canvases;
- desktop and mobile viewport checks show no horizontal document overflow;
- inline citation/reference links have `target="_blank"` and
  `rel="noopener noreferrer"` where they leave the site;
- `git diff --check` passes in the website repo.

If website copy depends on a current publisher license, DOI landing page, or
other unstable public fact, verify it from the web and cite/link the source in
the public page.

## Git Hygiene

Both repos may be dirty for unrelated reasons. Never revert user changes. Audit
both worktrees before editing:

```powershell
git status --short --branch
git diff --stat
```

When both repos are touched, finish with both statuses visible. If committing,
commit this repo and the website repo separately with messages that name the
implementation/report change and the public-site change.
