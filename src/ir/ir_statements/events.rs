use super::*;

/// Lower a Solidity `emit Event(arg1, arg2, ...)` expression.
///
/// Two emission shapes exist, selected per event declaration:
///
/// 1. **Native NEP token shape** — when the declaration matches a NEP
///    standard `Transfer` signature (see `native_transfer_standard`):
///    NEP-17 `Transfer(address, address, uint256)` or NEP-11
///    `Transfer(address, address, uint256, bytes32|bytes)`, any
///    indexed-ness, non-anonymous. The notification carries the NATIVE Neo
///    payload `[from, to, amount(, tokenId)]` with NO topic0 so wallets,
///    indexers and NEP trackers can read it. `from`/`to` are 20-byte
///    ByteStrings and the zero address maps to `Null` at runtime (NEP-17
///    spec: mint => from is null, burn => to is null); `amount` is
///    CONVERTed to Integer; `tokenId` (NEP-11) passes through as lowered
///    (ByteString). The manifest declares `from: Hash160, to: Hash160,
///    amount: Integer (, tokenId)` — see the events section of
///    cli_manifest/build.rs, which shares the `native_transfer_standard`
///    predicate.
///
/// 2. **EVM-canonical log shape** — every other event. Per the EVM ABI
///    spec (Solidity handbook §"Events"), an `emit` produces:
///
///    ```text
///    topic[0]   = keccak256("Name(type1,type2,...)") — the canonical
///                 signature hash, always 32 bytes, present for
///                 non-anonymous events only.
///    topic[1..] = one 32-byte slot per `indexed` parameter:
///                 - static types (uint, int, bool, address, bytesN,
///                   enum): left-padded big-endian value
///                 - dynamic types (string, bytes, arrays, structs):
///                   keccak256(value).
///    data       = abi.encode(non_indexed_args...) — a single ByteArray
///                 produced by the StdLib.abiEncode helper (BE-padded,
///                 concatenated 32-byte slots per Task #44).
///    ```
///
/// # Bytecode shape delivered to System.Runtime.Notify
///
/// The runtime expects `Notify(eventName, stateArray)`. We co-opt that
/// signature to carry the EVM shape:
///
///   eventName  = human-readable event name (UTF-8, manifest-compatible)
///   `stateArray = [topic[0], topic[1], ..., data]`
///
/// The eventName is ALWAYS the declared Solidity event name — including for
/// `anonymous` events. Neo nodes >= 3.6 (HF_Basilisk) validate every
/// notification against the contract manifest (the event name must be
/// declared and the state-item count must match the declared parameter
/// count), so an empty / synthesized name would fault on-chain. Anonymous-
/// ness therefore only controls the EVM topic model (topic0 suppressed,
/// state = `[topic1, ..., topicN, data]`); the Neo-level event name and the
/// manifest declaration use the declaration name. The manifest declares the
/// matching wire shape (`[topic0,] indexed..., data` — all ByteArray) so
/// post-Basilisk validation passes.
///
/// The bundled emulator's `System.Runtime.Notify` handler detects the EVM
/// shape by checking for a 32-byte first state item and rebuilds a proper
/// EVM `LogEntry { topics, data }`. Legacy `Syscalls.notify(short_name,
/// payload)` paths without the 32-byte marker remain on the original
/// Neo-native path, as do native-shaped NEP Transfer notifications (whose
/// first item is a 20-byte ByteString or Null).
///
/// # No silent fallback
///
/// Emitting an event that is not declared in the manifest — or whose
/// state-item count mismatches the declared parameter count — faults on
/// every Neo node >= 3.6. The lowering therefore records a compile ERROR
/// instead of falling back to a mismatched legacy `Notify` when:
///   * the event name has no registered `EventSignature` (e.g. the
///     inherited-event resolution lost the declaration), or
///   * the emit's argument count differs from the declaration, or
///   * an argument expression cannot be lowered.
pub(crate) fn lower_emit(
    expr: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let Expression::FunctionCall(_, func, args) = expr else {
        return;
    };
    let Expression::Variable(identifier) = func.as_ref() else {
        return;
    };

    let Some(signature) = ctx.event_evm_signature(&identifier.name).cloned() else {
        // No declaration => nothing in the manifest to validate against.
        // Notifying anyway would fault on Neo nodes >= 3.6 (HF_Basilisk),
        // so fail the compile loudly instead of shipping a faulting emit.
        ctx.record_error_with_suggestion(
            format!(
                "emit references event `{}` which has no resolved declaration; the event would \
                 be missing from the contract manifest and the notification would fault on Neo \
                 nodes >= 3.6 (HF_Basilisk validates notifications against the manifest)",
                identifier.name
            ),
            "declare the event in the emitting contract (or a base contract it inherits)",
        );
        return;
    };

    if args.len() != signature.params.len() {
        ctx.record_error_with_suggestion(
            format!(
                "emit of event `{}` passes {} argument(s) but the declaration has {} \
                 parameter(s); the notification would mismatch the manifest declaration and \
                 fault on Neo nodes >= 3.6",
                identifier.name,
                args.len(),
                signature.params.len()
            ),
            "match the emit's argument list to the event declaration",
        );
        return;
    }

    let canonical_types: Vec<String> = signature
        .params
        .iter()
        .map(|param| param.canonical_type.clone())
        .collect();
    if native_transfer_standard(&identifier.name, signature.is_anonymous, &canonical_types)
        .is_some()
    {
        lower_emit_native_transfer(&identifier.name, args, ctx, instructions);
        return;
    }

    lower_emit_evm_shape(&identifier.name, &signature, args, ctx, instructions);
}

/// Lower a NEP-17 / NEP-11 standard `Transfer` emit to the NATIVE Neo
/// notification payload `[from, to, amount(, tokenId)]` (no topic0).
///
/// `from` / `to` get a runtime zero-address => `Null` coercion so mint /
/// burn notifications follow the NEP convention (`from == null` for mint,
/// `to == null` for burn) that wallets and indexers expect. `amount` is
/// CONVERTed to Integer so the manifest's declared `Integer` type is
/// honoured even if the value arrives as a byte string.
pub(crate) fn lower_emit_native_transfer(
    event_name: &str,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let original_len = instructions.len();
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        event_name.as_bytes().to_vec(),
    )));

    for (idx, arg) in args.iter().enumerate() {
        if !lower_expression(arg, ctx, instructions) {
            instructions.truncate(original_len);
            ctx.record_error(format!(
                "could not lower argument {idx} of `emit {event_name}(...)`; refusing to emit a \
                 partial notification"
            ));
            return;
        }
        match idx {
            // from / to: map the zero address to Null per NEP-17/NEP-11.
            0 | 1 => emit_zero_address_to_null(ctx, instructions),
            // amount: canonicalize to Integer (manifest declares Integer).
            2 => instructions.push(Instruction::Convert {
                target: ConvertTarget::Integer,
            }),
            // tokenId (NEP-11): pass through as lowered (ByteString).
            _ => {}
        }
    }

    if let Some(index) = ctx.event_index_map.get(event_name) {
        instructions.push(Instruction::EmitEvent {
            event_index: *index,
            arg_count: args.len(),
        });
    } else {
        instructions.push(Instruction::EmitEventByName {
            name: event_name.to_string(),
            arg_count: args.len(),
        });
    }
}

/// Replace the 20-byte address on top of the stack with `Null` when it is
/// the zero address (numeric value 0); leave any other value untouched.
///
/// The check is numeric (`CONVERT Integer; PUSH0; EQUAL`) rather than a
/// byte comparison because the lowered address may surface as a ByteString
/// OR a Buffer (e.g. via `coerce_to_fixed_bytes` / MEMCPY) — NeoVM's EQUAL
/// compares Buffers by reference, so a byte-equality probe against a
/// zero-literal would silently miss Buffer-shaped zero addresses on real
/// nodes. CONVERT-to-Integer handles ByteString, Buffer, Integer and
/// Boolean uniformly (any all-zero encoding => 0).
pub(crate) fn emit_zero_address_to_null(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let end_label = ctx.next_label();

    // [addr] -> [addr, addr] -> [addr, addr_as_int] -> [addr, is_zero]
    instructions.push(Instruction::Dup);
    instructions.push(Instruction::Convert {
        target: ConvertTarget::Integer,
    });
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    // IR `JumpIf` lowers to NeoVM JMPIFNOT_L: it branches when the
    // condition is FALSE — i.e. skip the Null substitution for any
    // non-zero address and fall through for the zero address.
    instructions.push(Instruction::JumpIf { target: end_label });
    instructions.push(Instruction::Drop(ValueType::Address));
    instructions.push(Instruction::PushLiteral(LiteralValue::Null));
    instructions.push(Instruction::Label(end_label));
}

pub(crate) fn lower_emit_evm_shape(
    event_name: &str,
    signature: &EventSignature,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) {
    let original_len = instructions.len();

    // Partition args into (indexed, non_indexed) by parameter index. The
    // caller has already verified `args.len() == signature.params.len()`.
    let mut indexed: Vec<&Expression> = Vec::new();
    let mut indexed_params: Vec<&EventParamInfo> = Vec::new();
    let mut non_indexed: Vec<&Expression> = Vec::new();
    for (arg, info) in args.iter().zip(signature.params.iter()) {
        if info.indexed {
            indexed.push(arg);
            indexed_params.push(info);
        } else {
            non_indexed.push(arg);
        }
    }

    // Step 1: push the `eventName` argument for System.Runtime.Notify.
    //
    // The eventName must be a UTF-8-decodable string because Neo's RPC and
    // log-emit layers treat it as a printable identifier (it's the same
    // string declared in the manifest's `events` array). Stuffing the
    // 32-byte keccak hash in here — as an earlier revision of this code
    // did — breaks on real Neo nodes whenever the keccak's first byte
    // isn't valid UTF-8. The classic case is
    // `Transfer(address,address,uint256)` whose keccak begins with 0xDD;
    // the testnet runtime faults with "Unable to translate bytes [DD]
    // from specified code page to Unicode."
    //
    // We push the human-readable event name here and carry topic[0] (the
    // signature keccak) as the FIRST element of the state array below.
    // Anonymous events ALSO push their declared name: Neo nodes >= 3.6
    // reject notifications whose event name is not declared in the
    // manifest, so the previous empty-string sentinel faulted on-chain.
    // Anonymous-ness only suppresses the EVM topic0 slot (Step 1b).
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        event_name.as_bytes().to_vec(),
    )));

    // Step 1b: for non-anonymous events, push topic[0] = keccak(signature)
    // as the first element of the state array. The runtime peels this back
    // off when reconstructing the EVM-canonical LogEntry. Its 32-byte length
    // is the structural marker the runtime uses to distinguish EVM-shape
    // from legacy.
    if !signature.is_anonymous {
        instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
            signature.topic0.to_vec(),
        )));
    }

    // Step 2: for each indexed arg, lower the expression and then coerce to
    // a 32-byte topic slot.
    //   - static types → direct 32-byte BE-padded slot (via
    //     `lower_static_abi_slots_for_expr`, which produces real EVM-canonical
    //     bytes instead of the `StdLib.serialize` / JSON-shaped fallback).
    //   - dynamic types → Keccak256(value).
    //
    // Falling back to `CallBuiltin::AbiEncode(1)` is ONLY safe when the
    // expression isn't a static value type — that path goes through
    // `StdLib.serialize([arg])` and produces a Neo-native / JSON-shaped
    // blob rather than a 32-byte EVM topic slot. See the fuzz harness
    // `event_with_indexed_and_dynamic_args_lowers` which caught this when
    // `msg.sender` (an indexed static address) surfaced as ~130 bytes.
    let mut success = true;
    for (arg, info) in indexed.iter().zip(indexed_params.iter()) {
        if info.is_dynamic {
            let is_bytes_like = info.canonical_type == "string" || info.canonical_type == "bytes";
            if is_bytes_like {
                // `string`/`bytes`: the runtime value IS the byte payload, so
                // keccak256(value) == keccak256(abi.encodePacked(value)) — the
                // Solidity indexed-topic rule (ethers hashes the raw bytes).
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                    break;
                }
            } else {
                // Dynamic array / dynamic struct: the indexed topic is
                // keccak256(abi.encode(value)) (ethers:
                // keccak256(abiCoder.encode([type], [value]))). Hashing the raw
                // Array stack item instead would hash a non-conformant blob and
                // FAULT on a real node (CryptoLib.keccak256 requires a
                // ByteString). Route through the conformant ABI encoder.
                match lower_abi_encode_args_direct(&[*arg], ctx, instructions) {
                    Some(true) => {}
                    _ => {
                        success = false;
                        break;
                    }
                }
            }
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::Keccak256,
                arg_count: 1,
            });
            continue;
        }
        // A bare hex literal / named constant declared as an indexed `bytesN`
        // topic is INTEGER-backed; the generic static-slot encoder infers it as
        // a number and emits a RIGHT-aligned (uint-style) slot, corrupting the
        // topic. `bytesN` topics are LEFT-aligned, so resolve the literal to its
        // big-endian bytes and emit a left-aligned 32-byte slot directly
        // (mirrors the multi-return + abi.encode bytesN fixes).
        if let Some(n) = info
            .canonical_type
            .strip_prefix("bytes")
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|n| (1..=32).contains(n))
        {
            if is_integer_backed_bytesn_operand(arg, ctx) {
                if let Some(be) = fixed_len_bytes_be_from_hex_or_const(arg, n, ctx) {
                    let mut slot = be;
                    slot.resize(32, 0);
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(slot)));
                    continue;
                }
            }
        }
        let pre = instructions.len();
        match lower_static_abi_slots_for_expr(arg, ctx, instructions, false) {
            Some(0) => {
                success = false;
                break;
            }
            Some(1) => {
                // 32-byte BE-padded slot already on the stack.
            }
            Some(n) => {
                // Struct-as-indexed: concat the per-field slots and hash,
                // matching Solidity's `keccak256(abi.encode(struct))` rule
                // for indexed reference-type args.
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::BytesConcat,
                    arg_count: n,
                });
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::Keccak256,
                    arg_count: 1,
                });
            }
            None => {
                instructions.truncate(pre);
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                    break;
                }
                instructions.push(Instruction::CallBuiltin {
                    builtin: BuiltinCall::AbiEncode,
                    arg_count: 1,
                });
            }
        }
    }

    // Step 3: push `data = abi.encode(non_indexed_args...)`. Must come
    // AFTER the indexed topics so PACK captures them in the right order.
    //
    // Task #109 — `bytesN(..)` / `address(..)` cast args route through
    // `coerce_to_fixed_bytes`, which leaves the MEMCPY-returned destination
    // buffer on the stack BENEATH the canonical ByteString result (MEMCPY
    // pushes `dst` back per C-style memcpy semantics — see
    // src/runtime/execution/execution_impl_part3_bytes.rs::memcpy_bytes).
    // When such a cast appears among non-indexed args, the subsequent PACK
    // inside `AbiEncode` picks up the leaked buffer instead of an earlier
    // scalar arg, corrupting `data` (e.g. `emit E(uint256 x, bytes32 y)`
    // yields `bytes32(y) || bytes32(y)` instead of `bytes32(x) || bytes32(y)`,
    // and `x` then leaks into the stateArray as a separate topic).
    // Mirror the Task #66/#89 packed-encoding fix in builtins.rs: follow
    // each leaky arg with `Swap; Drop` to discard the leaked buffer and
    // keep only the canonical result.
    if success {
        if non_indexed.is_empty() {
            instructions.push(Instruction::PushLiteral(
                LiteralValue::ByteArray(Vec::new()),
            ));
        } else {
            let pre_data = instructions.len();
            let refs: Vec<&Expression> = non_indexed.clone();
            match lower_abi_encode_args_direct(&refs, ctx, instructions) {
                Some(true) => {
                    // EVM-canonical data bytes already on the stack.
                }
                Some(false) => {
                    success = false;
                }
                None => {
                    instructions.truncate(pre_data);
                    for arg in &non_indexed {
                        if !lower_expression(arg, ctx, instructions) {
                            success = false;
                            break;
                        }
                    }
                    if success {
                        instructions.push(Instruction::CallBuiltin {
                            builtin: BuiltinCall::AbiEncode,
                            arg_count: non_indexed.len(),
                        });
                    }
                }
            }
        }
    }

    if !success {
        // A partially-lowered notification would either corrupt the state
        // array or mismatch the manifest-declared shape (which faults on
        // Neo nodes >= 3.6) — fail the compile instead.
        instructions.truncate(original_len);
        ctx.record_error(format!(
            "could not lower the arguments of `emit {event_name}(...)`; refusing to emit a \
             partial notification"
        ));
        return;
    }

    // Step 4: emit the `EmitEvent` / `EmitEventByName` IR node. The
    // bytecode emitter PACKs the state-array elements and calls
    // Runtime.Notify; the runtime peels them apart again to build a
    // proper EVM-canonical LogEntry.
    //
    // For non-anonymous events the layout is:
    //   stateArray = [topic0, topic1, ..., topicN, data]
    // For anonymous events the layout is:
    //   stateArray = [topic1, ..., topicN, data]
    // (topic0 is suppressed per EVM ABI for anonymous events.)
    //
    // The manifest's events section declares exactly this shape (see
    // cli_manifest/build.rs) so the post-Basilisk count validation passes.
    let topic0_slot = if signature.is_anonymous { 0 } else { 1 };
    let state_item_count = topic0_slot + indexed.len() + 1; // +1 for the data slot.
    if let Some(index) = ctx.event_index_map.get(event_name) {
        instructions.push(Instruction::EmitEvent {
            event_index: *index,
            arg_count: state_item_count,
        });
    } else {
        instructions.push(Instruction::EmitEventByName {
            name: event_name.to_string(),
            arg_count: state_item_count,
        });
    }
}
