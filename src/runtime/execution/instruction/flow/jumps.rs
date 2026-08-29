use super::*;
use crate::opcode::OpCode;

// ============================================================
// Jump Entry Point — dispatches to sub-handlers
// ============================================================

impl ExecutionContext {
    pub(crate) fn execute_flow_jumps(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        if self.execute_flow_jumps_unconditional(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_jumps_conditional(opcode)? {
            return Ok(true);
        }
        if self.execute_flow_jumps_compare(opcode)? {
            return Ok(true);
        }
        Ok(false)
    }
}

// ============================================================
// Unconditional Jumps (JMP, JMP_L, NOP)
// ============================================================

impl ExecutionContext {
    pub(crate) fn execute_flow_jumps_unconditional(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        const NOP: u8 = OpCode::NOP.byte();
        const JMP: u8 = OpCode::JMP.byte();
        const JMP_L: u8 = OpCode::JMP_L.byte();

        match opcode {
            NOP => {
                self.instruction_pointer += 1;
                Ok(true)
            }
            JMP => {
                let offset = self.read_i8_offset("JMP")? as i32;
                let target = self.compute_offset_target("JMP", self.instruction_pointer, offset)?;
                self.instruction_pointer = target;
                Ok(true)
            }
            JMP_L => {
                let offset = self.read_i32_offset("JMP_L")?;
                let target =
                    self.compute_offset_target("JMP_L", self.instruction_pointer, offset)?;
                self.instruction_pointer = target;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// ============================================================
// Conditional Jumps (JMPIF, JMPIFNOT and their long variants)
// ============================================================

impl ExecutionContext {
    pub(crate) fn execute_flow_jumps_conditional(
        &mut self,
        opcode: u8,
    ) -> Result<bool, RuntimeError> {
        const JMPIF: u8 = OpCode::JMPIF.byte();
        const JMPIF_L: u8 = OpCode::JMPIF_L.byte();
        const JMPIFNOT: u8 = OpCode::JMPIFNOT.byte();
        const JMPIFNOT_L: u8 = OpCode::JMPIFNOT_L.byte();

        match opcode {
            JMPIF => {
                let condition = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPIF")? as i32;
                if condition.is_truthy() {
                    let target =
                        self.compute_offset_target("JMPIF", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPIF_L => {
                let condition = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPIF_L")?;
                if condition.is_truthy() {
                    let target =
                        self.compute_offset_target("JMPIF_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPIFNOT => {
                let condition = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPIFNOT")? as i32;
                if !condition.is_truthy() {
                    let target =
                        self.compute_offset_target("JMPIFNOT", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPIFNOT_L => {
                let condition = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPIFNOT_L")?;
                if !condition.is_truthy() {
                    let target =
                        self.compute_offset_target("JMPIFNOT_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// ============================================================
// Compare-and-Jump (JMPEQ, JMPNE, JMPGT, JMPGE, JMPLT, JMPLE)
// ============================================================

impl ExecutionContext {
    pub(crate) fn execute_flow_jumps_compare(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const JMPEQ: u8 = OpCode::JMPEQ.byte();
        const JMPEQ_L: u8 = OpCode::JMPEQ_L.byte();
        const JMPNE: u8 = OpCode::JMPNE.byte();
        const JMPNE_L: u8 = OpCode::JMPNE_L.byte();
        const JMPGT: u8 = OpCode::JMPGT.byte();
        const JMPGT_L: u8 = OpCode::JMPGT_L.byte();
        const JMPGE: u8 = OpCode::JMPGE.byte();
        const JMPGE_L: u8 = OpCode::JMPGE_L.byte();
        const JMPLT: u8 = OpCode::JMPLT.byte();
        const JMPLT_L: u8 = OpCode::JMPLT_L.byte();
        const JMPLE: u8 = OpCode::JMPLE.byte();
        const JMPLE_L: u8 = OpCode::JMPLE_L.byte();

        match opcode {
            JMPEQ => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPEQ")? as i32;
                if self.stack_items_equal(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPEQ", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPEQ_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPEQ_L")?;
                if self.stack_items_equal(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPEQ_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPNE => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPNE")? as i32;
                if !self.stack_items_equal(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPNE", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPNE_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPNE_L")?;
                if !self.stack_items_equal(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPNE_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPGT => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPGT")? as i32;
                if self.greater_than(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPGT", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPGT_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPGT_L")?;
                if self.greater_than(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPGT_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPGE => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPGE")? as i32;
                let ge = self.greater_than(&a, &b)? || self.stack_items_equal(&a, &b)?;
                if ge {
                    let target =
                        self.compute_offset_target("JMPGE", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPGE_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPGE_L")?;
                let ge = self.greater_than(&a, &b)? || self.stack_items_equal(&a, &b)?;
                if ge {
                    let target =
                        self.compute_offset_target("JMPGE_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPLT => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPLT")? as i32;
                if self.less_than(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPLT", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPLT_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPLT_L")?;
                if self.less_than(&a, &b)? {
                    let target =
                        self.compute_offset_target("JMPLT_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            JMPLE => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i8_offset("JMPLE")? as i32;
                let le = self.less_than(&a, &b)? || self.stack_items_equal(&a, &b)?;
                if le {
                    let target =
                        self.compute_offset_target("JMPLE", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 2;
                }
                Ok(true)
            }
            JMPLE_L => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let offset = self.read_i32_offset("JMPLE_L")?;
                let le = self.less_than(&a, &b)? || self.stack_items_equal(&a, &b)?;
                if le {
                    let target =
                        self.compute_offset_target("JMPLE_L", self.instruction_pointer, offset)?;
                    self.instruction_pointer = target;
                } else {
                    self.instruction_pointer += 5;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
