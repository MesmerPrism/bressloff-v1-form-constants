# Web Hosting Plan

Updated: 2026-05-23

The current viewer can be hosted as a real web app, but not as a fully static
site if the public version should keep live model generation. The browser UI
calls `/api/generate`, so the hosted version needs the Rust server running
behind the same origin as `viewer/index.html`.

## Recommended First Deployment

Use the included `Dockerfile` to deploy the Rust server plus the static viewer
assets to a small container host such as Fly.io, Render, Railway, a VPS, or any
container-capable host.

The container:

- builds `rust-v1-sim` in release mode;
- serves `viewer/`, `reports/`, and the public project metadata;
- binds to `0.0.0.0:$PORT`;
- excludes `private/`, local build output, logs, and large orientation exports
  through `.dockerignore`.

Local container smoke test:

```powershell
docker build -t bressloff-v1 .
docker run --rm -p 8080:8080 bressloff-v1
```

Open:

```text
http://127.0.0.1:8080/viewer/index.html
```

## Public-Safety Notes

The API already clamps grid sizes, frame counts, and parameter ranges, but a
public deployment should still be treated as CPU-bound interactive compute.
Before pointing high traffic at it, add or configure:

- provider-level request timeout;
- provider-level rate limiting or a CDN/WAF rule for `/api/generate`;
- a small instance size with autoscaling disabled until usage is understood;
- server log capture for failed or slow model runs;
- no `private/` files in the build context or image.

## Static-Only Alternative

A static host can serve `viewer/index.html` and `viewer/frames.json`, but the
controls will fall back to the precomputed payload and will not regenerate model
frames. That is acceptable for a gallery/demo page, not for the current
interactive lab.

## Longer-Term Option

A WebAssembly build could move the model generator into the browser and make the
interactive app static-hostable. That would require splitting the single Rust
binary into a reusable model library plus a WASM frontend binding, so it should
come after the server-backed deployment is stable.
