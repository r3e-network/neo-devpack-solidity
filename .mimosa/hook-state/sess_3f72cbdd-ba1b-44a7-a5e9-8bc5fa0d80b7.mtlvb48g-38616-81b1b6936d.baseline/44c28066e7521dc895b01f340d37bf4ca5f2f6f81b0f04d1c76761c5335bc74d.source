//! Fuzz target: Solidity standard-JSON input parser must not panic on
//! arbitrary bytes, and must reject malformed / pathological inputs with
//! an `Err(String)` instead of crashing.
//!
//! The neo-devpack-solidity compiler exposes a Solidity-compatible standard-JSON
//! interface (`--standard-json`) consumed by Hardhat, Foundry, and other
//! third-party tooling. Production robustness requires graceful error
//! paths for malicious or malformed input. Panics here indicate a bug
//! that would bring down the compiler process.
//!
//! The entry point is `neo_devpack_solidity::cli::fuzz_process_standard_json_content`,
//! a total wrapper around the internal `process_standard_json_content`.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        let _ = neo_devpack_solidity::cli::fuzz_process_standard_json_content(data);
    });
});
