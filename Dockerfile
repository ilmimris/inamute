# Runtime stage - distroless (no shell, no package manager, minimal attack surface)
FROM gcr.io/distroless/cc-debian12

LABEL maintainer="Rafiul Ilmi <muhammadrafiulilmi@gmail.com>"
LABEL description="Indonesian Commute Schedule API"
LABEL version="0.1.0"

WORKDIR /app

# Copy binary and migrations from builder
COPY target/release/inamute .
COPY migrations ./migrations

EXPOSE 8080

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

# Run as non-root user (distroless has built-in nonroot user)
USER nonroot:nonroot

CMD ["./inamute"]
