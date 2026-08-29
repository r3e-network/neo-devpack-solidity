use super::*;

pub(crate) fn string_literal_bytes(parts: &[PtStringLiteral]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for part in parts {
        // Solang's lexer hands us the raw source bytes between the quotes,
        // including any `\n`, `\t`, `\xHH`, `\uNNNN`, ... escape sequences
        // verbatim. Per the Solidity spec (and C-string semantics) these
        // must be lowered into their single-byte / UTF-8 equivalents here;
        // otherwise `"line1\n"` flows into the VM as the two literal bytes
        // 0x5C 0x6E instead of the newline 0x0A it represents.
        bytes.extend_from_slice(&unescape_solidity_string(&part.string));
    }
    bytes
}

/// Process Solidity / C-style escape sequences in a string literal body.
///
/// Handles:
/// - `\n`  -> 0x0A
/// - `\t`  -> 0x09
/// - `\r`  -> 0x0D
/// - `\"`  -> 0x22
/// - `\'`  -> 0x27
/// - `\\`  -> 0x5C
/// - `\0`  -> 0x00
/// - `\xHH` -> single hex byte (exactly 2 hex digits)
/// - `\uNNNN` -> 4-hex-digit Unicode code point, encoded as UTF-8
///
/// Unknown escape sequences (e.g. `\z`) are left verbatim as two bytes
/// (backslash + char) so we don't silently drop data — the analyser
/// rejects malformed escapes before lowering.
pub(crate) fn unescape_solidity_string(raw: &str) -> Vec<u8> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c != b'\\' || i + 1 >= bytes.len() {
            out.push(c);
            i += 1;
            continue;
        }
        let next = bytes[i + 1];
        match next {
            b'n' => {
                out.push(0x0A);
                i += 2;
            }
            b't' => {
                out.push(0x09);
                i += 2;
            }
            b'r' => {
                out.push(0x0D);
                i += 2;
            }
            b'"' => {
                out.push(0x22);
                i += 2;
            }
            b'\'' => {
                out.push(0x27);
                i += 2;
            }
            b'\\' => {
                out.push(0x5C);
                i += 2;
            }
            b'0' => {
                out.push(0x00);
                i += 2;
            }
            b'x' if i + 3 < bytes.len() => {
                let hi = (bytes[i + 2] as char).to_digit(16);
                let lo = (bytes[i + 3] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(((h << 4) | l) as u8);
                    i += 4;
                } else {
                    // Malformed \x escape — emit verbatim.
                    out.push(c);
                    i += 1;
                }
            }
            b'u' if i + 5 < bytes.len() => {
                let d0 = (bytes[i + 2] as char).to_digit(16);
                let d1 = (bytes[i + 3] as char).to_digit(16);
                let d2 = (bytes[i + 4] as char).to_digit(16);
                let d3 = (bytes[i + 5] as char).to_digit(16);
                if let (Some(a), Some(b), Some(c_), Some(d)) = (d0, d1, d2, d3) {
                    let cp = (a << 12) | (b << 8) | (c_ << 4) | d;
                    if let Some(ch) = char::from_u32(cp) {
                        let mut buf = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        i += 6;
                    } else {
                        // Surrogate / invalid scalar — emit verbatim.
                        out.push(c);
                        i += 1;
                    }
                } else {
                    out.push(c);
                    i += 1;
                }
            }
            _ => {
                // Unknown escape — preserve verbatim (backslash + next char).
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

pub(crate) fn extract_signature_string(expr: &Expression) -> Option<String> {
    match expr {
        Expression::StringLiteral(parts) => {
            Some(String::from_utf8_lossy(&string_literal_bytes(parts)).to_string())
        }
        Expression::FunctionCall(_, func, args) => {
            if args.len() == 1 {
                match func.as_ref() {
                    Expression::Type(_, _) => extract_signature_string(&args[0]),
                    Expression::Variable(id) if id.name == "bytes" || id.name == "string" => {
                        extract_signature_string(&args[0])
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn resolve_selector_method_name(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<String> {
    match expr {
        Expression::Variable(identifier) => {
            if let Some(state_index) = ctx.state_index_map.get(&identifier.name).copied() {
                if let Some(meta) = ctx.state_metadata(state_index) {
                    if meta.is_constant {
                        if let Some(initializer) = meta.initializer.as_ref() {
                            return resolve_selector_method_name(initializer, ctx);
                        }
                    }
                }
            }
            None
        }
        Expression::MemberAccess(_, inner, member) => {
            if member.name == "selector" {
                match inner.as_ref() {
                    Expression::MemberAccess(_, _, function_name)
                        if !function_name.name.trim().is_empty() =>
                    {
                        return Some(function_name.name.clone());
                    }
                    // Custom-error pattern: `ErrorName.selector`
                    Expression::Variable(function_name)
                        if !function_name.name.trim().is_empty() =>
                    {
                        return Some(function_name.name.clone());
                    }
                    _ => {}
                }
            }
            None
        }
        Expression::FunctionCall(_, func, args) => {
            // Ignore simple casts like bytes4(...) around selector expressions.
            if matches!(func.as_ref(), Expression::Type(_, _)) && args.len() == 1 {
                return resolve_selector_method_name(&args[0], ctx);
            }

            if let Expression::Variable(id) = func.as_ref() {
                if id.name == "keccak256" && args.len() == 1 {
                    if let Some(signature) = extract_signature_string(&args[0]) {
                        if let Some(name) = crate::utils::method_name_from_signature(&signature) {
                            return Some(name);
                        }
                    }
                }
            }

            None
        }
        _ => None,
    }
}
