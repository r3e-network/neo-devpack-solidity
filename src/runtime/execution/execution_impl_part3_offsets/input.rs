use super::*;

impl ExecutionContext {
    /// Return the length of the calldata buffer
    pub fn input_size(&self) -> usize {
        self.input_data.len()
    }

    /// Read a slice of calldata, zero-padding when the requested range exceeds the buffer.
    ///
    /// Returns an error if `length` exceeds `memory_limit`. Without this guard,
    /// any future opcode that wires `calldatacopy` / `copy_input_to_memory` through
    /// to user-controlled length would inherit the bug-#15 OOM-DoS pattern (the
    /// `vec![0u8; length]` would allocate before `write_memory` got a chance to
    /// reject the request). Mirrors the PUSHDATA4 length check.
    pub fn read_input_slice(&self, offset: usize, length: usize) -> Result<Vec<u8>, RuntimeError> {
        if length > self.memory_limit {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "calldata read length {} exceeds memory limit {}",
                    length, self.memory_limit
                ),
            });
        }
        let mut buffer = vec![0u8; length];
        if offset >= self.input_data.len() {
            return Ok(buffer);
        }

        let end = offset
            .checked_add(length)
            .ok_or(RuntimeError::ExecutionError {
                message: "calldata read range overflow".to_string(),
            })?;
        let available = end.min(self.input_data.len());
        let copy_len = available.saturating_sub(offset);
        buffer[..copy_len].copy_from_slice(&self.input_data[offset..offset + copy_len]);
        Ok(buffer)
    }

    /// Copy calldata into linear memory, expanding memory as required
    pub fn copy_input_to_memory(
        &mut self,
        memory_offset: usize,
        input_offset: usize,
        length: usize,
    ) -> Result<(), RuntimeError> {
        if length == 0 {
            return Ok(());
        }

        let data = self.read_input_slice(input_offset, length)?;
        self.write_memory(memory_offset, &data)
    }

    /// Size of the calldata buffer
    pub fn calldatasize(&self) -> usize {
        self.input_data.len()
    }

    /// Load a 32-byte word from calldata starting at `offset`
    pub fn calldataload_word(&self, offset: usize) -> [u8; 32] {
        // 32 is a compile-time constant well under any reasonable memory_limit;
        // unwrap is safe.
        let data = self
            .read_input_slice(offset, 32)
            .expect("calldataload_word: 32-byte read cannot exceed memory_limit");
        let mut word = [0u8; 32];
        word.copy_from_slice(&data);
        word
    }

    /// Copy calldata into linear memory
    pub fn calldatacopy(
        &mut self,
        memory_offset: usize,
        data_offset: usize,
        length: usize,
    ) -> Result<(), RuntimeError> {
        if length == 0 {
            return Ok(());
        }
        let data = self.read_input_slice(data_offset, length)?;
        self.write_memory(memory_offset, &data)
    }
}
