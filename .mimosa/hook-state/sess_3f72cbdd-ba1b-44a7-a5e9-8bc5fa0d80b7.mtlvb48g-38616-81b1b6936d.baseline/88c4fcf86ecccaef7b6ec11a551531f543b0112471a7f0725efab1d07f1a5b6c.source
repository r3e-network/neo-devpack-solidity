use super::*;

pub(crate) fn try_lower_resolved_builtin_call(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let builtin = resolve_builtin_call(func)?;

    if matches!(&builtin, BuiltinCall::RuntimeNotify) {
        validate_runtime_notify_call(args, ctx);
    }

    if matches!(&builtin, BuiltinCall::TypeOf) {
        ctx.record_error(
            "`type(...)` is only supported via members like `type(uint256).max`, `type(int256).min`, or `type(IInterface).interfaceId`",
        );
        return Some(false);
    }

    let (min_args, max_args) = match &builtin {
        BuiltinCall::RuntimeNotify => (2, Some(2)),
        BuiltinCall::RuntimeCheckWitness => (1, Some(1)),
        // Solidity allows abi.encode()/encodePacked() with zero arguments.
        BuiltinCall::AbiEncode | BuiltinCall::AbiEncodePacked | BuiltinCall::AbiEncodeCall => {
            (0, None)
        }
        BuiltinCall::AbiEncodeWithSignature => (1, None),
        BuiltinCall::AbiDecode => (2, None),
        BuiltinCall::Keccak256 => (1, Some(1)),
        BuiltinCall::Ecrecover => (4, Some(4)),
        BuiltinCall::StorageFind => (1, None),
        BuiltinCall::StoragePut => (2, Some(2)),
        BuiltinCall::StorageGet | BuiltinCall::StorageDelete => (1, Some(1)),
        BuiltinCall::ContractCall => (3, Some(3)),
        BuiltinCall::ContractCallWithFlags => (4, Some(4)),
        BuiltinCall::NotifySerialized => (1, Some(1)),
        BuiltinCall::VerifySignature => (3, Some(3)),
        BuiltinCall::DeployContract => (2, Some(3)),
        BuiltinCall::GetContract | BuiltinCall::GetContractScript => (1, Some(1)),
        BuiltinCall::GetNeoAccountState => (1, Some(1)),
        BuiltinCall::NativeCall { .. } => (0, None),
        BuiltinCall::Syscall(_) => (0, None),
        BuiltinCall::TypeOf => (1, Some(1)),
        BuiltinCall::BytesConcat => (0, None),
        BuiltinCall::PrecompileEcrecover | BuiltinCall::PrecompileModexp => (1, Some(1)),
    };

    if args.len() < min_args || max_args.is_some_and(|max| args.len() > max) {
        ctx.record_error(format!(
            "builtin call requires between {} and {} argument(s), got {}",
            min_args,
            max_args
                .map(|v| v.to_string())
                .unwrap_or_else(|| "∞".to_string()),
            args.len()
        ));
        return Some(false);
    }

    if matches!(builtin, BuiltinCall::AbiEncode | BuiltinCall::AbiEncodeCall) {
        if let Some(result) = lower_abi_encode_args_direct_from_slice(args, ctx, instructions) {
            return Some(result);
        }
    }

    if matches!(builtin, BuiltinCall::AbiEncodePacked) {
        if let Some(result) =
            lower_abi_encode_packed_args_direct_from_slice(args, ctx, instructions)
        {
            return Some(result);
        }
    }

    if matches!(builtin, BuiltinCall::AbiDecode) {
        if let Some(result) = lower_abi_decode_direct(args, ctx, instructions) {
            return Some(result);
        }
    }

    // Task #66 — `abi.encodePacked` width-aware packing for narrow integers.
    //
    // Per Solidity spec, `abi.encodePacked(uintN)` emits exactly N/8 bytes in
    // big-endian form (not a 32-byte slot). The runtime `abiencodepacked`
    // handler can't recover the Solidity type from the StackItem (it sees an
    // `Integer(5)` whether the source was `uint8(5)` or `uint256(5)`). When
    // ALL args have narrow integer types (bits in {8,16,32,64,128}) we
    // side-step the runtime path and lower to a compile-time CAT chain of
    // fixed-width BE byte arrays — `coerce_to_fixed_bytes(N, reverse=true)`
    // converts the LE integer-bytes NeoVM emits into right-aligned BE of
    // exactly N bytes. uint256/bytes/strings/address/bool args fall through
    // to the existing runtime handler (preserves Task #44 semantics).
    //
    // `coerce_to_fixed_bytes` leaves the MEMCPY-returned destination buffer
    // on the stack beneath the canonical ByteString result (its original
    // callers `bytesN(..)`/`address(..)` are single-use so the leak is
    // unobservable). Chaining N packed args would stack up N leaked items
    // which the subsequent `BytesConcat` (only pops top 2) would then
    // mis-concatenate. We follow each coercion with `Swap; Drop` to discard
    // the leak and keep only the canonical result.
    if matches!(builtin, BuiltinCall::AbiEncodePacked) && !args.is_empty() {
        let narrow_widths: Option<Vec<usize>> = args
            .iter()
            .map(|arg| match infer_type_from_expression(arg, ctx) {
                // Bug #23 (packed variant): only UNSIGNED widths may take the
                // zero-filling `coerce_to_fixed_bytes` fast path. Negative
                // signed values need 0xff sign-extension, which the direct
                // path's `emit_abi_fixed_buffer_signed` handles.
                Some(ValueType::Integer {
                    bits,
                    signed: false,
                }) if matches!(bits, 8 | 16 | 32 | 64 | 128) => Some(bits as usize),
                _ => None,
            })
            .collect();
        if let Some(widths) = narrow_widths {
            let mut success = true;
            for (arg, bits) in args.iter().zip(widths.iter()) {
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                    break;
                }
                // Convert the integer to its natural LE byte array (NeoVM
                // stores integers little-endian); then coerce into a fixed
                // N-byte buffer and reverse in place to get right-aligned BE.
                instructions.push(Instruction::Convert {
                    target: ConvertTarget::ByteArray,
                });
                coerce_to_fixed_bytes(bits / 8, true, ctx, instructions);
                // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
            }
            if success {
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: args.len(),
                });
            }
            return Some(success);
        }
    }

    let mut success = true;
    // Task #124 — track flattened arg count for `AbiEncode` when struct args
    // are expanded into per-field stack items. For non-AbiEncode builtins and
    // the struct-less fast path this stays equal to `args.len()`.
    let mut flat_arg_count: usize = args.len();
    match &builtin {
        BuiltinCall::AbiDecode => {
            if let Some(first) = args.first() {
                if !lower_expression(first, ctx, instructions) {
                    success = false;
                }
            }
            // Task #84 — `abi.decode(buf, (T, U, …))` must PANIC when `buf`
            // is too short to hold every declared slot. Emit a bytecode-level
            // guard `DUP ; SIZE ; PUSH expected ; NE ; JMPIFNOT decode_ok ;
            // emit_panic(0x41)` keyed on the second argument's tuple
            // arity when every element is a static 32-byte type. Dynamic
            // tuples (string/bytes, nested) keep the legacy no-guard path.
            //
            // Task #107 — canonical EVM Panic(uint256) envelope so
            // `catch Panic(uint code)` can bind code = 0x41.
            if success {
                if let Some(expected_bytes) = abi_decode_expected_static_bytes(args) {
                    let decode_ok_label = ctx.next_label();
                    instructions.push(Instruction::Dup);
                    instructions.push(Instruction::GetSize);
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(expected_bytes),
                    )));
                    // Solidity's `abi.decode` only requires the buffer to be LONG
                    // ENOUGH (it ignores trailing bytes); only an UNDER-length
                    // buffer reverts. Using `<` (not `!=`) accepts over-length
                    // input, matching ethers/foundry/solc decoding.
                    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
                    instructions.push(Instruction::JumpIf {
                        target: decode_ok_label,
                    });
                    emit_panic(0x41, instructions);
                    instructions.push(Instruction::Label(decode_ok_label));
                }
            }
        }
        BuiltinCall::AbiEncode => {
            // Task #109 — `bytesN(..)` / `address(..)` casts route through
            // `coerce_to_fixed_bytes` which leaves the MEMCPY-returned dst
            // buffer on the stack BENEATH the canonical ByteString result
            // (MEMCPY pushes `dst` back per C-style memcpy semantics — see
            // src/runtime/execution/execution_impl_part3_bytes.rs::memcpy_bytes).
            // When such a cast appears as an argument to a multi-arg builtin
            // like `abi.encode(uint256 x, bytes32 y)`, the subsequent PACK
            // picks up the leaked buffer instead of the earlier scalar arg,
            // corrupting the encoded payload.
            //
            // Task #124 — struct-typed args must be flattened into per-field
            // stack items before the runtime `abiencode` handler sees them.
            // Otherwise the struct's `StackItem::Array` shape hits the
            // dynamic-type classifier and emits Task #121's offset+length+
            // elements buffer instead of the EVM-canonical field-concat
            // (e.g. `keccak(abi.encode(Voucher{a,b,c}))` diverges from the
            // 96-byte-BE oracle). Detect all-static structs and emit
            // `lower_expression(expr); push i; ArrayGet` per field so the
            // AbiEncode builtin receives N scalar items (Integer / ByteArray
            // / Boolean) that `abi_pad32_be` can slot-encode directly.
            //
            // Non-struct args follow the standard lower-then-leak-drop path.
            flat_arg_count = 0;
            for arg in args {
                if let Some(field_count) =
                    try_flatten_struct_arg_for_abi_encode(arg, ctx, instructions, &mut success)
                {
                    flat_arg_count += field_count;
                    continue;
                }
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                    continue;
                }
                // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
                flat_arg_count += 1;
            }
        }
        _ => {
            // Task #109 — `bytesN(..)` / `address(..)` casts route through
            // `coerce_to_fixed_bytes` which leaves the MEMCPY-returned dst
            // buffer on the stack BENEATH the canonical ByteString result
            // (MEMCPY pushes `dst` back per C-style memcpy semantics — see
            // src/runtime/execution/execution_impl_part3_bytes.rs::memcpy_bytes).
            // When such a cast appears as an argument to a multi-arg builtin
            // like `abi.encode(uint256 x, bytes32 y)`, the subsequent PACK
            // picks up the leaked buffer instead of the earlier scalar arg,
            // corrupting the encoded payload (e.g. abi.encode produces
            // bytes32(y) || bytes32(y) instead of bytes32(x) || bytes32(y)).
            // Mirror the Task #66/#89 packed-encoding fix in builtins.rs /
            // member_access.rs: follow each leaky arg with `Swap; Drop` to
            // discard the leaked buffer and keep only the canonical result.
            for arg in args {
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                    continue;
                }
                // No Swap; Drop needed: real NeoVM MEMCPY pushes nothing.
            }
        }
    }

    if success {
        match builtin {
            BuiltinCall::RuntimeNotify
            | BuiltinCall::RuntimeCheckWitness
            | BuiltinCall::AbiEncodePacked
            | BuiltinCall::AbiEncodeCall
            | BuiltinCall::AbiDecode
            | BuiltinCall::Keccak256
            | BuiltinCall::Ecrecover
            | BuiltinCall::StorageFind
            | BuiltinCall::StoragePut
            | BuiltinCall::StorageGet
            | BuiltinCall::StorageDelete
            | BuiltinCall::ContractCallWithFlags
            | BuiltinCall::NotifySerialized
            | BuiltinCall::VerifySignature
            | BuiltinCall::DeployContract
            | BuiltinCall::GetContract
            | BuiltinCall::GetContractScript
            | BuiltinCall::GetNeoAccountState
            | BuiltinCall::NativeCall { .. }
            | BuiltinCall::Syscall(_)
            | BuiltinCall::BytesConcat
            | BuiltinCall::PrecompileEcrecover
            | BuiltinCall::PrecompileModexp => {
                instructions.push(Instruction::CallBuiltin {
                    builtin,
                    arg_count: args.len(),
                });
            }
            BuiltinCall::AbiEncode => {
                // Task #124 — use the flattened arg count (per-field for
                // struct args) so the runtime `abiencode` handler PACKs the
                // correct number of scalar slots. For struct-less calls this
                // equals `args.len()` so the static/dynamic encoding paths
                // (Tasks #44/#72/#73/#121) keep their existing semantics.
                instructions.push(Instruction::CallBuiltin {
                    builtin,
                    arg_count: flat_arg_count,
                });
            }
            BuiltinCall::ContractCall => {
                if ctx.is_safe {
                    // In safe (view/pure) contexts, default contract calls to read-only.
                    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
                        BigInt::from(0x05u8),
                    )));
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::ContractCallWithFlags,
                        arg_count: 4,
                    });
                } else {
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::ContractCall,
                        arg_count: args.len(),
                    });
                }
            }
            BuiltinCall::AbiEncodeWithSignature => {
                let mut selector = Vec::new();
                if let Some(Expression::StringLiteral(parts)) = args.first() {
                    let bytes = string_literal_bytes(parts);
                    let mut hasher = Keccak256::new();
                    hasher.update(&bytes);
                    let digest = hasher.finalize();
                    selector.extend_from_slice(&digest[..4]);
                }
                for _ in args {
                    instructions.push(Instruction::Drop(ValueType::Any));
                }
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(selector)));
            }
            BuiltinCall::TypeOf => {}
        }
    }

    Some(success)
}

/// Task #84 — return `arity * 32` when `abi.decode(buf, types)`'s `types`
/// argument is a tuple whose elements are all static 32-byte Solidity types
/// (`uintN`, `intN`, `address[ payable]`, `bool`, `bytesN`). Returns `None`
/// for dynamic or mixed tuples; callers then fall back to the legacy
/// no-guard path since a simple size comparison is insufficient.
pub(crate) fn abi_decode_expected_static_bytes(args: &[Expression]) -> Option<u32> {
    let slot_count: u32 = match args.get(1)? {
        Expression::List(_, params) => {
            let mut count: u32 = 0;
            for (_, param) in params {
                if !is_static_abi_type(&param.as_ref()?.ty) {
                    return None;
                }
                count = count.checked_add(1)?;
            }
            count
        }
        Expression::Parenthesis(_, inner) => {
            if !is_static_abi_type(inner.as_ref()) {
                return None;
            }
            1
        }
        other => {
            if !is_static_abi_type(other) {
                return None;
            }
            1
        }
    };
    slot_count.checked_mul(32)
}

/// Task #84 — true iff `expr` names a Solidity type whose ABI encoding is
/// exactly one 32-byte slot (no offset/length tail).
pub(crate) fn is_static_abi_type(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Type(
            _,
            PtType::Uint(_)
                | PtType::Int(_)
                | PtType::Address
                | PtType::AddressPayable
                | PtType::Payable
                | PtType::Bool
                | PtType::Bytes(_)
        )
    )
}

/// Task #127 — true iff `expr` names a Solidity type whose ABI encoding
/// uses the dynamic-tail layout: `bytes`, `string`, or `T[]` (dynamic
/// array). In the ABI head these take a single 32-byte offset slot
/// that points to a length + payload tail section. Used by the mixed
/// static+dynamic tuple passthrough in `return_revert.rs` to recognise
/// `return abi.decode(buf, (uint, string, address))`-style shapes
/// whose canonical output IS the input buffer verbatim.
pub(crate) fn is_dynamic_abi_type(expr: &Expression) -> bool {
    match expr {
        Expression::Type(_, PtType::DynamicBytes | PtType::String) => true,
        // `T[]` appears as an ArraySubscript with no length index.
        Expression::ArraySubscript(_, _inner, None) => true,
        _ => false,
    }
}
