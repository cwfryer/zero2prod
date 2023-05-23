FROM rust:1 as builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build --release --bin zero2prod

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration

RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
