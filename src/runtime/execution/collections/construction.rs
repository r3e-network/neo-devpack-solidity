use super::*;

/// NeoVM's `ExecutionEngineLimits.MaxStackSize`. Every live StackItem — eval
/// stack, all slots, and every element nested inside arrays/maps — counts
/// toward it; exceeding it FAULTs.
pub(crate) const NEOVM_MAX_STACK_SIZE: usize = 2048;

impl ExecutionContext {
    pub(crate) fn new_array0(&mut self) -> Result<(), RuntimeError> {
        self.push_stack(StackItem::array(Vec::new()))
    }

    /// Count the total number of live StackItems reachable from every root
    /// (eval stack + indexed slots + saved call frames), recursing into
    /// arrays/maps, stopping once `cap` is exceeded.
    ///
    /// Shared array/map/bytestring objects are de-duplicated by pointer
    /// identity (mirroring NeoVM's per-item ReferenceCounter), and map keys —
    /// stored here as raw bytes rather than primitive items — are not counted.
    /// The result is therefore conservative w.r.t. compound/byte storage; a
    /// `> NEOVM_MAX_STACK_SIZE` result is dominated by genuinely-distinct
    /// collection elements (the only way a contract realistically reaches the
    /// limit), so it signals a real on-chain MaxStackSize fault rather than a
    /// false positive on a valid contract.
    pub(crate) fn live_stack_item_count(&self, cap: usize) -> usize {
        use std::rc::Rc;
        let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut count = 0usize;
        let mut work: Vec<StackItem> = Vec::new();
        work.extend(self.stack.iter().cloned());
        work.extend(self.locals.iter().cloned());
        work.extend(self.args.iter().cloned());
        work.extend(self.static_fields.iter().cloned());
        // Indexed slots saved on the call stack. `local_variables` (a parallel
        // named map) is intentionally NOT walked — it can alias `saved_locals`
        // and would double-count.
        for frame in &self.call_stack {
            work.extend(frame.saved_locals.iter().cloned());
            work.extend(frame.saved_args.iter().cloned());
        }
        while let Some(item) = work.pop() {
            if count > cap {
                return count;
            }
            match item {
                StackItem::Array(rc) => {
                    if seen.insert(Rc::as_ptr(&rc) as usize) {
                        count += 1;
                        work.extend(rc.borrow().iter().cloned());
                    }
                }
                StackItem::Map(rc) => {
                    if seen.insert(Rc::as_ptr(&rc) as usize) {
                        count += 1;
                        work.extend(rc.borrow().values().cloned());
                    }
                }
                StackItem::ByteArray { data: rc, .. } => {
                    if seen.insert(Rc::as_ptr(&rc) as usize) {
                        count += 1;
                    }
                }
                _ => count += 1,
            }
        }
        count
    }

    /// Fault when the live item count exceeds NeoVM's MaxStackSize. The global
    /// backstop to the cheap single-collection checks in `new_array` /
    /// `append_item`: catches the case where many separate collections, each
    /// individually under the limit, together exceed it (e.g. an array of many
    /// arrays). A real node FAULTs here; without this the simulator could
    /// report success for a script that reverts on-chain.
    pub(crate) fn enforce_global_stack_limit(&self) -> Result<(), RuntimeError> {
        if self.live_stack_item_count(NEOVM_MAX_STACK_SIZE) > NEOVM_MAX_STACK_SIZE {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "total live stack items exceed NeoVM MaxStackSize ({NEOVM_MAX_STACK_SIZE})"
                ),
            });
        }
        Ok(())
    }

    pub(crate) fn new_struct0(&mut self) -> Result<(), RuntimeError> {
        self.push_stack(StackItem::array(Vec::new()))
    }

    pub(crate) fn new_struct(&mut self) -> Result<(), RuntimeError> {
        self.new_array()
    }

    pub(crate) fn new_array(&mut self) -> Result<(), RuntimeError> {
        let count = self.pop_usize("NEWARRAY")?;
        // NeoVM's ExecutionEngineLimits.MaxStackSize (2048) counts EVERY item,
        // including elements inside arrays/maps/structs. A single collection that
        // exceeds 2048 therefore always blows the global limit and FAULTs on a
        // real node — model that here so the simulator does not hide an on-chain
        // fault by reporting success. (Conservative: a valid contract can never
        // hold a 2048+-element collection, so this never false-faults.)
        if count > 2048 {
            return Err(RuntimeError::ExecutionError {
                message: format!("NEWARRAY: {count} elements exceeds NeoVM MaxStackSize (2048)"),
            });
        }
        // Bound user-supplied count against memory_limit. Without this check,
        // a malicious PUSHINT + NEWARRAY/NEWSTRUCT/NEWARRAY_T sequence can
        // request ~2^31 elements and OOM-abort the host before gas accounting
        // fires. Mirrors the PUSHDATA4 length check in instruction/push.rs.
        let item_size = std::mem::size_of::<StackItem>();
        let required = count.saturating_mul(item_size);
        if required > self.memory_limit {
            return Err(RuntimeError::ExecutionError {
                message: format!(
                    "NEWARRAY: requested {} items ({} bytes) exceeds memory limit {}",
                    count, required, self.memory_limit
                ),
            });
        }
        let items = vec![StackItem::Null; count];
        self.push_stack(StackItem::array(items))?;
        // Global backstop for accumulating many separate collections.
        self.enforce_global_stack_limit()
    }

    pub(crate) fn new_map(&mut self) -> Result<(), RuntimeError> {
        self.push_stack(StackItem::map(std::collections::HashMap::new()))
    }
}
