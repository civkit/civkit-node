# Stage 1: Building the binary
FROM rust:latest as builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler

# Set the working directory in the Docker image
WORKDIR /usr/src/civkit-node

# Copy the source code into the Docker image
COPY . .

# Build the application
RUN cargo build

# Stage 2: Setup the runtime environment
FROM ubuntu:latest

# Install runtime dependencies
# Including CA certificates
RUN apt-get update && apt-get install -y libsqlite3-0 libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binaries from the builder stage
COPY --from=builder /usr/src/civkit-node/target/debug/civkitd /usr/local/bin/civkitd
COPY --from=builder /usr/src/civkit-node/target/debug/civkit-cli /usr/local/bin/civkit-cli
COPY --from=builder /usr/src/civkit-node/target/debug/civkit-sample /usr/local/bin/civkit-sample

# Set the default command to run the main binary
CMD ["civkitd"]

