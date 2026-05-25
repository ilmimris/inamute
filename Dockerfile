# Builder stage
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY static ./static

# Build release binary
RUN cargo build --release

# Runtime stage - distroless (no shell, no package manager, minimal attack surface)
FROM gcr.io/distroless/cc-debian12

LABEL maintainer="Rafiul Ilmi <muhammadrafiulilmi@gmail.com>"
LABEL description="Indonesian Commute Schedule API"
LABEL version="0.1.0"

WORKDIR /app

# Copy binary, migrations, and static from builder
COPY --from=builder /app/target/release/inamute .
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/static ./static

EXPOSE 8080

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

# Run as non-root user (distroless has built-in nonroot user)
USER nonroot:nonroot

CMD ["./inamute"]
