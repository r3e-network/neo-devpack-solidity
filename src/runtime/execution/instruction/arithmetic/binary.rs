use super::*;
use crate::opcode::OpCode;

/// Per-byte gas surcharge for computationally expensive arithmetic ops
/// (MUL, DIV, MOD, POW, MODMUL, MODPOW). On Neo N3, BigInt operations
/// scale with operand size — larger operands consume more gas.
/// Formula: `max(byte_len(a), byte_len(b)) * ARITH_PER_BYTE_GAS`.
const ARITH_PER_BYTE_GAS: u64 = 3;

impl ExecutionContext {
    /// Charge additional gas proportional to the larger of the two operand
    /// byte lengths. Used after the base opcode gas has already been charged
    /// in `execute_instruction`.
    fn charge_operand_size_gas(
        &mut self,
        a: &StackItem,
        b: &StackItem,
    ) -> Result<(), RuntimeError> {
        let byte_len = |item: &StackItem| -> u64 {
            match item {
                StackItem::ByteArray { data: bytes, .. } => bytes.borrow().len() as u64,
                StackItem::Integer(_) | StackItem::UnsignedInteger(_) => 8,
                StackItem::Boolean(_) => 1,
                StackItem::Null => 0,
                _ => 8, // arrays, maps — conservative estimate
            }
        };
        let len_a = byte_len(a);
        let len_b = byte_len(b);
        let extra = len_a.max(len_b).saturating_mul(ARITH_PER_BYTE_GAS);
        if extra == 0 {
            return Ok(());
        }
        let projected = self.gas_used.saturating_add(extra);
        if projected > self.gas_limit {
            return Err(RuntimeError::OutOfGas {
                used: projected,
                limit: self.gas_limit,
            });
        }
        self.gas_used = projected;
        Ok(())
    }

    pub(crate) fn execute_arithmetic_binary(&mut self, opcode: u8) -> Result<bool, RuntimeError> {
        const ADD: u8 = OpCode::ADD.byte();
        const SUB: u8 = OpCode::SUB.byte();
        const MUL: u8 = OpCode::MUL.byte();
        const DIV: u8 = OpCode::DIV.byte();
        const MOD: u8 = OpCode::MOD.byte();
        const POW: u8 = OpCode::POW.byte();
        const MODMUL: u8 = OpCode::MODMUL.byte();
        const MODPOW: u8 = OpCode::MODPOW.byte();

        match opcode {
            ADD => {
                // ADD
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.add_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            SUB => {
                // SUB
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = self.sub_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MUL => {
                // MUL
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.charge_operand_size_gas(&a, &b)?;
                let result = self.mul_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            DIV => {
                // DIV
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.charge_operand_size_gas(&a, &b)?;
                let result = self.div_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MOD => {
                // MOD
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.charge_operand_size_gas(&a, &b)?;
                let result = self.mod_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            POW => {
                // POW
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.charge_operand_size_gas(&a, &b)?;
                let result = self.pow_stack_items(a, b)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MODMUL => {
                // MODMUL
                let modulus = self.pop_stack()?;
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.charge_operand_size_gas(&a, &b)?;
                self.charge_operand_size_gas(&b, &modulus)?;
                let result = self.modmul_stack_items(a, b, modulus)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            MODPOW => {
                // MODPOW
                let modulus = self.pop_stack()?;
                let exponent = self.pop_stack()?;
                let base = self.pop_stack()?;
                let result = self.modpow_stack_items(base, exponent, modulus)?;
                self.push_stack(result)?;
                self.instruction_pointer += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
