use super::*;

pub(crate) fn manifest_type_name(ty: ManifestType) -> &'static str {
    match ty {
        ManifestType::Integer => "Integer",
        ManifestType::Boolean => "Boolean",
        ManifestType::String => "String",
        ManifestType::Hash160 => "Hash160",
        ManifestType::Hash256 => "Hash256",
        ManifestType::ByteArray => "ByteArray",
        ManifestType::Array => "Array",
        ManifestType::Map => "Map",
        ManifestType::Any => "Any",
    }
}

pub(crate) fn value_type_satisfies_manifest_type(
    actual: &ValueType,
    expected: ManifestType,
) -> Option<bool> {
    if expected == ManifestType::Any {
        return Some(true);
    }

    match actual {
        ValueType::Any => None,
        ValueType::Integer { .. } => Some(expected == ManifestType::Integer),
        ValueType::Boolean => Some(expected == ManifestType::Boolean),
        ValueType::String => Some(matches!(
            expected,
            ManifestType::String | ManifestType::ByteArray
        )),
        ValueType::Address => Some(matches!(
            expected,
            ManifestType::Hash160 | ManifestType::ByteArray
        )),
        ValueType::ByteArray { fixed_len } => match expected {
            ManifestType::Hash160 => Some(matches!(fixed_len, Some(20))),
            ManifestType::Hash256 => Some(matches!(fixed_len, Some(32))),
            ManifestType::ByteArray => Some(true),
            _ => Some(false),
        },
        ValueType::Array(_) => Some(expected == ManifestType::Array),
        ValueType::Mapping { .. } => Some(expected == ManifestType::Map),
        ValueType::Struct { .. } => Some(expected == ManifestType::Array),
    }
}

pub(crate) fn extract_string_literal(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(parts) => {
            Some(String::from_utf8_lossy(&string_literal_bytes(parts)).to_string())
        }
        _ => None,
    }
}

pub(crate) fn extract_abi_encode_args(expr: &Expression) -> Option<&[Expression]> {
    let Expression::FunctionCall(_, func, args) = expr else {
        return None;
    };

    let Expression::MemberAccess(_, inner, member) = func.as_ref() else {
        return None;
    };
    let Expression::Variable(base) = inner.as_ref() else {
        return None;
    };

    if base.name == "abi"
        && (member.name == "encode" || member.name == "encodePacked" || member.name == "encodeCall")
    {
        Some(args.as_slice())
    } else {
        None
    }
}

/// Task #124 — true iff a `ValueType` is a STATIC EVM ABI type (all-static
/// structs fold to per-field 32-byte slots, no offset/length header). Mirrors
/// the `is_static_abi_type` predicate used by `abi_decode_expected_static_bytes`
/// but operates on the IR `ValueType` rather than a solang `PtType`.
pub(crate) fn is_static_abi_type_value(ty: &ValueType) -> bool {
    match ty {
        ValueType::Integer { .. } => true,
        ValueType::Boolean => true,
        ValueType::Address => true,
        ValueType::ByteArray { fixed_len: Some(_) } => true,
        // Unknown-width bytes arrays (`bytes memory`), strings, arrays, and
        // mappings are dynamic and must keep the head+tail layout.
        ValueType::ByteArray { fixed_len: None } => false,
        ValueType::String => false,
        ValueType::Array(_) => false,
        ValueType::Mapping { .. } => false,
        // Nested structs: conservative — keep at single-level and treat as
        // dynamic (so the outer encode uses the existing StackItem::Array
        // fallback path for nested structs). Nested-struct flattening is
        // a follow-up if harnesses need it.
        ValueType::Struct { .. } => false,
        ValueType::Any => false,
    }
}

/// Predicate matching dynamic ABI value types whose `abi.decode` lowering
/// is implemented by [`emit_abi_decode_dynamic_top_level`] /
/// [`emit_abi_decode_dynamic_tuple_member`].
///
/// Currently: `string`, `bytes` (= `ByteArray { fixed_len: None }`), and
/// `T[]` where `T` is a static ABI type (uint/int/address/bool/bytesN, or
/// an all-static struct occupying `abi_static_slot_count(T)` slots per
/// element) OR a supported dynamic type — i.e. nested-dynamic shapes
/// (`string[]`, `bytes[]`, `T[][]`) walked via
/// [`emit_abi_decode_nested_array_tail_runtime`]. The recursion mirrors the
/// ENCODE side's `abi_dynamic_value_type_is_supported` exactly, so every
/// canonically-encoded shape round-trips through `abi.decode`.
pub(crate) fn abi_dynamic_decode_value_type_is_supported(value_type: &ValueType) -> bool {
    match value_type {
        ValueType::ByteArray { fixed_len: None } | ValueType::String => true,
        ValueType::Array(element_type) => {
            abi_static_slot_count(element_type).is_some()
                || abi_dynamic_decode_value_type_is_supported(element_type)
        }
        // Dynamic struct: decodable when every field is static or a supported
        // dynamic shape (mirrors the encode-side predicate).
        ValueType::Struct { fields, .. } => {
            fields.iter().any(|f| abi_value_type_is_dynamic(&f.ty))
                && fields.iter().all(|f| {
                    abi_static_slot_count(&f.ty).is_some()
                        || abi_dynamic_decode_value_type_is_supported(&f.ty)
                })
        }
        _ => false,
    }
}

pub(crate) fn abi_static_slot_count(value_type: &ValueType) -> Option<usize> {
    match value_type {
        ValueType::Struct { fields, .. }
            if !fields.is_empty()
                && fields
                    .iter()
                    .all(|field| is_static_abi_type_value(&field.ty)) =>
        {
            Some(fields.len())
        }
        other if is_static_abi_type_value(other) => Some(1),
        _ => None,
    }
}

pub(crate) fn abi_value_type_is_dynamic(value_type: &ValueType) -> bool {
    match value_type {
        ValueType::ByteArray { fixed_len: None } | ValueType::String | ValueType::Array(_) => true,
        // A struct is dynamic iff any field is (recursively) dynamic — solc's
        // tuple rule. An all-static struct stays static (a flat run of inline
        // head slots); a struct with a string/bytes/array/dynamic-struct field
        // needs the head+tail layout.
        ValueType::Struct { fields, .. } => fields.iter().any(|f| abi_value_type_is_dynamic(&f.ty)),
        _ => false,
    }
}

pub(crate) fn abi_dynamic_value_type_is_supported(value_type: &ValueType) -> bool {
    match value_type {
        ValueType::ByteArray { fixed_len: None } | ValueType::String => true,
        // A `T[]` is encodable when its elements are either fixed-width
        // (head-only, e.g. `uint256[]`) or themselves a supported dynamic
        // shape (head+tail, e.g. `string[]`, `bytes[]`, `uint256[][]`). The
        // recursion mirrors `emit_abi_dynamic_tail_for_value_type`.
        ValueType::Array(element_type) => {
            abi_static_slot_count(element_type).is_some()
                || abi_dynamic_value_type_is_supported(element_type)
        }
        // Dynamic struct (has ≥1 dynamic field): encodable when EVERY field is
        // itself encodable (static or supported-dynamic). Encoded as a tuple.
        ValueType::Struct { fields, .. } => {
            fields.iter().any(|f| abi_value_type_is_dynamic(&f.ty))
                && fields.iter().all(|f| {
                    abi_static_slot_count(&f.ty).is_some()
                        || abi_dynamic_value_type_is_supported(&f.ty)
                })
        }
        _ => false,
    }
}
