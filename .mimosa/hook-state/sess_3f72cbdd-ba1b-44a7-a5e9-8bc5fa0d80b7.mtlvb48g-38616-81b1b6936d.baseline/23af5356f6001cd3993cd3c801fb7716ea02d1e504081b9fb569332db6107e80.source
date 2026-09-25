//! Shared Utility Functions
//!
//! Common utilities used across the compiler.

use std::collections::HashMap;

/// Convert a Solidity type string to its canonical form for ABI encoding.
///
/// Handles special cases like `struct Foo` and `enum Bar` by extracting
/// just the type name.
///
/// # Examples
/// ```
/// use neo_devpack_solidity::utils::canonical_param_type;
/// assert_eq!(canonical_param_type("uint256"), "uint256");
/// assert_eq!(canonical_param_type("struct MyStruct"), "MyStruct");
/// assert_eq!(canonical_param_type("enum MyEnum"), "MyEnum");
/// ```
pub fn canonical_param_type(ty: &str) -> String {
    let mut parts = ty.split_whitespace();
    match parts.next() {
        Some("struct" | "enum") => parts.next().unwrap_or_default().to_string(),
        Some(first) => first.to_string(),
        None => String::new(),
    }
}

/// Simple canonical type extraction (first word only).
///
/// Use this when struct/enum special handling is not needed.
pub fn canonical_param_type_simple(ty: &str) -> String {
    ty.split_whitespace().next().unwrap_or_default().to_string()
}

/// Extract the method name from a `name(args)` signature string.
///
/// Returns `None` if the signature is empty after trimming — a missing
/// name is treated as a malformed input, not a successful empty-string
/// match. Used by the four call sites that accept Solidity-style
/// `encodeWithSignature(...)` / `keccak256(...)` arguments and need the
/// bare method name without the parameter list. Centralising this here
/// removes five near-identical inline implementations that previously
/// drifted apart in subtle ways (some returned `""`, some returned
/// the whole signature, some used `split_once` etc).
///
/// # Examples
/// ```
/// use neo_devpack_solidity::utils::method_name_from_signature;
/// assert_eq!(method_name_from_signature("transfer(address,uint256)"), Some("transfer".to_string()));
/// assert_eq!(method_name_from_signature("name"), Some("name".to_string()));
/// assert_eq!(method_name_from_signature(""), None);
/// ```
pub fn method_name_from_signature(signature: &str) -> Option<String> {
    let name = signature.split('(').next().unwrap_or(signature).trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Task #106 — Convert a Solidity type string to its EVM-canonical ABI
/// signature form, expanding struct types into parenthesized tuple
/// representations.
///
/// Unlike `canonical_param_type`, which returns the bare struct NAME, this
/// version recursively expands struct field types so selectors match the
/// Ethereum spec:
///   * `struct P { uint256 a; bool b; }` → `"(uint256,bool)"`
///   * `struct Outer { P inner; address who; }` → `"((uint256,bool),address)"`
///
/// `struct_fields` maps struct NAME to its ordered `(name, ty_string)` list.
/// Enum types canonicalize to `uint8` per Solidity's `abi.encode` rules.
/// Unknown user-defined types fall back to the bare name (same as
/// `canonical_param_type`).
pub fn canonical_param_type_with_structs(
    ty: &str,
    struct_fields: &HashMap<String, Vec<(String, String)>>,
) -> String {
    // Strip Solidity data-location keywords that never participate in the
    // canonical signature.
    let trimmed = ty.trim();
    let without_location = trimmed
        .replace(" memory", "")
        .replace(" calldata", "")
        .replace(" storage", "");
    let t = without_location.trim();

    // Peel a trailing array suffix (`[]` or `[N]`), canonicalize the ELEMENT
    // type, then re-append the suffix. Without this, an array-of-struct like
    // `P[]` tokenizes to the whole `"P[]"` (not a struct-map key) and passes
    // through verbatim, producing a non-conformant selector instead of the
    // canonical `(uint256,bool)[]`.
    if t.ends_with(']') {
        if let Some(open) = t.rfind('[') {
            let (element, suffix) = t.split_at(open);
            let inner = canonical_param_type_with_structs(element.trim(), struct_fields);
            return format!("{inner}{suffix}");
        }
    }

    // Handle `struct Name` → expand to `(field_types)`.
    // Handle `enum Name` → uint8.
    // Handle bare `Name` that matches a known struct → expand.
    // Otherwise → passthrough (plain scalar or unknown user-defined type).
    let mut parts = t.split_whitespace();
    match parts.next() {
        Some("struct") => {
            let name = parts.next().unwrap_or_default();
            if name.is_empty() {
                return String::new();
            }
            // Guard against infinite recursion on self-referential struct names
            // by tracking a visited set.
            let mut visited = std::collections::HashSet::new();
            expand_struct_canonical(name, struct_fields, &mut visited)
        }
        Some("enum") => "uint8".to_string(),
        Some(first) => {
            // Bare-name path: solang-parser emits `"P"` (no `struct` prefix)
            // for struct parameters in some code paths. If `first` names a
            // known struct in the map, expand it to the tuple form.
            if struct_fields.contains_key(first) {
                let mut visited = std::collections::HashSet::new();
                expand_struct_canonical(first, struct_fields, &mut visited)
            } else {
                first.to_string()
            }
        }
        None => String::new(),
    }
}

fn expand_struct_canonical(
    name: &str,
    struct_fields: &HashMap<String, Vec<(String, String)>>,
    visited: &mut std::collections::HashSet<String>,
) -> String {
    if !visited.insert(name.to_string()) {
        // Recursive reference — fall back to the bare name to avoid infinite
        // recursion. Solidity itself forbids recursive value-type structs, but
        // keep the fallback defensive.
        return name.to_string();
    }
    let Some(fields) = struct_fields.get(name) else {
        // Unknown struct — fall back to bare name (Task #65 compat).
        visited.remove(name);
        return name.to_string();
    };
    let field_sigs: Vec<String> = fields
        .iter()
        .map(|(_, field_ty)| {
            // Strip location keywords on fields too.
            let tt = field_ty
                .replace(" memory", "")
                .replace(" calldata", "")
                .replace(" storage", "");
            let tt = tt.trim();
            let mut parts = tt.split_whitespace();
            match parts.next() {
                Some("struct") => {
                    let sub_name = parts.next().unwrap_or_default();
                    if sub_name.is_empty() {
                        String::new()
                    } else {
                        expand_struct_canonical(sub_name, struct_fields, visited)
                    }
                }
                Some("enum") => "uint8".to_string(),
                Some(first) => {
                    if struct_fields.contains_key(first) {
                        expand_struct_canonical(first, struct_fields, visited)
                    } else {
                        first.to_string()
                    }
                }
                None => String::new(),
            }
        })
        .collect();
    visited.remove(name);
    format!("({})", field_sigs.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_param_type() {
        assert_eq!(canonical_param_type("uint256"), "uint256");
        assert_eq!(canonical_param_type("struct MyStruct"), "MyStruct");
        assert_eq!(canonical_param_type("enum MyEnum"), "MyEnum");
        assert_eq!(canonical_param_type("address payable"), "address");
        assert_eq!(canonical_param_type(""), "");
    }

    #[test]
    fn test_canonical_param_type_simple() {
        assert_eq!(canonical_param_type_simple("uint256"), "uint256");
        assert_eq!(canonical_param_type_simple("struct MyStruct"), "struct");
        assert_eq!(canonical_param_type_simple("address payable"), "address");
    }

    #[test]
    fn test_canonical_param_type_with_structs_flat() {
        let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        map.insert(
            "P".to_string(),
            vec![
                ("a".to_string(), "uint256".to_string()),
                ("b".to_string(), "bool".to_string()),
            ],
        );
        assert_eq!(
            canonical_param_type_with_structs("struct P", &map),
            "(uint256,bool)"
        );
        assert_eq!(
            canonical_param_type_with_structs("struct P memory", &map),
            "(uint256,bool)"
        );
        assert_eq!(
            canonical_param_type_with_structs("uint256", &map),
            "uint256"
        );
    }

    #[test]
    fn test_canonical_param_type_with_structs_nested() {
        let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        map.insert(
            "Inner".to_string(),
            vec![("x".to_string(), "uint256".to_string())],
        );
        map.insert(
            "Outer".to_string(),
            vec![
                ("inner".to_string(), "struct Inner".to_string()),
                ("who".to_string(), "address".to_string()),
            ],
        );
        assert_eq!(
            canonical_param_type_with_structs("struct Outer", &map),
            "((uint256),address)"
        );
    }

    #[test]
    fn test_canonical_param_type_with_structs_unknown_fallback() {
        let map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        // Unknown struct falls back to bare name.
        assert_eq!(
            canonical_param_type_with_structs("struct Unknown", &map),
            "Unknown"
        );
    }

    #[test]
    fn test_canonical_param_type_with_structs_enum() {
        let map: HashMap<String, Vec<(String, String)>> = HashMap::new();
        assert_eq!(
            canonical_param_type_with_structs("enum MyEnum", &map),
            "uint8"
        );
    }
}
