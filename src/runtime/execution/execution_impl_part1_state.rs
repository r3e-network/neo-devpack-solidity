use super::types::context::{ManifestMethods, ManifestPermission};
use super::*;

impl ExecutionContext {
    /// Get gas limit
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// Get gas used so far
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Get pending block height override, if any.
    pub fn pending_block_height(&self) -> Option<u64> {
        self.pending_block_height
    }

    /// Get pending timestamp override, if any.
    pub fn pending_timestamp(&self) -> Option<u64> {
        self.pending_timestamp
    }

    /// Get pending caller account override bytes, if any.
    pub fn pending_caller_account(&self) -> Option<&[u8]> {
        self.pending_caller_account.as_deref()
    }

    /// Task #113 — Get pending msg.value override, if any.
    pub fn pending_msg_value(&self) -> Option<u64> {
        self.pending_msg_value
    }

    /// Task #113 — Get the active msg.value for the current execution.
    /// `None` indicates no override was set (compiled `GetMsgValue`
    /// syscall coalesces None → 0 at push time).
    pub fn msg_value(&self) -> Option<u64> {
        self.msg_value
    }

    /// Get the active block height for the current execution.
    pub fn block_height(&self) -> Option<u64> {
        self.block_height
    }

    /// Get the active timestamp for the current execution.
    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }

    /// Get the active caller account for the current execution.
    pub fn caller_account(&self) -> Option<&[u8]> {
        self.caller_account.as_deref()
    }

    /// Get the default account bytes configured for this context.
    pub fn default_account_bytes(&self) -> &[u8] {
        &self.default_account_bytes
    }

    /// Disable the bytecode-derived `default_account_bytes` behaviour.
    ///
    /// When a test harness seeds a deterministic `default_account_bytes`
    /// (for example to exercise native-contract balance tracking or an
    /// explicit caller account), it can call this after construction to
    /// pin the explicitly configured value. Subsequent `initialize` calls
    /// will NOT overwrite `default_account_bytes` with `Hash160(bytecode)`.
    ///
    /// This is the escape hatch that keeps pre-existing tests (which set
    /// `contract_account` to a non-default value) unaffected by the
    /// self-hash derivation introduced for `address(this)` support.
    pub fn force_default_account_explicit_for_tests(&mut self) {
        self.default_account_derived = false;
    }

    /// Number of contracts currently tracked by the in-memory registry.
    pub fn contract_registry_len(&self) -> usize {
        self.contract_registry.len()
    }

    /// Update counter for a registered contract (if present).
    pub fn contract_update_counter(&self, hash: &[u8]) -> Option<u32> {
        self.contract_registry.get(hash).map(|c| c.update_counter)
    }

    /// Known contract hashes (20-byte each).
    pub fn contract_hashes(&self) -> Vec<[u8; 20]> {
        self.contract_registry
            .keys()
            .filter_map(|k| {
                if k.len() == 20 {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(k);
                    Some(arr)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get maximum stack depth
    pub fn max_stack_depth(&self) -> u32 {
        self.max_stack_depth
    }

    /// Task #70: Install the `(method_name → (offset, arg_count))` table that
    /// `handle_contract_call` uses to route `this.someFn()` self external
    /// calls to compiled method offsets. Populated from `manifest.abi.methods`
    /// by `NeoRuntime::call_method` before each invocation.
    pub fn set_self_method_table(&mut self, table: Vec<(String, u32, u16)>) {
        self.self_method_offsets.clear();
        self.self_method_arg_counts.clear();
        for (name, offset, arg_count) in table {
            self.self_method_offsets.insert(name.clone(), offset);
            self.self_method_arg_counts.insert(name, arg_count);
        }
    }

    /// Task #70: Clear the self-method table. Called between `execute()`
    /// invocations that don't run through `call_method`.
    pub fn clear_self_method_table(&mut self) {
        self.self_method_offsets.clear();
        self.self_method_arg_counts.clear();
    }

    /// S6 follow-up — parse the manifest's `permissions` array into the
    /// structured matchers `manifest_permits` consumes. Called by
    /// `NeoRuntime::call_method` alongside `set_self_method_table`.
    pub fn set_manifest_permissions(&mut self, manifest: &serde_json::Value) {
        self.manifest_permissions = Some(parse_manifest_permissions(manifest));
    }

    /// S6 follow-up — return `true` when the executing contract's manifest
    /// declares permission to call `target_hash_le`/`method`. Self-calls are
    /// exempt by construction (the self-offsets arm of `handle_contract_call`
    /// returns before reaching this check — the compiler deliberately omits
    /// self-permissions, see `permissions/calls.rs`). When no manifest is
    /// loaded (`None`, the raw `execute()` path), the check is skipped.
    pub(crate) fn manifest_permits(&self, target_hash_le: &[u8; 20], method: &str) -> bool {
        let Some(perms) = &self.manifest_permissions else {
            return true;
        };
        perms.iter().any(|p| {
            let contract_ok = match &p.contract {
                None => true,
                Some(c) => *c == *target_hash_le,
            };
            let methods_ok = match &p.methods {
                ManifestMethods::All => true,
                ManifestMethods::Some(set) => set.contains(method),
            };
            contract_ok && methods_ok
        })
    }
}

/// S6 follow-up — parse `manifest["permissions"]` into structured matchers.
/// Each entry is `{"contract": "0x<be>" | "*", "methods": ["m1",..] | "*"}`.
/// The big-endian hex is reversed to little-endian for direct byte comparison
/// with the eval-stack UInt160. Malformed entries are skipped.
fn parse_manifest_permissions(manifest: &serde_json::Value) -> Vec<ManifestPermission> {
    let Some(arr) = manifest["permissions"].as_array() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let contract = match entry["contract"].as_str() {
            Some("*") | None => None,
            Some(hex_str) => match parse_hash_le(hex_str) {
                Some(h) => Some(h),
                None => continue,
            },
        };
        let methods = match &entry["methods"] {
            serde_json::Value::String(s) if s == "*" => ManifestMethods::All,
            serde_json::Value::Array(items) => {
                let set: HashSet<String> = items
                    .iter()
                    .filter_map(|m| m.as_str().map(str::to_string))
                    .collect();
                ManifestMethods::Some(set)
            }
            _ => ManifestMethods::All,
        };
        out.push(ManifestPermission { contract, methods });
    }
    out
}

/// Decode `"0x<big-endian-hex>"` into little-endian bytes for direct eval-stack
/// comparison. Returns `None` for anything that isn't a 20-byte hex string.
fn parse_hash_le(s: &str) -> Option<[u8; 20]> {
    let hex = s.strip_prefix("0x").unwrap_or(s);
    if hex.len() != 40 {
        return None;
    }
    let mut be = [0u8; 20];
    hex::decode_to_slice(hex, &mut be).ok()?;
    let mut le = be;
    le.reverse();
    Some(le)
}
