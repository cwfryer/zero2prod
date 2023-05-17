############### Builder stage ###############

# We use the latest Rust stable release as base image
FROM rust:1.49 AS builder

WORKDIR /app

RUN cargo install --locked --branch master \
    --git https://github.com/eeff/cargo-build-deps

# Build the dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo build-deps --release

# Enforce sqlx offline mode
ENV SQLX_OFFLINE true

COPY . .

# Build out application, leveraging the cached deps
RUN cargo build --release --bin zero2prod


############### Runtime stage ###############

FROM debian:buster-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some the dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Using production environment
ENV APP_ENVIRONMENT production

# Lauch our binary
ENTRYPOINT ["./zero2prod"]

# We need the configuration file at runtime
COPY configuration configuration

# Copy the compiled binary from the builder environment
COPY --from=builder /app/target/release/zero2prod zero2prod
