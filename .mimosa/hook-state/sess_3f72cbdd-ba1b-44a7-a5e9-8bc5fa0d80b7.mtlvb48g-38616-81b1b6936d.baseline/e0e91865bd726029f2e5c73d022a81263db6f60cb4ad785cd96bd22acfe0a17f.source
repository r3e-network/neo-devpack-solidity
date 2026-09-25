use super::*;

impl<'a> LoweringContext<'a> {
    pub(crate) fn next_label(&mut self) -> usize {
        let label = self.label_counter;
        // `label_counter` is `usize`; a real contract can never generate
        // 2^64 labels. Use checked_add so the (impossible) overflow path
        // surfaces with a clear panic rather than wrapping silently.
        self.label_counter = self
            .label_counter
            .checked_add(1)
            .expect("label counter overflow: contract exceeds usize::MAX labels");
        label
    }

    pub(crate) fn push_loop(&mut self, continue_label: usize, break_label: usize) {
        self.loop_stack.push(LoopLabels {
            continue_label,
            break_label,
        });
    }

    pub(crate) fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    pub(crate) fn break_target(&self) -> Option<usize> {
        self.loop_stack.last().map(|labels| labels.break_label)
    }

    pub(crate) fn continue_target(&self) -> Option<usize> {
        self.loop_stack.last().map(|labels| labels.continue_label)
    }

    pub(crate) fn allocate_local(&mut self, name: String, value_type: Option<ValueType>) -> usize {
        let index = self.local_count as usize;
        // `local_count` is `u16`. A function needing > 65 535 local slots is
        // unrepresentable, but it is reachable from valid (if pathological)
        // Solidity — many lowerings allocate fresh per-statement temporaries
        // (e.g. each Yul `mstore`/`div`, each bytesN pad). A `.expect()` here
        // would ABORT the whole compiler. Instead SATURATE: `local_count`
        // pins at `u16::MAX`, which the per-method guard in
        // `bytecode_core.rs` (`local_count > 255`) turns into the normal
        // "exceeds NeoVM's 255-slot limit" compile ERROR — the same graceful
        // path the 256..=65 535 case already takes. (Saturated slots collide,
        // but the recorded error stops emission before any bytecode is used.)
        self.local_count = self.local_count.saturating_add(1);
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.push(name.clone());
        }
        self.local_index_map.entry(name).or_default().push(index);
        if let Some(ty) = value_type {
            self.local_types.insert(index, ty);
        }
        index
    }

    /// Return `n` shared scratch local slots for the inline uint256 routines,
    /// allocating (and caching) more on first demand. Reused across every
    /// uint256 arith site in the current function.
    pub(crate) fn u256_scratch_locals(&mut self, n: usize) -> Vec<usize> {
        while self.u256_scratch.len() < n {
            let i = self.u256_scratch.len();
            let idx = self.allocate_local(format!("__u256_scratch_{i}"), None);
            self.u256_scratch.push(idx);
        }
        self.u256_scratch[..n].to_vec()
    }

    /// Return `n` reusable scratch locals for the nested-dynamic ABI
    /// encoder/decoder at nesting `depth`. Locals are lazily allocated and
    /// shared across every call site reaching the same depth (see
    /// [`Self::abi_nested_scratch`]).
    pub(crate) fn abi_nested_scratch_locals(&mut self, depth: usize, n: usize) -> Vec<usize> {
        while self.abi_nested_scratch.len() <= depth {
            self.abi_nested_scratch.push(Vec::new());
        }
        while self.abi_nested_scratch[depth].len() < n {
            let i = self.abi_nested_scratch[depth].len();
            let idx = self.allocate_local(format!("__abi_nested_{depth}_{i}"), None);
            self.abi_nested_scratch[depth].push(idx);
        }
        self.abi_nested_scratch[depth][..n].to_vec()
    }

    pub(crate) fn resolve_local(&self, name: &str) -> Option<usize> {
        self.local_index_map
            .get(name)
            .and_then(|stack| stack.last().copied())
    }

    pub(crate) fn ensure_local(&mut self, name: &str) -> usize {
        if let Some(index) = self.resolve_local(name) {
            index
        } else {
            self.allocate_local(name.to_string(), None)
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        if let Some(names) = self.scope_stack.pop() {
            for name in names {
                if let Some(stack) = self.local_index_map.get_mut(&name) {
                    if let Some(index) = stack.pop() {
                        self.local_types.remove(&index);
                    }
                    if stack.is_empty() {
                        self.local_index_map.remove(&name);
                    }
                }
                self.storage_aliases.remove(&name);
            }
        }
    }

    pub(crate) fn is_local_in_current_scope(&self, name: &str) -> bool {
        self.scope_stack
            .last()
            .is_some_and(|scope| scope.iter().any(|existing| existing == name))
    }

    pub(crate) fn set_storage_alias(&mut self, name: String, alias: StorageReference) {
        self.storage_aliases.insert(name, alias);
    }

    pub(crate) fn storage_alias(&self, name: &str) -> Option<&StorageReference> {
        self.storage_aliases.get(name)
    }
}
