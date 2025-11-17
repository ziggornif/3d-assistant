# Multi-stage Dockerfile for 3D Print Quote Service
# Stage 1: Build Rust backend
FROM rust:1.91-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust project files
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build release binary
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/quote-service /app/quote-service

# Copy static assets (CSS, JS)
COPY static/ /app/static/

# Copy templates (for SSR)
COPY templates/ /app/templates/

# Create necessary directories
RUN mkdir -p /app/data /app/uploads

# Create non-root user
RUN useradd -m -u 1000 appuser && \
    chown -R appuser:appuser /app
USER appuser

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=sqlite:///app/data/quotes.db?mode=rwc
ENV UPLOAD_DIR=/app/uploads
ENV STATIC_DIR=/app/static
ENV TEMPLATE_DIR=/app/templates
ENV MAX_FILE_SIZE_MB=50
ENV SESSION_EXPIRY_HOURS=24
ENV ADMIN_TOKEN=changeme-in-production
ENV RUST_LOG=info

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/materials || exit 1

# Run the application
CMD ["/app/quote-service"]
