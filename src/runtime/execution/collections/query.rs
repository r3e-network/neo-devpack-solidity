use super::*;

impl ExecutionContext {
    pub(crate) fn size_of(&mut self) -> Result<(), RuntimeError> {
        let item = self.pop_stack()?;
        let size = match item {
            StackItem::Array(items) => items.borrow().len(),
            StackItem::Map(map) => map.borrow().len(),
            StackItem::ByteArray { data: bytes, .. } => bytes.borrow().len(),
            StackItem::Integer(_)
            | StackItem::UnsignedInteger(_)
            | StackItem::Boolean(_)
            | StackItem::Null => {
                return Err(RuntimeError::ExecutionError {
                    message: "SIZE: unsupported type".to_string(),
                })
            }
        };
        self.push_stack(StackItem::Integer(size as i64))
    }

    pub(crate) fn haskey(&mut self) -> Result<(), RuntimeError> {
        let key = self.pop_stack()?;
        let collection = self.pop_stack()?;
        let exists = match collection {
            StackItem::Array(items) => match key {
                StackItem::Integer(i) if i >= 0 => (i as usize) < items.borrow().len(),
                StackItem::UnsignedInteger(u) => (u as usize) < items.borrow().len(),
                _ => false,
            },
            StackItem::Map(map) => {
                let key_bytes = Self::stack_item_to_bytes(key);
                map.borrow().contains_key(&key_bytes)
            }
            StackItem::ByteArray { data: bytes, .. } => match key {
                StackItem::Integer(i) if i >= 0 => (i as usize) < bytes.borrow().len(),
                StackItem::UnsignedInteger(u) => (u as usize) < bytes.borrow().len(),
                _ => false,
            },
            _ => false,
        };
        self.push_stack(StackItem::Boolean(exists))
    }

    pub(crate) fn keys(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        let keys = match collection {
            StackItem::Map(map) => StackItem::array(
                map.borrow()
                    .keys()
                    .cloned()
                    .map(StackItem::byte_array)
                    .collect::<Vec<_>>(),
            ),
            _ => {
                return Err(RuntimeError::ExecutionError {
                    message: "KEYS: only supported for maps".to_string(),
                })
            }
        };
        self.push_stack(keys)
    }

    pub(crate) fn values(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        let values = match collection {
            StackItem::Map(map) => StackItem::array(map.borrow().values().cloned().collect()),
            StackItem::Array(items) => StackItem::Array(items),
            StackItem::ByteArray { data: bytes, .. } => StackItem::array(
                bytes
                    .borrow()
                    .iter()
                    .map(|b| StackItem::byte_array(vec![*b]))
                    .collect(),
            ),
            _ => {
                return Err(RuntimeError::ExecutionError {
                    message: "VALUES: unsupported target".to_string(),
                })
            }
        };
        self.push_stack(values)
    }

    pub(crate) fn unpack(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        let items = match collection {
            StackItem::Array(items) => items.borrow().clone(),
            StackItem::ByteArray { data: bytes, .. } => bytes
                .borrow()
                .iter()
                .map(|b| StackItem::byte_array(vec![*b]))
                .collect(),
            StackItem::Map(map) => map.borrow().values().cloned().collect(),
            other => {
                return Err(RuntimeError::ExecutionError {
                    message: format!("UNPACK: unsupported target {other:?}"),
                })
            }
        };
        let count = items.len();
        for item in items.into_iter().rev() {
            self.push_stack(item)?;
        }
        self.push_stack(StackItem::Integer(count as i64))
    }
}
