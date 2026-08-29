use super::*;

pub(crate) fn try_lower_storage_member_builtin(
    base: &Identifier,
    member: &Identifier,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if base.name != "Storage" {
        return None;
    }

    match member.name.as_str() {
        // NOTE: `getUsage` used to be special-cased here as a hardcoded
        // PUSH0 stub. Neo N3 has no `System.Storage.GetUsage` syscall and the
        // compiler does not track byte-level storage usage, so the stub
        // silently returned 0 for every contract. It was removed so calls
        // fail compilation loudly instead of miscompiling.
        "putContractMetadata" => {
            if args.len() != 4 {
                ctx.record_error(format!(
                    "Storage.putContractMetadata requires 4 argument(s), got {}",
                    args.len()
                ));
                return Some(false);
            }

            // putContractMetadata(name, version, author, extra) expands to:
            // Storage.put("__CONTRACT_METADATA__", abi.encode(name, version, author, extra, block.timestamp))
            //
            // We emit the key first so the encoded metadata lands above it on the stack.
            instructions.push(Instruction::PushLiteral(LiteralValue::String(
                b"__CONTRACT_METADATA__".to_vec(),
            )));

            let mut success = true;
            for arg in args {
                if !lower_expression(arg, ctx, instructions) {
                    success = false;
                }
            }

            if !success {
                return Some(false);
            }

            instructions.push(Instruction::LoadRuntimeValue(RuntimeValue::BlockTimestamp));
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::AbiEncode,
                arg_count: 5,
            });
            instructions.push(Instruction::CallBuiltin {
                builtin: BuiltinCall::StoragePut,
                arg_count: 2,
            });
            Some(true)
        }
        _ => None,
    }
}
