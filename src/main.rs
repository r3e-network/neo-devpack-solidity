//! Neo DevPack for Solidity (neo-solc)
//!
//! Command-line compiler for Neo N3 smart contracts written in Solidity.
//!
//! # Usage
//!
//! ```bash
//! # Compile a single file (writes .nef + .manifest.json)
//! neo-solc input.sol -o build/
//!
//! # Batch compile multiple input files into a directory
//! neo-solc contracts/*.sol -o build/
//!
//! # Use standard JSON interface (stdin/stdout)
//! neo-solc --standard-json < input.json > output.json
//!
//! # Or via explicit files
//! neo-solc --standard-json --input input.json --output output.json
//!
//! # Set optimization level (0-3)
//! neo-solc input.sol -o build/ -O 3
//! ```
//!
//! # Output Files
//!
//! - `*.nef` - Neo Executable Format bytecode
//! - `*.manifest.json` - Contract manifest for deployment

fn main() {
    neo_devpack_solidity::cli::run();
}
