use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_slots_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        // Pin OpCode byte values once at the top so the range patterns below
        // (which must be `u8` ranges, not `bool`) compile cleanly.
        const INITSLOT: u8 = OpCode::INITSLOT.byte();
        const INITSSLOT: u8 = OpCode::INITSSLOT.byte();
        const LDSFLD0: u8 = OpCode::LDSFLD0.byte();
        const LDSFLD6_END: u8 = OpCode::LDSFLD6.byte();
        const LDSFLD: u8 = OpCode::LDSFLD.byte();
        const STSFLD0: u8 = OpCode::STSFLD0.byte();
        const STSFLD6: u8 = OpCode::STSFLD6.byte();
        const STSFLD: u8 = OpCode::STSFLD.byte();
        const LDLOC0: u8 = OpCode::LDLOC0.byte();
        const LDLOC6: u8 = OpCode::LDLOC6.byte();
        const LDLOC: u8 = OpCode::LDLOC.byte();
        const STLOC0: u8 = OpCode::STLOC0.byte();
        const STLOC6: u8 = OpCode::STLOC6.byte();
        const STLOC: u8 = OpCode::STLOC.byte();
        const LDARG0: u8 = OpCode::LDARG0.byte();
        const LDARG6: u8 = OpCode::LDARG6.byte();
        const LDARG: u8 = OpCode::LDARG.byte();
        const STARG0: u8 = OpCode::STARG0.byte();
        const STARG6: u8 = OpCode::STARG6.byte();
        const STARG: u8 = OpCode::STARG.byte();

        match opcode {
            INITSLOT => {
                // INITSLOT locals, args
                if self.instruction_pointer + 2 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "INITSLOT: insufficient bytecode".to_string(),
                    });
                }
                let local_count = self.bytecode[self.instruction_pointer as usize + 1] as usize;
                let arg_count = self.bytecode[self.instruction_pointer as usize + 2] as usize;
                self.locals = vec![StackItem::Null; local_count];
                // Pop arg_count items from the evaluation stack into arg slots.
                // NeoVM pops in order so that arg0 = first popped (top of stack).
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    if self.stack.is_empty() {
                        args.push(StackItem::Null);
                    } else {
                        args.push(self.pop_stack()?);
                    }
                }
                self.args = args;
                self.instruction_pointer += 3;
            }
            INITSSLOT => {
                // INITSSLOT static slots
                if self.instruction_pointer + 1 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "INITSSLOT: insufficient bytecode".to_string(),
                    });
                }
                let static_count = self.bytecode[self.instruction_pointer as usize + 1] as usize;
                self.static_fields = vec![StackItem::Null; static_count];
                self.instruction_pointer += 2;
            }
            LDSFLD0..=LDSFLD6_END => {
                // LDSFLD0-6
                let index = (opcode - LDSFLD0) as usize;
                let value = self.get_static(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 1;
            }
            LDSFLD => {
                // LDSFLD
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "LDSFLD: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.get_static(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 2;
            }
            STSFLD0..=STSFLD6 => {
                // STSFLD0-6
                let index = (opcode - STSFLD0) as usize;
                let value = self.pop_stack()?;
                self.set_static(index, value)?;
                self.instruction_pointer += 1;
            }
            STSFLD => {
                // STSFLD
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "STSFLD: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.pop_stack()?;
                self.set_static(index, value)?;
                self.instruction_pointer += 2;
            }
            LDLOC0..=LDLOC6 => {
                // LDLOC0-6
                let index = (opcode - LDLOC0) as usize;
                let value = self.get_local(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 1;
            }
            LDLOC => {
                // LDLOC
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "LDLOC: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.get_local(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 2;
            }
            STLOC0..=STLOC6 => {
                // STLOC0-6
                let index = (opcode - STLOC0) as usize;
                let value = self.pop_stack()?;
                self.set_local(index, value)?;
                self.instruction_pointer += 1;
            }
            STLOC => {
                // STLOC
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "STLOC: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.pop_stack()?;
                self.set_local(index, value)?;
                self.instruction_pointer += 2;
            }
            LDARG0..=LDARG6 => {
                // LDARG0-6
                let index = (opcode - LDARG0) as usize;
                let value = self.get_arg(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 1;
            }
            LDARG => {
                // LDARG
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "LDARG: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.get_arg(index)?;
                self.push_stack(value)?;
                self.instruction_pointer += 2;
            }
            STARG0..=STARG6 => {
                // STARG0-6
                let index = (opcode - STARG0) as usize;
                let value = self.pop_stack()?;
                self.set_arg(index, value)?;
                self.instruction_pointer += 1;
            }
            STARG => {
                // STARG
                let idx_pos = self.instruction_pointer as usize + 1;
                if idx_pos >= self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "STARG: missing index".to_string(),
                    });
                }
                let index = self.bytecode[idx_pos] as usize;
                let value = self.pop_stack()?;
                self.set_arg(index, value)?;
                self.instruction_pointer += 2;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
