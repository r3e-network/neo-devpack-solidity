use super::*;

impl ExecutionContext {
    pub(crate) fn sign_stack_item(&self, value: StackItem) -> Result<i64, RuntimeError> {
        match value {
            StackItem::Integer(v) => Ok(v.signum()),
            StackItem::UnsignedInteger(v) => Ok(if v == 0 { 0 } else { 1 }),
            StackItem::Boolean(b) => Ok(if b { 1 } else { 0 }),
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for SIGN".to_string(),
            }),
        }
    }

    pub(crate) fn abs_stack_item(&self, value: StackItem) -> Result<StackItem, RuntimeError> {
        match value {
            StackItem::Integer(v) => {
                v.checked_abs()
                    .map(StackItem::Integer)
                    .ok_or(RuntimeError::ExecutionError {
                        message: "ABS overflow".to_string(),
                    })
            }
            StackItem::UnsignedInteger(v) => Ok(StackItem::UnsignedInteger(v)),
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for ABS".to_string(),
            }),
        }
    }

    pub(crate) fn negate_stack_item(&self, value: StackItem) -> Result<StackItem, RuntimeError> {
        match value {
            StackItem::Integer(v) => {
                v.checked_neg()
                    .map(StackItem::Integer)
                    .ok_or(RuntimeError::ExecutionError {
                        message: "NEGATE overflow: cannot negate i64::MIN".to_string(),
                    })
            }
            StackItem::UnsignedInteger(v) => {
                if v <= i64::MAX as u64 {
                    Ok(StackItem::Integer(-(v as i64)))
                } else {
                    Err(RuntimeError::ExecutionError {
                        message: "NEGATE overflow for unsigned value".to_string(),
                    })
                }
            }
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for NEGATE".to_string(),
            }),
        }
    }

    pub(crate) fn inc_stack_item(&self, value: StackItem) -> Result<StackItem, RuntimeError> {
        match value {
            StackItem::Integer(v) => {
                if self.strict_arithmetic {
                    v.checked_add(1)
                        .map(StackItem::Integer)
                        .ok_or(RuntimeError::ExecutionError {
                            message: format!("Integer overflow in INC: {v}"),
                        })
                } else {
                    Ok(StackItem::Integer(v.wrapping_add(1)))
                }
            }
            StackItem::UnsignedInteger(v) => {
                if self.strict_arithmetic {
                    v.checked_add(1).map(StackItem::UnsignedInteger).ok_or(
                        RuntimeError::ExecutionError {
                            message: format!("Unsigned integer overflow in INC: {v}"),
                        },
                    )
                } else {
                    Ok(StackItem::UnsignedInteger(v.wrapping_add(1)))
                }
            }
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for INC".to_string(),
            }),
        }
    }

    pub(crate) fn dec_stack_item(&self, value: StackItem) -> Result<StackItem, RuntimeError> {
        match value {
            StackItem::Integer(v) => {
                if self.strict_arithmetic {
                    v.checked_sub(1)
                        .map(StackItem::Integer)
                        .ok_or(RuntimeError::ExecutionError {
                            message: format!("Integer underflow in DEC: {v}"),
                        })
                } else {
                    Ok(StackItem::Integer(v.wrapping_sub(1)))
                }
            }
            StackItem::UnsignedInteger(v) => {
                if self.strict_arithmetic {
                    v.checked_sub(1).map(StackItem::UnsignedInteger).ok_or(
                        RuntimeError::ExecutionError {
                            message: format!("Unsigned integer underflow in DEC: {v}"),
                        },
                    )
                } else {
                    Ok(StackItem::UnsignedInteger(v.wrapping_sub(1)))
                }
            }
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for DEC".to_string(),
            }),
        }
    }
}
