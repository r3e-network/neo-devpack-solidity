use super::*;

pub(crate) fn lower_yul_statement(
    stmt: &solang_parser::pt::YulStatement,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    use solang_parser::pt::YulStatement;
    match stmt {
        YulStatement::VariableDeclaration(_, idents, init) => {
            if idents.len() != 1 {
                // Multi-return yul locals (`let a, b := f()`) are out of
                // scope for Task #99.
                return false;
            }
            let ident = &idents[0];
            let name = ident.id.name.clone();
            let slot = ctx.allocate_local(format!("__yul_var_{name}"), None);
            state.yul_locals.insert(name, slot);
            if let Some(expr) = init {
                if !lower_yul_expression(expr, state, ctx, instructions) {
                    return false;
                }
                instructions.push(Instruction::StoreLocal(slot));
            } else {
                // yul default-inits decls to 0.
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                    BigInt::zero(),
                )));
                instructions.push(Instruction::StoreLocal(slot));
            }
            true
        }
        YulStatement::Assign(_, targets, value) => {
            if targets.len() != 1 {
                return false;
            }
            let solang_parser::pt::YulExpression::Variable(ident) = &targets[0] else {
                return false;
            };
            // Task #100 — resolution order matches yul's reference semantics:
            // (1) yul-local `let x` bindings in this block; (2) outer Solidity
            // locals the assembly block has visibility on (e.g. `uint v; assembly { v := tload(0) }`).
            // Task #183 — also (3) outer Solidity parameters (`function f(uint v) { assembly { v := tload(0) }}`).
            // Parameters live in `param_index_map` and are written via
            // `StoreParameter` / NeoVM STARG. Mirror Task #156's tuple-assign
            // fix (TupleTarget::ExistingParameter) on the yul write side.
            enum YulAssignTarget {
                Local(usize),
                Parameter(usize),
            }
            let target = if let Some(&slot) = state.yul_locals.get(&ident.name) {
                YulAssignTarget::Local(slot)
            } else if let Some(&param_index) = ctx.param_index_map.get(&ident.name) {
                YulAssignTarget::Parameter(param_index)
            } else if let Some(slot) = ctx.resolve_local(&ident.name) {
                YulAssignTarget::Local(slot)
            } else {
                // Assigning to an un-declared yul identifier: out of scope.
                return false;
            };
            if !lower_yul_expression(value, state, ctx, instructions) {
                return false;
            }
            match target {
                YulAssignTarget::Local(slot) => {
                    instructions.push(Instruction::StoreLocal(slot));
                }
                YulAssignTarget::Parameter(param_index) => {
                    instructions.push(Instruction::StoreParameter(param_index));
                }
            }
            true
        }
        YulStatement::FunctionCall(call) => {
            lower_yul_function_call_as_statement(call, state, ctx, instructions)
        }
        YulStatement::Block(inner) => {
            lower_yul_block_stmts(&inner.statements, state, ctx, instructions)
        }
        // Task #200 — yul `if <cond> <body>`. The cond expression evaluates
        // to a yul uint256 (0 ⇒ false, non-zero ⇒ true). The NeoVM IR
        // `JumpIf` jumps when the top-of-stack is FALSY (see
        // `src/cli/bytecode/bytecode_emit_ir.rs:319` — "IR JumpIf branches
        // when the condition is false."), so the lowering is:
        //     <eval cond>
        //     JumpIf end_label      ; skip body when cond == 0
        //     <eval body>
        //     Label(end_label)
        YulStatement::If(_, cond, body) => {
            let end_label = ctx.next_label();
            if !lower_yul_expression(cond, state, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::JumpIf { target: end_label });
            if !lower_yul_block_stmts(&body.statements, state, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::Label(end_label));
            true
        }
        // Task #200 — yul `for { init } cond { post } { body }`. Classic
        // condition-top loop. `init` statements run once before entering;
        // `cond` is re-evaluated at the top of every iteration and a FALSY
        // value exits the loop; `post` runs after each body iteration and
        // is the continue-target (so `continue` re-enters at post, then
        // falls through to the condition check). Mirrors the canonical
        // Solidity `for` lowering in
        // src/ir/statements/dispatch/control_flow.rs::lower_for_statement.
        YulStatement::For(for_stmt) => {
            // Init statements are lowered in the enclosing scope so any
            // yul-locals they declare (via `let i := 0`) remain visible
            // to the condition / post / body — which matches yul semantics
            // (`for { let i := 0 } lt(i, n) { i := add(i, 1) } { ... }`).
            if !lower_yul_block_stmts(&for_stmt.init_block.statements, state, ctx, instructions) {
                return false;
            }

            let loop_start = ctx.next_label();
            let post_label = ctx.next_label();
            let loop_end = ctx.next_label();

            instructions.push(Instruction::Label(loop_start));
            if !lower_yul_expression(&for_stmt.condition, state, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::JumpIf { target: loop_end });

            // Register break/continue targets so yul `break` / `continue`
            // (if/when we lower them) and — for symmetry with the Solidity
            // control-flow lowering — land on the right labels. `continue`
            // jumps to `post_label` (run post, then re-check cond).
            ctx.push_loop(post_label, loop_end);
            let body_ok = lower_yul_block_stmts(
                &for_stmt.execution_block.statements,
                state,
                ctx,
                instructions,
            );
            ctx.pop_loop();
            if !body_ok {
                return false;
            }

            instructions.push(Instruction::Label(post_label));
            if !lower_yul_block_stmts(&for_stmt.post_block.statements, state, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::Jump { target: loop_start });
            instructions.push(Instruction::Label(loop_end));
            true
        }
        // Task #200 — yul `switch <expr> case v1 { ... } ... default { ... }`.
        // Evaluate the discriminant once into a fresh local, then emit a
        // linear case chain: for each case, compare the local against the
        // case literal and `JumpIf` over the body when unequal. After the
        // body, `Jump` to the shared end label. The default block (if any)
        // sits just before the end label so an unmatched dispatch falls
        // through to it naturally. Yul guarantees (per foundry-solang-parser
        // at solang-parser-0.3.5/src/pt.rs:1593) that `cases` contains only
        // `YulSwitchOptions::Case` and `default` is exactly `Default`.
        YulStatement::Switch(switch_stmt) => {
            if !lower_yul_expression(&switch_stmt.condition, state, ctx, instructions) {
                return false;
            }
            let disc_label = ctx.next_label();
            let disc_local = ctx.allocate_local(format!("__yul_switch_disc_{disc_label}"), None);
            instructions.push(Instruction::StoreLocal(disc_local));

            let end_label = ctx.next_label();

            for case_opt in &switch_stmt.cases {
                let solang_parser::pt::YulSwitchOptions::Case(_, value_expr, body) = case_opt
                else {
                    // Parser guarantees only Case here; defensive bail if
                    // that invariant ever slips.
                    return false;
                };
                let next_case_label = ctx.next_label();
                // Compare disc against the case literal; `JumpIf` skips the
                // body when they differ (Eq ⇒ 1 truthy, stays; Ne ⇒ 0 falsy,
                // JumpIf fires).
                instructions.push(Instruction::LoadLocal(disc_local));
                if !lower_yul_expression(value_expr, state, ctx, instructions) {
                    return false;
                }
                instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::Integer,
                });
                instructions.push(Instruction::JumpIf {
                    target: next_case_label,
                });
                if !lower_yul_block_stmts(&body.statements, state, ctx, instructions) {
                    return false;
                }
                instructions.push(Instruction::Jump { target: end_label });
                instructions.push(Instruction::Label(next_case_label));
            }

            if let Some(default_opt) = &switch_stmt.default {
                let solang_parser::pt::YulSwitchOptions::Default(_, default_body) = default_opt
                else {
                    return false;
                };
                if !lower_yul_block_stmts(&default_body.statements, state, ctx, instructions) {
                    return false;
                }
            }
            instructions.push(Instruction::Label(end_label));
            true
        }
        // Task #200 — yul `break` / `continue` jump to the innermost loop's
        // break / continue labels (both pushed by the `For` arm above). If
        // these appear outside a loop, solang's parser already rejects the
        // source, but defensively we also return false so the enclosing
        // assembly bails to the no-op warning path rather than emit a Jump
        // with no matching Label.
        YulStatement::Break(_) => {
            if let Some(label) = ctx.break_target() {
                instructions.push(Instruction::Jump { target: label });
                true
            } else {
                false
            }
        }
        YulStatement::Continue(_) => {
            if let Some(label) = ctx.continue_target() {
                instructions.push(Instruction::Jump { target: label });
                true
            } else {
                false
            }
        }
        // Out of scope: leave/FunctionDefinition.
        _ => false,
    }
}

/// Lower a yul function-call used as a top-level statement. Handles the
/// side-effect opcodes (mstore, return) whose yul signatures have no return
/// values. `mload`, `add`, etc. are only valid as expressions.
pub(crate) fn lower_yul_function_call_as_statement(
    call: &solang_parser::pt::YulFunctionCall,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let name = call.id.name.as_str();
    match name {
        "mstore" => {
            if call.arguments.len() != 2 {
                return false;
            }
            lower_yul_mstore(
                &call.arguments[0],
                &call.arguments[1],
                state,
                ctx,
                instructions,
            )
        }
        "mstore8" => {
            // mstore8 would require a single-byte write path distinct from
            // the 32-byte MEMCPY we use for mstore.
            false
        }
        "tstore" => {
            // Task #100 — EIP-1153 transient store. `tstore(slot, value)`
            // writes `value` into the per-invocation `__yul_transient` map
            // under key `slot`. No persistence beyond the current function
            // frame (which matches EIP-1153's per-tx semantics because each
            // runtime call is one tx in this host).
            if call.arguments.len() != 2 {
                return false;
            }
            lower_yul_tstore(
                &call.arguments[0],
                &call.arguments[1],
                state,
                ctx,
                instructions,
            )
        }
        "return" => {
            if call.arguments.len() != 2 {
                return false;
            }
            lower_yul_return(
                &call.arguments[0],
                &call.arguments[1],
                state,
                ctx,
                instructions,
            )
        }
        "returndatacopy" => {
            // Task #184 — `returndatacopy(dst, src, len)` copies `len` bytes
            // from the last-call returndata buffer into yul memory at `dst`,
            // reading from returndata offset `src`. Currently modeled against
            // a lazily-initialised empty `__yul_returndata` buffer (see
            // `ensure_returndata` comments): any non-zero-length read panics
            // with `"returndata: read past returndatasize"` because no prior
            // external call has populated the buffer. A follow-up task can
            // extend `Target(t).f()` / CALLT / DYNCALL sites to stash the
            // callee's return bytes into the same slot so this opcode
            // recovers the real payload.
            if call.arguments.len() != 3 {
                return false;
            }
            lower_yul_returndatacopy(
                &call.arguments[0],
                &call.arguments[1],
                &call.arguments[2],
                state,
                ctx,
                instructions,
            )
        }
        _ => false,
    }
}

/// Lower a yul expression, leaving its integer value on the NeoVM stack.
pub(crate) fn lower_yul_expression(
    expr: &solang_parser::pt::YulExpression,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    use solang_parser::pt::YulExpression;
    match expr {
        YulExpression::NumberLiteral(_, integer, _exp, _) => match integer.parse::<BigInt>() {
            Ok(value) => {
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                true
            }
            Err(_) => false,
        },
        YulExpression::HexNumberLiteral(_, raw, _) => {
            let digits = raw.trim_start_matches("0x").trim_start_matches("0X");
            match BigInt::parse_bytes(digits.as_bytes(), 16) {
                Some(value) => {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(value)));
                    true
                }
                None => false,
            }
        }
        YulExpression::BoolLiteral(_, value, _) => {
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::from(u8::from(*value)),
            )));
            true
        }
        YulExpression::Variable(ident) => {
            if let Some(&slot) = state.yul_locals.get(&ident.name) {
                instructions.push(Instruction::LoadLocal(slot));
                return true;
            }
            // Task #183 — a yul identifier may also resolve to a function
            // parameter (e.g. `function f(bytes32 x) { assembly { mstore(0, x) }}`).
            // Parameters live in `param_index_map` and are read via
            // `LoadParameter` / NeoVM LDARG. Task #99 covered yul-locals and
            // Solidity-locals (both in `local_index_map`) but missed params.
            // Mirror `lower_variable_expression`: check params first, then
            // Solidity-locals.
            if let Some(&param_index) = ctx.param_index_map.get(&ident.name) {
                instructions.push(Instruction::LoadParameter(param_index));
                return true;
            }
            if let Some(slot) = ctx.resolve_local(&ident.name) {
                instructions.push(Instruction::LoadLocal(slot));
                return true;
            }
            false
        }
        YulExpression::FunctionCall(call) => {
            lower_yul_function_call_as_expression(call, state, ctx, instructions)
        }
        _ => false,
    }
}

pub(crate) fn lower_yul_function_call_as_expression(
    call: &solang_parser::pt::YulFunctionCall,
    state: &mut YulLoweringState,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let name = call.id.name.as_str();
    match name {
        "mload" => {
            if call.arguments.len() != 1 {
                return false;
            }
            lower_yul_mload(&call.arguments[0], state, ctx, instructions)
        }
        "tload" => {
            // Task #100 — EIP-1153 transient load. Returns the value stored
            // under `slot` in the `__yul_transient` map, or 0 if the slot
            // has never been tstore'd in this invocation.
            if call.arguments.len() != 1 {
                return false;
            }
            lower_yul_tload(&call.arguments[0], state, ctx, instructions)
        }
        "returndatasize" => {
            // Task #184 — `returndatasize()` returns the byte length of the
            // last-call returndata buffer. Because the Task #184 minimal
            // surface leaves `__yul_returndata` at its empty-byte seed, this
            // evaluates to 0 on any top-level call. Added here so yul bodies
            // that guard `returndatacopy` with a `returndatasize()` check
            // (idiomatic EVM pattern: `if lt(returndatasize(), len) { revert }`)
            // compile without dropping to the legacy no-op warning path.
            if !call.arguments.is_empty() {
                return false;
            }
            let rd_local = state.ensure_returndata(ctx);
            instructions.push(Instruction::LoadLocal(rd_local));
            instructions.push(Instruction::GetSize);
            true
        }
        "add" | "sub" | "mul" => {
            lower_two_arg_yul_call(call, state, ctx, instructions, |n| match n {
                "add" => BinaryOperator::Add,
                "sub" => BinaryOperator::Sub,
                _ => BinaryOperator::Mul, // "mul" — the only remaining arm
            })
        }
        "div" | "mod" => {
            // Yul (EVM) semantics: division/modulo by zero yields 0, NOT a fault.
            // (Unlike high-level Solidity, which Panics 0x12 — that guard lives in
            // the binary-expression path, not here.)
            if call.arguments.len() != 2 {
                return false;
            }
            if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
                return false;
            }
            if !lower_yul_expression(&call.arguments[1], state, ctx, instructions) {
                return false;
            }
            let tmp = ctx.next_label();
            let b_local = ctx.allocate_local(format!("__yul_div_b_{tmp}"), None);
            let a_local = ctx.allocate_local(format!("__yul_div_a_{tmp}"), None);
            instructions.push(Instruction::StoreLocal(b_local)); // divisor (top)
            instructions.push(Instruction::StoreLocal(a_local)); // dividend
            let nonzero = ctx.next_label();
            let done = ctx.next_label();
            // `JumpIf` jumps when the condition is FALSE: divisor != 0 -> divide.
            instructions.push(Instruction::LoadLocal(b_local));
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
            instructions.push(Instruction::JumpIf { target: nonzero });
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::Jump { target: done });
            instructions.push(Instruction::Label(nonzero));
            instructions.push(Instruction::LoadLocal(a_local));
            instructions.push(Instruction::LoadLocal(b_local));
            // EVM div/mod are UNSIGNED; native NeoVM DIV/MOD are signed and
            // wrong for operands >= 2^255. Route through the software unsigned
            // divmod (the divisor is provably non-zero here).
            emit_u256_divmod_ir(ctx, instructions, name == "mod");
            instructions.push(Instruction::Label(done));
            true
        }
        "and" | "or" | "xor" => {
            lower_two_arg_yul_call(call, state, ctx, instructions, |n| match n {
                "and" => BinaryOperator::BitAnd,
                "or" => BinaryOperator::BitOr,
                _ => BinaryOperator::BitXor, // "xor" — the only remaining arm
            })
        }
        "shl" | "shr" => {
            // Yul shift args are (shift_amount, value); NeoVM's BinaryOp
            // Shl/Shr take (value, shift_amount) bottom-up.
            if call.arguments.len() != 2 {
                return false;
            }
            if !lower_yul_expression(&call.arguments[1], state, ctx, instructions) {
                return false;
            }
            if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
                return false;
            }
            // Stack: [value, shift]. EVM `shr` is a LOGICAL shift; native NeoVM
            // SHR is arithmetic and sign-extends a high-bit-set 256-bit word.
            if name == "shr" {
                emit_u256_logical_shr_ir(ctx, instructions);
            } else {
                instructions.push(Instruction::BinaryOp(BinaryOperator::Shl));
            }
            true
        }
        "lt" | "gt" | "eq" => {
            if call.arguments.len() != 2 {
                return false;
            }
            if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
                return false;
            }
            if !lower_yul_expression(&call.arguments[1], state, ctx, instructions) {
                return false;
            }
            // EVM `lt`/`gt` are UNSIGNED comparisons; native NeoVM LT/GT are
            // signed and wrong for 256-bit words >= 2^255 (e.g. after a `sub`
            // underflow produces a negative-looking value). Route lt/gt through
            // the unsigned-256 compare; `eq` is sign-agnostic.
            match name {
                "lt" => emit_u256_unsigned_compare(instructions, BinaryOperator::Lt),
                "gt" => emit_u256_unsigned_compare(instructions, BinaryOperator::Gt),
                _ => instructions.push(Instruction::BinaryOp(BinaryOperator::Eq)),
            }
            // Yul returns 0/1; convert the NeoVM Boolean to an Integer so
            // the value can chain into arithmetic (e.g. `add(lt(...), 1)`).
            instructions.push(Instruction::Convert {
                target: ConvertTarget::Integer,
            });
            true
        }
        "iszero" => {
            if call.arguments.len() != 1 {
                return false;
            }
            if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
                return false;
            }
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                BigInt::zero(),
            )));
            instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
            instructions.push(Instruction::Convert {
                target: ConvertTarget::Integer,
            });
            true
        }
        "not" => {
            if call.arguments.len() != 1 {
                return false;
            }
            if !lower_yul_expression(&call.arguments[0], state, ctx, instructions) {
                return false;
            }
            // EVM `not(x)` is the 256-bit complement `2^256-1-x`, NOT NeoVM
            // INVERT's arbitrary-precision `-x-1`. XOR with the 256-bit all-ones
            // literal routes through the runtime's wide BitXor, which masks the
            // result to 256 bits (so `not(0)` == 2^256-1, and `and(x, not(mask))`
            // is correct).
            let max_u256: BigInt = (BigInt::one() << 256usize) - BigInt::one();
            instructions.push(Instruction::PushLiteral(LiteralValue::Integer(max_u256)));
            instructions.push(Instruction::BinaryOp(BinaryOperator::BitXor));
            true
        }
        _ => false,
    }
}
