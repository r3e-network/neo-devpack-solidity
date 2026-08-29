use super::*;

pub(crate) fn infer_literal_array_element_type(elements: &[Expression]) -> ValueType {
    if elements.is_empty() {
        // Empty array literals (`[]`) have no elements to infer from.
        // Callers should use assignment-target context when available.
        return ValueType::Any;
    }

    let mut inferred: Option<ValueType> = None;
    for expr in elements {
        let candidate = match literal_from_expression(expr) {
            Some(LiteralValue::Boolean(_)) => Some(ValueType::Boolean),
            Some(LiteralValue::Integer(_)) => Some(ValueType::Integer {
                signed: false,
                bits: 256,
            }),
            Some(LiteralValue::String(_)) => Some(ValueType::String),
            Some(LiteralValue::ByteArray(_)) => Some(ValueType::ByteArray { fixed_len: None }),
            Some(LiteralValue::Address(_)) => Some(ValueType::Address),
            Some(LiteralValue::Null) | None => None,
        };

        if let Some(ty) = candidate {
            if let Some(ref prev) = inferred {
                if *prev != ty {
                    // Mixed element types — fall back to Any.
                    return ValueType::Any;
                }
            } else {
                inferred = Some(ty);
            }
        }
        // Skip unparseable elements; keep searching for a concrete type.
    }

    inferred.unwrap_or(ValueType::Any)
}

pub(crate) fn builtin_struct_type(base: &str, member: &str) -> Option<ValueType> {
    use crate::storage_key::compute_state_slot;

    pub(crate) fn mk_struct(name: &str, fields: Vec<(&str, ValueType)>) -> ValueType {
        ValueType::Struct {
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(field_name, ty)| StructField {
                    name: field_name.to_string(),
                    ty,
                    key: compute_state_slot(&format!("{name}::{field_name}")),
                })
                .collect(),
        }
    }

    let u256 = ValueType::Integer {
        signed: false,
        bits: 256,
    };

    match (base, member) {
        ("NativeCalls", "NeoCandidate") => Some(mk_struct(
            "NativeCalls.NeoCandidate",
            vec![
                ("publicKey", ValueType::ByteArray { fixed_len: None }),
                ("votes", u256.clone()),
            ],
        )),
        ("NativeCalls", "AccountState") => Some(mk_struct(
            "NativeCalls.AccountState",
            vec![
                ("balance", u256.clone()),
                ("balanceHeight", u256.clone()),
                ("voteTo", ValueType::ByteArray { fixed_len: None }),
                ("lastGasPerVote", u256),
            ],
        )),
        ("NativeCalls", "ContractState") => Some(mk_struct(
            "NativeCalls.ContractState",
            vec![
                ("hash", ValueType::Address),
                ("nef", ValueType::ByteArray { fixed_len: None }),
                ("manifest", ValueType::ByteArray { fixed_len: None }),
                ("updateCounter", u256),
            ],
        )),
        ("NativeCalls", "WhitelistedContract") => Some(mk_struct(
            "NativeCalls.WhitelistedContract",
            vec![
                ("contractHash", ValueType::Address),
                ("method", ValueType::String),
                (
                    "argCount",
                    ValueType::Integer {
                        signed: false,
                        bits: 256,
                    },
                ),
                (
                    "fixedFee",
                    ValueType::Integer {
                        signed: true,
                        bits: 256,
                    },
                ),
            ],
        )),
        ("NativeCalls", "NetworkConfig") => Some(mk_struct(
            "NativeCalls.NetworkConfig",
            vec![
                ("feePerByte", u256.clone()),
                (
                    "execFeeFactor",
                    ValueType::Integer {
                        signed: false,
                        bits: 32,
                    },
                ),
                ("storagePrice", u256.clone()),
                ("gasPerBlock", u256.clone()),
                ("oraclePrice", u256.clone()),
                ("minimumDeploymentFee", u256),
            ],
        )),
        _ => None,
    }
}

pub(crate) fn infer_defined_struct_type_by_name(
    ctx: &LoweringContext,
    name: &str,
) -> Option<ValueType> {
    ctx.defined_struct_types
        .iter()
        .chain(ctx.state_types.iter())
        .chain(ctx.param_types.iter())
        .chain(ctx.return_types.iter())
        .chain(ctx.local_types.values())
        .find_map(|ty| find_named_struct_type(ty, name))
}

pub(crate) fn infer_struct_constructor_type(
    func: &Expression,
    ctx: &LoweringContext,
) -> Option<ValueType> {
    match func {
        Expression::Variable(identifier) => {
            infer_defined_struct_type_by_name(ctx, &identifier.name)
        }
        Expression::MemberAccess(_, _, identifier) => {
            infer_defined_struct_type_by_name(ctx, &identifier.name)
        }
        _ => None,
    }
}

/// Returns true when the expression is structurally a Solidity type expression
/// rather than a value expression. Used to distinguish fixed-size array types
/// (`T[N]`) from value-subscripts (`arr[i]`) when both parse as
/// `Expression::ArraySubscript(inner, Some(_))`. A type expression is either a
/// primitive `Type` node or nested `ArraySubscript` whose base is itself a
/// type expression (covers nested fixed-size arrays like `uint[3][2]`).
pub(crate) fn is_type_expression(expr: &Expression) -> bool {
    match expr {
        Expression::Type(_, _) => true,
        Expression::ArraySubscript(_, inner, _) => is_type_expression(inner),
        Expression::Parenthesis(_, inner) => is_type_expression(inner),
        _ => false,
    }
}

// `stacker::maybe_grow` wrapper — this function recurses through
// `Parenthesis`, `Conditional`, and `MemberAccess` arms. Deeply nested
// sources (e.g. 30k-paren chains, 10k-long `a.b.c.d...` selectors) would
// otherwise stack-overflow the compiler. See sibling guards in
// `src/ir/expressions/dispatch/entry.rs`.
pub(crate) fn infer_type_from_expression(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<ValueType> {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        infer_type_from_expression_inner(expr, ctx)
    })
}

pub(crate) fn infer_type_from_expression_inner(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<ValueType> {
    match expr {
        Expression::Parenthesis(_, inner) => infer_type_from_expression(inner, ctx),
        // A ternary `c ? a : b` has the common type of its two branches. Without
        // this arm the result inferred `None`, which silently disabled the
        // bytesN-vs-hex-literal canonicalization in comparison lowering: e.g.
        // `(c ? a : b) == 0x..` (bytesN operands) left the literal as an Integer
        // and emitted a type-strict `EQUAL(ByteString, Integer)` that is always
        // false on a real node (defeating sentinel/role `!=` guards). Prefer the
        // then-branch's type, falling back to the else-branch's.
        Expression::ConditionalOperator(_, _, then_expr, else_expr) => {
            infer_type_from_expression(then_expr, ctx)
                .or_else(|| infer_type_from_expression(else_expr, ctx))
        }
        // A bitwise `a & b` / `a | b` / `a ^ b` has the common type of its
        // operands. Without this arm the result inferred `None`, which for
        // `bytesN` operands suppressed the byte-reversal in a following
        // `uint256(a | b)` / `int256(a & b)` cast: the bitwise op leaves a
        // byte-REVERSED word (NeoVM AND/OR/XOR read the big-endian ByteString
        // as a little-endian integer), and the cast's reverse is what recovers
        // the value — but only fires when the cast argument infers to `bytesN`.
        // Prefer the left operand's type, falling back to the right (bug-hunt
        // #14). Integer operands infer to `Integer`, so integer bitwise flows
        // are unchanged (no reverse added).
        Expression::BitwiseAnd(_, left, right)
        | Expression::BitwiseOr(_, left, right)
        | Expression::BitwiseXor(_, left, right) => {
            infer_type_from_expression(left, ctx).or_else(|| infer_type_from_expression(right, ctx))
        }
        // A bytesN shift keeps the bytesN type of its LEFT operand (the shift
        // amount doesn't participate). This is scoped to bytesN ONLY: without
        // it `uint256(b32 >> 8)` inferred `None`, so the cast skipped the
        // BE-bytes reversal and read the shifted bytes32 little-endian (same
        // class as the bitwise arm above / bug-hunt #14). Integer shifts keep
        // inferring `None` here — reporting `uintN` would flip the reverse
        // decision of a wrapping `bytesN(uint256 << k)` cast (whose shift
        // output is not a plain integer) and byte-corrupt it.
        Expression::ShiftLeft(_, left, _) | Expression::ShiftRight(_, left, _) => {
            infer_type_from_expression(left, ctx)
        }
        Expression::BoolLiteral(_, _) => Some(ValueType::Boolean),
        Expression::NumberLiteral(_, _, _, _)
        | Expression::HexNumberLiteral(_, _, _)
        | Expression::RationalNumberLiteral(_, _, _, _, _) => Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
        Expression::StringLiteral(_) => Some(ValueType::String),
        Expression::HexLiteral(_) => Some(ValueType::ByteArray { fixed_len: None }),
        Expression::AddressLiteral(_, _) => Some(ValueType::Address),
        Expression::Type(_, ty) => value_type_from_ptype(ty),
        Expression::FunctionCall(_, func, args)
            if args.len() == 1 && matches!(func.as_ref(), Expression::Type(_, _)) =>
        {
            if let Expression::Type(_, ty) = func.as_ref() {
                value_type_from_ptype(ty)
            } else {
                None
            }
        }
        Expression::FunctionCall(_, func, args) if args.len() == 1 => {
            if let Some(ty) = infer_struct_constructor_type(func.as_ref(), ctx) {
                return Some(ty);
            }

            if let Expression::Variable(identifier) = func.as_ref() {
                match identifier.name.as_str() {
                    "keccak256" | "sha256" => {
                        return Some(ValueType::ByteArray {
                            fixed_len: Some(32),
                        });
                    }
                    "ripemd160" => {
                        return Some(ValueType::ByteArray {
                            fixed_len: Some(20),
                        });
                    }
                    _ => {}
                }
            }

            if let Expression::MemberAccess(_, inner, member) = func.as_ref() {
                if let Expression::Variable(base) = inner.as_ref() {
                    match (base.name.as_str(), member.name.as_str()) {
                        ("Syscalls", "scriptHashToAddress") => return Some(ValueType::Address),
                        ("Syscalls", "addressToScriptHash") => {
                            return Some(ValueType::ByteArray {
                                fixed_len: Some(20),
                            })
                        }
                        ("Syscalls", "isValidAddress") => return Some(ValueType::Boolean),
                        _ => {}
                    }
                }
            }

            // Contract/interface type-casts like `IPool(addr)` and namespace-qualified
            // imports like `NS.IPool(addr)` evaluate to an address-like value.
            if let Expression::Variable(type_id) = func.as_ref() {
                if ctx.is_contract_type_name(&type_id.name) {
                    return Some(ValueType::Address);
                }
            }

            if let Expression::MemberAccess(_, namespace_expr, type_id) = func.as_ref() {
                if matches!(
                    namespace_expr.as_ref(),
                    Expression::Variable(namespace_id)
                        if !ctx.param_index_map.contains_key(&namespace_id.name)
                            && ctx.resolve_local(&namespace_id.name).is_none()
                            && !ctx.state_index_map.contains_key(&namespace_id.name)
                            && !ctx.is_contract_type_name(&namespace_id.name)
                ) && ctx.is_contract_type_name(&type_id.name)
                {
                    return Some(ValueType::Address);
                }
            }

            // Fall through to user-defined function return-type inference (the
            // N-arg arm below does the same): a 1-arg call like `this.f(x)` or
            // `IPool(addr).f(x)` must resolve to its DECLARED return type, not
            // None — otherwise `call(x) == <bytesN literal/constant>` cannot
            // canonicalize the literal and the type-strict `EQUAL` is wrong on a
            // real node. The cast / `Interface(addr)` / `Syscalls.*` fast paths
            // above already returned, so this only fires for genuine method
            // calls.
            if let Expression::Variable(identifier) = func.as_ref() {
                if let Some(ty) = ctx.get_function_return_type(&identifier.name, args.len()) {
                    return Some(ty.clone());
                }
            }
            if let Expression::MemberAccess(_, _, method) = func.as_ref() {
                if let Some(ty) = ctx.get_function_return_type(&method.name, args.len()) {
                    return Some(ty.clone());
                }
            }

            None
        }
        // Task #191 — infer a user-defined function's return type from the
        // registered `function_return_types` map so chained calls like
        // `c.inc().value` can resolve the struct-field index. The specific
        // cases already handled above (`T(x)` casts, `Interface(addr)`,
        // `Syscalls.*`, etc.) stay on their fast paths; this branch covers
        // bare `f(...)` where `f` is a contract method or injected free
        // function (e.g. `using { inc } for T` with inc at file scope).
        //
        // Two call shapes produce a user-defined function name here:
        //   - `Expression::Variable(id)` → `f(args)` (free function / same-
        //     contract method after Task #187 injection).
        //   - `Expression::MemberAccess(_, Variable(id), method)` → method
        //     call on `this` / library-qualified call. We key the lookup on
        //     the method name with the literal argument count, matching how
        //     `function_return_types` is populated at module-build time.
        Expression::FunctionCall(_, func, args) => {
            if let Some(ty) = infer_struct_constructor_type(func.as_ref(), ctx) {
                return Some(ty);
            }

            if let Expression::Variable(identifier) = func.as_ref() {
                match identifier.name.as_str() {
                    "keccak256" | "sha256" => {
                        return Some(ValueType::ByteArray {
                            fixed_len: Some(32),
                        });
                    }
                    "ripemd160" => {
                        return Some(ValueType::ByteArray {
                            fixed_len: Some(20),
                        });
                    }
                    _ => {}
                }
            }

            if let Expression::Variable(identifier) = func.as_ref() {
                if let Some(ty) = ctx.get_function_return_type(&identifier.name, args.len()) {
                    return Some(ty.clone());
                }
            }
            if let Expression::MemberAccess(_, inner, method) = func.as_ref() {
                if method.name == "concat" {
                    match inner.as_ref() {
                        Expression::Type(_, solang_parser::pt::Type::String) => {
                            return Some(ValueType::String);
                        }
                        Expression::Type(_, solang_parser::pt::Type::DynamicBytes) => {
                            return Some(ValueType::ByteArray { fixed_len: None });
                        }
                        _ => {}
                    }
                }

                // `x.f(args)` attached via `using { f } for T;` lowers to
                // `f(x, args)` — the registered return-type key therefore uses
                // `args.len() + 1` for the library-attach form. Try both
                // shapes before giving up.
                if ctx.has_using_directives() {
                    if let Some(ty) = ctx.get_function_return_type(&method.name, args.len() + 1) {
                        // Only honour the receiver-attached form when the
                        // inner expression isn't a namespace-style
                        // `Library.f(...)` call (which keeps its literal
                        // arg count). Distinguishing the two without full
                        // type info: treat a bare `Variable` inner that
                        // doesn't name a library/contract as a receiver.
                        if let Expression::Variable(base) = inner.as_ref() {
                            let is_namespace = ctx.is_contract_type_name(&base.name);
                            if !is_namespace {
                                return Some(ty.clone());
                            }
                        } else {
                            // Chained-call receiver (e.g. `c.inc().inc()`):
                            // the inner isn't a namespace, so the outer is
                            // genuinely receiver-attached.
                            return Some(ty.clone());
                        }
                    }
                }
                if let Some(ty) = ctx.get_function_return_type(&method.name, args.len()) {
                    return Some(ty.clone());
                }
            }
            None
        }
        Expression::NamedFunctionCall(_, func, _) => infer_struct_constructor_type(func, ctx),
        Expression::ArrayLiteral(_, elements) => Some(ValueType::Array(Box::new(
            infer_literal_array_element_type(elements),
        ))),
        Expression::ArraySubscript(_, array, index) => {
            // PERF: infer the base type ONCE. The fall-through chain below
            // previously re-called `infer_type_from_expression(array, ctx)` in
            // each `else if` arm — up to 4 full re-descents of the same
            // sub-tree, exponential in nesting depth for `a[i][j][k]`.
            let array_ty = infer_type_from_expression(array, ctx);
            // Type expression: `T[]` (dynamic array) is represented by solang-parser as an
            // ArraySubscript with a missing index expression. Use this to infer array element
            // types for locals/parameters so struct member access can be lowered correctly.
            if index.is_none() {
                array_ty.map(|inner| ValueType::Array(Box::new(inner)))
            } else if let Some(ValueType::Array(inner)) = &array_ty {
                // Value expression: `arr[i]`
                Some((**inner).clone())
            } else if let Some(ValueType::Mapping { value, .. }) = &array_ty {
                // Value expression: `m[key]` -> the mapping's declared VALUE
                // type. Without this, a `mapping(K => uintN)` read (N < 256)
                // inferred to `None`, so the downstream arithmetic lowering
                // treated the operand as full-width and skipped both the
                // checked-overflow Panic(0x11) guard and the `unchecked`
                // mod-2^N truncation — a silent-overflow divergence from
                // Solidity 0.8. Array elements / struct fields / locals already
                // carry their width, so only the mapping-read path was affected.
                Some((**value).clone())
            } else if matches!(array_ty, Some(ValueType::ByteArray { .. })) {
                // A byte index of `bytes`/`bytesN` (`b[i]`) is a `bytes1` value.
                // The lowering coerces it to a 1-byte ByteString (see
                // `try_lower_expression_primary`), so report `bytes1` here too:
                // this makes `b[i] == <hex literal>` canonicalize the literal to
                // a ByteString (otherwise it stays an Integer and the type-strict
                // `EQUAL(ByteString, Integer)` is false on a real node) and keeps
                // cast / abi-encode paths consistent with the runtime type.
                Some(ValueType::ByteArray { fixed_len: Some(1) })
            } else if is_type_expression(array) {
                // Task #185: Fixed-size array type expression `T[N]` (e.g. the outer
                // element type of a multi-dim declaration `uint[3][2] memory a;`).
                // solang-parser represents this as `ArraySubscript(T, Some(N))` in type
                // contexts — structurally identical to a value subscript `arr[i]`, but the
                // base resolves to a scalar/struct type rather than an Array. When the
                // base is structurally a type expression (`Type(_)` or nested
                // `ArraySubscript(type_expr, _)`), wrap the inferred base type in
                // `Array(..)` so inner-dimension `new T[N]` allocations receive the
                // correct element type.
                array_ty.map(|inner| ValueType::Array(Box::new(inner)))
            } else {
                None
            }
        }
        Expression::Variable(identifier) => {
            if identifier.name == "this" || ctx.is_contract_type_name(&identifier.name) {
                Some(ValueType::Address)
            } else if ctx.enum_variant_map.contains_key(&identifier.name) {
                // Solidity enums lower to unsigned integers. We model enum-typed values as
                // uint8 for IR inference to support constructs like `new MyEnum[](n)`.
                Some(ValueType::Integer {
                    signed: false,
                    bits: 8,
                })
            } else {
                ctx.variable_type(&identifier.name).or_else(|| {
                    // In type contexts solang-parser represents user-defined type names (e.g.,
                    // structs) as `Expression::Variable`. Resolve by scanning known value types
                    // with scope-priority ordering: local → param → return → state → defined
                    // structs. This prevents cross-scope type collisions when the same name
                    // appears at multiple levels.
                    infer_defined_struct_type_by_name(ctx, &identifier.name)
                })
            }
        }
        Expression::MemberAccess(_, inner, member) => {
            if member.name == "length"
                && matches!(
                    infer_type_from_expression(inner, ctx),
                    Some(ValueType::ByteArray { .. } | ValueType::String | ValueType::Array(_))
                )
            {
                return Some(ValueType::Integer {
                    signed: false,
                    bits: 256,
                });
            }

            // `<address>.balance` is a uint256 (the account's GAS balance).
            // Guarding on the inner inferring to `Address` excludes struct
            // fields literally named `balance` (e.g. NativeCalls.AccountState).
            // Without this, an `address.balance` argument infers to `None`, so
            // overload resolution of a same-arity overload set (e.g.
            // `assertEq(uint256,uint256,string)` vs its int/bool/… siblings)
            // matches nothing and the call traps with "no compiled body".
            if member.name == "balance"
                && matches!(
                    infer_type_from_expression(inner, ctx),
                    Some(ValueType::Address)
                )
            {
                return Some(ValueType::Integer {
                    signed: false,
                    bits: 256,
                });
            }

            // `<address>.codehash` is a bytes32; `<address>.code` is bytes.
            if matches!(
                infer_type_from_expression(inner, ctx),
                Some(ValueType::Address)
            ) {
                if member.name == "codehash" {
                    return Some(ValueType::ByteArray {
                        fixed_len: Some(32),
                    });
                }
                if member.name == "code" {
                    return Some(ValueType::ByteArray { fixed_len: None });
                }
            }

            if member.name == "selector" {
                return Some(ValueType::ByteArray { fixed_len: Some(4) });
            }

            if member.name == "interfaceId" {
                if let Expression::FunctionCall(_, func, args) = inner.as_ref() {
                    if args.len() == 1
                        && matches!(func.as_ref(), Expression::Variable(id) if id.name == "type")
                    {
                        return Some(ValueType::ByteArray { fixed_len: Some(4) });
                    }
                }
            }

            if matches!(member.name.as_str(), "max" | "min") {
                if let Expression::FunctionCall(_, func, args) = inner.as_ref() {
                    if args.len() == 1
                        && matches!(func.as_ref(), Expression::Variable(id) if id.name == "type")
                    {
                        if let Expression::Type(_, ty) = &args[0] {
                            if let Some(value_type) = value_type_from_ptype(ty) {
                                return Some(value_type);
                            }
                        }
                    }
                }
            }

            // Treat known NativeCalls/NativeContracts native contract hash constants as
            // addresses so downstream lowering can recognize static native targets.
            if matches!(
                inner.as_ref(),
                Expression::Variable(id)
                    if id.name == "NativeCalls" || id.name == "NativeContracts"
            ) && matches!(
                member.name.as_str(),
                "NEO_CONTRACT"
                    | "GAS_CONTRACT"
                    | "CONTRACT_MANAGEMENT"
                    | "POLICY_CONTRACT"
                    | "ORACLE_CONTRACT"
                    | "ROLE_MANAGEMENT"
                    | "NOTARY_CONTRACT"
                    | "TREASURY_CONTRACT"
                    | "LEDGER_CONTRACT"
                    | "CRYPTO_LIB"
                    | "STD_LIB"
            ) {
                return Some(ValueType::Address);
            }

            // Treat known Syscalls.* native contract hash constants as addresses so
            // member access is typed consistently with NativeCalls constants.
            if matches!(inner.as_ref(), Expression::Variable(id) if id.name == "Syscalls")
                && matches!(
                    member.name.as_str(),
                    "CONTRACT_MANAGEMENT"
                        | "POLICY_CONTRACT"
                        | "ORACLE_CONTRACT"
                        | "ROLE_MANAGEMENT"
                        | "LEDGER_CONTRACT"
                        | "CRYPTO_LIB"
                        | "STD_LIB"
                )
            {
                return Some(ValueType::Address);
            }

            // Type-qualified user-defined structs from interfaces/contracts: `Interface.StructName`.
            // solang-parser represents these as `MemberAccess(Variable("Interface"), "StructName")`.
            if let Expression::Variable(base) = inner.as_ref() {
                if ctx.is_contract_type_name(&base.name) {
                    if let Some(struct_ty) = ctx
                        .defined_struct_types
                        .iter()
                        .chain(ctx.state_types.iter())
                        .chain(ctx.param_types.iter())
                        .chain(ctx.return_types.iter())
                        .chain(ctx.local_types.values())
                        .find_map(|ty| find_named_struct_type(ty, &member.name))
                    {
                        return Some(struct_ty);
                    }
                }
            }

            if let Expression::Variable(base) = inner.as_ref() {
                match (base.name.as_str(), member.name.as_str()) {
                    ("msg", "sender") => return Some(ValueType::Address),
                    ("msg", "value") => {
                        return Some(ValueType::Integer {
                            signed: false,
                            bits: 256,
                        })
                    }
                    ("msg", "data") => return Some(ValueType::ByteArray { fixed_len: None }),
                    ("tx", "origin") => return Some(ValueType::Address),
                    ("block", "timestamp" | "number" | "chainid") => {
                        return Some(ValueType::Integer {
                            signed: false,
                            bits: 256,
                        })
                    }
                    _ => {}
                }
            }

            if let Expression::Variable(base) = inner.as_ref() {
                if let Some(struct_ty) = builtin_struct_type(&base.name, &member.name) {
                    return Some(struct_ty);
                }
            }

            // Best-effort struct member typing (e.g., `tmp.field`). This is required for
            // patterns like `IERC20(req.token).transfer(...)` where `req.token` must be
            // recognized as address-like for external call lowering.
            let inner_ty = infer_type_from_expression(inner, ctx)?;
            if let ValueType::Struct { fields, .. } = &inner_ty {
                if let Some(field) = fields.iter().find(|field| field.name == member.name) {
                    return Some(field.ty.clone());
                }
            }

            Some(inner_ty)
        }
        // Arithmetic inherits its operand type (left, then right). Without
        // these arms nested arithmetic like `(x + x) + 100` inferred `None`,
        // which defeated BOTH overflow-guard gates in
        // `ir_expressions/dispatch/binary_predicates.rs`:
        //   - the narrow guard (`is_narrow_result` → `narrow_unsigned_bits`
        //     needs the operand type), so `uint8` overflow silently produced
        //     out-of-range values (`f(100) == 300` instead of `Panic(0x11)`), and
        //   - the uint256 guard (`is_uint256_operand` bails on `None` for
        //     non-literals), so the outermost add of nested arithmetic fell
        //     through to a bare `BinaryOp` with no overflow check at all.
        // In valid Solidity these operators only exist on integers, so
        // inheriting the left operand's type is always sound.
        Expression::Add(_, left, right)
        | Expression::Subtract(_, left, right)
        | Expression::Multiply(_, left, right)
        | Expression::Divide(_, left, right)
        | Expression::Modulo(_, left, right)
        | Expression::Power(_, left, right) => {
            infer_type_from_expression(left, ctx).or_else(|| infer_type_from_expression(right, ctx))
        }
        // Comparisons and short-circuit logical operators yield `bool`.
        Expression::Less(_, _, _)
        | Expression::More(_, _, _)
        | Expression::LessEqual(_, _, _)
        | Expression::MoreEqual(_, _, _)
        | Expression::Equal(_, _, _)
        | Expression::NotEqual(_, _, _)
        | Expression::And(_, _, _)
        | Expression::Or(_, _, _)
        | Expression::Not(_, _) => Some(ValueType::Boolean),
        // Unary minus on a NUMBER LITERAL yields a SIGNED constant: Solidity
        // types `-4` as int256, and overload resolution (see
        // `calls/variable_calls.rs` `arg_types`) keys on that — inferring the
        // operand's uint256 here made `-4` look unsigned, so a call like
        // `assertEq(x, -4)` matched no overload and emitted a runtime trap
        // ("'assertEq'/2 has no compiled body").
        Expression::Negate(_, inner)
            if matches!(
                inner.as_ref(),
                Expression::NumberLiteral(..) | Expression::HexNumberLiteral(..)
            ) =>
        {
            Some(ValueType::Integer {
                signed: true,
                bits: 256,
            })
        }
        // Unary minus/plus and bitwise-not keep their operand's type.
        Expression::Negate(_, inner)
        | Expression::UnaryPlus(_, inner)
        | Expression::BitwiseNot(_, inner) => infer_type_from_expression(inner, ctx),
        _ => None,
    }
}

pub(crate) fn value_type_from_ptype(ty: &PtType) -> Option<ValueType> {
    match ty {
        PtType::Bool => Some(ValueType::Boolean),
        PtType::Address | PtType::AddressPayable | PtType::Payable => Some(ValueType::Address),
        PtType::Uint(bits) => Some(ValueType::Integer {
            signed: false,
            bits: *bits,
        }),
        PtType::Int(bits) => Some(ValueType::Integer {
            signed: true,
            bits: *bits,
        }),
        PtType::String => Some(ValueType::String),
        PtType::Bytes(len) => Some(ValueType::ByteArray {
            fixed_len: Some(*len as u16),
        }),
        PtType::DynamicBytes => Some(ValueType::ByteArray { fixed_len: None }),
        // Solidity `fixed`/`ufixed` rational literals resolve to integers at compile time.
        PtType::Rational => Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
        // Mapping type: extract key/value from inner type expressions when possible.
        PtType::Mapping { key, value, .. } => {
            let key_ty = if let Expression::Type(_, inner) = key.as_ref() {
                value_type_from_ptype(inner)?
            } else {
                ValueType::Any
            };
            let val_ty = if let Expression::Type(_, inner) = value.as_ref() {
                value_type_from_ptype(inner)?
            } else {
                ValueType::Any
            };
            Some(ValueType::Mapping {
                key: Box::new(key_ty),
                value: Box::new(val_ty),
            })
        }
        // Function types are not representable on NeoVM.
        PtType::Function { .. } => None,
    }
}

pub(crate) fn infer_array_element_type(
    expr: &Expression,
    ctx: &LoweringContext,
) -> Option<ValueType> {
    match infer_type_from_expression(expr, ctx) {
        Some(ValueType::Array(inner)) => Some(*inner.clone()),
        _ => None,
    }
}
