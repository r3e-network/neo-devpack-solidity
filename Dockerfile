# Multi-stage build for Neo DevPack for Solidity
# Use a Rust toolchain new enough for Edition 2024 dependencies.
FROM rust:1.88 AS rust-builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build optimized release
RUN cargo build --release

# Final runtime image
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts
COPY --from=rust-builder /app/target/release/neo-solc /usr/local/bin/neo-solc

# Copy project files
COPY devpack/ /opt/neo-devpack-solidity/devpack/
COPY examples/ /opt/neo-devpack-solidity/examples/
COPY *.md /opt/neo-devpack-solidity/docs/

# Create entrypoint script
RUN echo '#!/bin/bash' > /usr/local/bin/entrypoint.sh \
    && echo 'if [ "$1" = "neo-solc" ]; then' >> /usr/local/bin/entrypoint.sh \
    && echo '  shift' >> /usr/local/bin/entrypoint.sh \
    && echo '  exec neo-solc "$@"' >> /usr/local/bin/entrypoint.sh \
    && echo 'else' >> /usr/local/bin/entrypoint.sh \
    && echo '  exec "$@"' >> /usr/local/bin/entrypoint.sh \
    && echo 'fi' >> /usr/local/bin/entrypoint.sh \
    && chmod +x /usr/local/bin/entrypoint.sh

# Set working directory
WORKDIR /workspace

# Labels
LABEL org.opencontainers.image.title="Neo DevPack for Solidity"
LABEL org.opencontainers.image.description="Complete Solidity-to-NeoVM compilation system"
LABEL org.opencontainers.image.vendor="R3E Network"
LABEL org.opencontainers.image.authors="Jimmy <jimmy@r3e.network>"
LABEL org.opencontainers.image.source="https://github.com/r3e-network/neo-devpack-solidity"
LABEL org.opencontainers.image.licenses="MIT"

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["neo-solc", "--help"]
