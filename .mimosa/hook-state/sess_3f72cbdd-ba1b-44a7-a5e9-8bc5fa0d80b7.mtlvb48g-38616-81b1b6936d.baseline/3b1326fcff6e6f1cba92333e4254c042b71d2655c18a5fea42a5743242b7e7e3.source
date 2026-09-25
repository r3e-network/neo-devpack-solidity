use super::*;

pub(crate) fn try_lower_expression_calls(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    match expr {
        Expression::FunctionCallBlock(_, call, block) => {
            let _ = (call, block);
            ctx.record_warning_with_suggestion(
                "function call options (`{...}`) are ignored on Neo N3. Neo N3 requires explicit NEP-17 transfers instead of attached value.",
                "Replace {value: x} with explicit NativeCalls.gasTransfer() or NativeCalls.neoTransfer() before the call if value transfer is intended.",
            );
            // We still need to process the inner call expression to push a value on the stack.
            // Solang parser parses `a{value: 1}()` as a `FunctionCall` wrapping a `FunctionCallBlock`.
            // The `Expression::FunctionCallBlock` matching here usually handles orphaned blocks
            // or blocks without parens. We emit a dummy null value so compilation can proceed.
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            Some(true)
        }
        Expression::NamedFunctionCall(_, func, args) => {
            // Try struct constructor first (e.g., `MyStruct({x: 1, y: 2})`).
            if let Some(result) =
                try_lower_struct_constructor_named_call(func.as_ref(), args, ctx, instructions)
            {
                return Some(result);
            }

            // Try reordering named args into positional order for known functions.
            if let Some(result) =
                try_lower_named_function_call(func.as_ref(), args, ctx, instructions)
            {
                return Some(result);
            }

            ctx.record_error_with_suggestion(
                "named argument calls are not supported for this callee",
                "use positional arguments instead: f(arg1, arg2) rather than f({x: arg1, y: arg2})",
            );
            Some(false)
        }
        Expression::FunctionCall(_, func, args) => Some(lower_function_call_expression(
            func.as_ref(),
            args,
            ctx,
            instructions,
        )),
        Expression::New(_, expr) => Some(lower_new_expression(expr.as_ref(), ctx, instructions)),
        Expression::Type(_, ty) => {
            push_default_for_type(ty, instructions);
            Some(true)
        }
        Expression::Parenthesis(_, inner) => Some(lower_expression(inner, ctx, instructions)),
        Expression::MemberAccess(_, inner, member) => Some(lower_member_access_expression(
            expr,
            inner.as_ref(),
            member,
            ctx,
            instructions,
        )),
        _ => None,
    }
}

pub(crate) fn lower_new_expression(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    match expr {
        Expression::FunctionCall(_, func, args) => {
            // `new X{salt: s}(args)` — CREATE2-style salted creation. solang-parser
            // represents the `{...}` options as a `FunctionCallBlock` wrapping the
            // type expression. Neo N3 has no CREATE2 (deployment is an external
            // transaction, and a deterministic address can't be derived from a
            // salt here), so evaluate the option values for side effects, warn
            // they're ignored, and lower as a plain `new X(args)` — mirroring the
            // `{gas: ...}` / `{value: ...}` handling for ordinary method calls.
            if let Expression::FunctionCallBlock(_, inner, block) = func.as_ref() {
                if let Statement::Args(_, named_args) = block.as_ref() {
                    let names: Vec<String> =
                        named_args.iter().map(|a| a.name.name.clone()).collect();
                    ctx.record_warning_with_suggestion(
                        format!(
                            "call options ({}) on `new` are ignored on Neo N3; the contract is \
                             deployed as a normal `new` (Neo N3 has no CREATE2 salt).",
                            names.join(", ")
                        ),
                        "A deterministic address is not derived from the salt here; use a factory \
                         / ContractManagement if you need the deployed contract hash.",
                    );
                    for a in named_args {
                        if lower_expression(&a.expr, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                }
                return lower_new_expression(
                    &Expression::FunctionCall(Default::default(), inner.clone(), args.clone()),
                    ctx,
                    instructions,
                );
            }
            // `new bytes(n)` / `new string(n)`
            if matches!(
                func.as_ref(),
                Expression::Type(_, PtType::DynamicBytes | PtType::String)
            ) {
                if args.len() != 1 {
                    ctx.record_error_with_suggestion(
                        "new bytes/string expects exactly one length argument",
                        "usage: new bytes(length) or new string(length)",
                    );
                    for arg in args {
                        if lower_expression(arg, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                } else if !lower_expression(&args[0], ctx, instructions) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                }

                instructions.push(Instruction::NewBuffer);
                return true;
            }
            // `new T[](n)` dynamic arrays and `new T[N]()` fixed-size arrays.
            if let Expression::ArraySubscript(_, array_type_expr, index) = func.as_ref() {
                if let Some(fixed_len_expr) = index.as_ref() {
                    if !args.is_empty() {
                        ctx.record_error(
                            "new fixed-size array constructor does not accept runtime arguments",
                        );
                        for arg in args {
                            if lower_expression(arg, ctx, instructions) {
                                instructions.push(Instruction::Drop(ValueType::Any));
                            }
                        }
                    }

                    return lower_new_array_allocation(
                        array_type_expr.as_ref(),
                        fixed_len_expr.as_ref(),
                        ctx,
                        instructions,
                    );
                }

                if args.len() != 1 {
                    ctx.record_error("new array expects exactly one length argument");
                    for arg in args {
                        if lower_expression(arg, ctx, instructions) {
                            instructions.push(Instruction::Drop(ValueType::Any));
                        }
                    }

                    let zero = Expression::NumberLiteral(
                        Default::default(),
                        "0".to_string(),
                        "".to_string(),
                        None,
                    );
                    return lower_new_array_allocation(
                        array_type_expr.as_ref(),
                        &zero,
                        ctx,
                        instructions,
                    );
                }

                return lower_new_array_allocation(
                    array_type_expr.as_ref(),
                    &args[0],
                    ctx,
                    instructions,
                );
            }

            // `new Contract(args)` — Neo N3 has no contract-creation-from-contract
            // syscall (deployment is an external transaction), so we simulate the
            // `new` expression by (a) invoking the sibling's constructor body
            // in-line (merged as a name-mangled regular function
            // `__ctor__<Name>`; see Task #198 in `solidity_analyse.rs`) so its
            // state-variable assignments persist against the already-merged
            // sibling state slots (Task #197), and (b) pushing a 20-byte zero
            // placeholder for the "address". Subsequent `c.method()` calls on
            // the zero-hash route through `self_method_offsets` (Task #83), so
            // getters against the merged state observe the ctor-written values.
            if let Expression::Variable(identifier) = func.as_ref() {
                if ctx.is_contract_type_name(&identifier.name) {
                    let mangled = format!("__ctor__{}", identifier.name);
                    let mangled_resolves = ctx.neo_function_name(&mangled, args.len()).is_some();
                    if mangled_resolves {
                        let mut success = true;
                        for arg in args {
                            if !lower_expression(arg, ctx, instructions) {
                                success = false;
                            }
                        }
                        if success {
                            if let Some(neo_name) = ctx.neo_function_name(&mangled, args.len()) {
                                instructions.push(Instruction::CallFunction {
                                    name: neo_name,
                                    arg_count: args.len(),
                                });
                            }
                        }
                    } else {
                        // No matching constructor (sibling either has no
                        // constructor or has one with a different arity).
                        // Preserve the pre-Task-#198 behaviour: evaluate the
                        // arguments for side effects and discard them.
                        for arg in args {
                            if lower_expression(arg, ctx, instructions) {
                                instructions.push(Instruction::Drop(ValueType::Any));
                            }
                        }
                    }
                    instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                        0u8;
                        20
                    ])));
                    return true;
                }
            }

            ctx.record_error_with_suggestion(
                "unsupported `new` expression",
                "Neo N3 supports `new bytes(n)`, `new string(n)`, `new T[](n)`, and `new T[N]`; use ContractManagement for contract deployment",
            );
            for arg in args {
                if lower_expression(arg, ctx, instructions) {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
            }
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            true
        }
        Expression::ArraySubscript(_, array_type_expr, Some(length_expr)) => {
            lower_new_array_allocation(
                array_type_expr.as_ref(),
                length_expr.as_ref(),
                ctx,
                instructions,
            )
        }
        Expression::FunctionCallBlock(_, _, _) => {
            ctx.record_warning_with_suggestion(
                "function call options on `new` are ignored on Neo N3.",
                "Neo N3 does not support value transfers via call options; use explicit NEP-17 transfers if needed.",
            );
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            true
        }
        _ => {
            ctx.record_error_with_suggestion(
                "unsupported `new` expression",
                "Neo N3 supports `new bytes(n)`, `new string(n)`, `new T[](n)`, and `new T[N]`",
            );
            instructions.push(Instruction::PushLiteral(LiteralValue::Null));
            true
        }
    }
}

pub(crate) fn lower_new_array_allocation(
    array_type_expr: &Expression,
    length_expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let Some(element_type) = infer_type_from_expression(array_type_expr, ctx) else {
        ctx.record_error(format!(
            "unable to infer element type for new array allocation (`new {array_type_expr}`)"
        ));
        if lower_expression(length_expr, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
        instructions.push(Instruction::NewArray {
            element_type: ValueType::Any,
        });
        return true;
    };

    let tmp_id = ctx.next_label();
    let len_local = ctx.allocate_local(format!("__new_array_len_{tmp_id}"), None);
    if !lower_expression(length_expr, ctx, instructions) {
        instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
            BigInt::zero(),
        )));
    }
    instructions.push(Instruction::StoreLocal(len_local));

    let array_type = ValueType::Array(Box::new(element_type.clone()));
    let array_local = ctx.allocate_local(format!("__new_array_{tmp_id}"), Some(array_type));

    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::NewArray {
        element_type: element_type.clone(),
    });
    instructions.push(Instruction::StoreLocal(array_local));

    // Solidity initializes new memory arrays with element defaults. NeoVM NEWARRAY
    // fills with nulls, so explicitly write default values for value types.
    let idx_local = ctx.allocate_local(format!("__new_array_idx_{tmp_id}"), None);
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(idx_local));

    let loop_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::LoadLocal(len_local));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: end_label });

    instructions.push(Instruction::LoadLocal(array_local));
    instructions.push(Instruction::LoadLocal(idx_local));
    // Task #185: Recurse for nested fixed-size array types (e.g. `uint[3][2]`).
    // When the element type is itself a `T[N]` fixed-size array, emit a full
    // `new T[N]` allocation so the sub-array is pre-sized with default-
    // initialized slots; `push_default_for_value_type(Array, ..)` only
    // produces an empty length-0 array, which would fault at the first
    // nested `a[i][j] = ..` with "SETITEM: index out of bounds".
    if let Expression::ArraySubscript(_, inner_ty_expr, Some(inner_len_expr)) = array_type_expr {
        lower_new_array_allocation(
            inner_ty_expr.as_ref(),
            inner_len_expr.as_ref(),
            ctx,
            instructions,
        );
    } else {
        push_default_for_value_type(&element_type, ctx, instructions);
    }
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(idx_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(idx_local));

    instructions.push(Instruction::Jump { target: loop_label });
    instructions.push(Instruction::Label(end_label));

    instructions.push(Instruction::LoadLocal(array_local));
    true
}

/// Reorder named function call arguments into positional order and delegate
/// to the standard function call lowering path.
///
/// Returns `None` if the callee is not a known function (caller should fall
/// through to the error path). Returns `Some(bool)` on success or error.
pub(crate) fn try_lower_named_function_call(
    func: &Expression,
    named_args: &[solang_parser::pt::NamedArgument],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let Expression::Variable(identifier) = func else {
        return None;
    };

    // Look up parameter names for this function with matching arg count.
    // Clone to release the immutable borrow on `ctx` so we can call
    // `ctx.record_error()` later.
    let param_names: Vec<String> = ctx
        .get_function_param_names(&identifier.name, named_args.len())?
        .to_vec();

    // Build name→index mapping from the function's parameter list.
    let name_to_index: HashMap<&str, usize> = param_names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    // Reorder named arguments into positional order.
    let mut positional: Vec<Option<&Expression>> = vec![None; named_args.len()];
    let mut has_error = false;

    for arg in named_args {
        if let Some(&index) = name_to_index.get(arg.name.name.as_str()) {
            if positional[index].is_some() {
                ctx.record_error(format!(
                    "duplicate named argument '{}' in call to '{}'",
                    arg.name.name, identifier.name
                ));
                has_error = true;
            } else {
                positional[index] = Some(&arg.expr);
            }
        } else {
            ctx.record_error(format!(
                "unknown parameter '{}' in call to '{}'; expected one of: {}",
                arg.name.name,
                identifier.name,
                param_names.join(", ")
            ));
            has_error = true;
        }
    }

    if has_error {
        // Evaluate all args for side effects, then return false.
        for arg in named_args {
            if lower_expression(&arg.expr, ctx, instructions) {
                instructions.push(Instruction::Drop(ValueType::Any));
            }
        }
        return Some(false);
    }

    // Build positional args vector and delegate to normal call path.
    let mut ordered_exprs: Vec<&Expression> = Vec::with_capacity(positional.len());
    for (idx, opt) in positional.into_iter().enumerate() {
        if let Some(expr) = opt {
            ordered_exprs.push(expr);
        } else {
            ctx.record_error(format!(
                "missing argument at position {idx} in call to '{}'",
                identifier.name
            ));
            return Some(false);
        }
    }

    // Infer argument types before lowering (which consumes them) for
    // type-directed same-arity overload resolution.
    let arg_types: Vec<Option<ValueType>> = ordered_exprs
        .iter()
        .map(|expr| infer_type_from_expression(expr, ctx))
        .collect();
    // Lower each argument in positional order, then emit the call.
    let mut success = true;
    for expr in &ordered_exprs {
        if !lower_expression(expr, ctx, instructions) {
            success = false;
        }
    }

    if success {
        let arg_is_int_literal: Vec<bool> = ordered_exprs
            .iter()
            .map(|e| matches!(e, Expression::NumberLiteral(..)))
            .collect();
        if let Some(neo_name) = ctx.resolve_overload(
            &identifier.name,
            named_args.len(),
            &arg_types,
            &arg_is_int_literal,
        ) {
            instructions.push(Instruction::CallFunction {
                name: neo_name,
                arg_count: named_args.len(),
            });
        } else {
            ctx.record_error(format!(
                "no overload of '{}' with {} argument(s)",
                identifier.name,
                named_args.len()
            ));
            success = false;
        }
    }

    if ctx.is_void_function(&identifier.name) {
        return Some(false);
    }
    Some(success)
}
