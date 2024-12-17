# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Download dependencies
RUN cargo fetch

# Copy the source code into the container
COPY src ./src

# Build the Rust application in release mode
RUN cargo build --release

# Use a smaller base image for the final image
FROM debian:bullseye-slim

# Install necessary dependencies (e.g., for running the Rust app)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/my_app /usr/local/bin/my_app

# Set the default command to run the application
CMD ["cargo run"]
