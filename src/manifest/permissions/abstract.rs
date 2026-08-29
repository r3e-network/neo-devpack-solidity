use super::*;

pub(crate) fn pop_value(stack: &mut Vec<AbstractValue>) -> Result<AbstractValue, ()> {
    stack.pop().ok_or(())
}

pub(crate) fn pop_n(stack: &mut Vec<AbstractValue>, n: usize) -> Result<(), ()> {
    if stack.len() < n {
        return Err(());
    }
    for _ in 0..n {
        stack.pop();
    }
    Ok(())
}

pub(crate) fn apply_instruction(
    state: &mut AbstractState,
    instr: &ir::Instruction,
    ir_module: &ir::Module,
) -> Result<(), ()> {
    use ir::Instruction::{
        Abort, AbortMsg, ArrayGet, ArraySet, BinaryOp, BitwiseNot, CallBuiltin, CallFunction,
        CallIndirect, Convert, Drop, Dup, EmitEvent, EmitEventByName, EndTry, GetSize, HasKey,
        IsType, Jump, JumpIf, Label, LoadLocal, LoadMappingElement, LoadParameter,
        LoadRuntimeValue, LoadState, LoadStorageDynamic, LoadStructArrayElement, LoadStructField,
        LoadStructFieldMappingElement, LogicalNot, MemCpy, NewArray, NewBuffer, NewMap,
        PushFunctionOffset, PushLiteral, Return, ReturnDefault, ReturnVoid, ReverseItems,
        StoreArrayDeepCopy, StoreLocal, StoreMappingElement, StoreParameter, StoreState,
        StoreStructArrayElement, StoreStructField, StoreStructFieldMappingElement, Substr, Swap,
        Throw, Try,
    };

    match instr {
        Drop(_) => {
            pop_value(&mut state.stack)?;
        }
        LoadParameter(_) => state.stack.push(AbstractValue::Unknown),
        StoreParameter(_) => {
            // Task #156 — write-to-parameter mirrors StoreLocal for stack-shape
            // analysis: consumes the top-of-stack value and discards it from
            // the abstract tracker (we don't model parameter slot state).
            pop_value(&mut state.stack)?;
        }
        PushLiteral(lit) => state.stack.push(AbstractValue::literal(lit.clone())),
        Return | ReturnVoid | ReturnDefault(_) | Abort => {}
        AbortMsg | Throw => {
            pop_value(&mut state.stack)?;
        }
        BinaryOp(_) => {
            pop_n(&mut state.stack, 2)?;
            state.stack.push(AbstractValue::Unknown);
        }
        LoadState(_) => state.stack.push(AbstractValue::Unknown),
        StoreState(_) => {
            pop_value(&mut state.stack)?;
        }
        LoadStorageDynamic => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        LoadLocal(index) => {
            let value = state
                .locals
                .get(*index)
                .cloned()
                .unwrap_or(AbstractValue::Unknown);
            state.stack.push(value);
        }
        StoreLocal(index) => {
            let value = pop_value(&mut state.stack)?;
            if let Some(slot) = state.locals.get_mut(*index) {
                *slot = value;
            }
        }
        LoadMappingElement { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len())?;
            state.stack.push(AbstractValue::Unknown);
        }
        StoreMappingElement { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len() + 1)?;
        }
        StoreArrayDeepCopy { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len() + 1)?;
        }
        LoadStructField { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len())?;
            state.stack.push(AbstractValue::Unknown);
        }
        StoreStructField { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len() + 1)?;
        }
        LoadStructArrayElement { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len() + 1)?;
            state.stack.push(AbstractValue::Unknown);
        }
        StoreStructArrayElement { key_types, .. } => {
            pop_n(&mut state.stack, key_types.len() + 2)?;
        }
        LoadStructFieldMappingElement {
            key_types,
            trailing_key_types,
            ..
        } => {
            pop_n(&mut state.stack, key_types.len() + trailing_key_types.len())?;
            state.stack.push(AbstractValue::Unknown);
        }
        StoreStructFieldMappingElement {
            key_types,
            trailing_key_types,
            ..
        } => {
            pop_n(
                &mut state.stack,
                key_types.len() + trailing_key_types.len() + 1,
            )?;
        }
        LoadRuntimeValue(_) => state.stack.push(AbstractValue::Unknown),
        GetSize => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        CallFunction { name, arg_count } => {
            pop_n(&mut state.stack, *arg_count)?;
            let returns_value = ir_module
                .get_function(name)
                .map(|f| !f.returns.is_empty())
                .unwrap_or(true);
            if returns_value {
                state.stack.push(AbstractValue::Unknown);
            }
        }
        CallBuiltin { builtin, arg_count } => {
            pop_n(&mut state.stack, *arg_count)?;
            match builtin {
                ir::BuiltinCall::Syscall(name)
                    if name == "System.Runtime.GetExecutingScriptHash" =>
                {
                    state.stack.push(AbstractValue::executing_script_hash());
                }
                _ => state.stack.push(AbstractValue::Unknown),
            }
        }
        PushFunctionOffset { .. } => {
            // Task #186 — pushes the target function's bytecode offset as an
            // integer literal. Stack effect: +1 (one new opaque integer).
            state.stack.push(AbstractValue::Unknown);
        }
        CallIndirect {
            arg_count,
            has_return,
        } => {
            // Task #186 — indirect call through a function-pointer value.
            // Consumes `arg_count` arguments + 1 target offset; pushes a
            // return value iff the callee returns something.
            pop_n(&mut state.stack, *arg_count + 1)?;
            if *has_return {
                state.stack.push(AbstractValue::Unknown);
            }
        }
        EmitEvent { arg_count, .. } | EmitEventByName { arg_count, .. } => {
            pop_n(&mut state.stack, arg_count + 1)?;
        }
        Convert { .. } => {
            let value = pop_value(&mut state.stack)?;
            state.stack.push(value);
        }
        IsType { .. } => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        NewBuffer => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        NewArray { .. } => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        NewMap => {
            state.stack.push(AbstractValue::Unknown);
        }
        ArrayGet => {
            pop_n(&mut state.stack, 2)?;
            state.stack.push(AbstractValue::Unknown);
        }
        ArraySet => {
            pop_n(&mut state.stack, 3)?;
        }
        HasKey => {
            pop_n(&mut state.stack, 2)?;
            state.stack.push(AbstractValue::Unknown);
        }
        MemCpy => {
            // Real NeoVM: Pop 5, Push 0 (in-place buffer modification).
            pop_n(&mut state.stack, 5)?;
        }
        Substr => {
            // Substr pops [bytes, index, count] and pushes [bytes_substr].
            pop_n(&mut state.stack, 3)?;
            state.stack.push(AbstractValue::Unknown);
        }
        ReverseItems => {
            pop_value(&mut state.stack)?;
        }
        BitwiseNot | LogicalNot => {
            pop_value(&mut state.stack)?;
            state.stack.push(AbstractValue::Unknown);
        }
        Try { .. } | EndTry { .. } | Jump { .. } | Label(_) => {}
        JumpIf { .. } => {
            pop_value(&mut state.stack)?;
        }
        Dup => {
            let value = state
                .stack
                .last()
                .cloned()
                .unwrap_or(AbstractValue::Unknown);
            state.stack.push(value);
        }
        Swap => {
            let len = state.stack.len();
            if len >= 2 {
                state.stack.swap(len - 1, len - 2);
            }
        }
    }

    Ok(())
}
