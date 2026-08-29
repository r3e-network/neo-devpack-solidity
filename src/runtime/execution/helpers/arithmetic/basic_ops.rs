use super::*;

/// Arithmetic operation helpers with overflow/underflow checking.
///
/// All arithmetic operations follow the same pattern:
/// 1. Determine if unsigned arithmetic should be used
/// 2. Coerce both operands to appropriate type (i64/u64)
/// 3. Perform operation with overflow checking (if strict_arithmetic is enabled)
/// 4. Return result as StackItem
///
/// The macros below generate the repetitive boilerplate for each operation.
/// Macro to generate arithmetic operation functions (ADD, SUB, MUL).
///
/// # Syntax
/// ```text
/// arithmetic_op!(fn_name, op_name, op_sym, checked_fn, wrapping_fn, error_kind);
/// ```
///
/// # Example
/// ```text
/// arithmetic_op!(add_stack_items, "ADD", "+", checked_add, wrapping_add, "overflow");
/// ```
macro_rules! arithmetic_op {
    ($fn_name:ident, $op_name:literal, $op_sym:literal, $checked:ident, $wrapping:ident, $error_kind:literal) => {
        fn $fn_name(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
            let use_unsigned = matches!(a, StackItem::UnsignedInteger(_))
                || matches!(b, StackItem::UnsignedInteger(_));

            if use_unsigned {
                let x =
                    self.coerce_item_to_u64(&a)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                let y =
                    self.coerce_item_to_u64(&b)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                if self.strict_arithmetic {
                    x.$checked(y)
                        .map(StackItem::UnsignedInteger)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: format!(
                                "Unsigned integer {} in {}: {} {} {}",
                                $error_kind, $op_name, x, $op_sym, y
                            ),
                        })
                } else {
                    Ok(StackItem::UnsignedInteger(x.$wrapping(y)))
                }
            } else {
                let x =
                    self.coerce_item_to_i64(&a)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                let y =
                    self.coerce_item_to_i64(&b)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                if self.strict_arithmetic {
                    x.$checked(y).map(StackItem::Integer).ok_or_else(|| {
                        RuntimeError::ExecutionError {
                            message: format!(
                                "Integer {} in {}: {} {} {}",
                                $error_kind, $op_name, x, $op_sym, y
                            ),
                        }
                    })
                } else {
                    Ok(StackItem::Integer(x.$wrapping(y)))
                }
            }
        }
    };
}

/// Macro to generate division/modulo operation functions (DIV, MOD).
///
/// Checks for division by zero AND the signed overflow case `i64::MIN / -1`
/// (which panics in Rust debug mode and wraps in release mode).
macro_rules! divmod_op {
    ($fn_name:ident, $op_name:literal, $checked_fn:ident, $error_msg:literal) => {
        fn $fn_name(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
            let use_unsigned = matches!(a, StackItem::UnsignedInteger(_))
                || matches!(b, StackItem::UnsignedInteger(_));

            if use_unsigned {
                let x =
                    self.coerce_item_to_u64(&a)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                let y =
                    self.coerce_item_to_u64(&b)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                if y == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: $error_msg.to_string(),
                    });
                }
                x.$checked_fn(y)
                    .map(StackItem::UnsignedInteger)
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: format!(
                            "Unsigned integer overflow in {}: {} op {}",
                            $op_name, x, y
                        ),
                    })
            } else {
                let x =
                    self.coerce_item_to_i64(&a)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                let y =
                    self.coerce_item_to_i64(&b)
                        .ok_or_else(|| RuntimeError::ExecutionError {
                            message: concat!("Invalid operands for ", $op_name).to_string(),
                        })?;
                if y == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: $error_msg.to_string(),
                    });
                }
                x.$checked_fn(y).map(StackItem::Integer).ok_or_else(|| {
                    RuntimeError::ExecutionError {
                        message: format!("Signed integer overflow in {}: {} / -1", $op_name, x),
                    }
                })
            }
        }
    };
}

impl ExecutionContext {
    /// Coerce a stack item to a `num_bigint::BigInt` (signed arbitrary-precision),
    /// decoding `ByteArray` via `from_signed_bytes_le`.
    ///
    /// Used by the comparison helpers (Task #30 slice 1 Part C) when at least one
    /// operand is a wide (> 8 byte) ByteArray and the narrow i64/u64 coercion
    /// would silently truncate, producing wrong comparison results for values
    /// that exceed 64 bits (e.g. `type(uint256).max` vs `low64(max) + 1`).
    pub(crate) fn coerce_item_to_bigint(&self, item: &StackItem) -> Option<num_bigint::BigInt> {
        use num_bigint::BigInt;
        match item {
            StackItem::Integer(value) => Some(BigInt::from(*value)),
            StackItem::UnsignedInteger(value) => Some(BigInt::from(*value)),
            StackItem::Boolean(value) => Some(BigInt::from(if *value { 1 } else { 0 })),
            StackItem::Null => Some(BigInt::from(0)),
            StackItem::ByteArray { data: bytes, .. } => {
                let bytes = bytes.borrow();
                if bytes.is_empty() {
                    Some(BigInt::from(0))
                } else {
                    Some(BigInt::from_signed_bytes_le(&bytes))
                }
            }
            _ => None,
        }
    }

    /// True when either operand is a ByteArray wider than 8 bytes, meaning the
    /// i64/u64 coercion path would truncate. Comparisons and divmod guards
    /// route through `coerce_item_to_bigint` in that case.
    pub(crate) fn cmp_needs_bigint_path(&self, a: &StackItem, b: &StackItem) -> bool {
        let is_wide = |item: &StackItem| -> bool {
            if let StackItem::ByteArray { data: bytes, .. } = item {
                bytes.borrow().len() > 8
            } else {
                false
            }
        };
        is_wide(a) || is_wide(b)
    }

    pub(crate) fn coerce_item_to_i64(&self, item: &StackItem) -> Option<i64> {
        match item {
            StackItem::Integer(value) => Some(*value),
            StackItem::UnsignedInteger(value) => i64::try_from(*value).ok(),
            StackItem::Boolean(value) => Some(if *value { 1 } else { 0 }),
            StackItem::Null => Some(0),
            StackItem::ByteArray { data: bytes, .. } => {
                let bytes = bytes.borrow();
                if bytes.is_empty() {
                    return Some(0);
                }
                // NeoVM integers are arbitrary-precision little-endian byte arrays.
                // This runtime uses i64 internally; truncation to the low 8 bytes
                // is intentional — the compiler emits masking ops (AND, SHL, etc.)
                // to handle width reduction at the Solidity level.
                let len = bytes.len().min(8);
                let mut buf = [0u8; 8];
                buf[..len].copy_from_slice(&bytes[..len]);
                Some(i64::from_le_bytes(buf))
            }
            _ => None,
        }
    }

    pub(crate) fn coerce_item_to_u64(&self, item: &StackItem) -> Option<u64> {
        match item {
            StackItem::UnsignedInteger(value) => Some(*value),
            StackItem::Integer(value) => {
                if *value < 0 {
                    None
                } else {
                    Some(*value as u64)
                }
            }
            StackItem::Boolean(value) => Some(if *value { 1 } else { 0 }),
            StackItem::Null => Some(0),
            StackItem::ByteArray { data: bytes, .. } => {
                let bytes = bytes.borrow();
                if bytes.is_empty() {
                    return Some(0);
                }
                // See coerce_item_to_i64 comment — truncation to low 8 bytes
                // is the correct NeoVM runtime behavior.
                let len = bytes.len().min(8);
                let mut buf = [0u8; 8];
                buf[..len].copy_from_slice(&bytes[..len]);
                Some(u64::from_le_bytes(buf))
            }
            _ => None,
        }
    }

    // Generate all arithmetic operations using the macros (narrow i64/u64 paths).
    arithmetic_op!(
        add_stack_items_narrow,
        "ADD",
        "+",
        checked_add,
        wrapping_add,
        "overflow"
    );
    arithmetic_op!(
        sub_stack_items_narrow,
        "SUB",
        "-",
        checked_sub,
        wrapping_sub,
        "underflow"
    );
    arithmetic_op!(
        mul_stack_items_narrow,
        "MUL",
        "*",
        checked_mul,
        wrapping_mul,
        "overflow"
    );
    divmod_op!(
        div_stack_items_narrow,
        "DIV",
        checked_div,
        "Division by zero"
    );
    divmod_op!(mod_stack_items_narrow, "MOD", checked_rem, "Modulo by zero");

    /// Encode a `BigInt` back as a `StackItem`, preserving signed-BigInt shape
    /// for downstream BigInt comparisons. Small-fit values return narrow
    /// `Integer` so the existing narrow comparison path keeps working for
    /// legacy tests; wider values return the signed-LE ByteArray directly.
    fn bigint_to_stack_item(value: num_bigint::BigInt) -> StackItem {
        use num_bigint::{BigInt, Sign};
        if value == BigInt::from(0) {
            return StackItem::Integer(0);
        }
        let bytes = value.to_signed_bytes_le();
        // NEGATIVE wide-arithmetic results are 256-bit values stored in
        // two's-complement (a `uint256` >= 2^255 "looks negative"). Emit them as
        // a fixed 32-byte two's-complement ByteArray — NOT a narrow i64 Integer —
        // so they stay distinguishable from narrow ints and serialize/return as
        // the full 256-bit word (the truncated i64 form would decode to the wrong
        // unsigned value). Out-of-range magnitudes (> 32 bytes) keep their raw
        // signed-LE shape so the checked-overflow guards can still observe them.
        if value.sign() == Sign::Minus && bytes.len() <= 32 {
            let mut buf = vec![0xFFu8; 32];
            buf[..bytes.len()].copy_from_slice(&bytes);
            return StackItem::byte_array(buf);
        }
        if bytes.len() <= 8 {
            // Non-negative fits in i64 — return narrow Integer for efficient
            // narrow comparisons (preserves pre-slice-1 behavior at small widths).
            let mut buf = [0u8; 8];
            buf[..bytes.len()].copy_from_slice(&bytes);
            StackItem::Integer(i64::from_le_bytes(buf))
        } else {
            StackItem::byte_array(bytes)
        }
    }

    /// Task #30: wide-operand ADD path using BigInt so uint256 values pushed as
    /// ByteArray retain full precision. Entered when either operand is a wide
    /// ByteArray (see `cmp_needs_bigint_path`). The raw BigInt result is
    /// returned without wrapping; the compiler-emitted checked-arithmetic
    /// guard (when applicable) panics before the caller observes an
    /// out-of-range value. `unchecked { }` blocks see the raw result shape.
    fn add_stack_items_wide(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
        let x = self
            .coerce_item_to_bigint(&a)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for ADD".to_string(),
            })?;
        let y = self
            .coerce_item_to_bigint(&b)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for ADD".to_string(),
            })?;
        Ok(Self::bigint_to_stack_item(x + y))
    }

    fn sub_stack_items_wide(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
        let x = self
            .coerce_item_to_bigint(&a)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for SUB".to_string(),
            })?;
        let y = self
            .coerce_item_to_bigint(&b)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for SUB".to_string(),
            })?;
        Ok(Self::bigint_to_stack_item(x - y))
    }

    fn mul_stack_items_wide(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
        let x = self
            .coerce_item_to_bigint(&a)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for MUL".to_string(),
            })?;
        let y = self
            .coerce_item_to_bigint(&b)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for MUL".to_string(),
            })?;
        Ok(Self::bigint_to_stack_item(x * y))
    }

    fn div_stack_items_wide(
        &mut self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        let x = self
            .coerce_item_to_bigint(&a)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for DIV".to_string(),
            })?;
        let y = self
            .coerce_item_to_bigint(&b)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for DIV".to_string(),
            })?;
        if y == num_bigint::BigInt::from(0) {
            return Err(RuntimeError::ExecutionError {
                message: "Division by zero".to_string(),
            });
        }
        // Task #30 slice 4: detect the unrepresentable INT256_MIN / -1 case
        // (quotient would be INT256_MAX + 1 = 2^255, which is not a valid
        // int256) and throw Panic(0x11).
        //
        // Task #108 — route through the canonical EVM `Panic(uint256)`
        // envelope instead of the legacy `"Panic: 0x11"` ByteString message.
        // Shape: `keccak256("Panic(uint256)")[..4] || abi.encode(0x11)` = 36
        // bytes (selector `0x4e487b71` then 32-byte big-endian panic code).
        // Populate `self.revert_payload` so `dispatch_exception` surfaces the
        // canonical payload to `catch Panic(uint code)` (via
        // `try_frames.rs::dispatch_exception` at `revert_payload.clone()`)
        // AND so the bridge's uncaught-error path at
        // `bridge/bridge_impl_core/execute.rs:161` routes it into
        // `ExecutionResult.return_data`. Keep the marker substring `"THROW"`
        // in the error message so `execute.rs:142` classifies this as
        // `ExceptionType::RevertExecution`, not `Fault` — otherwise
        // `return_data` would be cleared at `execute.rs:164` and the caller
        // would observe an empty payload.
        let one: num_bigint::BigInt = num_bigint::BigInt::from(1);
        let shifted: num_bigint::BigInt = one << 255u32;
        let int256_min: num_bigint::BigInt = -shifted;
        let neg_one: num_bigint::BigInt = num_bigint::BigInt::from(-1);
        if x == int256_min && y == neg_one {
            let mut envelope: Vec<u8> = Vec::with_capacity(36);
            // keccak256("Panic(uint256)")[0..4]
            envelope.extend_from_slice(&[0x4e, 0x48, 0x7b, 0x71]);
            // abi.encode(uint256 0x11) = 32-byte big-endian payload.
            envelope.extend_from_slice(&[0u8; 31]);
            envelope.push(0x11);
            self.revert_payload = envelope;
            return Err(RuntimeError::ExecutionError {
                message: "THROW: Panic(uint256) 0x11 (INT256_MIN / -1)".to_string(),
            });
        }
        Ok(Self::bigint_to_stack_item(x / y))
    }

    fn mod_stack_items_wide(&self, a: StackItem, b: StackItem) -> Result<StackItem, RuntimeError> {
        let x = self
            .coerce_item_to_bigint(&a)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for MOD".to_string(),
            })?;
        let y = self
            .coerce_item_to_bigint(&b)
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: "Invalid operands for MOD".to_string(),
            })?;
        if y == num_bigint::BigInt::from(0) {
            return Err(RuntimeError::ExecutionError {
                message: "Modulo by zero".to_string(),
            });
        }
        Ok(Self::bigint_to_stack_item(x % y))
    }

    /// Dispatch: if either operand is a wide ByteArray, take the BigInt path;
    /// otherwise preserve the existing narrow-i64 semantics.
    pub(crate) fn add_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        if self.cmp_needs_bigint_path(&a, &b) {
            self.add_stack_items_wide(a, b)
        } else {
            // NeoVM integers are arbitrary-precision (up to 32 bytes); an i64
            // narrow overflow is not a VM fault (Solidity's checked-overflow is
            // enforced by emitted guard code, not the ADD itself), so promote to
            // the BigInt path instead of faulting.
            self.add_stack_items_narrow(a.clone(), b.clone())
                .or_else(|_| self.add_stack_items_wide(a, b))
        }
    }

    pub(crate) fn sub_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        if self.cmp_needs_bigint_path(&a, &b) {
            self.sub_stack_items_wide(a, b)
        } else {
            self.sub_stack_items_narrow(a.clone(), b.clone())
                .or_else(|_| self.sub_stack_items_wide(a, b))
        }
    }

    pub(crate) fn mul_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        if self.cmp_needs_bigint_path(&a, &b) {
            self.mul_stack_items_wide(a, b)
        } else {
            self.mul_stack_items_narrow(a.clone(), b.clone())
                .or_else(|_| self.mul_stack_items_wide(a, b))
        }
    }

    pub(crate) fn div_stack_items(
        &mut self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        if self.cmp_needs_bigint_path(&a, &b) {
            self.div_stack_items_wide(a, b)
        } else {
            // Task #30 slice 4: runtime-side INT256_MIN / -1 guard. The narrow
            // path already handles i64::MIN / -1 via checked_div, but when
            // operands arrive as wide ByteArrays that truncate to narrow
            // values the BigInt path catches the unrepresentable quotient.
            self.div_stack_items_narrow(a, b)
        }
    }

    pub(crate) fn mod_stack_items(
        &self,
        a: StackItem,
        b: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        if self.cmp_needs_bigint_path(&a, &b) {
            self.mod_stack_items_wide(a, b)
        } else {
            self.mod_stack_items_narrow(a, b)
        }
    }
}
