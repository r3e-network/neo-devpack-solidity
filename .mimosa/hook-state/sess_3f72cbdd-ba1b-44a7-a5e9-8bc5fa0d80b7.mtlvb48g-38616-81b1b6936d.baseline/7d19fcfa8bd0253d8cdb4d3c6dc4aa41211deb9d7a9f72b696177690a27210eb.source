use super::*;

pub(crate) fn try_lower_type_bound_max(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "max" {
        return None;
    }

    // Solidity uses `type(uint256).max` (and `type(int256).max`) to query numeric
    // bounds. solang-parser represents `type(T)` as a function call to the `type`
    // keyword.
    if let Some(type_arg) = typeof_argument(inner) {
        match type_arg {
            Expression::Type(_, PtType::Uint(bits)) => {
                let mut value = BigInt::one();
                value <<= *bits as usize;
                value -= BigInt::one();
                if let Some(msg) = neovm_integer_limit_warning(&value) {
                    ctx.record_warning(msg);
                }
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                return Some(true);
            }
            Expression::Type(_, PtType::Int(bits)) => {
                if *bits == 0 {
                    ctx.record_error("type(int0).max is invalid");
                    return Some(false);
                }

                let mut value = BigInt::one();
                value <<= (*bits as usize).saturating_sub(1);
                value -= BigInt::one();
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                return Some(true);
            }
            _ => {
                // Compatibility fallback for unknown/UDT type bounds.
                let mut value = BigInt::one();
                value <<= 256usize;
                value -= BigInt::one();
                if let Some(msg) = neovm_integer_limit_warning(&value) {
                    ctx.record_warning(msg);
                }
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                return Some(true);
            }
        }
    }

    // Fallback: tolerate `uint256.max`-style expressions if they appear in the AST.
    if let Expression::Type(_, PtType::Uint(bits)) = inner {
        let mut value = BigInt::one();
        value <<= *bits as usize;
        value -= BigInt::one();
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
        return Some(true);
    }

    if let Expression::Type(_, PtType::Int(bits)) = inner {
        if *bits == 0 {
            ctx.record_error("type(int0).max is invalid");
            return Some(false);
        }

        let mut value = BigInt::one();
        value <<= (*bits as usize).saturating_sub(1);
        value -= BigInt::one();
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
        return Some(true);
    }

    None
}

pub(crate) fn try_lower_type_bound_min(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "min" {
        return None;
    }

    if let Some(type_arg) = typeof_argument(inner) {
        match type_arg {
            Expression::Type(_, PtType::Uint(_)) => {
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                return Some(true);
            }
            Expression::Type(_, PtType::Int(bits)) => {
                if *bits == 0 {
                    ctx.record_error("type(int0).min is invalid");
                    return Some(false);
                }

                let mut value = BigInt::one();
                value <<= (*bits as usize).saturating_sub(1);
                value = -value;
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                return Some(true);
            }
            _ => {
                ctx.record_error("unsupported type(...).min expression");
                return Some(false);
            }
        }
    }

    if let Expression::Type(_, PtType::Uint(_)) = inner {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        return Some(true);
    }

    if let Expression::Type(_, PtType::Int(bits)) = inner {
        if *bits == 0 {
            ctx.record_error("type(int0).min is invalid");
            return Some(false);
        }

        let mut value = BigInt::one();
        value <<= (*bits as usize).saturating_sub(1);
        value = -value;
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
        return Some(true);
    }

    None
}

pub(crate) fn try_lower_type_name(
    inner: &Expression,
    member: &Identifier,
    _ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "name" {
        return None;
    }

    let type_arg = typeof_argument(inner)?;

    let name_str = match type_arg {
        Expression::Variable(id) => id.name.clone(),
        Expression::Type(_, pt_type) => format!("{pt_type}"),
        _ => return None,
    };

    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        name_str.into_bytes(),
    )));
    Some(true)
}

pub(crate) fn try_lower_type_code(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if !matches!(member.name.as_str(), "creationCode" | "runtimeCode") {
        return None;
    }

    // Solidity: `type(C).creationCode` / `type(C).runtimeCode` evaluates to
    // deployment/runtime bytecode of contract `C`. CREATE2 deterministic-
    // address schemes hash these bytes, so a shared empty blob across all
    // contracts collapses every derived address to the same hash. We emit a
    // deterministic NEF-framed envelope whose script uniquely identifies the
    // child contract by name. The envelope is stable across compiles and
    // differs per contract name, so `keccak256(type(A).creationCode) !=
    // keccak256(type(B).creationCode)` — preserving CREATE2 semantics. Neo
    // deployment does not consume this blob (it is an off-Neo compatibility
    // artefact used only for hashing), but it mimics the NEF3 layout so
    // tools that inspect the bytes see a well-formed envelope.
    if let Some(type_arg) = typeof_argument(inner) {
        let contract_name = match type_arg {
            Expression::Variable(id) => id.name.clone(),
            Expression::Type(_, PtType::Address) => "address".to_string(),
            Expression::Type(_, pt_type) => format!("{pt_type}"),
            _ => {
                ctx.record_error_with_suggestion(
                    format!("unsupported type(...).{} expression", member.name),
                    "type(C).creationCode/runtimeCode requires a concrete contract or type name",
                );
                return Some(false);
            }
        };

        // Require compile-time availability: the contract must be visible in
        // the current compilation unit. Cross-file references would require a
        // multi-pass compiler; reject explicitly rather than emit colliding
        // empty bytes.
        if matches!(type_arg, Expression::Variable(_))
            && !ctx.is_contract_type_name(&contract_name)
            && !ctx.is_interface_type_name(&contract_name)
        {
            ctx.record_error_with_suggestion(
                format!(
                    "type({}).{} requires the referenced contract to be declared in the same compilation unit",
                    contract_name, member.name
                ),
                "move the child contract into the same source, or inline its bytecode via a hex literal",
            );
            return Some(false);
        }

        let payload = creation_code_payload(&contract_name, &member.name);
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(payload)));
        return Some(true);
    }

    None
}

/// Build a deterministic NEF3-shaped envelope unique per contract name.
///
/// The layout is a minimal subset of the NEF3 format:
///   - 4 byte magic `NEF3`
///   - 64 byte compiler field (`"neo-devpack-solidity-type-code"` + zero pad)
///   - varstring source identifier (`"creationCode:<name>"` or similar)
///   - reserved 0x00
///   - token count = 0 (varint)
///   - reserved 0x00 0x00
///   - varbytes script = `keccak256(<member>:<name>)` (32 bytes)
///   - checksum = first 4 bytes of sha256(sha256(prefix))
///
/// We inline the construction here rather than calling `crate::neo::build_nef`
/// to keep the IR crate's dependency surface unchanged.
pub(crate) fn creation_code_payload(contract_name: &str, member_name: &str) -> Vec<u8> {
    let mut script_hasher = Keccak256::new();
    script_hasher.update(member_name.as_bytes());
    script_hasher.update(b":");
    script_hasher.update(contract_name.as_bytes());
    let script = script_hasher.finalize();

    let mut buffer: Vec<u8> = Vec::with_capacity(128);
    buffer.extend_from_slice(b"NEF3");
    let compiler = b"neo-devpack-solidity-type-code";
    buffer.extend_from_slice(compiler);
    buffer.extend(std::iter::repeat_n(0u8, 64 - compiler.len()));
    let source = format!("{member_name}:{contract_name}");
    let source_bytes = source.as_bytes();
    // NEF source is varstring; short names always fit in a single byte.
    let source_len = source_bytes.len().min(252);
    buffer.push(source_len as u8);
    buffer.extend_from_slice(&source_bytes[..source_len]);
    buffer.push(0); // reserved
    buffer.push(0); // varint token count = 0
    buffer.extend_from_slice(&[0, 0]); // reserved
                                       // varbytes script (length <= 252 so single-byte varint)
    buffer.push(script.len() as u8);
    buffer.extend_from_slice(&script);
    // Checksum: first 4 bytes of a deterministic hash over all preceding
    // bytes. NEF3 uses double SHA-256; we use double Keccak256 to avoid
    // importing an additional digest dependency. The envelope is purely an
    // off-Neo hashing artefact — tooling that strictly parses the NEF3
    // checksum should treat these as opaque blobs for CREATE2 hashing.
    let mut first = Keccak256::new();
    first.update(&buffer);
    let first_digest = first.finalize();
    let mut second = Keccak256::new();
    second.update(first_digest);
    let digest = second.finalize();
    buffer.extend_from_slice(&digest[..4]);
    buffer
}

pub(crate) fn try_lower_interface_id(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if member.name != "interfaceId" {
        return None;
    }

    if let Some(type_arg) = typeof_argument(inner) {
        if let Expression::Variable(type_name) = type_arg {
            if !ctx.is_interface_type_name(&type_name.name) {
                ctx.record_error_with_suggestion(
                    format!(
                        "type({}).interfaceId is only supported for interface types",
                        type_name.name
                    ),
                    "interfaceId can only be computed for interface definitions, not contracts or other types",
                );
                return Some(false);
            }

            if let Some(interface_id) = ctx.interface_id_for_type(&type_name.name) {
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                    interface_id.to_vec(),
                )));
                return Some(true);
            }

            ctx.record_error_with_suggestion(
                format!("unable to compute interfaceId for '{}'", type_name.name),
                "ensure the interface has at least one function declaration",
            );
            return Some(false);
        }

        ctx.record_error("unsupported type(...).interfaceId expression");
        return Some(false);
    }

    ctx.record_error("interfaceId is only supported as `type(Interface).interfaceId`");
    Some(false)
}
