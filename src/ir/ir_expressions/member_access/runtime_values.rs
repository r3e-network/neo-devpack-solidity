use super::*;

pub(crate) fn try_lower_runtime_member_access(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    let is_nep17_payment = ctx.function_name == "onNEP17Payment";
    let is_nep11_payment = ctx.function_name == "onNEP11Payment";
    let is_payment_callback = is_nep17_payment || is_nep11_payment;

    match member.name.as_str() {
        "sender" => {
            if let Expression::Variable(base) = inner {
                if base.name == "msg" {
                    if is_payment_callback {
                        instructions.push(Instruction::LoadParameter(0));
                    } else {
                        instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::MsgSender));
                    }
                    return Some(true);
                }
            }
            None
        }
        "value" => {
            if let Expression::Variable(base) = inner {
                if base.name == "msg" {
                    if is_payment_callback {
                        // Both onNEP17Payment and onNEP11Payment have amount at param 1
                        instructions.push(Instruction::LoadParameter(1));
                    } else {
                        // Neo N3 has no "attached value" for calls; msg.value is only
                        // meaningful inside payment callbacks. Outside that context we
                        // emit a warning (via the payable modifier check) and return 0
                        // via RuntimeValue::MsgValue for source compatibility.
                        instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::MsgValue));
                    }
                    return Some(true);
                }
            }
            None
        }
        "data" => {
            if let Expression::Variable(base) = inner {
                if base.name == "msg" {
                    if is_nep17_payment {
                        // onNEP17Payment(from, amount, data) — data is param 2
                        instructions.push(Instruction::LoadParameter(2));
                    } else if is_nep11_payment {
                        // onNEP11Payment(from, amount, tokenId, data) — data is param 3
                        instructions.push(Instruction::LoadParameter(3));
                    } else {
                        // msg.data lowers to the runtime's `input_data` (the raw calldata
                        // bytes passed to `execute`), exposed via the `Script` field of
                        // `System.Runtime.GetScriptContainer`. The bytecode emitter for
                        // `RuntimeValue::MsgData` fetches that field. See
                        // `src/cli/bytecode/bytecode_helpers/array_runtime.rs`.
                        //
                        // For entry-point invocations this is the exact calldata: fallback()
                        // observes `msg.data.length == injected_calldata.len()`, and
                        // external functions observe `selector || abi.encode(args)` as the
                        // runtime dispatches them from input_data.
                        //
                        // Across internal contract-to-contract calls Neo N3's script
                        // container still reflects the *transaction* script, so observers
                        // can see a mismatch vs. EVM (which repopulates calldata on each
                        // internal call). The surviving informational warning captures
                        // that residual difference.
                        ctx.record_warning_with_suggestion(
                            "msg.data is approximated on Neo N3 as `selector || abi.encode(current args)` for cross-contract calls where the Neo script container still reflects the entry-point transaction script rather than the per-call payload. Entry-point fallback/receive observe the exact injected calldata.",
                            "Pass the bytes payload explicitly (e.g. `function f(bytes calldata data)`) when cross-contract call input must be recovered bit-for-bit.",
                        );
                        instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::MsgData));
                    }
                    return Some(true);
                }
            }
            None
        }
        "sig" => {
            if let Expression::Variable(base) = inner {
                if base.name == "msg" {
                    ctx.record_warning_with_suggestion(
                        "msg.sig is approximated on Neo N3 using the current function selector. This differs from EVM semantics across internal calls, where msg.sig preserves the original external-call selector.",
                        "Use explicit method-name logic or interface IDs when you need cross-call-stable dispatch identity.",
                    );
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        ctx.current_function_selector().to_vec(),
                    )));
                    return Some(true);
                }
            }
            None
        }
        "origin" => {
            if let Expression::Variable(base) = inner {
                if base.name == "tx" {
                    // Non-fatal warning: tx.origin compiles but has different semantics on Neo.
                    ctx.record_warning_with_suggestion(
                        "tx.origin has different semantics on Neo N3. Neo uses multi-signature witnesses instead of a single origin.",
                        "Use msg.sender or Runtime.checkWitness() for authorization instead.",
                    );
                    instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::TxOrigin));
                    return Some(true);
                }
            }
            None
        }
        "gasprice" => {
            if let Expression::Variable(base) = inner {
                if base.name == "tx" {
                    // Neo N3 auto-compat: tx.gasprice → Policy.getFeePerByte()
                    ctx.record_warning_with_suggestion(
                        "tx.gasprice auto-mapped to Policy.getFeePerByte() on Neo N3. Neo fees are determined by script size and syscall costs.",
                        "Use Policy.getFeePerByte() directly when targeting Neo.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract: NativeContract::Policy,
                            method: "getFeePerByte".to_string(),
                        },
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "hash" => {
            if let Expression::Variable(base) = inner {
                if base.name == "tx" {
                    // Neo N3 auto-compat: tx.hash → System.Runtime.GetScriptContainer
                    // Returns the transaction that triggered execution
                    ctx.record_warning_with_suggestion(
                        "tx.hash auto-mapped to System.Runtime.GetScriptContainer on Neo N3. This returns the current transaction as a ScriptContainer.",
                        "Use System.Runtime.GetScriptContainer directly if you need the current Neo transaction container.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::Syscall(
                            "System.Runtime.GetScriptContainer".to_string(),
                        ),
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "timestamp" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::BlockTimestamp));
                    return Some(true);
                }
            }
            None
        }
        "number" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::BlockNumber));
                    return Some(true);
                }
            }
            None
        }
        "chainid" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Solidity `block.chainid` is a uint256 chain identifier. Neo N3 exposes a
                    // network "magic" number via `System.Runtime.GetNetwork`; use that as the
                    // closest equivalent.
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::Syscall("System.Runtime.GetNetwork".to_string()),
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "coinbase" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Neo N3 auto-compat: block.coinbase → address(0)
                    // dBFT has no PoW miner; there is no single "coinbase" address.
                    // Return address(0) to match EVM type semantics (address return).
                    ctx.record_warning_with_suggestion(
                        "block.coinbase auto-mapped to address(0) on Neo N3 because dBFT consensus has no block miner.",
                        "Use Neo.getNextBlockValidators() if you need the current validator set, or Runtime.checkWitness() for authorization.",
                    );
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(vec![
                        0u8;
                        20
                    ])));
                    return Some(true);
                }
            }
            None
        }
        "difficulty" | "prevrandao" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Neo N3 auto-compat: block.difficulty/prevrandao → Runtime.getRandom()
                    ctx.record_warning_with_suggestion(
                        format!(
                            "block.{} auto-mapped to Runtime.getRandom() on Neo N3 because dBFT consensus has no PoW difficulty.",
                            member.name
                        ),
                        "Review any randomness assumptions; Neo's Runtime.getRandom() is not equivalent to EVM difficulty/prevrandao.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::Syscall("System.Runtime.GetRandom".to_string()),
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "gaslimit" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Neo N3 auto-compat: block.gaslimit → Policy.getExecFeeFactor()
                    ctx.record_warning_with_suggestion(
                        "block.gaslimit auto-mapped to Policy.getExecFeeFactor() on Neo N3. Neo uses GAS token fees, not per-block gas limits.",
                        "Avoid relying on EVM block gas-limit semantics on Neo.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract: NativeContract::Policy,
                            method: "getExecFeeFactor".to_string(),
                        },
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "basefee" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Neo N3 auto-compat: block.basefee → Policy.getFeePerByte()
                    ctx.record_warning_with_suggestion(
                        "block.basefee auto-mapped to Policy.getFeePerByte() on Neo N3. Neo does not use EIP-1559 base fees.",
                        "Review any fee-market logic before deploying on Neo.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract: NativeContract::Policy,
                            method: "getFeePerByte".to_string(),
                        },
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "parenthash" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    // Neo N3 auto-compat: block.parenthash → Ledger.currentHash
                    // EVM's blockhash(block.number - 1) returns the parent block hash.
                    // On Neo, Ledger.currentHash returns the current block's hash.
                    ctx.record_warning_with_suggestion(
                        "block.parenthash auto-mapped to Ledger.currentHash on Neo N3.",
                        "Use Ledger.getBlock(currentIndex - 1).hash if you need the actual parent block hash.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract: NativeContract::Ledger,
                            method: "currentHash".to_string(),
                        },
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        "sha3" => {
            if let Expression::Variable(base) = inner {
                if base.name == "block" {
                    ctx.record_warning_with_suggestion(
                        "block.sha3 is deprecated in Solidity 0.8+ and not fully available on Neo N3. On EVM it returns keccak256 of the current block. On Neo, this is approximated as Ledger.currentHash (the current block's hash).",
                        "Use Ledger.currentHash() directly if you need the current block hash on Neo.",
                    );
                    instructions.push(Instruction::CallBuiltin {
                        builtin: BuiltinCall::NativeCall {
                            contract: NativeContract::Ledger,
                            method: "currentHash".to_string(),
                        },
                        arg_count: 0,
                    });
                    return Some(true);
                }
            }
            None
        }
        _ => None,
    }
}
