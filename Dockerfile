# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

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

# Build the application with real source code
# This will be fast because dependencies are already compiled
RUN cargo build --release

# Runtime stage - Using distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/animal-shelter-donation-thermometer /app/animal-shelter-donation-thermometer

# Expose port
EXPOSE 8080

# Run the web server
CMD ["/app/animal-shelter-donation-thermometer"]
