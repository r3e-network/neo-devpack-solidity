use super::*;

impl ExecutionContext {
    /// Read from memory, expanding zero-initialized regions as needed
    pub fn read_memory(&mut self, address: usize, length: usize) -> Result<&[u8], RuntimeError> {
        if length == 0 {
            if address > self.memory_limit {
                return Err(RuntimeError::ExecutionError {
                    message: "Memory access exceeds configured limit".to_string(),
                });
            }
            return Ok(&[]);
        }

        let required = address
            .checked_add(length)
            .ok_or(RuntimeError::ExecutionError {
                message: "Memory access overflow".to_string(),
            })?;
        if required > self.memory_limit {
            return Err(RuntimeError::ExecutionError {
                message: "Memory access exceeds configured limit".to_string(),
            });
        }

        if address > self.memory.len() {
            return Err(RuntimeError::ExecutionError {
                message: "Memory access exceeds allocated size".to_string(),
            });
        }

        if required > self.memory.len() {
            self.memory.resize(required, 0);
        }
        Ok(&self.memory[address..required])
    }

    /// Write to memory
    pub fn write_memory(&mut self, address: usize, data: &[u8]) -> Result<(), RuntimeError> {
        if data.is_empty() {
            return Ok(());
        }

        let required = address
            .checked_add(data.len())
            .ok_or(RuntimeError::ExecutionError {
                message: "Memory write overflow".to_string(),
            })?;
        if required > self.memory_limit {
            return Err(RuntimeError::ExecutionError {
                message: "Memory write exceeds configured limit".to_string(),
            });
        }

        if required > self.memory.len() {
            self.memory.resize(required, 0);
        }
        self.memory[address..required].copy_from_slice(data);
        Ok(())
    }

    /// Current size of the linear memory buffer
    pub fn memory_size(&self) -> usize {
        self.memory.len()
    }
}
