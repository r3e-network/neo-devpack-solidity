use super::*;
use crate::opcode::OpCode;

impl ExecutionContext {
    pub(crate) fn execute_push_instruction(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const PUSHINT256: u8 = OpCode::PUSHINT256.byte();
        const PUSHINT128: u8 = OpCode::PUSHINT128.byte();
        const PUSHT: u8 = OpCode::PUSHT.byte();
        const PUSHF: u8 = OpCode::PUSHF.byte();
        const PUSHM1: u8 = OpCode::PUSHM1.byte();
        const PUSHINT8: u8 = OpCode::PUSHINT8.byte();
        const PUSHINT16: u8 = OpCode::PUSHINT16.byte();
        const PUSHINT32: u8 = OpCode::PUSHINT32.byte();
        const PUSHINT64: u8 = OpCode::PUSHINT64.byte();
        const PUSHA: u8 = OpCode::PUSHA.byte();
        const PUSHNULL: u8 = OpCode::PUSHNULL.byte();
        const PUSHDATA1: u8 = OpCode::PUSHDATA1.byte();
        const PUSHDATA2: u8 = OpCode::PUSHDATA2.byte();
        const PUSHDATA4: u8 = OpCode::PUSHDATA4.byte();
        const PUSH0: u8 = OpCode::PUSH0.byte();
        const PUSH1: u8 = OpCode::PUSH1.byte();
        const PUSH16: u8 = OpCode::PUSH16.byte();

        match opcode {
            // Push operations (0x00-0x4F)
            PUSHINT256 => {
                // PUSHINT256
                if self.instruction_pointer + 32 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT256: insufficient bytecode".to_string(),
                    });
                }
                let start = self.instruction_pointer as usize + 1;
                let end = start + 32;
                let value = self.bytecode[start..end].to_vec();
                self.push_stack(StackItem::byte_array(value))?;
                self.instruction_pointer += 33;
            }
            PUSHINT128 => {
                // PUSHINT128
                if self.instruction_pointer + 16 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT128: insufficient bytecode".to_string(),
                    });
                }
                let start = self.instruction_pointer as usize + 1;
                let end = start + 16;
                let value = self.bytecode[start..end].to_vec();
                self.push_stack(StackItem::byte_array(value))?;
                self.instruction_pointer += 17;
            }
            PUSHT => {
                // PUSHT
                self.push_stack(StackItem::Boolean(true))?;
                self.instruction_pointer += 1;
            }
            PUSHF => {
                // PUSHF
                self.push_stack(StackItem::Boolean(false))?;
                self.instruction_pointer += 1;
            }
            PUSHM1 => {
                // PUSHM1
                self.push_stack(StackItem::Integer(-1))?;
                self.instruction_pointer += 1;
            }
            PUSHINT8 => {
                // PUSHINT8
                if self.instruction_pointer + 1 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT8: insufficient bytecode".to_string(),
                    });
                }
                let value = self.bytecode[self.instruction_pointer as usize + 1] as i8 as i64;
                self.push_stack(StackItem::Integer(value))?;
                self.instruction_pointer += 2;
            }
            PUSHINT16 => {
                // PUSHINT16
                if self.instruction_pointer + 2 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT16: insufficient bytecode".to_string(),
                    });
                }
                let bytes = &self.bytecode
                    [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 3];
                let value = i16::from_le_bytes([bytes[0], bytes[1]]) as i64;
                self.push_stack(StackItem::Integer(value))?;
                self.instruction_pointer += 3;
            }
            PUSHINT32 => {
                // PUSHINT32
                if self.instruction_pointer + 4 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT32: insufficient bytecode".to_string(),
                    });
                }
                let bytes = &self.bytecode
                    [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 5];
                let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64;
                self.push_stack(StackItem::Integer(value))?;
                self.instruction_pointer += 5;
            }
            PUSHINT64 => {
                // PUSHINT64
                if self.instruction_pointer + 8 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHINT64: insufficient bytecode".to_string(),
                    });
                }
                let bytes = &self.bytecode
                    [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 9];
                let mut array = [0u8; 8];
                array.copy_from_slice(bytes);
                let value = i64::from_le_bytes(array);
                self.push_stack(StackItem::Integer(value))?;
                self.instruction_pointer += 9;
            }
            PUSHA => {
                // PUSHA — push a code Pointer. The operand is a SIGNED offset
                // relative to this opcode, so the absolute target position is
                // `instruction_pointer + operand` (mirroring real NeoVM's
                // `Pointer{ CurrentContext.InstructionPointer + operand }`).
                // Modeled here as the absolute position later consumed by CALLA.
                let rel = self.read_i32_offset("PUSHA")?;
                let target = (self.instruction_pointer as i64 + rel as i64).max(0) as u64;
                self.push_stack(StackItem::UnsignedInteger(target))?;
                self.instruction_pointer += 5;
            }
            PUSHNULL => {
                // PUSHNULL
                self.push_stack(StackItem::Null)?;
                self.instruction_pointer += 1;
            }
            PUSHDATA1 => {
                // PUSHDATA1
                if self.instruction_pointer + 1 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA1: insufficient bytecode for length".to_string(),
                    });
                }
                let length = self.bytecode[self.instruction_pointer as usize + 1] as usize;
                if self.instruction_pointer as usize + 2 + length > self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA1: insufficient bytecode for data".to_string(),
                    });
                }
                let data = self.bytecode[self.instruction_pointer as usize + 2
                    ..self.instruction_pointer as usize + 2 + length]
                    .to_vec();
                self.push_stack(StackItem::byte_array(data))?;
                self.instruction_pointer += 2 + length as u32;
            }
            PUSHDATA2 => {
                // PUSHDATA2
                if self.instruction_pointer + 2 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA2: insufficient bytecode for length".to_string(),
                    });
                }
                let len_bytes = &self.bytecode
                    [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 3];
                let length = u16::from_le_bytes([len_bytes[0], len_bytes[1]]) as usize;
                if self.instruction_pointer as usize + 3 + length > self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA2: insufficient bytecode for data".to_string(),
                    });
                }
                let data = self.bytecode[self.instruction_pointer as usize + 3
                    ..self.instruction_pointer as usize + 3 + length]
                    .to_vec();
                self.push_stack(StackItem::byte_array(data))?;
                self.instruction_pointer += 3 + length as u32;
            }
            PUSHDATA4 => {
                // PUSHDATA4
                if self.instruction_pointer + 4 >= self.bytecode.len() as u32 {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA4: insufficient bytecode for length".to_string(),
                    });
                }
                let len_bytes = &self.bytecode
                    [self.instruction_pointer as usize + 1..self.instruction_pointer as usize + 5];
                let length =
                    u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]])
                        as usize;
                if length > self.memory_limit {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "PUSHDATA4: data length {} exceeds memory limit {}",
                            length, self.memory_limit
                        ),
                    });
                }
                if self.instruction_pointer as usize + 5 + length > self.bytecode.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "PUSHDATA4: insufficient bytecode for data".to_string(),
                    });
                }
                let data = self.bytecode[self.instruction_pointer as usize + 5
                    ..self.instruction_pointer as usize + 5 + length]
                    .to_vec();
                self.push_stack(StackItem::byte_array(data))?;
                self.instruction_pointer += 5 + length as u32;
            }
            PUSH0 => {
                // PUSH0
                self.push_stack(StackItem::Integer(0))?;
                self.instruction_pointer += 1;
            }
            PUSH1..=PUSH16 => {
                // PUSH1-PUSH16
                let value = (opcode - PUSH0) as i64;
                self.push_stack(StackItem::Integer(value))?;
                self.instruction_pointer += 1;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
