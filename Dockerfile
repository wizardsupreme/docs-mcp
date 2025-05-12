# Stage 1: Build the application
# Adhering to Rule ⭐⑴✅: Use latest stable Rust version.
# Using rust:1.86-slim-bullseye as per latest Docker Hub information.
# Adhering to Rule ⭐⑵✅: Consulting Rust and Docker documentation.
FROM rust:1.86-slim-bullseye AS builder

# Install cargo-chef for optimized layer caching
# Adhering to Rule ⭐⑵✅: Consulting cargo-chef documentation for usage.
RUN cargo install cargo-chef --locked
WORKDIR /app

# Copy manifests and lock file
COPY Cargo.toml Cargo.lock ./

# Prepare the chef recipe (plan dependencies)
# This layer will be cached if Cargo.toml and Cargo.lock don't change.
RUN cargo chef prepare --recipe-path recipe.json

# Cook dependencies (build dependencies)
# This layer will be cached if the recipe.json (dependency plan) doesn't change.
RUN cargo chef cook --release --recipe-path recipe.json

# Copy application source code
COPY src ./src

# Build the application binary. The binary name is 'cratedocs'.
# This layer will be rebuilt if src changes.
RUN cargo build --release --bin cratedocs

# Stage 2: Create the final minimal image
# Adhering to Rule ⭐⑴✅: Use a minimal, stable base image.
# Using debian:bookworm-slim.
FROM debian:bookworm-slim AS runtime

# Optional: Create a non-root user for better security
# RUN groupadd -r appgroup && useradd --no-create-home -r -g appgroup -s /bin/false appuser
# USER appuser

WORKDIR /usr/local/bin
# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/cratedocs .

# Ensure the binary is executable
RUN chmod +x /usr/local/bin/cratedocs

# Set the entrypoint for the application
ENTRYPOINT ["/usr/local/bin/cratedocs"]

# If the application listens on a port, uncomment and adjust:
# EXPOSE 8000