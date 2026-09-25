use super::*;

impl ExecutionContext {
    pub(crate) fn stack_item_to_bytes(item: StackItem) -> Vec<u8> {
        match item {
            StackItem::ByteArray { data: bytes, .. } => bytes.borrow().clone(),
            StackItem::Integer(value) => value.to_le_bytes().to_vec(),
            StackItem::UnsignedInteger(value) => value.to_le_bytes().to_vec(),
            StackItem::Boolean(value) => vec![value as u8],
            StackItem::Map(_) | StackItem::Array(_) => {
                serde_json::to_vec(&item).unwrap_or_default()
            }
            StackItem::Null => Vec::new(),
        }
    }

    /// Convert a StackItem to an integer value.
    pub(crate) fn stack_item_to_int(item: StackItem) -> i64 {
        match item {
            StackItem::Integer(v) => v,
            StackItem::UnsignedInteger(v) => v as i64,
            StackItem::Boolean(v) => v as i64,
            StackItem::ByteArray { data: bytes, .. } => {
                let b = bytes.borrow();
                let mut buf = [0u8; 8];
                for (i, byte) in b.iter().take(8).enumerate() {
                    buf[i] = *byte;
                }
                i64::from_le_bytes(buf)
            }
            _ => 0,
        }
    }

    /// Extract the first integer argument from a params StackItem (Array wrapper).
    pub(crate) fn extract_first_int(params: &StackItem) -> u64 {
        if let StackItem::Array(args) = params {
            let borrowed = args.borrow();
            if let Some(item) = borrowed.first() {
                return match item {
                    StackItem::UnsignedInteger(v) => *v,
                    StackItem::Integer(v) => *v as u64,
                    StackItem::ByteArray { data: bytes, .. } => {
                        let b = bytes.borrow();
                        let mut buf = [0u8; 8];
                        for (i, byte) in b.iter().take(8).enumerate() {
                            buf[i] = *byte;
                        }
                        u64::from_le_bytes(buf)
                    }
                    _ => 0,
                };
            }
        }
        0
    }

    /// Extract the first byte-array argument from a params StackItem (Array wrapper).
    pub(crate) fn extract_first_bytes(params: &StackItem) -> Vec<u8> {
        if let StackItem::Array(args) = params {
            let borrowed = args.borrow();
            if let Some(item) = borrowed.first() {
                return Self::stack_item_to_bytes(item.clone());
            }
        }
        Vec::new()
    }

    fn strip_hex_prefix(value: &str) -> &str {
        value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .unwrap_or(value)
    }

    pub(crate) fn normalize_account(account: &str) -> Result<String, RuntimeError> {
        let trimmed = account.trim();
        let without_prefix = Self::strip_hex_prefix(trimmed);
        if !without_prefix.len().is_multiple_of(2) {
            return Err(RuntimeError::ConfigurationError {
                message: "contract account hex string has odd length".to_string(),
            });
        }
        if without_prefix.len() != 40 {
            return Err(RuntimeError::ConfigurationError {
                message: format!(
                    "contract account must be a UInt160 (40 hex chars, got {})",
                    without_prefix.len()
                ),
            });
        }
        let lower = without_prefix.to_ascii_lowercase();
        Ok(format!("0x{lower}"))
    }

    pub(crate) fn account_string_to_bytes(account: &str) -> Result<Vec<u8>, RuntimeError> {
        let normalized = Self::normalize_account(account)?;
        let le = crate::neo::parse_uint160_hex_be(&normalized).map_err(|msg| {
            RuntimeError::ConfigurationError {
                message: format!("invalid contract account: {msg}"),
            }
        })?;
        Ok(le.to_vec())
    }

    pub fn bind_storage(
        &mut self,
        account: &str,
        storage: &mut storage::StorageManager,
    ) -> Result<(), RuntimeError> {
        let normalized = Self::normalize_account(account)?;
        self.storage_account = Some(normalized);
        self.storage_host = Some(NonNull::from(storage));
        self.storage_overlay.clear();
        // Seed native contract balances for default account
        self.neo_balances
            .insert(self.default_account_bytes.clone(), self.neo_total_supply);
        self.gas_balances
            .insert(self.default_account_bytes.clone(), self.gas_total_supply);
        self.try_stack.clear();
        Ok(())
    }
}
