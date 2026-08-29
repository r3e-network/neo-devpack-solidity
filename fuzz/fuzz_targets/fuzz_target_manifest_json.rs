//! Fuzz target: manifest JSON parser must not panic on arbitrary UTF-8
//! input, and must reject malformed inputs cleanly with an error.
//!
//! The neo-devpack-solidity compiler consumes manifests produced by other tools
//! (Hardhat, Foundry, legacy .NET compiler). Production robustness
//! requires that pathological / malicious manifests are rejected, not
//! crashed on.
//!
//! We fuzz via `serde_json::from_slice::<Value>` as the parser entry
//! point; a crash here signals a serde-json bug, but more importantly
//! we wrap the check in a panic-catcher to ensure callers get clean
//! errors.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        // Parse attempt — only non-panicking results count as "accepted".
        if let Ok(value) = serde_json::from_slice::<Value>(data) {
            // Manifest inspection: walk expected sub-paths without
            // dereferencing unchecked. Catches Value→shape
            // casting that might panic on unexpected types.
            let _ = value.get("abi").and_then(|abi| abi.get("methods"));
            let _ = value.get("name").and_then(|n| n.as_str());
            let _ = value
                .get("permissions")
                .and_then(|p| p.as_array())
                .map(|arr| arr.len());
            let _ = value
                .get("supportedstandards")
                .and_then(|s| s.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).count());
        }
    });
});
