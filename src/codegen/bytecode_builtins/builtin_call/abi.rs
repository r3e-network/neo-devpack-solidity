use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_abi_encode(
    bytecode: &mut Vec<u8>,
    arg_count: usize,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    if arg_count == 0 {
        // `abi.encode()` is the empty byte string. Lower it directly rather
        // than routing through the pseudo-native StdLib helper, which does
        // not exist on real Neo N3 / Neo-Express.
        push_data(bytecode, &[]);
        return;
    }

    let total_bigint = BigInt::from(arg_count);
    push_integer_bigint(bytecode, &total_bigint);
    bytecode.push(OpCode::PACK.byte());
    // PACK pops values from the stack and therefore reverses their order.
    // For `abi.encode(a, b, c)` we must preserve the original argument order.
    if arg_count > 1 {
        bytecode.push(OpCode::DUP.byte()); // DUP (keep a reference to the array on the stack)
        bytecode.push(OpCode::REVERSEITEMS.byte()); // REVERSEITEMS (consumes one reference, reverses in-place)
    }
    // Production Neo N3 has no StdLib.abiEncode helper. The high-level IR
    // lowers many static ABI shapes directly before bytecode emission; this
    // fallback keeps remaining shapes executable on chain by using real Neo
    // StdLib serialization instead of a pseudo-native test-runtime method.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "serialize",
        1,
        use_callt,
        token_patches,
    );
}

pub(crate) fn emit_abi_encode_packed(
    bytecode: &mut Vec<u8>,
    arg_count: usize,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    if arg_count == 0 {
        // `abi.encodePacked()` is also the empty byte string.
        push_data(bytecode, &[]);
        return;
    }

    let total_bigint = BigInt::from(arg_count);
    push_integer_bigint(bytecode, &total_bigint);
    bytecode.push(OpCode::PACK.byte());
    if arg_count > 1 {
        bytecode.push(OpCode::DUP.byte());
        bytecode.push(OpCode::REVERSEITEMS.byte());
    }
    // Production Neo N3 has no StdLib.abiEncodePacked helper. Static/narrow
    // packed forms are lowered before this point; use real StdLib.serialize
    // for the remaining fallback instead of emitting a nonexistent method.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "serialize",
        1,
        use_callt,
        token_patches,
    );
}

pub(crate) fn emit_abi_decode(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Production Neo N3 has no StdLib.abiDecode helper. Decode the fallback
    // Neo-serialized ABI payloads with real StdLib.deserialize; static
    // canonical ABI decode fast paths are handled in IR where target types are
    // still available.
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "deserialize",
        1,
        use_callt,
        token_patches,
    );
}

pub(crate) fn emit_notify_serialized(
    bytecode: &mut Vec<u8>,
    use_callt: bool,
    token_patches: &mut Vec<MethodTokenPatch>,
) {
    // Syscalls.notify expects a serialized payload: [eventName, stateArray]
    emit_native_contract_call(
        bytecode,
        ir::NativeContract::StdLib,
        "deserialize",
        1,
        use_callt,
        token_patches,
    );
    // Extract eventName/state via PICKITEM and call Runtime.Notify.
    bytecode.push(OpCode::DUP.byte()); // DUP (payload)
    bytecode.push(OpCode::PUSH0.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> eventName
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [eventName, payload]
    bytecode.push(OpCode::DUP.byte()); // DUP (payload)
    bytecode.push(OpCode::PUSH1.byte());
    bytecode.push(OpCode::PICKITEM.byte()); // PICKITEM -> state
    bytecode.push(OpCode::NIP.byte()); // NIP (drop payload) -> [eventName, state]
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [state, eventName]
    emit_syscall(bytecode, "System.Runtime.Notify");
    bytecode.push(OpCode::PUSH1.byte());
}
