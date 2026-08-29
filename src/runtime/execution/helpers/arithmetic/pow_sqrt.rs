use super::*;

impl ExecutionContext {
    pub(crate) fn pow_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        let exponent: u32 = match b {
            StackItem::Integer(e) => {
                if e < 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "POW exponent must be non-negative".to_string(),
                    });
                }
                e as u32
            }
            StackItem::UnsignedInteger(e) => {
                e.try_into().map_err(|_| RuntimeError::ExecutionError {
                    message: "POW exponent too large".to_string(),
                })?
            }
            _ => {
                return Err(RuntimeError::ExecutionError {
                    message: "Invalid exponent for POW".to_string(),
                })
            }
        };

        match a {
            StackItem::Integer(base) => base
                .checked_pow(exponent)
                .map(StackItem::Integer)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: format!("POW overflow: {base}^{exponent}"),
                }),
            StackItem::UnsignedInteger(base) => base
                .checked_pow(exponent)
                .map(StackItem::UnsignedInteger)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: format!("POW overflow: {base}^{exponent}"),
                }),
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid base for POW".to_string(),
            }),
        }
    }

    pub(crate) fn sqrt_stack_item(&self, value: StackItem) -> Result<StackItem, RuntimeError> {
        fn int_sqrt(n: u128) -> u128 {
            let mut x0 = n;
            let mut x1 = (x0 + 1) >> 1;
            while x1 < x0 {
                x0 = x1;
                x1 = (x1 + n / x1) >> 1;
            }
            x0
        }

        match value {
            StackItem::Integer(v) => {
                if v < 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "SQRT of negative value".to_string(),
                    });
                }
                Ok(StackItem::Integer(int_sqrt(v as u128) as i64))
            }
            StackItem::UnsignedInteger(v) => {
                Ok(StackItem::UnsignedInteger(int_sqrt(v as u128) as u64))
            }
            _ => Err(RuntimeError::ExecutionError {
                message: "Invalid operand for SQRT".to_string(),
            }),
        }
    }
}
