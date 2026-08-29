use super::*;

impl ExecutionContext {
    pub(crate) fn pick_item(&mut self) -> Result<(), RuntimeError> {
        let key = self.pop_stack()?;
        let collection = self.pop_stack()?;
        let value = Self::pick_from_collection(collection, key)?;
        self.push_stack(value)
    }

    pub(crate) fn set_item(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_stack()?;
        let key = self.pop_stack()?;
        let collection = self.pop_stack()?;
        Self::set_in_collection(collection, key, value)
    }

    fn pick_from_collection(
        collection: StackItem,
        key: StackItem,
    ) -> Result<StackItem, RuntimeError> {
        match collection {
            StackItem::Array(items) => {
                let index = match key {
                    StackItem::Integer(i) if i >= 0 => i as usize,
                    StackItem::UnsignedInteger(u) => u as usize,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "PICKITEM: array index must be non-negative integer"
                                .to_string(),
                        })
                    }
                };
                items
                    .borrow()
                    .get(index)
                    .cloned()
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: "PICKITEM: index out of bounds".to_string(),
                    })
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let index = match key {
                    StackItem::Integer(i) if i >= 0 => i as usize,
                    StackItem::UnsignedInteger(u) => u as usize,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "PICKITEM: byte index must be non-negative integer"
                                .to_string(),
                        })
                    }
                };
                bytes
                    .borrow()
                    .get(index)
                    .map(|b| StackItem::byte_array(vec![*b]))
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: "PICKITEM: index out of bounds".to_string(),
                    })
            }
            StackItem::Map(map) => {
                let key_bytes = Self::stack_item_to_bytes(key);
                map.borrow()
                    .get(&key_bytes)
                    .cloned()
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: "PICKITEM: key not found".to_string(),
                    })
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("PICKITEM: unsupported target {other:?}"),
            }),
        }
    }

    fn set_in_collection(
        collection: StackItem,
        key: StackItem,
        value: StackItem,
    ) -> Result<(), RuntimeError> {
        match collection {
            StackItem::Array(items) => {
                let index = match key {
                    StackItem::Integer(i) if i >= 0 => i as usize,
                    StackItem::UnsignedInteger(u) => u as usize,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "SETITEM: array index must be non-negative integer"
                                .to_string(),
                        })
                    }
                };
                let mut items = items.borrow_mut();
                if index >= items.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "SETITEM: index out of bounds".to_string(),
                    });
                }
                items[index] = value;
                Ok(())
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let index = match key {
                    StackItem::Integer(i) if i >= 0 => i as usize,
                    StackItem::UnsignedInteger(u) => u as usize,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "SETITEM: byte index must be non-negative integer".to_string(),
                        })
                    }
                };
                let mut bytes = bytes.borrow_mut();
                if index >= bytes.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "SETITEM: index out of bounds".to_string(),
                    });
                }
                let as_byte = match value {
                    StackItem::ByteArray { data: v, .. } => {
                        let v = v.borrow();
                        if v.len() == 1 {
                            v[0]
                        } else {
                            return Err(RuntimeError::ExecutionError {
                                message: "SETITEM: value not representable as byte".to_string(),
                            });
                        }
                    }
                    StackItem::Integer(i) if (0..256).contains(&i) => i as u8,
                    StackItem::UnsignedInteger(u) if u < 256 => u as u8,
                    _ => {
                        return Err(RuntimeError::ExecutionError {
                            message: "SETITEM: value not representable as byte".to_string(),
                        })
                    }
                };
                bytes[index] = as_byte;
                Ok(())
            }
            StackItem::Map(map) => {
                let key_bytes = Self::stack_item_to_bytes(key);
                map.borrow_mut().insert(key_bytes, value);
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("SETITEM: unsupported target {other:?}"),
            }),
        }
    }

    pub(crate) fn index_from_key(
        &self,
        key: &StackItem,
        opname: &str,
    ) -> Result<usize, RuntimeError> {
        match key {
            StackItem::Integer(i) if *i >= 0 => Ok(*i as usize),
            StackItem::UnsignedInteger(u) => Ok(*u as usize),
            _ => Err(RuntimeError::ExecutionError {
                message: format!("{opname}: index must be non-negative integer"),
            }),
        }
    }
}
