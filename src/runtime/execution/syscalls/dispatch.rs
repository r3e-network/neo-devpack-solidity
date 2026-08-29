use super::*;

impl ExecutionContext {
    pub(crate) fn handle_syscall(&mut self, id: [u8; 4]) -> Result<(), RuntimeError> {
        let name = match spec::SYSCALLS.get(&id) {
            Some(spec) => spec.name,
            None => {
                return Err(RuntimeError::ExecutionError {
                    message: format!(
                        "Unsupported syscall id {:02X?} at ip {}",
                        id, self.instruction_pointer
                    ),
                })
            }
        };

        if self.handle_storage_syscall(name)? {
            return Ok(());
        }
        if self.handle_runtime_syscall(name)? {
            return Ok(());
        }
        if self.handle_crypto_syscall(name)? {
            return Ok(());
        }
        if self.handle_iterator_syscall(name)? {
            return Ok(());
        }
        if self.handle_contract_syscall(name)? {
            return Ok(());
        }

        Err(RuntimeError::ExecutionError {
            message: format!("Unimplemented syscall {name}"),
        })
    }
}
