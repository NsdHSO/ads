# syntax=docker/dockerfile:1.6

# --- Build stage ---
FROM rust:1-bullseye as build
WORKDIR /work

# Cache deps
COPY Cargo.toml rust-toolchain.toml ./
COPY crates/jseries/Cargo.toml crates/jseries/Cargo.toml
COPY crates/e2ee/Cargo.toml crates/e2ee/Cargo.toml
COPY apps/bridge/Cargo.toml apps/bridge/Cargo.toml
RUN mkdir -p crates/jseries/src crates/e2ee/src apps/bridge/src \
    && echo "pub fn _dummy(){}" > crates/jseries/src/lib.rs \
    && echo "pub fn _dummy(){}" > crates/e2ee/src/lib.rs \
    && echo "fn main(){}" > apps/bridge/src/main.rs \
    && cargo build -p bridge --release

# Copy real sources and rebuild
COPY . .
RUN cargo build -p bridge --release

# --- Runtime stage (distroless nonroot) ---
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=build /work/target/release/bridge /app/bridge
USER nonroot
ENTRYPOINT ["/app/bridge"]
