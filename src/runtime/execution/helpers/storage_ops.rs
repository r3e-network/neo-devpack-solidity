use super::*;

/// Neo N3 `FindOptions` flags honoured by `System.Storage.Find` /
/// `System.Storage.Local.Find` (gap `nep11` — previously the bundled runtime
/// ignored the options argument and always yielded `[key, value]` structs,
/// diverging from real Neo N3 for `KeysOnly` / `RemovePrefix` / `ValuesOnly`
/// iterators such as the devpack NEP-11 `tokensOf` / `tokens` index scans).
mod find_options {
    // FindOptions.None is the absence of all flags (0x00).
    pub const KEYS_ONLY: i64 = 0x01;
    pub const REMOVE_PREFIX: i64 = 0x02;
    pub const VALUES_ONLY: i64 = 0x04;
    pub const DESERIALIZE_VALUES: i64 = 0x08;
    pub const PICK_FIELD_0: i64 = 0x10;
    pub const PICK_FIELD_1: i64 = 0x20;
    pub const BACKWARDS: i64 = 0x80;
    pub const ALL: i64 = KEYS_ONLY
        | REMOVE_PREFIX
        | VALUES_ONLY
        | DESERIALIZE_VALUES
        | PICK_FIELD_0
        | PICK_FIELD_1
        | BACKWARDS;
}

impl ExecutionContext {
    /// Validate a `FindOptions` bitmask with the same rules as the C# node
    /// (`Neo.SmartContract.ApplicationEngine.Storage`): unknown bits fault,
    /// `KeysOnly` excludes the value-shaping flags, `PickField*` are mutually
    /// exclusive and require `DeserializeValues`.
    pub(crate) fn validate_find_options(options: i64) -> Result<(), RuntimeError> {
        use find_options::*;
        let fail = |message: String| Err(RuntimeError::ExecutionError { message });
        if options & !ALL != 0 {
            return fail(format!(
                "Storage.Find: invalid FindOptions bits 0x{options:02x}"
            ));
        }
        if options & KEYS_ONLY != 0
            && options & (VALUES_ONLY | DESERIALIZE_VALUES | PICK_FIELD_0 | PICK_FIELD_1) != 0
        {
            return fail("Storage.Find: KeysOnly cannot be combined with value options".into());
        }
        if options & VALUES_ONLY != 0 && options & (KEYS_ONLY | REMOVE_PREFIX) != 0 {
            return fail("Storage.Find: ValuesOnly cannot be combined with key options".into());
        }
        if options & PICK_FIELD_0 != 0 && options & PICK_FIELD_1 != 0 {
            return fail("Storage.Find: PickField0 and PickField1 are mutually exclusive".into());
        }
        if options & (PICK_FIELD_0 | PICK_FIELD_1) != 0 && options & DESERIALIZE_VALUES == 0 {
            return fail("Storage.Find: PickField requires DeserializeValues".into());
        }
        Ok(())
    }

    /// Shape one `(key, value)` storage entry per the validated `options`
    /// bitmask, mirroring real Neo N3 iterator semantics.
    fn shape_find_entry(prefix: &[u8], options: i64, key: Vec<u8>, value: Vec<u8>) -> StackItem {
        use find_options::*;
        let key = if options & REMOVE_PREFIX != 0 {
            key[prefix.len().min(key.len())..].to_vec()
        } else {
            key
        };
        if options & KEYS_ONLY != 0 {
            return StackItem::byte_array(key);
        }
        let mut value_item = StackItem::byte_array(value);
        if options & DESERIALIZE_VALUES != 0 {
            let bytes = Self::stack_item_to_bytes(value_item);
            value_item = serde_json::from_slice::<StackItem>(&bytes).unwrap_or(StackItem::Null);
            let pick = if options & PICK_FIELD_0 != 0 {
                Some(0usize)
            } else if options & PICK_FIELD_1 != 0 {
                Some(1usize)
            } else {
                None
            };
            if let Some(index) = pick {
                value_item = match &value_item {
                    StackItem::Array(items) => items
                        .borrow()
                        .get(index)
                        .cloned()
                        .unwrap_or(StackItem::Null),
                    _ => StackItem::Null,
                };
            }
        }
        if options & VALUES_ONLY != 0 {
            return value_item;
        }
        StackItem::array(vec![StackItem::byte_array(key), value_item])
    }

    /// Allocate a streaming iterator that lazy-fetches pages from the host.
    ///
    /// The initial `entries` batch is placed in the buffer; subsequent pages
    /// are fetched by `refill_iterator_buffer` when `Iterator.Next` exhausts
    /// the buffer and a `StreamingCursor` is still active.
    pub(crate) fn allocate_streaming_iterator(
        &mut self,
        entries: Vec<StackItem>,
        cursor: StreamingCursor,
    ) -> StackItem {
        let id = self.next_iterator_id;
        self.next_iterator_id = self.next_iterator_id.saturating_add(1);
        self.iterators.insert(
            id,
            IteratorState {
                entries,
                index: 0,
                cursor: Some(cursor),
            },
        );
        StackItem::byte_array(id.to_le_bytes().to_vec())
    }

    /// Called by `Iterator.Next` when the buffer is exhausted and a streaming
    /// cursor exists. Fetches the next page from the storage host, merges
    /// pending overlay entries, and appends shaped items to the iterator's
    /// entry buffer.
    pub(crate) fn refill_iterator_buffer(&mut self, id: u64) -> Result<bool, RuntimeError> {
        // Take the cursor out so we can mutate it; we'll put it back.
        let cursor = match self.iterators.get(&id) {
            Some(state) => state.cursor.clone(),
            None => return Ok(false),
        };
        let Some(mut cursor) = cursor else {
            return Ok(false); // fully materialized iterator, nothing to refill
        };
        if cursor.exhausted {
            return Ok(false);
        }

        // Query the next page.
        let (new_entries, last_key) = self.query_storage_page(&cursor)?;

        if !new_entries.is_empty() {
            // Append to the iterator's entry buffer.
            if let Some(state) = self.iterators.get_mut(&id) {
                state.entries.extend(new_entries);
            }
            cursor.last_key = last_key;
        } else {
            cursor.exhausted = true;
        }

        // Put the updated cursor back.
        if let Some(state) = self.iterators.get_mut(&id) {
            state.cursor = Some(cursor);
        }
        Ok(true)
    }

    /// Query the storage host for a single page of entries, using the cursor
    /// for pagination. Returns the shaped entries and the last raw storage key
    /// (for the cursor), or None if the page is empty.
    pub(crate) fn query_storage_page(
        &self,
        cursor: &StreamingCursor,
    ) -> Result<(Vec<StackItem>, Option<Vec<u8>>), RuntimeError> {
        let mut entries: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        if let (Some(mut ptr), Some(account)) = (self.storage_host, self.storage_account.as_ref()) {
            // # Safety
            //
            // The storage_host pointer is valid for the execution context lifetime.
            // We only borrow mutably for the duration of this single operation.
            // No other code can access storage during this call since we're in a
            // single-threaded execution model with exclusive access to the context.
            let storage = unsafe { ptr.as_mut() };
            let query = storage::StorageQuery {
                account: account.clone(),
                key_prefix: Some(cursor.prefix.clone()),
                limit: Some(cursor.page_size),
                include_pending: true,
                start_after_key: cursor.last_key.clone(),
            };
            entries = storage.query(query)?;
        }
        self.shape_raw_entries(entries, cursor)
    }

    /// Shape raw (key, value) entries into StackItems, merging the overlay
    /// for keys within this page's range. Returns the shaped items and the
    /// last raw key (after overlay merge) for cursor pagination.
    pub(crate) fn shape_raw_entries(
        &self,
        mut entries: Vec<(Vec<u8>, Vec<u8>)>,
        cursor: &StreamingCursor,
    ) -> Result<(Vec<StackItem>, Option<Vec<u8>>), RuntimeError> {
        let prefix = &cursor.prefix;
        let options = cursor.options;

        // Merge overlay: only overlay keys within this page's key range matter.
        // Entries from the host query are already sorted by the storage manager.
        let start_key = cursor.last_key.clone();
        let end_key = entries.last().map(|(k, _)| k.clone());

        for (key, entry) in &self.storage_overlay {
            if !key.starts_with(prefix) {
                continue;
            }
            // Only merge if the overlay key falls in (start_key, end_key] range.
            // When both are None (first page or empty page), include all matching
            // overlay entries.
            let in_range = match (&start_key, &end_key) {
                (None, None) => true, // first page: include all overlay entries
                (Some(s), Some(e)) => {
                    key.as_slice() > s.as_slice() && key.as_slice() <= e.as_slice()
                }
                (None, Some(e)) => key.as_slice() <= e.as_slice(),
                (Some(s), None) => key.as_slice() > s.as_slice(),
            };
            if !in_range {
                continue;
            }
            match &entry.value {
                Some(value) => {
                    entries.retain(|(k, _)| k != key);
                    entries.push((key.clone(), value.clone()));
                }
                None => {
                    entries.retain(|(k, _)| k != key);
                }
            }
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));
        if options & find_options::BACKWARDS != 0 {
            entries.reverse();
        }
        let last_key = entries.last().map(|(k, _)| k.clone());
        let mut items = Vec::with_capacity(entries.len());
        for (k, v) in entries {
            items.push(Self::shape_find_entry(prefix, options, k, v));
        }
        Ok((items, last_key))
    }

    pub(crate) fn iterator_id_from_item(item: &StackItem) -> Option<u64> {
        if let StackItem::ByteArray { data: bytes, .. } = item {
            let bytes = bytes.borrow();
            if bytes.len() >= 8 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[..8]);
                return Some(u64::from_le_bytes(buf));
            }
        }
        None
    }

    pub(crate) fn register_contract(&mut self, nef: Vec<u8>, manifest: Vec<u8>) -> ContractState {
        fn nef_checksum(nef: &[u8]) -> u32 {
            if nef.len() >= 4 {
                let mut buf = [0u8; 4];
                let start = nef.len() - 4;
                buf.copy_from_slice(&nef[start..]);
                u32::from_le_bytes(buf)
            } else {
                let first = Sha256::digest(nef);
                let second = Sha256::digest(first);
                let mut checksum = [0u8; 4];
                checksum.copy_from_slice(&second[..4]);
                u32::from_le_bytes(checksum)
            }
        }

        fn manifest_name(manifest: &[u8]) -> Option<String> {
            let text = std::str::from_utf8(manifest).ok()?;
            let json: serde_json::Value = serde_json::from_str(text).ok()?;
            json.get("name")?.as_str().map(|s| s.to_string())
        }

        let checksum = nef_checksum(&nef);
        let name = manifest_name(&manifest).unwrap_or_default();
        let sender_bytes = self
            .caller_account
            .as_deref()
            .unwrap_or(&self.default_account_bytes);
        let mut sender_le = [0u8; 20];
        if sender_bytes.len() >= 20 {
            sender_le.copy_from_slice(&sender_bytes[..20]);
        } else {
            sender_le[..sender_bytes.len()].copy_from_slice(sender_bytes);
        }

        let hash = crate::neo::compute_contract_hash(sender_le, checksum, &name);
        let id = self.next_contract_id;
        self.next_contract_id = self.next_contract_id.saturating_add(1);
        let state = ContractState {
            id,
            hash,
            nef,
            manifest,
            update_counter: 0,
        };
        self.contract_registry.insert(hash.to_vec(), state.clone());
        state
    }

    pub(crate) fn update_contract(
        &mut self,
        hash: &[u8],
        nef: Vec<u8>,
        manifest: Vec<u8>,
    ) -> Option<ContractState> {
        if let Some(existing) = self.contract_registry.get_mut(hash) {
            existing.nef = nef;
            existing.manifest = manifest;
            existing.update_counter = existing.update_counter.saturating_add(1);
            return Some(existing.clone());
        }
        None
    }

    pub(crate) fn lookup_contract(&self, hash: &[u8]) -> Option<ContractState> {
        self.contract_registry.get(hash).cloned()
    }

    pub(crate) fn contract_to_stackitem(&self, state: &ContractState) -> StackItem {
        StackItem::array(vec![
            StackItem::UnsignedInteger(state.id as u64),
            StackItem::UnsignedInteger(state.update_counter as u64),
            StackItem::byte_array(state.hash.to_vec()),
            StackItem::byte_array(state.nef.clone()),
            StackItem::byte_array(state.manifest.clone()),
        ])
    }

    pub fn unbind_storage(&mut self) {
        self.storage_host = None;
        self.storage_account = Some(self.default_account.clone());
        self.storage_overlay.clear();
    }

    /// S7 fix test helper — insert (or overwrite) a storage-overlay entry for
    /// `key` with `value`, marked dirty. Lets a test pre-seed the overlay so
    /// it can observe whether a faulted execution clobbered it. Host-only API;
    /// compiled contracts reach storage via the syscalls.
    pub fn storage_overlay_insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.storage_overlay.insert(
            key,
            OverlayEntry {
                value: Some(value),
                dirty: true,
            },
        );
    }

    /// S7 fix test helper — read the current overlay value for `key`.
    /// Returns the inner bytes if the entry exists and is non-tombstone.
    pub fn storage_overlay_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage_overlay.get(key).and_then(|e| e.value.clone())
    }

    /// S7 fix — snapshot the current storage overlay so an inner-call fault
    /// can be rolled back to this point. Returns a cheap clone of the map;
    /// callers store it on the [`CallFrame`] being pushed so the matching
    /// [`Self::restore_storage_snapshot`] runs exactly when that frame unwinds.
    pub(crate) fn snapshot_storage_overlay(&self) -> HashMap<Vec<u8>, OverlayEntry> {
        self.storage_overlay.clone()
    }

    /// S7 fix — restore the storage overlay to a prior snapshot, discarding
    /// any writes the callee made between snapshot and fault.
    pub(crate) fn restore_storage_snapshot(&mut self, snapshot: HashMap<Vec<u8>, OverlayEntry>) {
        self.storage_overlay = snapshot;
    }

    pub fn drain_dirty_storage_overlay(&mut self) -> Option<(String, StorageOverlayEntries)> {
        let account = self.storage_account.clone()?;
        let mut entries = Vec::new();
        for (key, entry) in self.storage_overlay.drain() {
            if entry.dirty {
                entries.push((key, entry.value));
            }
        }

        if entries.is_empty() {
            None
        } else {
            Some((account, entries))
        }
    }

    pub(crate) fn fetch_storage_value(
        &mut self,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        match (self.storage_host, self.storage_account.as_ref()) {
            (Some(mut ptr), Some(account)) => {
                // # Safety
                //
                // Same guarantee as `query_storage_page()`:
                // The storage_host pointer is valid for the execution context lifetime.
                // We only borrow mutably for the duration of this single operation.
                // No other code can access storage during this call since we're in a
                // single-threaded execution model with exclusive access to the context.
                let storage = unsafe { ptr.as_mut() };
                storage.get(account, key)
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod s7_tests {
    use super::super::*;
    use crate::runtime::RuntimeConfig;

    fn fresh_ctx() -> ExecutionContext {
        ExecutionContext::new(&RuntimeConfig::default()).expect("ctx")
    }

    /// S7 fix — snapshot/restore isolation. A callee that writes storage
    /// and then faults must have its writes discarded; the caller's
    /// pre-existing entries must survive. This is the load-bearing unit
    /// for the `storage_snapshot` field wired through
    /// `handle_contract_call` → `CallFrame.storage_snapshot` →
    /// `dispatch_exception`'s unwind loop.
    #[test]
    fn snapshot_restore_discards_callee_writes_keeps_caller_state() {
        let mut ctx = fresh_ctx();
        // Caller pre-existing state: A = "caller_val".
        ctx.storage_overlay_insert(vec![b'A'], b"caller_val".to_vec());

        // Callee entry: snapshot the overlay.
        let snapshot = ctx.snapshot_storage_overlay();

        // Callee writes: clobbers A and adds a new key B.
        ctx.storage_overlay_insert(vec![b'A'], b"callee_clobber".to_vec());
        ctx.storage_overlay_insert(vec![b'B'], b"callee_only".to_vec());

        // Callee faults → restore the snapshot.
        ctx.restore_storage_snapshot(snapshot);

        // After rollback: A is back to caller_val, B is gone.
        assert_eq!(
            ctx.storage_overlay_get(b"A"),
            Some(b"caller_val".to_vec()),
            "S7: callee's clobber of A must be discarded on revert"
        );
        assert_eq!(
            ctx.storage_overlay_get(b"B"),
            None,
            "S7: callee-only write B must be discarded on revert"
        );
    }

    /// S7 fix regression guard — snapshot captures the *current* overlay,
    /// not an empty one.
    #[test]
    fn snapshot_is_not_empty_when_overlay_has_entries() {
        let mut ctx = fresh_ctx();
        ctx.storage_overlay_insert(vec![b'X'], b"v".to_vec());
        let snapshot = ctx.snapshot_storage_overlay();
        assert_eq!(snapshot.len(), 1, "snapshot must reflect the live overlay");
        assert_eq!(
            snapshot.get(&vec![b'X']).and_then(|e| e.value.clone()),
            Some(b"v".to_vec())
        );
    }
}
