use super::*;
use crate::opcode::OpCode;

pub(crate) fn emit_event(
    bytecode: &mut Vec<u8>,
    module: &ir::Module,
    index: usize,
    arg_count: usize,
) {
    if module.events.get(index).is_some() {
        emit_event_payload(bytecode, arg_count);
    }
}

pub(crate) fn emit_event_by_name(bytecode: &mut Vec<u8>, _name: &str, arg_count: usize) {
    emit_event_payload(bytecode, arg_count);
}

fn emit_event_payload(bytecode: &mut Vec<u8>, arg_count: usize) {
    // Neo N3 Runtime.Notify signature is: Notify(eventName, stateArray)
    // The lowering pushes eventName first, then the event arguments.
    let total_bigint = BigInt::from(arg_count);
    push_integer_bigint(bytecode, &total_bigint);
    bytecode.push(OpCode::PACK.byte()); // PACK arguments into an array, leaving eventName below it
                                        // PACK reverses argument order (it pops from the stack), but Neo ABI expects event
                                        // arguments to match the declared order in the manifest. Reverse the packed array
                                        // in-place to restore the original ordering.
    if arg_count > 1 {
        bytecode.push(OpCode::DUP.byte()); // DUP (keep a reference to the array on the stack)
        bytecode.push(OpCode::REVERSEITEMS.byte()); // REVERSEITEMS (consumes one reference, reverses in-place)
    }
    bytecode.push(OpCode::SWAP.byte()); // SWAP -> [stateArray, eventName]
    emit_syscall(bytecode, "System.Runtime.Notify");
}
