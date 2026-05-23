FROM rust:1-bookworm AS build

WORKDIR /app
COPY rust-v1-sim/Cargo.toml rust-v1-sim/Cargo.lock ./rust-v1-sim/
COPY rust-v1-sim/src ./rust-v1-sim/src
RUN cargo build --manifest-path rust-v1-sim/Cargo.toml --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=build /app/rust-v1-sim/target/release/bressloff-v1 /usr/local/bin/bressloff-v1
COPY viewer ./viewer
COPY reports ./reports
COPY README.md LICENSE NOTICE.md CITATION.cff ./

ENV PORT=8080
EXPOSE 8080

CMD ["sh", "-c", "bressloff-v1 serve --host 0.0.0.0 --port ${PORT} --root /app"]
