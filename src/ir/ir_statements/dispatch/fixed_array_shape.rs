use super::*;

/// Task #185 — parse a Solidity return type string like `uint[3][2]` into
/// its nested fixed-size dimensions in OUTER-FIRST order (matches Solidity
/// source convention: `uint[3][2]` is an array of 2 arrays of 3 uints, so
/// this returns `vec![2, 3]`).
///
/// Returns `None` when:
/// - the type isn't a nested fixed-size array (contains `[]` dynamic, or is
///   a scalar/struct),
/// - any dimension is non-numeric or zero,
/// - the leaf element type is not a static 32-byte ABI primitive (uintN/
///   intN/address/bool/bytesN); nested dynamics (string/bytes/T[]) or
///   structs would require recursive tail encoding and fall back to the
///   generic `AbiEncode(1)` path.
///
/// Mutates nothing. Whitespace-tolerant. Ignores trailing " memory" /
/// " calldata" / " storage" markers that `format!("{}", pt::Type)` may emit
/// on parameters.
pub(crate) fn parse_nested_fixed_array_shape(ty_str: &str) -> Option<Vec<usize>> {
    // Strip any storage-location suffix and surrounding whitespace.
    let cleaned: String = ty_str
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    let base = cleaned
        .strip_suffix("memory")
        .or_else(|| cleaned.strip_suffix("calldata"))
        .or_else(|| cleaned.strip_suffix("storage"))
        .unwrap_or(&cleaned);

    // Walk from the rightmost `[N]` inward, collecting dimensions. Solidity
    // source order `uint[3][2]` parses as `(uint[3])[2]`: the RIGHTMOST
    // bracket pair is the OUTERMOST dimension (here `[2]` → outer dim 2),
    // with each inner `[N]` one step closer to the leaf scalar type. Because
    // we walk right-to-left (outer first) and push in that order, the final
    // vector is ALREADY in outer-first order — no reversal needed.
    let mut remaining = base;
    let mut dims_outer_first: Vec<usize> = Vec::new();
    while let Some(close_idx) = remaining.rfind(']') {
        if close_idx + 1 != remaining.len() {
            // Trailing characters after the last `]` would mean something
            // like `uint[3][2]foo` — not a parseable array type.
            return None;
        }
        let open_idx = remaining.rfind('[')?;
        if open_idx >= close_idx {
            return None;
        }
        let dim_text = &remaining[open_idx + 1..close_idx];
        if dim_text.is_empty() {
            // Dynamic array `T[]` — cannot flat-encode at compile time.
            return None;
        }
        let dim: usize = dim_text.parse().ok()?;
        if dim == 0 {
            return None;
        }
        dims_outer_first.push(dim);
        remaining = &remaining[..open_idx];
    }

    if dims_outer_first.is_empty() {
        return None;
    }

    // Leaf type must be a static 32-byte ABI primitive. Anything else
    // (string / bytes / T[] / structs / mappings) needs dynamic-tail
    // encoding and falls outside this flat path.
    if !is_static_32_byte_leaf_type(remaining) {
        return None;
    }

    Some(dims_outer_first)
}

pub(crate) fn array_leaf_static_value_type(value_type: &ValueType) -> Option<ValueType> {
    let mut current = value_type;
    while let ValueType::Array(inner) = current {
        current = inner.as_ref();
    }
    if is_static_abi_type_value(current) {
        Some(current.clone())
    } else {
        None
    }
}

/// Task #185 — predicate for scalar Solidity types whose EVM canonical
/// encoding is a single 32-byte BE-padded slot. Matches the signature used
/// by `abi_is_dynamic` / `abi_pad32_be` in the runtime's StdLib handler so
/// the flat-array return path only fires for cases where the runtime's
/// static fast-path produces correct output.
pub(crate) fn is_static_32_byte_leaf_type(ty: &str) -> bool {
    let compact = ty.trim();
    match compact {
        "uint" | "int" | "address" | "bool" => true,
        _ => {
            if let Some(width) = compact
                .strip_prefix("uint")
                .or_else(|| compact.strip_prefix("int"))
            {
                if let Ok(bits) = width.parse::<u32>() {
                    return bits > 0 && bits <= 256 && bits.is_multiple_of(8);
                }
            }
            if let Some(width) = compact.strip_prefix("bytes") {
                if let Ok(len) = width.parse::<u32>() {
                    return (1..=32).contains(&len);
                }
            }
            false
        }
    }
}
