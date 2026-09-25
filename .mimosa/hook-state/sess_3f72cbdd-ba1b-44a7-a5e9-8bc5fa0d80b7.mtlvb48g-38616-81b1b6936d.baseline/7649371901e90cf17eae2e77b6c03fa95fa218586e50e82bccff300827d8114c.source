use super::*;

/// S6 fix — Neo N3 CallFlags bitmask constants. See
/// `ExecutionContext::active_call_flags`. These mirror the
/// `Neo.SmartContract.CallFlags` enum: a callee's effective flags are the
/// caller-requested subset, and each mutating/notification/call syscall faults
/// unless its bit is set in the active context.
/// ReadStates has no production gate today (storage reads are unconditionally
/// permitted); declared for parity with the `Neo.SmartContract.CallFlags` enum
/// and the upcoming read-only-context follow-up.
#[allow(dead_code)]
pub(crate) const CALL_FLAG_READ_STATES: u8 = 0b0001;
pub(crate) const CALL_FLAG_WRITE_STATES: u8 = 0b0010;
pub(crate) const CALL_FLAG_ALLOW_CALL: u8 = 0b0100;
pub(crate) const CALL_FLAG_ALLOW_NOTIFY: u8 = 0b1000;
/// `CallFlags.All` — top-level default. Every gate below is a subset bit.
pub(crate) const CALL_FLAG_ALL: u8 = 0b1111;

/// Storage syscall implementations for Neo N3 VM compatibility.
///
/// This module implements the System.Storage.* syscall interface, providing persistent
/// key-value storage operations compatible with Neo N3's storage model.
///
/// # Storage Model
///
/// - **Keys**: Byte arrays (typically SHA-256 hashed for deterministic slot calculation)
/// - **Values**: Byte arrays (empty value represents deleted key)
/// - **Context**: Opaque handle representing storage context (simplified as empty byte array)
/// - **Gas**: Storage operations consume gas based on operation type and data size
///
/// # Syscall Signatures
///
/// ## Context Management
/// - `GetContext()` - Returns a storage context for regular operations
/// - `GetReadOnlyContext()` - Returns a read-only storage context
/// - `AsReadOnly(context)` - Converts a context to read-only view
///
/// ## Data Operations
/// - `Get(context, key)` - Retrieve value for key, returns empty if not found
/// - `Put(context, key, value)` - Store key-value pair, empty value deletes key
/// - `Delete(context, key)` - Remove key from storage
///
/// ## Query Operations
/// - `Find(context, prefix, options)` - Returns iterator for prefix search
///
/// # Storage Overlay
///
/// During execution, modifications are tracked in `storage_overlay` as `OverlayEntry`:
/// - `value`: The value (Some) or deletion marker (None)
/// - `dirty`: Whether this entry needs to be persisted
/// - Changes are flushed via `drain_dirty_storage_overlay()` after execution
///
/// # Thread Safety
///
/// Storage operations modify internal state but are safe within single-threaded
/// execution model. Concurrent access must be synchronized externally.
///
/// # Gas Costs
///
/// - GetContext/GetReadOnlyContext/AsReadOnly: 1 gas
/// - Get: 100 gas
/// - Put: 1,000 gas
/// - Delete: 100 gas
/// - Find: 100 gas
///
/// # Examples
///
/// ## Basic storage operation
/// ```text
/// // In NeoVM bytecode (simplified)
/// PUSHDATA1 "my_key"
/// PUSHDATA1 "my_value"
/// SYSCALL System.Storage.GetContext
/// SYSCALL System.Storage.Put
/// ```
///
/// ## Prefix search with Find
/// ```text
/// PUSH0                      // options (unused)
/// PUSHDATA1 "prefix"         // search prefix
/// SYSCALL System.Storage.GetContext
/// SYSCALL System.Storage.Find
/// // Returns iterator token
/// SYSCALL System.Iterator.Next
/// SYSCALL System.Iterator.Value
/// ```
impl ExecutionContext {
    pub(crate) fn handle_storage_syscall(&mut self, name: &str) -> Result<bool, RuntimeError> {
        match name {
            "System.Storage.GetContext" | "System.Storage.GetReadOnlyContext" => {
                self.push_stack(StackItem::byte_array(Vec::new()))?;
                Ok(true)
            }
            "System.Storage.AsReadOnly" => {
                // Neo N3 converts a StorageContext into a readonly view. The embedded runtime
                // represents contexts as opaque byte arrays, so we simply pass it through.
                let context = self.pop_stack()?;
                self.push_stack(context)?;
                Ok(true)
            }
            "System.Storage.Get" => {
                let _context = self.pop_stack()?; // ignored
                let slot_item = self.pop_stack()?;
                let key = Self::stack_item_to_bytes(slot_item);

                let value = if let Some(entry) = self.storage_overlay.get(&key) {
                    entry.value.clone().unwrap_or_default()
                } else {
                    let fetched = self.fetch_storage_value(&key)?;
                    let bytes = fetched.clone().unwrap_or_default();
                    self.storage_overlay.insert(
                        key.clone(),
                        OverlayEntry {
                            value: fetched,
                            dirty: false,
                        },
                    );
                    bytes
                };

                self.push_stack(StackItem::byte_array(value))?;
                Ok(true)
            }
            "System.Storage.Put" => {
                // S6 fix: Storage.Put requires the WriteStates CallFlag.
                // A read-only context (staticcall-shaped, only ReadStates)
                // must FAULT here — before this check a read-only callee
                // could silently mutate state.
                if self.active_call_flags & CALL_FLAG_WRITE_STATES == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "Storage.Put requires CallFlag.WriteStates".to_string(),
                    });
                }
                // Syscall signature: System.Storage.Put(context, key, value)
                // Stack order: [value, key, context] (top-of-stack is `context`)
                let _context = self.pop_stack()?; // context
                let slot_item = self.pop_stack()?; // key
                let value_item = self.pop_stack()?; // value

                let key = Self::stack_item_to_bytes(slot_item);
                let value = Self::stack_item_to_bytes(value_item);

                // Neo N3 consensus limits: a Storage.Put FAULTs when the key
                // exceeds MaxStorageKeySize (64) or the value exceeds
                // MaxStorageValueSize (65535). Without this, a contract storing
                // an oversized dynamic value (e.g. a >64 KB string) succeeds in
                // the simulator but reverts on-chain.
                const MAX_STORAGE_KEY_SIZE: usize = 64;
                const MAX_STORAGE_VALUE_SIZE: usize = 65535;
                if key.len() > MAX_STORAGE_KEY_SIZE {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "Storage.Put: key length {} exceeds Neo N3 MaxStorageKeySize ({MAX_STORAGE_KEY_SIZE})",
                            key.len()
                        ),
                    });
                }
                if value.len() > MAX_STORAGE_VALUE_SIZE {
                    return Err(RuntimeError::ExecutionError {
                        message: format!(
                            "Storage.Put: value length {} exceeds Neo N3 MaxStorageValueSize ({MAX_STORAGE_VALUE_SIZE})",
                            value.len()
                        ),
                    });
                }

                self.enforce_storage_limit(&key, &value)?;

                let entry =
                    self.storage_overlay
                        .entry(key.clone())
                        .or_insert_with(|| OverlayEntry {
                            value: None,
                            dirty: false,
                        });
                entry.value = if value.is_empty() { None } else { Some(value) };
                entry.dirty = true;
                Ok(true)
            }
            "System.Storage.Delete" => {
                // S6 fix: Storage.Delete requires the WriteStates CallFlag.
                if self.active_call_flags & CALL_FLAG_WRITE_STATES == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "Storage.Delete requires CallFlag.WriteStates".to_string(),
                    });
                }
                // Syscall signature: System.Storage.Delete(context, key)
                // Stack order: [key, context] (top-of-stack is `context`)
                let _context = self.pop_stack()?; // context
                let slot_item = self.pop_stack()?; // key
                let key = Self::stack_item_to_bytes(slot_item);
                self.storage_overlay.insert(
                    key,
                    OverlayEntry {
                        value: None,
                        dirty: true,
                    },
                );
                Ok(true)
            }
            "System.Storage.Find" => {
                // Call signature: Find(context, prefix, options)
                let _context = self.pop_stack()?;
                let prefix = Self::stack_item_to_bytes(self.pop_stack()?);
                let options = Self::stack_item_to_int(self.pop_stack()?);

                Self::validate_find_options(options)?;

                const STREAMING_PAGE_SIZE: usize = 50;
                let cursor = StreamingCursor {
                    prefix: prefix.clone(),
                    options,
                    last_key: None,
                    exhausted: false,
                    page_size: STREAMING_PAGE_SIZE,
                };
                let (entries, last_key) = self.query_storage_page(&cursor)?;
                let exhausted = entries.len() < STREAMING_PAGE_SIZE;
                let cursor = StreamingCursor {
                    prefix,
                    options: cursor.options,
                    last_key,
                    exhausted,
                    page_size: STREAMING_PAGE_SIZE,
                };
                let token = self.allocate_streaming_iterator(entries, cursor);
                self.push_stack(token)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Bug #17: enforce `storage_limit` on every Put. Without this, the
    /// `storage_limit` field is dead — an attacker contract could `Storage.put`
    /// arbitrary-length values until the host process OOM-aborts. The check
    /// computes the cumulative byte footprint of the in-memory `storage_overlay`
    /// (key + value across all live entries) plus the new entry's contribution,
    /// and rejects when the total would exceed `storage_limit`. When replacing
    /// an existing key, the old value's bytes are subtracted before adding the
    /// new write.
    fn enforce_storage_limit(&self, key: &[u8], value: &[u8]) -> Result<(), RuntimeError> {
        // Existing total across live overlay entries.
        let mut current: usize = 0;
        for (k, entry) in self.storage_overlay.iter() {
            if let Some(v) = &entry.value {
                current = current.saturating_add(k.len()).saturating_add(v.len());
            }
        }
        // Subtract the entry being replaced, if any.
        let replaced = self
            .storage_overlay
            .get(key)
            .and_then(|e| e.value.as_ref().map(|v| key.len() + v.len()))
            .unwrap_or(0);
        let after_replace = current.saturating_sub(replaced);
        let new_entry = if value.is_empty() {
            0 // delete (overlay value=None) — no new bytes
        } else {
            key.len().saturating_add(value.len())
        };
        let projected = after_replace.saturating_add(new_entry);
        if projected > self.storage_limit {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "Storage.Put: would exceed storage_limit ({} + {} > {})",
                    after_replace, new_entry, self.storage_limit
                ),
            });
        }
        Ok(())
    }
}
