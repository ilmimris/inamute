# Build stage
FROM public.ecr.aws/docker/library/rust:1.88-slim AS builder

WORKDIR /usr/src/inamute

# Copy dependency manifests first for better layer caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy src to pre-build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source and rebuild
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# Runtime stage - distroless (no shell, no package manager, minimal attack surface)
FROM gcr.io/distroless/static-debian12

LABEL maintainer="Rafiul Ilmi <muhammadrafiulilmi@gmail.com>"
LABEL description="Indonesian Commute Schedule API"
LABEL version="0.1.0"

WORKDIR /app

# Copy binary and migrations from builder
COPY --from=builder /usr/src/inamute/target/release/inamute .
COPY --from=builder /usr/src/inamute/migrations ./migrations

# Set proper permissions (readable and executable only)
RUN chmod 555 /app/inamute

EXPOSE 8080

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

# Run as non-root user (distroless has built-in nonroot user)
USER nonroot:nonroot

CMD ["./inamute"]
