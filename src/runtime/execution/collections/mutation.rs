use super::*;
use crate::runtime::execution::types::stack::ByteArrayType;

impl ExecutionContext {
    pub(crate) fn append_item(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_stack()?;
        let collection = self.pop_stack()?;
        match collection {
            StackItem::Array(items) => {
                // NeoVM MaxStackSize (2048) counts every contained item; an array
                // that grows past it FAULTs on a real node. Enforce it so the
                // simulator does not hide an on-chain "MaxStackSize exceeded"
                // fault by appending without bound.
                if items.borrow().len() >= 2048 {
                    return Err(RuntimeError::ExecutionError {
                        message: "APPEND: array exceeds NeoVM MaxStackSize (2048)".to_string(),
                    });
                }
                items.borrow_mut().push(value);
                // Global backstop: the single-collection check above only sees
                // THIS array; many separate collections can still blow the
                // limit collectively.
                self.enforce_global_stack_limit()
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let append = Self::stack_item_to_bytes(value);
                bytes.borrow_mut().extend_from_slice(&append);
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("APPEND: unsupported target {other:?}"),
            }),
        }
    }

    pub(crate) fn remove_item(&mut self) -> Result<(), RuntimeError> {
        let key = self.pop_stack()?;
        let collection = self.pop_stack()?;
        match collection {
            StackItem::Array(items) => {
                let index = self.index_from_key(&key, "REMOVE")?;
                let mut items = items.borrow_mut();
                if index >= items.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "REMOVE: index out of bounds".to_string(),
                    });
                }
                items.remove(index);
                Ok(())
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let index = self.index_from_key(&key, "REMOVE")?;
                let mut bytes = bytes.borrow_mut();
                if index >= bytes.len() {
                    return Err(RuntimeError::ExecutionError {
                        message: "REMOVE: index out of bounds".to_string(),
                    });
                }
                bytes.remove(index);
                Ok(())
            }
            StackItem::Map(map) => {
                let key_bytes = Self::stack_item_to_bytes(key);
                map.borrow_mut().remove(&key_bytes);
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("REMOVE: unsupported target {other:?}"),
            }),
        }
    }

    pub(crate) fn clear_items(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        match collection {
            StackItem::Array(items) => {
                items.borrow_mut().clear();
                Ok(())
            }
            StackItem::ByteArray { data: bytes, .. } => {
                bytes.borrow_mut().clear();
                Ok(())
            }
            StackItem::Map(map) => {
                map.borrow_mut().clear();
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("CLEARITEMS: unsupported target {other:?}"),
            }),
        }
    }

    pub(crate) fn pop_item_from_collection(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        match collection {
            StackItem::Array(items) => {
                let value = items
                    .borrow_mut()
                    .pop()
                    .ok_or(RuntimeError::ExecutionError {
                        message: "POPITEM: array is empty".to_string(),
                    })?;
                self.push_stack(value)
            }
            StackItem::ByteArray { data: bytes, .. } => {
                let value = bytes
                    .borrow_mut()
                    .pop()
                    .ok_or(RuntimeError::ExecutionError {
                        message: "POPITEM: buffer is empty".to_string(),
                    })?;
                self.push_stack(StackItem::byte_array(vec![value]))
            }
            StackItem::Map(_) => Err(RuntimeError::ExecutionError {
                message: "POPITEM: unsupported for maps".to_string(),
            }),
            other => Err(RuntimeError::ExecutionError {
                message: format!("POPITEM: unsupported target {other:?}"),
            }),
        }
    }

    pub(crate) fn reverse_items(&mut self) -> Result<(), RuntimeError> {
        let collection = self.pop_stack()?;
        match collection {
            StackItem::Array(items) => {
                items.borrow_mut().reverse();
                Ok(())
            }
            StackItem::ByteArray {
                data: bytes,
                type_tag,
            } => {
                // NeoVM REVERSEITEMS only accepts Buffer (0x30) and Array;
                // reversing a ByteString (0x28) is disallowed on a real node.
                if type_tag == ByteArrayType::ByteString {
                    return Err(RuntimeError::ExecutionError {
                        message: "REVERSEITEMS: cannot reverse a ByteString".to_string(),
                    });
                }
                bytes.borrow_mut().reverse();
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("REVERSEITEMS: unsupported target {other:?}"),
            }),
        }
    }
}
