# Reference Source Clones

This directory is a local source-cache area for third-party projects that help
implementation planning. Large or nested upstream clones should stay ignored and
should not be committed as gitlinks or vendored code unless the project
explicitly decides to adopt that dependency.

Current local clones:

- `references/BifurcationKit.jl/`:
  `https://github.com/bifurcationkit/BifurcationKit.jl.git`, cloned as a
  shallow local audit copy. It is ignored by `.gitignore`.

Tracked audit outputs should live under `docs/`, for example
`docs/BIFURCATIONKIT_REFERENCE_AUDIT.md`.
