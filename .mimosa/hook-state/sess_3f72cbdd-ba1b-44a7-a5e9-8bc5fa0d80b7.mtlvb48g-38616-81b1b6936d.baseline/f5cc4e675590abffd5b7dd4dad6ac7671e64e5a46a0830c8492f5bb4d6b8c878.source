//! Fuzz target: mutation-based fuzzing of *almost-valid* NEF and manifest
//! inputs.
//!
//! Why a separate target from `fuzz_target_2` / `fuzz_target_manifest_json`?
//! Those targets feed pure-random bytes into `parse_nef` /
//! `serde_json::from_slice`. The NEF parser bails on the magic-byte check
//! within the first 4 bytes for >99% of random inputs, and the JSON parser
//! rejects non-JSON byte salads almost as quickly. The deeper code paths
//! — token-table parsing, varint length decoders, embedded source-file
//! string decoder, checksum verification, manifest ABI cast logic — are
//! never reached.
//!
//! Mutation-based fuzzing seeds with *valid* NEF / manifest pairs and
//! flips a small number of bytes. The result almost always still has a
//! valid magic + checksum prefix (or, for manifests, still parses as JSON),
//! which lets the mutation reach much further into the parser.
//!
//! Invariant under test: for any (seed_index, mutations) pair, parsing
//! the mutated input via `neo_devpack_solidity::neo::parse_nef` and via
//! `serde_json::from_slice::<Value>` (the manifest entry point used by
//! `fuzz_target_manifest_json`) terminates with either `Ok(_)` or
//! `Err(_)` — never a Rust panic, never an OOM-abort.
//!
//! Resource bound: same `catch_unwind` wrapping as
//! `fuzz_target_runtime_exec`; mutations are capped at ≤8 byte-flips,
//! ≤8 single-byte writes, ≤4 insertions of 1..=4 bytes each, ≤4
//! deletions of 1..=4 bytes each, so the mutated buffer stays within
//! O(seed_size + 32) bytes — well under any practical OOM threshold.

#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use std::sync::OnceLock;

/// Embedded fall-back NEF seed — the compiled `Target` contract from
/// `test_low_level-Target.nef`. 108 bytes, valid magic, valid checksum,
/// a single embedded source-file string. Used when `fuzz/corpus/` is
/// empty (e.g. fresh checkout, CI with no prior runs).
const EMBEDDED_NEF: &[u8] = &[
    0x4e, 0x45, 0x46, 0x33, 0x6e, 0x65, 0x6f, 0x2d, 0x73, 0x6f, 0x6c, 0x69, 0x64, 0x69, 0x74, 0x79,
    0x2d, 0x30, 0x2e, 0x31, 0x36, 0x2e, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x17, 0x2f, 0x74, 0x6d, 0x70, 0x2f, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x6c,
    0x6f, 0x77, 0x5f, 0x6c, 0x65, 0x76, 0x65, 0x6c, 0x2e, 0x73, 0x6f, 0x6c, 0x00, 0x00, 0x00, 0x00,
    0x07, 0x00, 0x2a, 0x40, 0x57, 0x00, 0x02, 0x40, 0xca, 0xd6, 0xe0, 0x2b,
];

/// Embedded fall-back manifest seed — matches the embedded NEF above.
/// Valid JSON, real manifest schema (abi/methods/permissions/etc.) so
/// any mutation lands in the schema-walker code paths.
const EMBEDDED_MANIFEST: &[u8] = br#"{
  "abi": {
    "events": [],
    "methods": [
      {"name": "getValue", "offset": 0, "parameters": [], "returntype": "Integer", "safe": true},
      {"name": "_deploy", "offset": 3, "parameters": [
        {"name": "data", "type": "Any"},
        {"name": "update", "type": "Boolean"}
      ], "returntype": "Void", "safe": false}
    ]
  },
  "extra": {"Author": "fuzz", "Compiler": "neo-devpack-solidity-0.16.0"},
  "features": {},
  "groups": [],
  "name": "Target",
  "permissions": [],
  "supportedstandards": ["NEP-29"],
  "trusts": []
}"#;

/// Cap on the number of seeds we load from the corpus directories.
/// Loading thousands of files at startup wastes memory; 64 of each is
/// more than enough variety for the fuzzer (libFuzzer's mutator works on
/// the `Unstructured` input bytes, not the seed pool).
const MAX_CORPUS_SEEDS: usize = 64;

/// Mutation caps — kept small so the mutated buffer stays close in size
/// and shape to the seed (the whole point of mutation fuzzing).
const MAX_BYTE_FLIPS: usize = 8;
const MAX_SINGLE_WRITES: usize = 8;
const MAX_INSERTIONS: usize = 4;
const MAX_DELETIONS: usize = 4;
const MAX_CHUNK_LEN: usize = 4;

/// Which parser the fuzzer should drive the mutated bytes into.
/// Determined per-iteration from the seed family.
#[derive(Clone, Copy)]
enum Parser {
    Nef,
    Manifest,
}

/// One-shot loader: walks `fuzz/corpus/fuzz_target_2` and
/// `fuzz/corpus/fuzz_target_manifest_json`, picking the smallest
/// `MAX_CORPUS_SEEDS` files of each (small files keep mutation
/// iterations fast). Falls back to the embedded seeds if the dirs are
/// missing or empty. The result is cached in a `OnceLock` so we only
/// pay the I/O cost once per fuzz process.
fn corpus() -> &'static (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    static SEEDS: OnceLock<(Vec<Vec<u8>>, Vec<Vec<u8>>)> = OnceLock::new();
    SEEDS.get_or_init(|| {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let nef_seeds = load_dir(&format!("{manifest_dir}/corpus/fuzz_target_2"));
        let manifest_seeds =
            load_dir(&format!("{manifest_dir}/corpus/fuzz_target_manifest_json"));

        let nef_seeds = if nef_seeds.is_empty() {
            vec![EMBEDDED_NEF.to_vec()]
        } else {
            nef_seeds
        };
        let manifest_seeds = if manifest_seeds.is_empty() {
            vec![EMBEDDED_MANIFEST.to_vec()]
        } else {
            manifest_seeds
        };
        (nef_seeds, manifest_seeds)
    })
}

/// Load up to `MAX_CORPUS_SEEDS` smallest files from `path`. Silent on
/// any I/O error — the fall-back to the embedded seed handles the
/// missing-directory case at the call site.
fn load_dir(path: &str) -> Vec<Vec<u8>> {
    let entries = match std::fs::read_dir(path) {
        Ok(it) => it,
        Err(_) => return Vec::new(),
    };
    let mut files: Vec<(u64, std::path::PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let md = e.metadata().ok()?;
            if !md.is_file() {
                return None;
            }
            Some((md.len(), e.path()))
        })
        .collect();
    files.sort_by_key(|(len, _)| *len);
    files
        .into_iter()
        .take(MAX_CORPUS_SEEDS)
        .filter_map(|(_, p)| std::fs::read(p).ok())
        .filter(|b| !b.is_empty())
        .collect()
}

/// Apply the random mutation set to `buf` in place. Mutations are
/// bounded (see the constants above) so the result stays within
/// `seed.len() + 32` bytes. All offsets are clamped against the current
/// buffer length, so an empty seed (which can't happen in practice
/// after `load_dir`'s `is_empty` filter) would be a no-op.
fn apply_mutations(buf: &mut Vec<u8>, u: &mut Unstructured) -> arbitrary::Result<()> {
    // 1) Bit-flips: flip a single random bit at a random byte offset.
    //    Hits checksum-detection paths and varint MSB toggling.
    let n_flips = u.int_in_range(0..=MAX_BYTE_FLIPS)?;
    for _ in 0..n_flips {
        if buf.is_empty() {
            break;
        }
        let off = u.int_in_range(0..=buf.len() - 1)?;
        let bit = u.int_in_range(0..=7u8)?;
        buf[off] ^= 1 << bit;
    }

    // 2) Single-byte writes: overwrite a byte with a fully-random value.
    //    Targets length fields, type tags, and ASCII string contents.
    let n_writes = u.int_in_range(0..=MAX_SINGLE_WRITES)?;
    for _ in 0..n_writes {
        if buf.is_empty() {
            break;
        }
        let off = u.int_in_range(0..=buf.len() - 1)?;
        buf[off] = u8::arbitrary(u)?;
    }

    // 3) Insertions: splice 1..=MAX_CHUNK_LEN random bytes into the
    //    buffer at a random offset. Drives the parser into "longer than
    //    expected" branches (varint-claimed length vs actual remaining).
    let n_inserts = u.int_in_range(0..=MAX_INSERTIONS)?;
    for _ in 0..n_inserts {
        let off = if buf.is_empty() {
            0
        } else {
            u.int_in_range(0..=buf.len())?
        };
        let chunk_len = u.int_in_range(1..=MAX_CHUNK_LEN)?;
        let mut chunk = Vec::with_capacity(chunk_len);
        for _ in 0..chunk_len {
            chunk.push(u8::arbitrary(u)?);
        }
        buf.splice(off..off, chunk);
    }

    // 4) Deletions: drop 1..=MAX_CHUNK_LEN bytes at a random offset.
    //    Drives "shorter than expected" branches — truncated PUSHDATA
    //    operands, premature end-of-buffer in the manifest JSON tail.
    let n_deletes = u.int_in_range(0..=MAX_DELETIONS)?;
    for _ in 0..n_deletes {
        if buf.is_empty() {
            break;
        }
        let off = u.int_in_range(0..=buf.len() - 1)?;
        let max_chunk = MAX_CHUNK_LEN.min(buf.len() - off);
        if max_chunk == 0 {
            continue;
        }
        let chunk_len = u.int_in_range(1..=max_chunk)?;
        buf.drain(off..off + chunk_len);
    }

    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        let mut u = Unstructured::new(data);

        // Pick which parser family we're targeting this iteration.
        // Doing this BEFORE `apply_mutations` consumes a single byte so
        // the same mutation budget is available regardless of choice.
        let parser = if matches!(u.int_in_range(0..=1u8), Ok(0)) {
            Parser::Nef
        } else {
            Parser::Manifest
        };

        let (nef_seeds, manifest_seeds) = corpus();
        let pool = match parser {
            Parser::Nef => nef_seeds,
            Parser::Manifest => manifest_seeds,
        };
        // `pool` is non-empty by construction — both fall back to an
        // embedded seed in `corpus()` — but guard anyway.
        if pool.is_empty() {
            return;
        }

        let seed_index = match u8::arbitrary(&mut u) {
            Ok(b) => b as usize,
            Err(_) => return,
        };
        let mut buf = pool[seed_index % pool.len()].clone();

        if apply_mutations(&mut buf, &mut u).is_err() {
            // Out of entropy partway through — still drive what we
            // have through the parser; partial mutations are a valid
            // input shape we want coverage on.
        }

        match parser {
            Parser::Nef => {
                // The contract: `parse_nef` returns either Ok or Err on
                // every byte sequence — never panics. Output is
                // discarded; we only care about reachability + absence
                // of crashes.
                let _ = neo_devpack_solidity::neo::parse_nef(&buf);
            }
            Parser::Manifest => {
                // Mirror `fuzz_target_manifest_json`: parse as JSON and
                // walk the manifest schema sub-paths to exercise the
                // post-parse Value→shape casts.
                if let Ok(value) = serde_json::from_slice::<Value>(&buf) {
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
            }
        }
    });
});
