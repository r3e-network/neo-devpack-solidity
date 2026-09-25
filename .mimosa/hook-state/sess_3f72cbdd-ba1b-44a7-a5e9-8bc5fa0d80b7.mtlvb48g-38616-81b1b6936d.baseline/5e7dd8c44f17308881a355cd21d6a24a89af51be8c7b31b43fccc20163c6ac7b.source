use super::*;

impl ExecutionContext {
    pub(crate) fn handle_iterator_syscall(&mut self, name: &str) -> Result<bool, RuntimeError> {
        match name {
            "System.Iterator.Next" => {
                let token = self.pop_stack()?;
                let mut has_next = false;
                if let Some(id) = Self::iterator_id_from_item(&token) {
                    // Check the current buffer first.
                    if let Some(state) = self.iterators.get(&id) {
                        has_next = state.index < state.entries.len();
                    }
                    // If the buffer is exhausted and there's a streaming cursor,
                    // try to fetch the next page.
                    if !has_next {
                        has_next = self.refill_iterator_buffer(id)?;
                    }
                    if has_next {
                        if let Some(state) = self.iterators.get_mut(&id) {
                            state.index += 1;
                        }
                    }
                }
                self.push_stack(StackItem::Boolean(has_next))?;
                Ok(true)
            }
            "System.Iterator.Value" => {
                let token = self.pop_stack()?;
                let mut value = StackItem::Null;
                if let Some(id) = Self::iterator_id_from_item(&token) {
                    if let Some(state) = self.iterators.get(&id) {
                        if state.index > 0 {
                            value = state
                                .entries
                                .get(state.index - 1)
                                .cloned()
                                .unwrap_or(StackItem::Null);
                        }
                    }
                }
                self.push_stack(value)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
