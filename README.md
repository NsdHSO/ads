# ADS Secure Translator (Template)

This workspace bootstraps the Phase 1–3 goals:

- Phase 1: Core J-Series parsing (crates/jseries), Zenoh bridge (apps/bridge), Kani proof harness scaffolding.
- Phase 2: Application-level E2EE (crates/e2ee) with hooks for rustls and optional PQC.
- Phase 3: Containerization and SBOM.

## Workspace

- crates/jseries: Bit-level parser/serializer for Link 16 J-Series messages (example J3.2 Air Track) using `deku`.
- crates/e2ee: Minimal E2EE layer and stubs for TLS/PQC integration.
- apps/bridge: Zenoh-based scoped bridge from JSON telemetry to J3.2 bytes and UDP sink.

## Quick start

Build everything:

```bash
cargo build --workspace
```

Run the bridge without Zenoh (prints help):

```bash
cargo run -p bridge -- --help
```

Enable Zenoh feature to activate pub/sub bridging:

```bash
cargo run -p bridge --features zenoh -- --subscribe drone/** --sink 127.0.0.1:5000
```

Generate SBOM (CycloneDX) locally (requires cargo-sbom):

```bash
cargo install cargo-sbom
cargo sbom --format cyclonedx --output sbom.json
```

Kani proofs (locally):

```bash
# Install Kani per upstream instructions, then
kani --enable-unstable --workspace
```

## Docker

```bash
# Build release image
DOCKER_BUILDKIT=1 docker build -t ads-bridge:dev -f Dockerfile .

# Run (example)
docker run --rm --network host ads-bridge:dev --features zenoh --subscribe drone/** --sink 127.0.0.1:5000
```
