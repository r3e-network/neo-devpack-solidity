use super::*;

impl ExecutionContext {
    pub(crate) fn min_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        let lt = self.less_than(&a, &b)?;
        if lt {
            Ok(a)
        } else {
            Ok(b)
        }
    }

    pub(crate) fn max_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        let gt = self.greater_than(&a, &b)?;
        if gt {
            Ok(a)
        } else {
            Ok(b)
        }
    }

    pub(crate) fn within_stack_items(
        &self,
        value: StackItem,
        min_item: StackItem,
        max_item: StackItem,
    ) -> Result<bool, RuntimeError> {
        let ge_min = !self.less_than(&value, &min_item)?;
        let lt_max = self.less_than(&value, &max_item)?;
        Ok(ge_min && lt_max)
    }
}
