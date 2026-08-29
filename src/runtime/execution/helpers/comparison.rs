use super::*;

/// Shared error for invalid comparison operands (used as a reusable `fn` so it
/// can be passed to `ok_or_else` multiple times).
fn cmp_err() -> RuntimeError {
    RuntimeError::ExecutionError {
        message: "Invalid operands for comparison".to_string(),
    }
}

impl ExecutionContext {
    pub(crate) fn stack_items_equal(
        &self,
        a: &StackItem,
        b: &StackItem,
    ) -> Result<bool, RuntimeError> {
        // Task #30 slice 1 Part C: for wide ByteArray operands (e.g. uint256 values
        // pushed via PUSHINT256/PUSHDATA1), route through BigInt so the overflow
        // guards emitted by the compiler compare at full 256-bit width instead of
        // truncating to the low 8 bytes.
        if self.cmp_needs_bigint_path(a, b) {
            let x = self
                .coerce_item_to_bigint(a)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            let y = self
                .coerce_item_to_bigint(b)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            return Ok(x == y);
        }
        match (a, b) {
            (StackItem::Integer(x), StackItem::Integer(y)) => Ok(x == y),
            (StackItem::UnsignedInteger(x), StackItem::UnsignedInteger(y)) => Ok(x == y),
            (StackItem::Boolean(x), StackItem::Boolean(y)) => Ok(x == y),
            // ByteString (0x28) vs Buffer (0x30) — NeoVM EQUAL is type-strict.
            // Equal bytes with different type tags must return false.
            (
                StackItem::ByteArray {
                    data: x,
                    type_tag: tx,
                },
                StackItem::ByteArray {
                    data: y,
                    type_tag: ty,
                },
            ) => Ok(tx == ty && x == y),
            // Recursive element-wise comparison — must NOT delegate to
            // `PartialEq` because that impl ignores `type_tag` for nested
            // `ByteArray` elements (ByteString vs Buffer with equal bytes
            // would incorrectly compare as equal).
            (StackItem::Array(x), StackItem::Array(y)) => {
                let x = x.borrow();
                let y = y.borrow();
                if x.len() != y.len() {
                    return Ok(false);
                }
                for (a_item, b_item) in x.iter().zip(y.iter()) {
                    if !self.stack_items_equal(a_item, b_item)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            // Map comparison — key-value pairs must match, with type-strict
            // value comparison (recursive).
            (StackItem::Map(x), StackItem::Map(y)) => {
                let x = x.borrow();
                let y = y.borrow();
                if x.len() != y.len() {
                    return Ok(false);
                }
                for (k, v) in x.iter() {
                    match y.get(k) {
                        Some(v2) => {
                            if !self.stack_items_equal(v, v2)? {
                                return Ok(false);
                            }
                        }
                        None => return Ok(false),
                    }
                }
                Ok(true)
            }
            (StackItem::Null, StackItem::Null) => Ok(true),
            // Cross-type comparisons
            (StackItem::Integer(x), StackItem::UnsignedInteger(y)) => {
                if *x < 0 {
                    Ok(false)
                } else {
                    Ok(*x as u64 == *y)
                }
            }
            (StackItem::UnsignedInteger(x), StackItem::Integer(y)) => {
                if *y < 0 {
                    Ok(false)
                } else {
                    Ok(*x == *y as u64)
                }
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn less_than(&self, a: &StackItem, b: &StackItem) -> Result<bool, RuntimeError> {
        // Task #30 slice 1 Part C: see stack_items_equal.
        if self.cmp_needs_bigint_path(a, b) {
            let x = self
                .coerce_item_to_bigint(a)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            let y = self
                .coerce_item_to_bigint(b)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            return Ok(x < y);
        }

        if self.cmp_mixed_sign_needs_bigint(a, b) {
            let x = self.coerce_item_to_bigint(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_bigint(b).ok_or_else(cmp_err)?;
            return Ok(x < y);
        }

        let use_unsigned = matches!(a, StackItem::UnsignedInteger(_))
            || matches!(b, StackItem::UnsignedInteger(_));

        if use_unsigned {
            let x = self.coerce_item_to_u64(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_u64(b).ok_or_else(cmp_err)?;
            Ok(x < y)
        } else {
            let x = self.coerce_item_to_i64(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_i64(b).ok_or_else(cmp_err)?;
            Ok(x < y)
        }
    }

    /// NeoVM integers are arbitrary-precision with no signed/unsigned tag, so an
    /// `UnsignedInteger` compared/divided against a NEGATIVE `Integer` must be
    /// evaluated as BigInt rather than faulting on a u64 coercion (Solidity never
    /// emits such a mix directly, but the runtime can produce it internally).
    pub(crate) fn cmp_mixed_sign_needs_bigint(&self, a: &StackItem, b: &StackItem) -> bool {
        let has_unsigned = matches!(a, StackItem::UnsignedInteger(_))
            || matches!(b, StackItem::UnsignedInteger(_));
        let has_negative = matches!(a, StackItem::Integer(v) if *v < 0)
            || matches!(b, StackItem::Integer(v) if *v < 0);
        has_unsigned && has_negative
    }

    pub(crate) fn greater_than(&self, a: &StackItem, b: &StackItem) -> Result<bool, RuntimeError> {
        // Task #30 slice 1 Part C: see stack_items_equal.
        if self.cmp_needs_bigint_path(a, b) {
            let x = self
                .coerce_item_to_bigint(a)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            let y = self
                .coerce_item_to_bigint(b)
                .ok_or_else(|| RuntimeError::ExecutionError {
                    message: "Invalid operands for comparison".to_string(),
                })?;
            return Ok(x > y);
        }

        if self.cmp_mixed_sign_needs_bigint(a, b) {
            let x = self.coerce_item_to_bigint(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_bigint(b).ok_or_else(cmp_err)?;
            return Ok(x > y);
        }

        let use_unsigned = matches!(a, StackItem::UnsignedInteger(_))
            || matches!(b, StackItem::UnsignedInteger(_));

        if use_unsigned {
            let x = self.coerce_item_to_u64(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_u64(b).ok_or_else(cmp_err)?;
            Ok(x > y)
        } else {
            let x = self.coerce_item_to_i64(a).ok_or_else(cmp_err)?;
            let y = self.coerce_item_to_i64(b).ok_or_else(cmp_err)?;
            Ok(x > y)
        }
    }
}
