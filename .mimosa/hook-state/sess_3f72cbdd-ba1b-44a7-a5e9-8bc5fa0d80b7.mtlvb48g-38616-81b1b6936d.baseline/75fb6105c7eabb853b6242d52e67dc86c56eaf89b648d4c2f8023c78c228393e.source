use super::*;

impl ExecutionContext {
    pub(crate) fn read_i8_offset(&self, opcode_label: &str) -> Result<i8, RuntimeError> {
        let idx = self.instruction_pointer as usize + 1;
        if idx >= self.bytecode.len() {
            return Err(RuntimeError::ExecutionError {
                message: format!("{opcode_label}: insufficient bytecode for offset"),
            });
        }
        Ok(self.bytecode[idx] as i8)
    }

    pub(crate) fn read_i32_offset(&self, opcode_label: &str) -> Result<i32, RuntimeError> {
        let start = self.instruction_pointer as usize + 1;
        let end = start + 4;
        if end > self.bytecode.len() {
            return Err(RuntimeError::ExecutionError {
                message: format!("{opcode_label}: insufficient bytecode for offset"),
            });
        }

        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.bytecode[start..end]);
        Ok(i32::from_le_bytes(buf))
    }

    pub(crate) fn compute_offset_target(
        &self,
        opcode_label: &str,
        base: u32,
        offset: i32,
    ) -> Result<u32, RuntimeError> {
        let target = (base as i64).checked_add(offset as i64).ok_or_else(|| {
            RuntimeError::ExecutionError {
                message: format!("{opcode_label}: jump target out of bounds"),
            }
        })?;

        if target < 0 || target as usize >= self.bytecode.len() {
            return Err(RuntimeError::ExecutionError {
                message: format!("{opcode_label}: jump target out of bounds"),
            });
        }

        Ok(target as u32)
    }
}
