use super::*;

impl ExecutionContext {
    /// Push value onto stack
    pub fn push_stack(&mut self, item: StackItem) -> Result<(), RuntimeError> {
        if self.stack.len() >= 2048 {
            // NeoVM stack limit
            return Err(RuntimeError::StackOverflow {
                depth: self.stack.len() as u32,
            });
        }
        self.stack.push(item);
        Ok(())
    }

    /// Pop value from stack
    pub fn pop_stack(&mut self) -> Result<StackItem, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::ExecutionError {
            message: "Stack underflow".to_string(),
        })
    }

    /// Peek at top stack item
    pub fn peek_stack(&self) -> Result<&StackItem, RuntimeError> {
        self.stack.last().ok_or(RuntimeError::ExecutionError {
            message: "Stack is empty".to_string(),
        })
    }

    /// Get stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }
}
