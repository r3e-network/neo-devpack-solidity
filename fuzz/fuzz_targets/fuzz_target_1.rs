#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Treat random bytes as a Solidity source string.
    // Valid UTF-8 is required for the parser; if invalid, just ignore.
    if let Ok(source) = std::str::from_utf8(data) {
        // Attempt compilation at all optimization levels.
        // We only care about panics/crashes, not semantic correctness.
        for opt in [0, 1, 2, 3] {
            let _ = std::panic::catch_unwind(|| {
                let _ = neo_devpack_solidity::cli::compile_contracts(source, false, opt);
            });
        }
    }
});
