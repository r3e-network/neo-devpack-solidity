use super::*;

impl ExecutionContext {
    pub(crate) fn logical_not(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_stack()?;
        self.push_stack(StackItem::Boolean(!value.is_truthy()))
    }

    pub(crate) fn logical_and(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.push_stack(StackItem::Boolean(a.is_truthy() && b.is_truthy()))
    }

    pub(crate) fn logical_or(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop_stack()?;
        let a = self.pop_stack()?;
        self.push_stack(StackItem::Boolean(a.is_truthy() || b.is_truthy()))
    }
}
