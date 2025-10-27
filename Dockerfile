# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install fonts for runtime
RUN apt-get update && \
    apt-get install -y fonts-dejavu-core fonts-liberation && \
    rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only - this layer will be cached
RUN cargo build --release

# Remove dummy source and compiled artifacts
RUN rm -rf src target/release/animal-shelter* target/release/deps/animal_shelter*

# Copy actual source code
COPY src ./src
COPY templates ./templates
COPY static ./static

# Build the application with real source code
# This will be fast because dependencies are already compiled
RUN cargo build --release

# Runtime stage - Using Debian slim for font support
FROM debian:12-slim

WORKDIR /app

# Install required runtime libraries and fonts
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        fonts-dejavu-core \
        fonts-liberation \
        libssl3 \
        fontconfig && \
    rm -rf /var/lib/apt/lists/* && \
    fc-cache -f -v

# Copy the binary from builder
COPY --from=builder /app/target/release/animal-shelter-donation-thermometer /app/animal-shelter-donation-thermometer
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# Expose port
EXPOSE 8080

# Run the web server
CMD ["/app/animal-shelter-donation-thermometer"]
