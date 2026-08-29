use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_stack_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const DEPTH: u8 = OpCode::DEPTH.byte();
        const DROP: u8 = OpCode::DROP.byte();
        const NIP: u8 = OpCode::NIP.byte();
        const XDROP: u8 = OpCode::XDROP.byte();
        const CLEAR: u8 = OpCode::CLEAR.byte();
        const DUP: u8 = OpCode::DUP.byte();
        const OVER: u8 = OpCode::OVER.byte();
        const PICK: u8 = OpCode::PICK.byte();
        const TUCK: u8 = OpCode::TUCK.byte();
        const SWAP: u8 = OpCode::SWAP.byte();
        const ROT: u8 = OpCode::ROT.byte();
        const ROLL: u8 = OpCode::ROLL.byte();
        const REVERSE3: u8 = OpCode::REVERSE3.byte();
        const REVERSE4: u8 = OpCode::REVERSE4.byte();
        const REVERSEN: u8 = OpCode::REVERSEN.byte();

        match opcode {
            // Stack operations (0x39-0x4F)
            DEPTH => {
                // DEPTH
                self.push_stack(StackItem::Integer(self.stack.len() as i64))?;
                self.instruction_pointer += 1;
            }
            DROP => {
                // DROP
                self.pop_stack()?;
                self.instruction_pointer += 1;
            }
            NIP => {
                // NIP
                self.nip()?;
                self.instruction_pointer += 1;
            }
            XDROP => {
                // XDROP
                self.xdrop()?;
                self.instruction_pointer += 1;
            }
            CLEAR => {
                // CLEAR
                self.clear_stack();
                self.instruction_pointer += 1;
            }
            DUP => {
                // DUP
                let top = self.peek_stack()?.clone();
                self.push_stack(top)?;
                self.instruction_pointer += 1;
            }
            OVER => {
                // OVER
                self.over()?;
                self.instruction_pointer += 1;
            }
            PICK => {
                // PICK (index on top)
                self.pick_n()?;
                self.instruction_pointer += 1;
            }
            TUCK => {
                // TUCK
                self.tuck()?;
                self.instruction_pointer += 1;
            }
            SWAP => {
                // SWAP
                let top = self.pop_stack()?;
                let second = self.pop_stack()?;
                self.push_stack(top)?;
                self.push_stack(second)?;
                self.instruction_pointer += 1;
            }
            ROT => {
                // ROT
                if self.stack.len() < 3 {
                    return Err(RuntimeError::ExecutionError {
                        message: "ROT: insufficient stack items".to_string(),
                    });
                }
                let c = self.pop_stack()?;
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.push_stack(b)?;
                self.push_stack(c)?;
                self.push_stack(a)?;
                self.instruction_pointer += 1;
            }
            ROLL => {
                // ROLL (index on top)
                self.roll()?;
                self.instruction_pointer += 1;
            }
            REVERSE3 => {
                // REVERSE3
                self.reverse_top_n(3)?;
                self.instruction_pointer += 1;
            }
            REVERSE4 => {
                // REVERSE4
                self.reverse_top_n(4)?;
                self.instruction_pointer += 1;
            }
            REVERSEN => {
                // REVERSEN (count on top)
                let count = self.pop_usize("REVERSEN")?;
                self.reverse_top_n(count)?;
                self.instruction_pointer += 1;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
