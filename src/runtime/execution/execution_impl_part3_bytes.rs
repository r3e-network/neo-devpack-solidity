use super::*;
use crate::runtime::execution::types::stack::ByteArrayType;

/// NeoVM `ExecutionEngineLimits.MaxItemSize` = `ushort.MaxValue * 2` = 131070
/// (verified against neo-vm ExecutionEngineLimits.cs). A ByteString/Buffer
/// whose length exceeds this FAULTs on a real node; NEWBUFFER, CAT and
/// integer/byte growth past it throw "MaxItemSize exceeded". Distinct from
/// the 65535-byte MaxStorageValueSize cap enforced separately in storage.rs.
pub(crate) const NEOVM_MAX_ITEM_SIZE: usize = u16::MAX as usize * 2;

impl ExecutionContext {
    pub(crate) fn new_buffer(&mut self) -> Result<(), RuntimeError> {
        let len = self.pop_usize("NEWBUFFER")?;
        // NeoVM faults when the requested size exceeds MaxItemSize (65535),
        // independent of the host memory budget — model it so the simulator
        // does not report success for a buffer a real node rejects.
        if len > NEOVM_MAX_ITEM_SIZE {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "NEWBUFFER: length {len} exceeds NeoVM MaxItemSize ({NEOVM_MAX_ITEM_SIZE})"
                ),
            });
        }
        // Bound user-supplied length against memory_limit to prevent OOM DoS
        // from attacker-supplied bytecode (PUSHINT + NEWBUFFER). Mirrors the
        // PUSHDATA4 length check in instruction/push.rs.
        if len > self.memory_limit {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "NEWBUFFER: requested length {} exceeds memory limit {}",
                    len, self.memory_limit
                ),
            });
        }
        self.push_stack(StackItem::buffer(vec![0u8; len]))
    }

    pub(crate) fn memcpy_bytes(&mut self) -> Result<(), RuntimeError> {
        // NeoVM MEMCPY stack order: [dst, dst_offset, src, src_offset, count]
        // (top-of-stack is `count`).
        let count = self.pop_usize("MEMCPY count")?;
        let src_offset = self.pop_usize("MEMCPY src_offset")?;
        let src = Self::stack_item_to_bytes(self.pop_stack()?);
        let dst_offset = self.pop_usize("MEMCPY dst_offset")?;
        let dst_item = self.pop_stack()?;

        match dst_item {
            StackItem::ByteArray {
                data: dst,
                type_tag,
            } => {
                // NeoVM MEMCPY only accepts Buffer (0x30) as destination;
                // copying into a ByteString (0x28) is disallowed on a real node.
                if type_tag == ByteArrayType::ByteString {
                    return Err(RuntimeError::ExecutionError {
                        message: "MEMCPY: destination must be a Buffer, not a ByteString"
                            .to_string(),
                    });
                }
                let src_end =
                    src_offset
                        .checked_add(count)
                        .ok_or(RuntimeError::ExecutionError {
                            message: "MEMCPY: source range overflow".to_string(),
                        })?;
                let dst_end =
                    dst_offset
                        .checked_add(count)
                        .ok_or(RuntimeError::ExecutionError {
                            message: "MEMCPY: destination range overflow".to_string(),
                        })?;

                {
                    let mut dst_ref = dst.borrow_mut();
                    if src_end > src.len() || dst_end > dst_ref.len() {
                        return Err(RuntimeError::ExecutionError {
                            message: "MEMCPY: range out of bounds".to_string(),
                        });
                    }

                    let src_slice = &src[src_offset..src_end];
                    dst_ref[dst_offset..dst_end].copy_from_slice(src_slice);
                }

                // Real NeoVM MEMCPY: Pop 5, Push 0. The destination buffer
                // is modified in-place via Rc<RefCell> interior mutability;
                // nothing is pushed back onto the evaluation stack.
                Ok(())
            }
            other => Err(RuntimeError::ExecutionError {
                message: format!("MEMCPY: unsupported destination {other:?}"),
            }),
        }
    }

    pub(crate) fn concat_bytes(&mut self) -> Result<(), RuntimeError> {
        let b = Self::stack_item_to_bytes(self.pop_stack()?);
        let mut a = Self::stack_item_to_bytes(self.pop_stack()?);
        // NeoVM CAT faults when the concatenated result exceeds MaxItemSize.
        if a.len().saturating_add(b.len()) > NEOVM_MAX_ITEM_SIZE {
            return Err(RuntimeError::ExecutionError {
                message: format!("CAT: result exceeds NeoVM MaxItemSize ({NEOVM_MAX_ITEM_SIZE})"),
            });
        }
        a.extend_from_slice(&b);
        self.push_stack(StackItem::buffer(a))
    }

    pub(crate) fn substr_bytes(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("SUBSTR")?;
        let index = self.pop_usize("SUBSTR")?;
        let data = Self::stack_item_to_bytes(self.pop_stack()?);
        // Checked add: `index + count` can overflow usize for crafted operands
        // and wrap past the bounds guard into a slice-index panic (mirrors
        // memcpy_bytes' checked arithmetic above).
        let end = index
            .checked_add(count)
            .ok_or(RuntimeError::ExecutionError {
                message: "SUBSTR: range overflow".to_string(),
            })?;
        if end > data.len() {
            return Err(RuntimeError::ExecutionError {
                message: "SUBSTR: out of bounds".to_string(),
            });
        }
        self.push_stack(StackItem::buffer(data[index..end].to_vec()))
    }

    pub(crate) fn left_bytes(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("LEFT")?;
        let data = Self::stack_item_to_bytes(self.pop_stack()?);
        if count > data.len() {
            return Err(RuntimeError::ExecutionError {
                message: "LEFT: out of bounds".to_string(),
            });
        }
        self.push_stack(StackItem::buffer(data[..count].to_vec()))
    }

    pub(crate) fn right_bytes(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("RIGHT")?;
        let data = Self::stack_item_to_bytes(self.pop_stack()?);
        if count > data.len() {
            return Err(RuntimeError::ExecutionError {
                message: "RIGHT: out of bounds".to_string(),
            });
        }
        self.push_stack(StackItem::buffer(data[data.len() - count..].to_vec()))
    }
}
