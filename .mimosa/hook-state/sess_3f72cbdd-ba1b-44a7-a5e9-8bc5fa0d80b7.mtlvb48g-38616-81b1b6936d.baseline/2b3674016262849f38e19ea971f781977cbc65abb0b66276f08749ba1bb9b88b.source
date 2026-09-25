use super::*;

pub(crate) fn validate_return_types(
    metadata: &ContractMetadata,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for function in &metadata.methods {
        let is_exposed = matches!(
            function.visibility,
            VisibilityKind::Public | VisibilityKind::External
        );

        if let Some(ret_param) = function.return_parameters.first() {
            if ret_param.neo_type.is_none() && is_exposed {
                let lower_ty = ret_param.ty.to_ascii_lowercase();
                if lower_ty.starts_with("fixed") || lower_ty.starts_with("ufixed") {
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "function '{}' return type '{}' is not supported on NeoVM",
                            function.name, ret_param.ty
                        ))
                        .with_suggestion(
                            "use scaled integer arithmetic instead (e.g., multiply by 10^18 for 18 decimal places)"
                        ),
                    );
                } else if !return_type_supported(&ret_param.ty)
                    && !is_devpack_iterator_type(&ret_param.ty)
                {
                    // Task #94 — accept parenthesised tuple return types
                    // (e.g. `((uint256, uint256), uint256)`) when every inner
                    // component is individually supported. `return_type_supported`
                    // handles the recursive walk with paren-depth tracking.
                    //
                    // Gap `nep11` — the devpack `Syscalls.Iterator` handle type
                    // is also accepted: returning it from a public method leaves
                    // the raw NeoVM iterator stack item as the return value
                    // (manifest returntype `InteropInterface`), which is how
                    // NEP-11 `tokensOf`/`tokens` are specified.
                    diagnostics.push(
                        Diagnostic::error(format!(
                            "function '{}' return type '{}' is unsupported",
                            function.name, ret_param.ty
                        ))
                        .with_code("UNSUPPORTED_RETURN_TYPE"),
                    );
                }
            }

            let supported = match ret_param.neo_type.as_ref() {
                Some(NeoType::Any) | None => {
                    let lowered = ret_param.ty.to_ascii_lowercase();
                    return_type_supported(&ret_param.ty)
                        || lowered.starts_with("syscalls.")
                        || lowered.starts_with("storage.iterator")
                        || is_devpack_iterator_type(&ret_param.ty)
                }
                Some(_) => true,
            };

            if !supported {
                diagnostics.push(
                    Diagnostic::warning(format!(
                        "function '{}' returns '{}', which may not map cleanly to Neo manifest types",
                        function.name, ret_param.ty
                    ))
                    .with_code("RETURN_TYPE_UNMAPPED"),
                );
            }
        }

        for ret_param in &function.return_parameters {
            if let Some(storage) = &ret_param.storage {
                if storage == "storage" {
                    diagnostics.push(
                        Diagnostic::warning(format!(
                            "function '{}' return value '{}' uses 'storage' data location (treated as Any)",
                            function.name, ret_param.ty
                        ))
                        .with_code("INVALID_STORAGE_RETURN"),
                    );
                }
            }
        }
    }
}

/// Gap `nep11` — true iff `ty` names the devpack NeoVM iterator handle type
/// (`Syscalls.Iterator` / `Storage.Iterator` / bare `Iterator`). The builtin
/// helper libraries are never struct-merged into user contracts (see
/// `is_builtin_library_name` in `solidity_analyse.rs`), so this type always
/// reaches validation with `neo_type == None` and must be allowed by name.
fn is_devpack_iterator_type(ty: &str) -> bool {
    let lowered = ty.trim().to_ascii_lowercase();
    matches!(
        lowered.as_str(),
        "iterator" | "syscalls.iterator" | "storage.iterator"
    )
}

/// Task #94 — returns `true` iff `s` looks like a parenthesised tuple type:
/// it starts with `(`, ends with `)`, and the matching close-paren is at the
/// very end (i.e. the outer parens wrap the whole string).
fn is_tuple_type(s: &str) -> bool {
    let trimmed = s.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'(' || bytes[bytes.len() - 1] != b')' {
        return false;
    }
    // Verify the opening `(` matches the final `)` and isn't closed earlier.
    let mut depth: i32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 && i != bytes.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

/// Task #94 — split a tuple body (the contents between outer `(` and `)`) on
/// top-level commas, respecting nested parens and bracket depth.
fn split_tuple_components(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let bytes = body.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(body[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < body.len() {
        let last = body[start..].trim();
        if !last.is_empty() {
            parts.push(last.to_string());
        }
    }
    parts
}

/// Task #94 — recursively decide whether a Solidity type string is accepted
/// by the return-type validator. Handles the leaf scalar/array/mapping/bytes
/// families directly, and for parenthesised tuples recurses on every top-
/// level component.
fn return_type_supported(ty: &str) -> bool {
    let trimmed = ty.trim();
    if is_tuple_type(trimmed) {
        let inner = &trimmed[1..trimmed.len() - 1];
        let parts = split_tuple_components(inner);
        if parts.is_empty() {
            return false;
        }
        return parts.iter().all(|p| return_type_supported(p));
    }
    let lowered = trimmed.to_ascii_lowercase();
    lowered.starts_with("uint")
        || lowered.starts_with("int")
        || lowered == "bool"
        || lowered == "string"
        || lowered == "address"
        || lowered == "bytes"
        || lowered == "bytearray"
        || lowered.starts_with("bytes")
        || lowered.ends_with("[]")
        || lowered.starts_with("mapping")
}
