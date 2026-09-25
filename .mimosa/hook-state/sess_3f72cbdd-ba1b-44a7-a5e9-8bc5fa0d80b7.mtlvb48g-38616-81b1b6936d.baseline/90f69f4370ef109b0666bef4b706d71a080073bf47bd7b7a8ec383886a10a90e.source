use super::*;

/// Maximum nested-struct resolution depth.
///
/// Solidity allows self-referencing structs (`struct Node { Node[] children; }`)
/// and mutually-recursive pairs (`struct A { B[] bs; } struct B { A[] as_; }`),
/// both of which are legal Solidity sources but would otherwise drive
/// `NeoType::from_solidity` into unbounded recursion when expanding
/// struct fields. Depth-capping the recursion at 64 is a generous
/// bound (deeper than any hand-written production schema) that
/// converts the cycle into a `NeoType::Any` leaf instead of a stack
/// overflow. Discovered via `fuzz/corpus/fuzz_target_1/.quarantined_seed_pathological_recursive_types.sol`.
const MAX_STRUCT_RESOLUTION_DEPTH: u32 = 64;

impl NeoType {
    pub fn from_solidity(
        ty: &str,
        structs: &[StructTypeMetadata],
        enums: &[EnumTypeMetadata],
        contract_types: &[String],
    ) -> Result<Self, TypeParseError> {
        let empty: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        Self::from_solidity_bounded(ty, structs, enums, contract_types, &empty, 0)
    }

    fn from_solidity_bounded(
        ty: &str,
        structs: &[StructTypeMetadata],
        enums: &[EnumTypeMetadata],
        contract_types: &[String],
        type_aliases: &std::collections::HashMap<String, String>,
        depth: u32,
    ) -> Result<Self, TypeParseError> {
        if depth >= MAX_STRUCT_RESOLUTION_DEPTH {
            // Cycle or pathologically deep type — break the recursion.
            // Returning `Any` preserves compilation for legal recursive
            // struct shapes (the outer pass still encodes the struct by
            // name; field-type introspection beyond this depth is lost
            // but rare in practice and always lossy in EVM compatibility
            // terms anyway).
            return Ok(NeoType::Any);
        }
        let ty = strip_data_location(ty);
        // NEP `Any` placeholder: the devpack declares `type Any is bytes;`
        // purely so Solidity sources can spell the NEP-17/NEP-11 `data`
        // parameter, but the manifest must expose it as `Any` per the NEP
        // specs. Resolve it BEFORE user-defined-value-type alias expansion so
        // the `bytes` desugaring does not demote it to ByteArray. The special
        // case is scoped to an underlying type of exactly `bytes` (or no
        // alias at all), so an unrelated `type Any is bytes32;` keeps its
        // alias semantics.
        if ty.eq_ignore_ascii_case("any") {
            match type_aliases.iter().find(|(alias, _)| alias.as_str() == ty) {
                None => return Ok(NeoType::Any),
                Some((_, underlying)) if underlying.trim() == "bytes" => {
                    return Ok(NeoType::Any);
                }
                Some(_) => {}
            }
        }
        // Resolve user-defined value-type aliases before recursing. A field
        // declared as `Slot0 slot0` (where `type Slot0 is bytes32;`) should
        // recurse with `"bytes32"` so the inner parse hits the built-in
        // `bytes32` branch. Doing this here — not only at the
        // `from_solidity_with_aliases` top entry — covers field/array
        // element/mapping value positions where the alias appears nested
        // inside another type expression.
        let resolved = type_aliases
            .iter()
            .find(|(alias, _)| alias.as_str() == ty)
            .map(|(_, underlying)| underlying.clone());
        let ty: &str = match resolved.as_deref() {
            Some(underlying) => underlying,
            None => ty,
        };
        let lower = ty.to_ascii_lowercase();

        // Arrays must be detected before scalar prefixes like `uint`/`int`.
        // Support dynamic arrays (`T[]`) and fixed-size arrays (`T[n]`).
        if lower.ends_with("[]") {
            let inner = &ty[..ty.len() - 2];
            let element = NeoType::from_solidity_bounded(
                inner,
                structs,
                enums,
                contract_types,
                type_aliases,
                depth + 1,
            )?;
            return Ok(NeoType::Array(Box::new(element), None));
        }

        if lower.ends_with(']') {
            if let Some(stripped) = lower.strip_suffix(']') {
                if let Some((_inner_ty, size_str)) = stripped.rsplit_once('[') {
                    if !size_str.is_empty() && size_str.chars().all(|c| c.is_ascii_digit()) {
                        let inner_original = &ty[..ty.len() - size_str.len() - 2];
                        let element = NeoType::from_solidity_bounded(
                            inner_original,
                            structs,
                            enums,
                            contract_types,
                            type_aliases,
                            depth + 1,
                        )?;
                        // Preserve the fixed length so the ABI signature is
                        // `T[N]` (correct selector) rather than `T[]`.
                        let fixed_len = size_str.parse::<usize>().ok();
                        return Ok(NeoType::Array(Box::new(element), fixed_len));
                    }
                }
            }
        }

        if let Some(rest) = lower.strip_prefix("uint") {
            if rest.is_empty() {
                return Ok(NeoType::Integer {
                    signed: false,
                    bits: 256,
                });
            }
            if rest.chars().all(|c| c.is_ascii_digit()) {
                let bits = rest.parse::<u16>().map_err(|_| {
                    TypeParseError::Unsupported(format!("invalid uint bit-width: '{ty}'"))
                })?;
                if bits == 0 || bits > 256 || !bits.is_multiple_of(8) {
                    return Err(TypeParseError::Unsupported(format!(
                        "uint bit-width must be a multiple of 8 in 8..=256, got {bits}"
                    )));
                }
                return Ok(NeoType::Integer {
                    signed: false,
                    bits,
                });
            }
            // remainder is not purely numeric (e.g. "interface" matched via "int" prefix) — fall through
        }

        if let Some(rest) = lower.strip_prefix("int") {
            if rest.is_empty() {
                return Ok(NeoType::Integer {
                    signed: true,
                    bits: 256,
                });
            }
            if rest.chars().all(|c| c.is_ascii_digit()) {
                let bits = rest.parse::<u16>().map_err(|_| {
                    TypeParseError::Unsupported(format!("invalid int bit-width: '{ty}'"))
                })?;
                if bits == 0 || bits > 256 || !bits.is_multiple_of(8) {
                    return Err(TypeParseError::Unsupported(format!(
                        "int bit-width must be a multiple of 8 in 8..=256, got {bits}"
                    )));
                }
                return Ok(NeoType::Integer { signed: true, bits });
            }
            // remainder is not purely numeric (e.g. "interface") — fall through
        }

        if lower == "bool" {
            return Ok(NeoType::Boolean);
        }

        if lower == "string" {
            return Ok(NeoType::String);
        }

        if lower == "address" || lower == "address payable" || lower == "hash160" {
            return Ok(NeoType::Address);
        }
        // NOTE: `bytes20` is NOT an address. Solidity treats `bytes20` and
        // `address` as distinct types with DIFFERENT ABI selectors (`f(bytes20)`
        // != `f(address)`); mapping it to `Address` produced the wrong selector /
        // interfaceId. It falls through to the `bytesN` branch below →
        // `ByteArray { fixed_len: Some(20) }` → canonical `bytes20`.

        if lower == "bytes" {
            return Ok(NeoType::ByteArray { fixed_len: None });
        }

        if let Some(hex_suffix) = lower.strip_prefix("bytes") {
            if let Ok(len) = hex_suffix.parse::<u16>() {
                // Solidity only defines `bytes1`..=`bytes32`; reject `bytes0` and
                // `bytes33`+ rather than silently accepting an invalid fixed-bytes
                // type (which would mis-encode/mis-pad downstream).
                if len == 0 || len > 32 {
                    return Err(TypeParseError::Unsupported(format!(
                        "bytesN width must be 1..=32, got {len}"
                    )));
                }
                return Ok(NeoType::ByteArray {
                    fixed_len: Some(len),
                });
            }
        }

        if lower.starts_with("mapping") {
            return parse_mapping_type_bounded(
                ty,
                structs,
                enums,
                contract_types,
                type_aliases,
                depth + 1,
            );
        }

        if let Some(struct_meta) = lookup_struct(ty, structs) {
            let mut fields = Vec::new();
            for field in &struct_meta.fields {
                let field_type = NeoType::from_solidity_bounded(
                    &field.ty,
                    structs,
                    enums,
                    contract_types,
                    type_aliases,
                    depth + 1,
                )
                .unwrap_or(NeoType::Any);
                fields.push(StructFieldType {
                    name: field.name.clone(),
                    ty: Box::new(field_type),
                });
            }
            return Ok(NeoType::Struct {
                name: struct_meta.name.clone(),
                fields,
            });
        }

        if lookup_enum(ty, enums).is_some() {
            // Solidity encodes enums as unsigned integers (typically uint8).
            return Ok(NeoType::Integer {
                signed: false,
                bits: 8,
            });
        }

        if lower == "any" {
            return Ok(NeoType::Any);
        }

        // Fixed-point types are not supported on NeoVM.
        if lower.starts_with("fixed") || lower.starts_with("ufixed") {
            return Err(TypeParseError::FixedPoint(ty.to_string()));
        }

        // Contract/interface-typed values are represented as Neo UInt160 addresses.
        let mut candidate = ty.trim();
        // Strip common Solidity prefixes (`contract Foo`, `interface Foo`) if present.
        let candidate_lower = candidate.to_ascii_lowercase();
        for prefix in ["contract ", "interface ", "library "] {
            if candidate_lower.starts_with(prefix) {
                candidate = candidate[prefix.len()..].trim_start();
                break;
            }
        }
        if let Some((_, last)) = candidate.rsplit_once('.') {
            candidate = last.trim();
        }
        if contract_types
            .iter()
            .any(|name| name.eq_ignore_ascii_case(candidate))
        {
            return Ok(NeoType::Address);
        }

        Err(TypeParseError::Unsupported(ty.to_string()))
    }

    /// Resolve user-defined value type aliases before parsing.
    ///
    /// If `ty` matches a key in `type_aliases`, the underlying type is used instead.
    /// This makes `type Price is uint256` transparent to the type system.
    /// Aliases are also threaded into the recursive `from_solidity_bounded`
    /// walk so they get resolved when the alias appears as a struct field,
    /// array element, or mapping value (not just at the top level).
    pub fn from_solidity_with_aliases(
        ty: &str,
        structs: &[StructTypeMetadata],
        enums: &[EnumTypeMetadata],
        contract_types: &[String],
        type_aliases: &std::collections::HashMap<String, String>,
    ) -> Result<Self, TypeParseError> {
        Self::from_solidity_bounded(ty, structs, enums, contract_types, type_aliases, 0)
    }
}

fn strip_data_location(ty: &str) -> &str {
    let mut trimmed = ty.trim();
    for suffix in [" storage", " memory", " calldata"] {
        if let Some(stripped) = trimmed.strip_suffix(suffix) {
            trimmed = stripped.trim_end();
        }
    }
    trimmed
}

fn lookup_struct<'a>(
    ty: &str,
    structs: &'a [StructTypeMetadata],
) -> Option<&'a StructTypeMetadata> {
    fn strip_qualifier(raw: &str) -> &str {
        let candidate = raw.trim_start_matches("struct ").trim();
        if let Some((_, last)) = candidate.rsplit_once('.') {
            last.trim()
        } else {
            candidate
        }
    }
    fn strip_struct_keyword(raw: &str) -> &str {
        raw.trim_start_matches("struct ").trim()
    }

    let qualified_name = strip_struct_keyword(ty);
    let short_name = strip_qualifier(ty);

    // Phase 1: exact (qualified) match. When two structs share the same short
    // name but live in different scopes (V4 PoolOperation.SwapParams vs
    // Pool.SwapParams), a qualified reference like "Pool.SwapParams" must
    // resolve to its exact namesake. We qualify nested struct names in
    // `frontend_convert::convert_contract`, so internal references inside the
    // owning library/contract have already been rewritten to the qualified
    // form before they reach this lookup.
    if let Some(found) = structs
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(qualified_name))
    {
        return Some(found);
    }

    // Phase 2: short-name fallback. Used for legacy references and for any
    // unqualified spelling that survives the rewrite pass. If multiple
    // structs share the short name, the first one wins — matches the
    // historical behaviour.
    structs
        .iter()
        .find(|s| strip_qualifier(&s.name).eq_ignore_ascii_case(short_name))
}

fn lookup_enum<'a>(ty: &str, enums: &'a [EnumTypeMetadata]) -> Option<&'a EnumTypeMetadata> {
    fn normalize(raw: &str) -> &str {
        let mut candidate = raw.trim_start_matches("enum ").trim();
        if let Some((_, last)) = candidate.rsplit_once('.') {
            candidate = last.trim();
        }
        candidate
    }

    let name = normalize(ty);
    enums
        .iter()
        .find(|e| normalize(&e.name).eq_ignore_ascii_case(name))
}

fn parse_mapping_type_bounded(
    ty: &str,
    structs: &[StructTypeMetadata],
    enums: &[EnumTypeMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
    depth: u32,
) -> Result<NeoType, TypeParseError> {
    // Expect "mapping(<key> => <value>)"
    let after_keyword = ty
        .trim_start()
        .strip_prefix("mapping")
        .ok_or_else(|| TypeParseError::Unsupported(ty.to_string()))?;

    let mut rest = after_keyword.trim_start();
    if !rest.starts_with('(') {
        return Err(TypeParseError::Unsupported(ty.to_string()));
    }
    rest = &rest[1..]; // skip '('

    // Find matching ')' — note: `paren_depth` here shadowed the outer
    // recursion-depth parameter `depth` before the bounded-parse rewrite,
    // so it's renamed to keep the two distinct.
    let mut paren_depth = 1isize;
    let mut idx = 0usize;
    let bytes = rest.as_bytes();
    while idx < bytes.len() && paren_depth > 0 {
        match bytes[idx] as char {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }
        if paren_depth == 0 {
            break;
        }
        idx += 1;
    }

    if paren_depth != 0 {
        return Err(TypeParseError::Unsupported(ty.to_string()));
    }

    let inner = &rest[..idx];
    let remaining = rest[idx + 1..].trim();
    if !remaining.is_empty() {
        return Err(TypeParseError::Unsupported(ty.to_string()));
    }

    // Split inner on top-level "=>"
    let mut split_index = None;
    let mut paren_depth = 0isize;
    let mut chars = inner.char_indices();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' => paren_depth += 1,
            ')' | ']' => paren_depth -= 1,
            '=' if paren_depth == 0 => {
                if let Some((_, next_char)) = chars.clone().next() {
                    if next_char == '>' {
                        split_index = Some(i);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    let split_idx = split_index.ok_or_else(|| TypeParseError::Unsupported(ty.to_string()))?;
    let key_str = inner[..split_idx].trim();
    let value_str = inner[split_idx + 2..].trim();

    if key_str.is_empty() || value_str.is_empty() {
        return Err(TypeParseError::Unsupported(ty.to_string()));
    }

    let key = parse_mapping_component_bounded(
        key_str,
        structs,
        enums,
        contract_types,
        type_aliases,
        depth,
    )?;

    // Solidity requires mapping keys to be elementary types (integers, bool,
    // address, string, bytes, enums, contract types). Arrays, structs, and
    // nested mappings are not valid because they lack a deterministic hash.
    match &key {
        NeoType::Array(..) | NeoType::Struct { .. } | NeoType::Mapping { .. } => {
            return Err(TypeParseError::Unsupported(format!(
                "invalid mapping key type '{key_str}'; only elementary types are allowed"
            )));
        }
        _ => {}
    }

    let value = parse_mapping_component_bounded(
        value_str,
        structs,
        enums,
        contract_types,
        type_aliases,
        depth,
    )?;

    Ok(NeoType::Mapping {
        key: Box::new(key),
        value: Box::new(value),
    })
}

fn parse_mapping_component_bounded(
    raw: &str,
    structs: &[StructTypeMetadata],
    enums: &[EnumTypeMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
    depth: u32,
) -> Result<NeoType, TypeParseError> {
    let trimmed = raw.trim();

    if let Ok(parsed) =
        NeoType::from_solidity_bounded(trimmed, structs, enums, contract_types, type_aliases, depth)
    {
        return Ok(parsed);
    }

    if let Some(stripped) = strip_named_mapping_component(trimmed) {
        if let Ok(parsed) = NeoType::from_solidity_bounded(
            stripped,
            structs,
            enums,
            contract_types,
            type_aliases,
            depth,
        ) {
            return Ok(parsed);
        }
    }

    Ok(NeoType::Any)
}

fn strip_named_mapping_component(raw: &str) -> Option<&str> {
    let trimmed = raw.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    // Consume the trailing identifier candidate (the optional mapping key/value name).
    let mut idx = trimmed.len();
    let mut saw_ident = false;
    while idx > 0 {
        let ch = trimmed[..idx].chars().next_back()?;
        if ch.is_ascii_alphanumeric() || ch == '_' {
            saw_ident = true;
            idx -= ch.len_utf8();
            continue;
        }
        break;
    }

    if !saw_ident || idx == trimmed.len() {
        return None;
    }

    // The optional name must be separated by whitespace from the type.
    let before = &trimmed[..idx];
    if !before
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace())
    {
        return None;
    }

    let stripped = before.trim_end();
    if stripped.is_empty() {
        return None;
    }

    Some(stripped)
}
