#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz the NEF parser with random bytes.
    let _ = neo_devpack_solidity::neo::parse_nef(data);
});
