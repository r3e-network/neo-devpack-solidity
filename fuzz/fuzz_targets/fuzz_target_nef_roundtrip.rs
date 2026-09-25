//! Fuzz target: NEF parse → re-serialize round-trip preserves bytes when
//! parsing succeeded. Equivalent to a differential check against an
//! idealised identity encoder.
//!
//! Invariant under test: if `parse_nef(input)` succeeds, then building a
//! fresh NEF from the parsed script + tokens must not crash. This catches
//! parse / serialize asymmetry bugs where the parser accepts inputs the
//! serializer can't reproduce (a common class of round-trip regression in
//! NEF header / method-token decoders).
//!
//! Crashes or `Err` from the re-serialize path signal a parser that
//! over-accepts or a serializer that fails on legitimate shapes.

#![no_main]

use libfuzzer_sys::fuzz_target;
use neo_devpack_solidity::neo::{build_nef_with_tokens, parse_nef};

fuzz_target!(|data: &[u8]| {
    match std::panic::catch_unwind(|| {
        let Ok(parsed) = parse_nef(data) else {
            return; // Parser rejected — nothing to round-trip.
        };
        // Parser accepted: the compiler would be willing to pick up this
        // NEF as a dependency. Serialize it back through the canonical
        // builder and assert that succeeds — if it panics or errors the
        // parse / serialize contract is broken.
        let rebuilt = build_nef_with_tokens(
            &parsed.script,
            "neo-devpack-solidity-fuzz",
            parsed.source.as_str(),
            &parsed.tokens,
        )
        .expect("a successfully parsed NEF must be serializable");
        let reparsed = parse_nef(&rebuilt).expect("serialized NEF must parse again");
        assert_eq!(reparsed.script, parsed.script, "NEF script round-trip changed");
        assert_eq!(reparsed.tokens.len(), parsed.tokens.len(), "NEF token count changed");
        for (index, (actual, expected)) in reparsed.tokens.iter().zip(parsed.tokens.iter()).enumerate() {
            assert_eq!(actual.hash, expected.hash, "NEF token {index} hash changed");
            assert_eq!(actual.method, expected.method, "NEF token {index} method changed");
            assert_eq!(
                actual.parameters_count,
                expected.parameters_count,
                "NEF token {index} parameter count changed"
            );
            assert_eq!(
                actual.has_return_value,
                expected.has_return_value,
                "NEF token {index} return flag changed"
            );
            assert_eq!(actual.call_flags, expected.call_flags, "NEF token {index} flags changed");
        }
    }) {
        Ok(()) => {}
        Err(payload) => std::panic::resume_unwind(payload),
    }
});
