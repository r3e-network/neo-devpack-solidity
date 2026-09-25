use super::*;

impl ExecutionContext {
    pub(crate) fn pop_usize(&mut self, opname: &str) -> Result<usize, RuntimeError> {
        let item = self.pop_stack()?;
        let as_usize = match item {
            StackItem::Integer(i) if i >= 0 => i as usize,
            StackItem::UnsignedInteger(u) => u as usize,
            _ => {
                return Err(RuntimeError::ExecutionError {
                    message: format!("{opname}: expected non-negative integer"),
                })
            }
        };
        Ok(as_usize)
    }

    pub(crate) fn clear_stack(&mut self) {
        self.stack.clear();
    }

    pub(crate) fn nip(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::ExecutionError {
                message: "NIP: insufficient stack items".to_string(),
            });
        }
        let top = self.pop_stack()?;
        self.pop_stack()?; // drop second
        self.push_stack(top)?;
        Ok(())
    }

    pub(crate) fn xdrop(&mut self) -> Result<(), RuntimeError> {
        let index = self.pop_usize("XDROP")?;
        if index >= self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: "XDROP: index out of bounds".to_string(),
            });
        }
        let remove_pos = self.stack.len() - 1 - index;
        self.stack.remove(remove_pos);
        Ok(())
    }

    pub(crate) fn over(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::ExecutionError {
                message: "OVER: insufficient stack items".to_string(),
            });
        }
        let second = self.stack[self.stack.len().saturating_sub(2)].clone();
        self.push_stack(second)
    }

    pub(crate) fn pick_n(&mut self) -> Result<(), RuntimeError> {
        let index = self.pop_usize("PICK")?;
        if index >= self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: "PICK: index out of bounds".to_string(),
            });
        }
        let pos = self.stack.len() - 1 - index;
        let item = self.stack[pos].clone();
        self.push_stack(item)
    }

    pub(crate) fn tuck(&mut self) -> Result<(), RuntimeError> {
        if self.stack.len() < 2 {
            return Err(RuntimeError::ExecutionError {
                message: "TUCK: insufficient stack items".to_string(),
            });
        }
        let top = self.pop_stack()?;
        let second = self.pop_stack()?;
        let duplicate = top.clone();
        self.push_stack(duplicate)?;
        self.push_stack(second)?;
        self.push_stack(top)?;
        Ok(())
    }

    pub(crate) fn roll(&mut self) -> Result<(), RuntimeError> {
        let index = self.pop_usize("ROLL")?;
        if index >= self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: "ROLL: index out of bounds".to_string(),
            });
        }
        let pos = self.stack.len() - 1 - index;
        let item = self.stack.remove(pos);
        self.push_stack(item)
    }

    pub(crate) fn reverse_top_n(&mut self, n: usize) -> Result<(), RuntimeError> {
        if n > self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: format!("REVERSEN: need {n} items"),
            });
        }
        let split_at = self.stack.len() - n;
        let mut tail = self.stack.split_off(split_at);
        tail.reverse();
        self.stack.extend(tail);
        Ok(())
    }
}
