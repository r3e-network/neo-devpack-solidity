use super::*;

pub(crate) fn try_lower_type_constructor_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    pub(crate) fn lower_contract_like_cast(
        args: &[Expression],
        ctx: &mut LoweringContext,
        instructions: &mut Vec<Instruction>,
    ) -> bool {
        if let Some(arg) = args.first() {
            if matches!(
                infer_type_from_expression(arg, ctx),
                Some(ValueType::Address)
            ) {
                return lower_expression(arg, ctx, instructions);
            }

            if let Some(bytes) = address_bytes_le_from_expression(arg) {
                instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                return true;
            }

            if lower_expression(arg, ctx, instructions) {
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::ByteArray,
                });
                coerce_to_fixed_bytes(20, false, ctx, instructions);
            } else {
                instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                    0u8;
                    20
                ])));
            }
        } else {
            instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                0u8;
                20
            ])));
        }

        true
    }

    // Contract/interface casts represented as plain identifiers:
    // `IFoo(target)` should behave like `address(target)` at runtime.
    if let Expression::Variable(type_id) = func {
        if ctx.is_contract_type_name(&type_id.name) && args.len() == 1 {
            return Some(lower_contract_like_cast(args, ctx, instructions));
        }
    }

    // `import * as NS` namespace-qualified contract/interface casts:
    // `NS.IFoo(target)` should behave like `IFoo(target)`.
    if let Expression::MemberAccess(_, namespace_expr, type_id) = func {
        let is_namespace = matches!(
            namespace_expr.as_ref(),
            Expression::Variable(namespace_id)
                if !ctx.param_index_map.contains_key(&namespace_id.name)
                    && ctx.resolve_local(&namespace_id.name).is_none()
                    && !ctx.state_index_map.contains_key(&namespace_id.name)
                    && !ctx.is_contract_type_name(&namespace_id.name)
        );

        if is_namespace && ctx.is_contract_type_name(&type_id.name) && args.len() == 1 {
            return Some(lower_contract_like_cast(args, ctx, instructions));
        }
    }

    if let Expression::Type(_, ty) = func {
        match ty {
            // Task #128 — `payable(x)` is a Solidity type-only cast (§4.3, §4.7.3).
            // It changes the static type from `address` to `address payable` but MUST
            // preserve the underlying 20-byte value identically. Previously this variant
            // fell through all `try_lower_*` paths and landed in the fallback at
            // `dispatch.rs` which dropped the arg and pushed `BigInt::zero()` (observed
            // as 8 zero bytes for a 20-byte probe) — catastrophic: every `.call{value:}`
            // routed through `payable(a)` would target `address(0)` instead of `a`.
            // The Solidity-spec semantics are a pure runtime identity: evaluate and
            // leave the value on the stack untouched.
            PtType::Payable => {
                if let Some(arg) = args.first() {
                    if !lower_expression(arg, ctx, instructions) {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                            0u8;
                            20
                        ])));
                    }
                } else {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                        0u8;
                        20
                    ])));
                }
                return Some(true);
            }
            PtType::Address | PtType::AddressPayable => {
                if let Some(arg) = args.first() {
                    // Preserve `this` as the executing script hash so manifest permission
                    // inference can recognize self-calls without requiring wildcard entries.
                    if matches!(arg, Expression::Variable(id) if id.name == "this") {
                        if !lower_expression(arg, ctx, instructions) {
                            instructions.push(Instruction::PushLiteral(LiteralValue::Address(
                                vec![0u8; 20],
                            )));
                        }
                        return Some(true);
                    }

                    // Prefer a fixed-width UInt160 literal for common address casts like
                    // `address(0x...)`. If we lowered the numeric literal as an Integer it
                    // may become 21 bytes due to sign-extension rules, which Neo N3 rejects
                    // for UInt160 inputs.
                    if let Some(bytes) = address_bytes_le_from_expression(arg) {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Address(bytes)));
                        return Some(true);
                    }

                    if lower_expression(arg, ctx, instructions) {
                        // Coerce to a fixed-width UInt160 (little-endian) so comparisons with
                        // `address(0)` and storage defaults behave consistently.
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::ByteArray,
                        });
                        coerce_to_fixed_bytes(20, false, ctx, instructions);
                    } else {
                        instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                            0u8;
                            20
                        ])));
                    }
                } else {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Address(vec![
                        0u8;
                        20
                    ])));
                }
                return Some(true);
            }
            PtType::Uint(bits) => {
                let bits = *bits as usize;
                let arg = args.first();
                if arg.is_none_or(|arg| !lower_expression(arg, ctx, instructions)) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                } else if let Some(arg) = arg {
                    // Solidity allows casting `bytesN` and `bool` into integers; NeoVM represents
                    // these as ByteString/Boolean stack items, so coerce them to Integer first.
                    //
                    // Task #111 — `uint256(bytes32)` is a bit-identity reinterpret: Solidity's
                    // bytes32 is a big-endian byte container (MSB at byte 0), but NeoVM's
                    // CONVERT→Integer decodes ByteArray as signed little-endian. Without a
                    // swap, `uint256(bytes32(0x00..01))` parses the low 8 bytes `[0,0,0,0,0,0,0,0]`
                    // as 0 instead of 1. Mirror the opposite direction's handling in
                    // `coerce_to_fixed_bytes` (which REVERSEs on `bytes32(uint)`): emit
                    // REVERSEITEMS on the fixed-bytes input so the LE-interpreting CONVERT
                    // recovers the correct magnitude. `bytes` dynamic gets the same treatment
                    // for consistency — cross-width reinterprets aren't valid Solidity, but if
                    // they sneak through the type checker we prefer "top byte first" semantics.
                    let arg_type = infer_type_from_expression(arg, ctx);
                    if matches!(arg_type, Some(ValueType::ByteArray { .. })) {
                        // Task #207 — real NeoVM accepts REVERSEITEMS for
                        // mutable Buffers/Arrays, but NOT for ByteString.
                        // The old lowering did:
                        //   CONVERT ByteArray; DUP; REVERSEITEMS
                        // which worked in the local runtime but faults under
                        // Neo-Express with:
                        //   Invalid type for REVERSEITEMS: ByteString
                        // Re-materialize the bytes into a NEWBUFFER first,
                        // then reverse the mutable buffer in place before the
                        // final CONVERT→Integer.
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::ByteArray,
                        });
                        materialize_byte_array_buffer(&mut *ctx, instructions, true);
                    }
                    if matches!(
                        arg_type,
                        Some(ValueType::ByteArray { .. } | ValueType::Address | ValueType::Boolean)
                    ) {
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::Integer,
                        });
                    }
                }

                // Solidity integer casts truncate modulo 2^bits.
                // For unsigned, this is equivalent to `value & ((1<<bits)-1)`.
                //
                // Avoid emitting a redundant `& (2^256-1)` mask for common no-op casts like
                // `uint256(1)` so bytecode patterns (and tests) remain stable.
                let needs_mask = if bits != 256 {
                    true
                } else if let Some(arg) = args.first() {
                    if matches!(arg, Expression::Negate(..)) {
                        true
                    } else if let Some(LiteralValue::Integer(value)) = literal_from_expression(arg)
                    {
                        value < BigInt::zero() || value >= (BigInt::one() << 256)
                    } else {
                        matches!(
                            infer_type_from_expression(arg, ctx),
                            Some(ValueType::Integer { signed: true, .. })
                        )
                    }
                } else {
                    false
                };

                if needs_mask {
                    let mask = (BigInt::one() << bits) - BigInt::one();
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
                    instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));
                }
                return Some(true);
            }
            PtType::Int(bits) => {
                let bits = *bits as usize;
                let arg = args.first();
                if arg.is_none_or(|arg| !lower_expression(arg, ctx, instructions)) {
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::zero(),
                    )));
                } else if let Some(arg) = arg {
                    let arg_type = infer_type_from_expression(arg, ctx);
                    if matches!(arg_type, Some(ValueType::ByteArray { .. })) {
                        // `bytesN` is BIG-endian; NeoVM `CONVERT→Integer` reads
                        // little-endian. Reverse first (via a mutable Buffer —
                        // REVERSEITEMS rejects ByteString) so the recovered
                        // magnitude matches the `uintN(bytesN)` path above.
                        // Without this, `int256(bytes32(uint256(1)))` decoded as
                        // 2^248 instead of 1, and diverged from `uintN(bytesN)`.
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::ByteArray,
                        });
                        materialize_byte_array_buffer(&mut *ctx, instructions, true);
                    }
                    if matches!(
                        arg_type,
                        Some(ValueType::ByteArray { .. } | ValueType::Address | ValueType::Boolean)
                    ) {
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::Integer,
                        });
                    }
                }

                // `int256(x)` is a pure bit-reinterpret no-op: every integer is
                // already stored as a 256-bit two's-complement value, so the
                // 256-bit signed range coincides with the stored bits. The
                // truncate + sign-adjust below is only meaningful for NARROW
                // widths; applying it at 256 bits would double-convert (and the
                // `value - 2^256` step needs a 33-byte `2^256` literal a real
                // node rejects).
                if bits == 256 {
                    return Some(true);
                }

                // Solidity signed integer casts:
                // 1) truncate to `bits` low bits (mod 2^bits)
                // 2) interpret as signed using two's complement (sign-extend).
                let mask = (BigInt::one() << bits) - BigInt::one();
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(mask)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::BitAnd));

                let tmp_id = ctx.next_label();
                let value_local = ctx.allocate_local(format!("__int_cast_{tmp_id}"), None);
                instructions.push(Instruction::StoreLocal(value_local));

                let sign_bit = BigInt::one() << (bits.saturating_sub(1));
                let modulus = BigInt::one() << bits;
                let positive_label = ctx.next_label();
                let end_label = ctx.next_label();

                // if value < sign_bit -> already positive
                instructions.push(Instruction::LoadLocal(value_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(sign_bit)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Ge));
                instructions.push(Instruction::JumpIf {
                    target: positive_label,
                });

                // negative: value - 2^bits
                instructions.push(Instruction::LoadLocal(value_local));
                instructions.push(Instruction::PushLiteral(LiteralValue::Integer(modulus)));
                instructions.push(Instruction::BinaryOp(BinaryOperator::Sub));
                instructions.push(Instruction::Jump { target: end_label });

                // positive: value
                instructions.push(Instruction::Label(positive_label));
                instructions.push(Instruction::LoadLocal(value_local));

                instructions.push(Instruction::Label(end_label));
                return Some(true);
            }
            PtType::Bytes(len) => {
                if let Some(arg) = args.first() {
                    let reverse = matches!(
                        infer_type_from_expression(arg, ctx),
                        Some(ValueType::Integer { .. } | ValueType::Boolean)
                    );

                    if lower_expression(arg, ctx, instructions) {
                        instructions.push(Instruction::Convert {
                            target: ConvertTarget::ByteArray,
                        });
                        coerce_to_fixed_bytes(*len as usize, reverse, ctx, instructions);
                    } else {
                        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                            vec![0u8; *len as usize],
                        )));
                    }
                } else {
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        vec![0u8; *len as usize],
                    )));
                }
                return Some(true);
            }
            PtType::DynamicBytes => {
                if args
                    .first()
                    .is_none_or(|arg| !lower_expression(arg, ctx, instructions))
                {
                    instructions.push(Instruction::PushLiteral(
                        LiteralValue::ByteArray(Vec::new()),
                    ));
                }
                return Some(true);
            }
            // Task #171 — `string(bytes_value)` cast is a semantic no-op on
            // NeoVM: Solidity's `string` and `bytes` are both ByteArrays at
            // the stack-item level (only the source-language interpretation
            // differs, with `string` being UTF-8 bytes). Without this handler
            // the `FunctionCall(Type(String), [value])` AST node falls
            // through the dispatch chain to the generic fallback in
            // `lower_function_call_expression`, which drops the argument
            // and pushes `Integer(0)` — corrupting multi-return tuples like
            // `return (string(left), string(right))` into zero scalars that
            // `AbiEncode` encodes as 64 bytes of zeros. Mirror the
            // `DynamicBytes` branch: lower the argument (passing the
            // underlying ByteArray through) and, on lowering failure, push
            // an empty ByteArray as a defensive default.
            PtType::String => {
                if args
                    .first()
                    .is_none_or(|arg| !lower_expression(arg, ctx, instructions))
                {
                    instructions.push(Instruction::PushLiteral(
                        LiteralValue::ByteArray(Vec::new()),
                    ));
                }
                return Some(true);
            }
            PtType::Rational => {
                ctx.record_error_with_suggestion(
                    "fixed-point types (fixed/ufixed) are not supported on NeoVM",
                    "use scaled integer arithmetic instead (e.g., multiply by 10^18 for 18 decimal places)",
                );
                return Some(false);
            }
            _ => {}
        }
    }

    None
}

pub(crate) fn coerce_to_fixed_bytes(
    len: usize,
    reverse: bool,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__bytes_cast_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__bytes_cast_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__bytes_cast_size_{tmp_id}"), None);
    let count_local = ctx.allocate_local(format!("__bytes_cast_count_{tmp_id}"), None);

    // Store the source bytes so we can compute its size.
    instructions.push(Instruction::StoreLocal(src_local));

    // Allocate zero-initialized destination buffer of fixed length.
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    // count = min(size, len)
    let ge_label = ctx.next_label();
    let end_label = ctx.next_label();
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: ge_label });

    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(ge_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::from(len as u64),
    )));
    instructions.push(Instruction::StoreLocal(count_local));
    instructions.push(Instruction::Label(end_label));

    // NeoVM MEMCPY stack order: [dst, dst_offset, src, src_offset, count]
    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(count_local));
    instructions.push(Instruction::MemCpy);
    // Note: MEMCPY modifies the buffer in-place and does NOT push anything
    // back onto the stack. dst_local already holds the buffer reference.

    if reverse {
        // Reverse in place, keeping one reference on the stack.
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::ReverseItems);
    } else {
        instructions.push(Instruction::LoadLocal(dst_local));
    }

    // Canonicalize fixed-byte casts to ByteString so equality checks against
    // storage loads (also ByteString) are value-based and deterministic.
    instructions.push(Instruction::Convert {
        target: ConvertTarget::ByteArray,
    });
}

pub(crate) fn materialize_byte_array_buffer(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    reverse: bool,
) {
    let tmp_id = ctx.next_label();
    let src_local = ctx.allocate_local(format!("__bytearray_src_{tmp_id}"), None);
    let dst_local = ctx.allocate_local(format!("__bytearray_dst_{tmp_id}"), None);
    let size_local = ctx.allocate_local(format!("__bytearray_size_{tmp_id}"), None);

    instructions.push(Instruction::StoreLocal(src_local));

    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::StoreLocal(size_local));

    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::NewBuffer);
    instructions.push(Instruction::StoreLocal(dst_local));

    instructions.push(Instruction::LoadLocal(dst_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(src_local));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::LoadLocal(size_local));
    instructions.push(Instruction::MemCpy);

    if reverse {
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::LoadLocal(dst_local));
        instructions.push(Instruction::ReverseItems);
    } else {
        instructions.push(Instruction::LoadLocal(dst_local));
    }
}
