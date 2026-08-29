# Installing the Solidity Compiler

## Versioning

Neo DevPack for Solidity versions follow semantic versioning. We recommend always using the latest release to ensure you have the most up-to-date EVM-to-NeoVM mappings, intrinsic library support, and security hardening patches.

## npm / Node.js

Use `npm` for a convenient and portable way to install `neo-devpack-solidity` locally into your JavaScript/TypeScript project. This is highly recommended if you are using Hardhat or other Node.js-based tooling.

```bash
npm install --save-dev hardhat@^2.28.6 @neo-devpack-solidity/hardhat-solc-neo
```

The Hardhat plugin will automatically manage the underlying compiler binary for your system. The
current plugin line targets Hardhat 2.28.x; Hardhat 3 requires a separate plugin/runtime migration.

## Docker

If you prefer to run the compiler inside a containerized environment to avoid installing Rust and other dependencies locally, build the checked-in Dockerfile from the source repository.

```bash
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity
docker build -t neo-solc:local .
docker run --rm -v "$(pwd)":/sources neo-solc:local -o /sources/build /sources/MyContract.sol
```

## Static Binaries (Linux, macOS, Windows)

We provide pre-built static binaries for major operating systems via our GitHub Releases page.

1. Navigate to [Neo DevPack for Solidity Releases](https://github.com/r3e-network/neo-devpack-solidity/releases).
2. Download the binary archive for your platform.
3. Extract the archive.
4. Move the `neo-solc` executable to a directory in your system `$PATH` (e.g., `/usr/local/bin`).

```bash
chmod +x neo-solc
sudo mv neo-solc /usr/local/bin/
```

## Building from Source

If you want the absolute latest features or are contributing to the compiler, building from source is the best option.

### Prerequisites

| Requirement        | Minimum Version | Purpose                                              |
| ------------------ | --------------- | ---------------------------------------------------- |
| **Rust toolchain** | 1.88+ (stable)  | The core compiler is written in Rust                 |
| **Git**            | 2.x             | To clone the repository                              |
| **Node.js**        | Active LTS: 20.19+, 22.12+, or 24.x | (Optional) For TypeScript tooling, VitePress docs, and Hardhat workflows |

If Rust is not installed, install it via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Clone and Build

```bash
git clone https://github.com/r3e-network/neo-devpack-solidity.git
cd neo-devpack-solidity

# Build the release binary
cargo build --release
```

The compiled binary will be located at `target/release/neo-solc`.

### Install System-Wide via Cargo

You can easily install the compiler system-wide directly through Cargo without needing to manually move the binary:

```bash
cargo install --path .
```

Ensure your `~/.cargo/bin` directory is in your system's `$PATH`.

## The Version String in Detail

The Neo DevPack for Solidity version string contains the primary version number, along with build metadata if compiled from a non-release branch.

```bash
neo-solc --version
```

Output format:
`neo-solc <major>.<minor>.<patch>` (e.g., `neo-solc 0.27.0`).

When debugging or reporting issues on GitHub, please always include the exact version string produced by this command.

## Optional: Local TestNet Environment (Neo-Express)

To effectively test compiled `.nef` files locally, we recommend installing Neo-Express, a local private blockchain for Neo N3.

```bash
# Requires .NET 8.0+ SDK
dotnet tool install -g Neo.Express
```

Once installed, you can use `neoxp` to create a local node, deploy the compiler's output, and run transactions locally. See [Deploying Contracts](/basics/deploying-contracts) for more details.
