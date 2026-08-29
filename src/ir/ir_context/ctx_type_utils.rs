use super::*;

pub(crate) fn normalize_solidity_like_type_signature(raw: &str) -> String {
    let compact = raw
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .replace("payable", "");
    let lowered = compact.to_ascii_lowercase();
    match lowered.as_str() {
        "uint" => "uint256".to_string(),
        "int" => "int256".to_string(),
        "byte" => "bytes1".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn value_type_signature(value_type: &ValueType) -> String {
    match value_type {
        ValueType::Integer { signed: true, bits } => format!("int{bits}"),
        ValueType::Integer {
            signed: false,
            bits,
        } => format!("uint{bits}"),
        ValueType::Boolean => "bool".to_string(),
        ValueType::String => "string".to_string(),
        ValueType::Address => "address".to_string(),
        ValueType::ByteArray {
            fixed_len: Some(len),
        } => format!("bytes{len}"),
        ValueType::ByteArray { fixed_len: None } => "bytes".to_string(),
        ValueType::Array(inner) => format!("{}[]", value_type_signature(inner)),
        ValueType::Mapping { key, value } => format!(
            "mapping({}=>{})",
            value_type_signature(key),
            value_type_signature(value)
        ),
        ValueType::Struct { name, .. } => name.to_ascii_lowercase(),
        ValueType::Any => "any".to_string(),
    }
}

/// Task #91 — match a `using X for T` directive target against a receiver
/// signature. The frontend renders `using L for L.Data;` as target
/// `"l.data"`, but `ValueType::Struct { name: "Data" }` normalises to
/// `"data"` (`lookup_struct` strips the qualifier). Fall back to matching
/// by the last `.`-separated segment so storage-pointer struct receivers
/// dispatch correctly.
pub(crate) fn using_target_matches_signature(target: &str, receiver_sig: &str) -> bool {
    target == receiver_sig
        || target
            .rsplit_once('.')
            .is_some_and(|(_, last)| last == receiver_sig)
}

pub(crate) fn is_implicitly_convertible(actual: &ValueType, expected: &ValueType) -> bool {
    match (actual, expected) {
        (_, ValueType::Any) | (ValueType::Any, _) => true,
        (
            ValueType::Integer {
                signed: actual_signed,
                bits: actual_bits,
            },
            ValueType::Integer {
                signed: expected_signed,
                bits: expected_bits,
            },
        ) => actual_signed == expected_signed && actual_bits <= expected_bits,
        (ValueType::Boolean, ValueType::Boolean)
        | (ValueType::String, ValueType::String)
        | (ValueType::Address, ValueType::Address) => true,
        (
            ValueType::ByteArray {
                fixed_len: actual_len,
            },
            ValueType::ByteArray {
                fixed_len: expected_len,
            },
        ) => match (actual_len, expected_len) {
            (_, None) => true,
            (Some(actual), Some(expected)) => actual == expected,
            (None, Some(_)) => false,
        },
        (ValueType::Array(actual_inner), ValueType::Array(expected_inner)) => {
            is_implicitly_convertible(actual_inner, expected_inner)
        }
        (
            ValueType::Mapping {
                key: actual_key,
                value: actual_value,
            },
            ValueType::Mapping {
                key: expected_key,
                value: expected_value,
            },
        ) => {
            is_implicitly_convertible(actual_key, expected_key)
                && is_implicitly_convertible(actual_value, expected_value)
        }
        (
            ValueType::Struct {
                name: actual_name, ..
            },
            ValueType::Struct {
                name: expected_name,
                ..
            },
        ) => actual_name.eq_ignore_ascii_case(expected_name),
        _ => false,
    }
}
