use super::*;

/// Task #181 — recursively render a `ValueType` as its EVM-canonical ABI
/// signature fragment. Used by `revert_arg_canonical_type` to expand struct
/// args into their parenthesised tuple form (`(address,uint256)`) instead of
/// the bare struct-name or fallback `bytes` token.
///
/// Mirrors `utils::canonical_param_type_with_structs` but operates on the
/// IR's `ValueType` tree (which already carries resolved field types) so
/// the revert-selector path doesn't need the solang-level string→fields map
/// that `solidity_analyse.rs` built for Task #106.
pub(crate) fn value_type_canonical_abi(ty: &ValueType) -> String {
    match ty {
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
        ValueType::Array(inner) => format!("{}[]", value_type_canonical_abi(inner)),
        ValueType::Struct { fields, .. } => {
            let parts: Vec<String> = fields
                .iter()
                .map(|f| value_type_canonical_abi(&f.ty))
                .collect();
            format!("({})", parts.join(","))
        }
        // Mapping args cannot actually appear in an ABI signature; fall back
        // to `bytes` for robustness (same policy as the legacy surface).
        ValueType::Mapping { .. } | ValueType::Any => "bytes".to_string(),
    }
}

/// Task #181 — resolve the struct type of a revert-arg expression.
///
/// `infer_type_from_expression` handles the easy cases (Variable bound to a
/// struct param/local/state, MemberAccess into a struct). The revert-arg
/// harness also hits STRUCT-CONSTRUCTOR CALLS — `revert NotAuthorized(Actor(addr, 5))` —
/// where the callee is a bare `Variable("Actor")` naming a struct and the
/// 2-arg function-call shape has no inference rule. Look that up via
/// `resolve_struct_type_by_name` (shared with the struct-constructor
/// lowering) so the selector and flatten paths see the same fields.
pub(crate) fn resolve_struct_type_for_revert_arg(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<ValueType> {
    let mut current = expr;
    while let Expression::Parenthesis(_, inner) = current {
        current = inner.as_ref();
    }
    if let Some(ty) = infer_type_from_expression(current, ctx) {
        if matches!(ty, ValueType::Struct { .. }) {
            return Some(ty);
        }
    }
    if let Expression::FunctionCall(_, func, _) = current {
        let name = match func.as_ref() {
            Expression::Variable(id) => Some(id.name.clone()),
            Expression::MemberAccess(_, _, id) => Some(id.name.clone()),
            _ => None,
        }?;
        return resolve_struct_type_by_name(ctx, &name);
    }
    None
}

/// Canonical-type signature for an argument passed to a custom-error revert.
///
/// Task #27 (compiler slice) — the EVM-style payload is
/// `keccak256("ErrorName(t1,t2,...)")[0..4] || abi.encode(args)`. For the
/// selector to match what Ethereum tooling expects, each `ti` must use
/// Solidity's canonical ABI form (`uint256`, not `uint`; `bytes32`, not a
/// struct display string).
///
/// We infer the type from the argument expression; for the common
/// integer-literal case this defaults to `uint256`, which matches the
/// Solidity language rule that number literals implicitly convert to the
/// error's declared parameter type. Unknown types degrade to `bytes` — a
/// conservative default that yields a stable (though non-matching)
/// selector so the payload still flows through the runtime.
///
/// Task #181 — struct args (both Variable references and constructor calls
/// like `Actor(addr, role)`) now expand to their canonical tuple form
/// `(t1,t2,...)` rather than falling through to `bytes`. Extends the Task
/// #106 struct-expansion pattern from `abi.encodeCall` to custom-error
/// reverts so `revert NotAuthorized(Actor(addr, 5))` emits the spec
/// selector `keccak256("NotAuthorized((address,uint256))")[..4]`.
pub(crate) fn revert_arg_canonical_type(expr: &Expression, ctx: &LoweringContext) -> String {
    // Struct-typed args first: cover both the Variable path (handled by
    // infer) AND the constructor-call path (`Actor(...)`) which has no
    // inference rule but resolves via `resolve_struct_type_by_name`.
    if let Some(struct_ty) = resolve_struct_type_for_revert_arg(expr, ctx) {
        return value_type_canonical_abi(&struct_ty);
    }
    match infer_type_from_expression(expr, ctx) {
        Some(ValueType::Integer { signed: true, bits }) => format!("int{bits}"),
        Some(ValueType::Integer {
            signed: false,
            bits,
        }) => format!("uint{bits}"),
        Some(ValueType::Boolean) => "bool".to_string(),
        Some(ValueType::String) => "string".to_string(),
        Some(ValueType::Address) => "address".to_string(),
        Some(ValueType::ByteArray {
            fixed_len: Some(len),
        }) => format!("bytes{len}"),
        Some(ValueType::ByteArray { fixed_len: None }) => "bytes".to_string(),
        Some(ValueType::Array(inner)) => format!("{}[]", value_type_canonical_abi(&inner)),
        _ => "bytes".to_string(),
    }
}

/// Task #181 — lower a single revert arg and, when the arg is a struct with
/// all-static fields, flatten it into per-field stack items so the downstream
/// `AbiEncode` builtin sees N scalar args (one per field) instead of a single
/// `StackItem::Array` that the runtime classifier would treat as dynamic and
/// wrap in Task #121's `offset || length || elements` layout.
///
/// Returns the number of stack items pushed (1 for non-struct args, N for
/// flattened struct args) so the caller can thread the correct `arg_count`
/// into `CallBuiltin { AbiEncode, arg_count }`. Returns 0 on lowering
/// failure; callers treat that as a fatal condition and fall back to the
/// bare error-name string path.
///
/// Mirrors the Task #124 `try_flatten_struct_arg_for_abi_encode` pattern but
/// (a) uses `resolve_struct_type_for_revert_arg` so struct-constructor calls
/// are covered, and (b) materialises the struct into a single temp local
/// before indexing so the constructor runs ONCE per arg (the Task #124
/// version re-lowers the expression per field — fine for bare Variable
/// references, wasteful for constructor calls that build the struct array
/// on every index).
pub(crate) fn lower_and_flatten_revert_arg(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> usize {
    if let Some(struct_ty) = resolve_struct_type_for_revert_arg(expr, ctx) {
        if let ValueType::Struct { fields, .. } = &struct_ty {
            if !fields.is_empty() && fields.iter().all(|f| is_static_abi_type_value(&f.ty)) {
                if !lower_expression(expr, ctx, instructions) {
                    return 0;
                }
                let tmp_id = ctx.next_label();
                let tmp_local = ctx
                    .allocate_local(format!("__revert_struct_{tmp_id}"), Some(struct_ty.clone()));
                instructions.push(Instruction::StoreLocal(tmp_local));
                for i in 0..fields.len() {
                    instructions.push(Instruction::LoadLocal(tmp_local));
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(i as u64),
                    )));
                    instructions.push(Instruction::ArrayGet);
                }
                return fields.len();
            }
        }
    }
    if !lower_expression(expr, ctx, instructions) {
        return 0;
    }
    1
}

/// Canonicalize a DECLARED error-parameter Solidity type string into its
/// EVM-canonical ABI signature form, resolving enums (→ `uint8`) and
/// struct names (→ tuple form) against the lowering context.
///
/// This is the declared-signature counterpart of
/// `revert_arg_canonical_type`: where that function INFERS a type from a
/// revert-site argument expression, this one normalizes the type string
/// written in the `error Name(t1 a, t2 b);` declaration — which is what
/// Solidity actually hashes for the selector
/// (`keccak256("Name(uint8)")`, not `keccak256("Name(uint256)")`, for
/// `error Name(uint8 code); revert Name(1);`).
pub(crate) fn declared_error_param_canonical(raw: &str, ctx: &LoweringContext) -> String {
    let cleaned = raw
        .replace(" memory", "")
        .replace(" calldata", "")
        .replace(" storage", "")
        .replace(" payable", "");
    let compact: String = cleaned
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    // Canonicalize the base type and keep any array suffix verbatim
    // (`uint[]` → `uint256[]`, `uint[3]` → `uint256[3]`).
    if let Some(idx) = compact.find('[') {
        let base = declared_error_param_canonical(&compact[..idx], ctx);
        return format!("{}{}", base, &compact[idx..]);
    }
    match compact.as_str() {
        "uint" => "uint256".to_string(),
        "int" => "int256".to_string(),
        "byte" => "bytes1".to_string(),
        "payable" => "address".to_string(),
        other => {
            // Enums hash as uint8 per the Solidity ABI spec.
            if ctx.enum_variant_map.contains_key(other) {
                return "uint8".to_string();
            }
            // Struct params expand to their canonical tuple form
            // `(t1,t2,...)` — same policy as the Task #181 inference path.
            let name = other.rsplit('.').next().unwrap_or(other);
            if let Some(struct_ty) = resolve_struct_type_by_name(ctx, name) {
                return value_type_canonical_abi(&struct_ty);
            }
            other.to_string()
        }
    }
}

/// Resolve the canonical ABI argument-type list for a custom-error
/// revert/require site.
///
/// Prefers the DECLARED `error` signature recorded in the lowering
/// context's registry (matching what solc hashes); falls back to
/// expression-type inference only when the error name was never declared
/// (e.g. a qualified `Lib.Error` whose library declaration is not merged)
/// or the argument count differs from the declaration.
pub(crate) fn revert_error_arg_types(
    error_name: &str,
    args: &[Expression],
    ctx: &LoweringContext,
) -> Vec<String> {
    let declared: Option<Vec<String>> = ctx
        .error_signature(error_name)
        .filter(|sig| sig.param_types.len() == args.len())
        .map(|sig| sig.param_types.clone());
    match declared {
        Some(declared_types) => declared_types
            .iter()
            .map(|ty| declared_error_param_canonical(ty, ctx))
            .collect(),
        None => args
            .iter()
            .map(|arg| revert_arg_canonical_type(arg, ctx))
            .collect(),
    }
}

/// Compute the 4-byte keccak selector for a custom error signature.
///
/// Signature form: `Name(t1,t2,...)` per Solidity's ABI spec.
pub(crate) fn revert_error_selector(name: &str, arg_types: &[String]) -> [u8; 4] {
    let signature = format!("{}({})", name, arg_types.join(","));
    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    [digest[0], digest[1], digest[2], digest[3]]
}

/// Build the canonical `Error(string)` revert envelope for a compile-time
/// string literal without routing through the on-chain `StdLib.abiEncode`
/// helper.
///
/// Layout:
/// - 4 bytes: selector = keccak256("Error(string)")[..4]
/// - 32 bytes: offset = 0x20
/// - 32 bytes: string length
/// - N bytes: UTF-8 string data
/// - padding: zero bytes to the next 32-byte boundary
pub(crate) fn error_string_literal_envelope(literal_bytes: &[u8]) -> Vec<u8> {
    let selector = revert_error_selector("Error", &["string".to_string()]);
    let padded_len = literal_bytes.len().div_ceil(32) * 32;
    let mut out = Vec::with_capacity(4 + 32 + 32 + padded_len);
    out.extend_from_slice(&selector);

    let mut offset_slot = [0u8; 32];
    offset_slot[31] = 0x20;
    out.extend_from_slice(&offset_slot);

    let mut len_slot = [0u8; 32];
    len_slot[24..].copy_from_slice(&(literal_bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(&len_slot);

    out.extend_from_slice(literal_bytes);
    out.resize(4 + 32 + 32 + padded_len, 0);
    out
}

/// True when a revert/require MESSAGE argument is string-typed: an inferred
/// `string` expression, a string literal, or a `string.concat(...)` call.
/// Shared predicate between `lower_revert_statement` and `lower_require`
/// so both wrap dynamic string messages in the same `Error(string)`
/// envelope.
pub(crate) fn revert_message_arg_is_string(arg: &Expression, ctx: &LoweringContext) -> bool {
    matches!(
        infer_type_from_expression(arg, ctx),
        Some(ValueType::String)
    ) || matches!(arg, Expression::StringLiteral(_))
        || matches!(
            arg,
            Expression::FunctionCall(_, func, _)
                if matches!(
                    func.as_ref(),
                    Expression::MemberAccess(_, inner, member)
                        if member.name == "concat"
                            && matches!(
                                inner.as_ref(),
                                Expression::Type(_, solang_parser::pt::Type::String)
                            )
                )
        )
}

/// Emit the EVM-canonical `Error(string)` revert envelope
/// `keccak256("Error(string)")[..4] || abi.encode(msg)` for a DYNAMIC
/// (non-literal) string message expression, followed by `Throw`.
///
/// Shared by `revert(msg)` and `require(cond, msg)` (Task #131 follow-up)
/// so both produce the payload shape that `catch Error(string)` selector
/// guards and Ethereum tooling expect — previously `require` with a
/// non-literal message threw the bare string bytes, so
/// `try this.f() catch Error(string)` silently missed it.
///
/// Returns `false` (leaving `instructions` unchanged) when the message
/// expression cannot be lowered; callers fall back to their legacy paths.
pub(crate) fn emit_error_string_envelope_throw(
    arg: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let selector = revert_error_selector("Error", &["string".to_string()]);
    let pre_len = instructions.len();
    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
        selector.to_vec(),
    )));
    if let Some(ok) =
        lower_abi_encode_args_direct_from_slice(std::slice::from_ref(arg), ctx, instructions)
    {
        if ok {
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
            instructions.push(Instruction::Throw);
            return true;
        }
    }
    instructions.truncate(pre_len);

    if lower_expression(arg, ctx, instructions) {
        let mut arg_instrs = instructions.split_off(pre_len);
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector.to_vec(),
        )));
        instructions.append(&mut arg_instrs);
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::AbiEncode,
            arg_count: 1,
        });
        instructions.push(Instruction::CallBuiltin {
            builtin: BuiltinCall::BytesConcat,
            arg_count: 2,
        });
        instructions.push(Instruction::Throw);
        return true;
    }
    instructions.truncate(pre_len);
    false
}

pub(crate) fn lower_revert_statement(
    ident: Option<&solang_parser::pt::IdentifierPath>,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(path) = ident {
        let name = path
            .identifiers
            .last()
            .map(|id| id.name.clone())
            .unwrap_or_default();

        // Task #27 (compiler slice) — emit the EVM-canonical revert payload
        // `keccak256("Name(t1,...)")[0..4] || abi.encode(args)` instead of
        // dropping the args and pushing the bare error name. The runtime
        // `THROW` captures this byte string verbatim into
        // `ExecutionResult.return_data`, giving callers the same surface
        // Ethereum tooling expects for `revert Name(args...)`.
        //
        // The selector is computed from the DECLARED `error` signature when
        // available (see `revert_error_arg_types`) — inferring from the
        // argument expressions yielded e.g. `keccak("E(uint256)")` for
        // `error E(uint8); revert E(1);`.
        let arg_types = revert_error_arg_types(&name, args, ctx);
        let selector = revert_error_selector(&name, &arg_types);

        let direct_static_path = args.iter().all(|arg| is_direct_static_revert_arg(arg, ctx));
        if direct_static_path {
            let pre_len = instructions.len();
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                selector.to_vec(),
            )));
            let mut success = true;
            for arg in args {
                if !lower_direct_static_revert_arg_slot(arg, ctx, instructions) {
                    success = false;
                    break;
                }
            }
            if success {
                if !args.is_empty() {
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::BytesConcat,
                        arg_count: args.len() + 1,
                    });
                }
                instructions.push(Instruction::Throw);
                return true;
            }
            instructions.truncate(pre_len);
        }

        let pre_len = instructions.len();
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector.to_vec(),
        )));
        if args.is_empty() {
            instructions.push(Instruction::Throw);
            return true;
        }
        if let Some(ok) = lower_abi_encode_args_direct_from_slice(args, ctx, instructions) {
            if ok {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: 2,
                });
                instructions.push(Instruction::Throw);
                return true;
            }
            instructions.truncate(pre_len);
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                name.into_bytes(),
            )));
            instructions.push(Instruction::Throw);
            return true;
        }
        instructions.truncate(pre_len);

        // Task #181 — struct args must be flattened into per-field stack
        // items (Task #124 pattern) so the AbiEncode builtin emits the
        // canonical 2-slot packed struct head (`addr || role`) instead of
        // the dynamic-array `offset || count || elements` shape the runtime
        // classifier produces for a `StackItem::Array`. Track `flat_count`
        // separately from `args.len()` so the `AbiEncode` call sees the
        // flattened slot count.
        let pre_len = instructions.len();
        let mut flat_count = 0usize;
        let mut success = true;
        for arg in args {
            let pushed = lower_and_flatten_revert_arg(arg, ctx, instructions);
            if pushed == 0 {
                success = false;
                break;
            }
            flat_count += pushed;
        }

        if !success {
            // Fall back to the legacy error-name string on lowering failure
            // so we still surface *something* on the error path.
            instructions.truncate(pre_len);
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                name.into_bytes(),
            )));
            instructions.push(Instruction::Throw);
            return true;
        }

        // Rewind the argument pushes so we can emit selector, then args, in
        // the correct order for AbiEncode + BytesConcat.
        let mut arg_instrs = instructions.split_off(pre_len);
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector.to_vec(),
        )));
        instructions.append(&mut arg_instrs);
        if flat_count == 0 {
            // `revert ErrorName()` — payload is just the 4-byte selector.
        } else {
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::AbiEncode,
                arg_count: flat_count,
            });
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
        }
        instructions.push(Instruction::Throw);
        return true;
    }

    // `revert("msg")` — EVM convention is to push
    // `Error(string)` selector (0x08c379a0) || abi.encode(msg). We follow
    // the same shape so tooling can decode the revert reason uniformly.
    if args.len() == 1 {
        if let Expression::StringLiteral(parts) = &args[0] {
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                error_string_literal_envelope(&string_literal_bytes(parts)),
            )));
            instructions.push(Instruction::Throw);
            return true;
        }
        if revert_message_arg_is_string(&args[0], ctx)
            && emit_error_string_envelope_throw(&args[0], ctx, instructions)
        {
            return true;
        }
        if lower_expression(&args[0], ctx, instructions) {
            instructions.push(Instruction::Throw);
            return true;
        }
    } else {
        for arg in args {
            if lower_expression(arg, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
        }
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::Null));
    instructions.push(Instruction::Throw);
    true
}

pub(crate) fn lower_revert_named_args(
    ident: Option<&solang_parser::pt::IdentifierPath>,
    args: &[solang_parser::pt::NamedArgument],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(path) = ident {
        let name = path
            .identifiers
            .last()
            .map(|id| id.name.clone())
            .unwrap_or_default();

        // Task #27 (compiler slice) — named-args form mirrors the positional
        // custom-error path: `selector(4) || abi.encode(args)`.
        //
        // solang preserves SOURCE order for named arguments, so
        // `revert E2({b: 2, a: 1})` arrives as [b, a] — the ABI encoding
        // (and the hashed signature) must instead follow DECLARATION order.
        // When the declared `error` signature is known, map each named
        // argument to its declared parameter slot and lower in that order;
        // otherwise keep the legacy source-order behavior. NOTE: reordering
        // also reorders evaluation of the argument expressions — acceptable
        // on a revert path, where the frame is unwinding anyway.
        let mut ordered_args: Vec<solang_parser::pt::NamedArgument> = args.to_vec();
        let mut declared_types: Option<Vec<String>> = None;
        if let Some(sig) = ctx.error_signature(&name).cloned() {
            if sig.param_names.len() == args.len()
                && sig.param_names.iter().all(|param| param.is_some())
            {
                let mut reordered = Vec::with_capacity(args.len());
                for param_name in sig.param_names.iter().flatten() {
                    match args.iter().find(|arg| &arg.name.name == param_name) {
                        Some(arg) => reordered.push(arg.clone()),
                        None => {
                            reordered.clear();
                            break;
                        }
                    }
                }
                if reordered.len() == args.len() {
                    ordered_args = reordered;
                    declared_types = Some(sig.param_types.clone());
                }
            }
        }
        let arg_types: Vec<String> = match &declared_types {
            Some(declared) => declared
                .iter()
                .map(|ty| declared_error_param_canonical(ty, ctx))
                .collect(),
            None => ordered_args
                .iter()
                .map(|arg| revert_arg_canonical_type(&arg.expr, ctx))
                .collect(),
        };
        let selector = revert_error_selector(&name, &arg_types);

        // Task #181 — named-args mirror of the positional custom-error
        // flatten. See `lower_revert_statement` for rationale.
        let direct_args: Vec<Expression> =
            ordered_args.iter().map(|arg| arg.expr.clone()).collect();
        let direct_pre_len = instructions.len();
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector.to_vec(),
        )));
        if direct_args.is_empty() {
            instructions.push(Instruction::Throw);
            return true;
        }
        if let Some(ok) = lower_abi_encode_args_direct_from_slice(&direct_args, ctx, instructions) {
            if ok {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: 2,
                });
                instructions.push(Instruction::Throw);
                return true;
            }
            instructions.truncate(direct_pre_len);
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                name.into_bytes(),
            )));
            instructions.push(Instruction::Throw);
            return true;
        }
        instructions.truncate(direct_pre_len);

        let pre_len = instructions.len();
        let mut flat_count = 0usize;
        let mut success = true;
        for arg in &ordered_args {
            let pushed = lower_and_flatten_revert_arg(&arg.expr, ctx, instructions);
            if pushed == 0 {
                success = false;
                break;
            }
            flat_count += pushed;
        }

        if !success {
            instructions.truncate(pre_len);
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                name.into_bytes(),
            )));
            instructions.push(Instruction::Throw);
            return true;
        }

        let mut arg_instrs = instructions.split_off(pre_len);
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            selector.to_vec(),
        )));
        instructions.append(&mut arg_instrs);
        if flat_count > 0 {
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::AbiEncode,
                arg_count: flat_count,
            });
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::BytesConcat,
                arg_count: 2,
            });
        }
        instructions.push(Instruction::Throw);
        return true;
    }

    for arg in args {
        if lower_expression(&arg.expr, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::Null));
    instructions.push(Instruction::Throw);
    true
}
