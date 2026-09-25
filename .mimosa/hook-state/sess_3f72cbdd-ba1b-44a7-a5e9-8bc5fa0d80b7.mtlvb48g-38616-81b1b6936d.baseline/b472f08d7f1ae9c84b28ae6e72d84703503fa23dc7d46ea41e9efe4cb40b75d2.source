use super::*;

impl ExecutionContext {
    pub(crate) fn pack_items(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("PACK")?;
        if count > self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: "PACK: insufficient stack items".to_string(),
            });
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(self.pop_stack()?);
        }
        self.push_stack(StackItem::array(items))?;
        self.enforce_global_stack_limit()
    }

    pub(crate) fn pack_map(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("PACKMAP")?;
        if count.saturating_mul(2) > self.stack.len() {
            return Err(RuntimeError::ExecutionError {
                message: "PACKMAP: insufficient stack items".to_string(),
            });
        }
        let mut map = std::collections::HashMap::new();
        for _ in 0..count {
            // NeoVM PACKMAP pops the KEY first (top of stack), then the value:
            //   `PrimitiveType key = Pop(); StackItem value = Pop();`
            let key = self.pop_stack()?;
            let value = self.pop_stack()?;
            let key_bytes = Self::stack_item_to_bytes(key);
            map.insert(key_bytes, value);
        }
        self.push_stack(StackItem::map(map))?;
        self.enforce_global_stack_limit()
    }
}
