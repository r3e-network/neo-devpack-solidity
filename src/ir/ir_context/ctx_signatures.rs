use super::*;

/// Task #91 — inlinable metadata for a library function whose first
/// parameter is a storage-pointer struct. `param_names[0]` is bound as a
/// storage alias to the receiver's `StorageReference`; `param_names[1..]`
/// are bound as locals populated from the call-site args. See
/// `member_calls.rs::inline_library_storage_call`.
#[derive(Debug, Clone)]
pub(crate) struct LibraryStorageBody {
    pub(crate) param_names: Vec<String>,
    pub(crate) value_param_types: Vec<Option<ValueType>>,
    pub(crate) body: Statement,
    pub(crate) return_type: Option<ValueType>,
}

/// Task #186 — per-binding metadata for an internal function-pointer local or
/// parameter. Knowing `arg_count` lets the bytecode emitter pick the correct
/// REVERSEN window size; `has_return` controls whether the `CallIndirect`
/// result is treated as a value or a statement-expression.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionPointerBinding {
    pub(crate) arg_count: usize,
    pub(crate) has_return: bool,
}

impl<'a> LoweringContext<'a> {
    pub(crate) fn state_type(&self, index: usize) -> Option<&ValueType> {
        self.state_types.get(index)
    }

    pub(crate) fn state_metadata(&self, index: usize) -> Option<&StateVariableMetadata> {
        self.state_variables.get(index)
    }

    /// Compile-time bound `N` when `struct_name.field_name` is declared as a
    /// fixed-size array (`T[N]`), `None` for dynamic (`T[]`) or non-array
    /// fields. See `struct_fixed_array_bounds`.
    pub(crate) fn struct_fixed_array_bound(
        &self,
        struct_name: &str,
        field_name: &str,
    ) -> Option<u64> {
        self.struct_fixed_array_bounds
            .get(&(struct_name.to_string(), field_name.to_string()))
            .copied()
    }

    pub(crate) fn can_write_state(&self, state_index: usize) -> bool {
        let Some(meta) = self.state_metadata(state_index) else {
            return true;
        };

        if !meta.is_immutable {
            return true;
        }

        // Sibling-merge (Task #198 in solidity_analyse.rs) renames sibling
        // constructors to `__ctor__<SiblingName>` and re-types them as
        // `FunctionTy::Function` so the host's `_deploy` prologue doesn't
        // accidentally re-run them. The body still semantically runs as
        // construction code (it's invoked from the host's `new Sibling(...)`
        // lowering), so it must remain allowed to assign to immutable
        // state vars — same as a regular constructor.
        //
        // Inheritance flattening additionally preserves a base constructor
        // body under `__super_<original>` / `__super2_<original>` etc. when
        // a derived contract overrides the constructor's name. Those super-
        // bodies also run as construction code from the derived constructor,
        // so they need the same exemption. Concrete repro: Chainlink
        // AutomationRegistry2_3 inherits from AutomationForwarder whose
        // constructor writes to `immutable i_target`; the flattener stores
        // the base body as `__super___ctor__AutomationForwarder` and the
        // immutable-write check rejected it without this clause.
        self.function_name == "constructor"
            || self.function_name == "_deploy"
            || self.function_name.starts_with("__ctor__")
            || self.function_name.contains("__ctor__")
    }

    pub(crate) fn ensure_state_writable(&mut self, state_index: usize) -> bool {
        if self.can_write_state(state_index) {
            return true;
        }

        let variable_name = self
            .state_metadata(state_index)
            .and_then(|meta| meta.name.as_deref())
            .unwrap_or("<unnamed>")
            .to_string();

        self.record_error_with_suggestion(
            format!(
                "cannot assign to immutable state variable '{variable_name}' outside constructor/deploy initialization"
            ),
            "initialize immutable values in the declaration or constructor only",
        );
        false
    }

    pub(crate) fn parameter_type(&self, name: &str) -> Option<&ValueType> {
        self.param_index_map
            .get(name)
            .and_then(|idx| self.param_types.get(*idx))
    }

    pub(crate) fn variable_type(&self, name: &str) -> Option<ValueType> {
        if let Some(reference) = self.storage_alias(name) {
            return Some(reference.value_type.clone());
        }
        if let Some(index) = self.state_index_map.get(name) {
            if let Some(ty) = self.state_type(*index) {
                return Some(ty.clone());
            }
        }
        if let Some(ty) = self.parameter_type(name) {
            return Some(ty.clone());
        }
        if let Some(local_index) = self.resolve_local(name) {
            if let Some(ty) = self.local_type(local_index) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Task #91 — fetch the inlinable body for a library function whose first
    /// parameter is a storage-pointer struct. Returns `None` when the call
    /// should go through the normal `CallFunction` path.
    pub(crate) fn library_storage_body(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Option<&LibraryStorageBody> {
        self.library_storage_bodies
            .get(&(name.to_string(), arg_count))
    }

    /// Task #196 — look up the state variable name aliased by a zero-arg
    /// internal function returning `T storage`. Returns `None` when the
    /// function isn't a simple storage-pointer alias. Used by the
    /// storage-reference resolver and the `.length` fast path to unwrap
    /// `fn()` into the backing state-var Expression so the caller can
    /// write to the actual storage slot instead of a materialised copy.
    pub(crate) fn storage_pointer_returning_fn(&self, name: &str) -> Option<&str> {
        self.storage_pointer_returning_fns
            .get(name)
            .map(|s| s.as_str())
    }

    pub(crate) fn event_signature(&self, event_name: &str) -> Option<&[ManifestType]> {
        self.event_signature_map
            .get(event_name)
            .map(|sig| sig.as_slice())
    }

    pub(crate) fn event_evm_signature(&self, event_name: &str) -> Option<&EventSignature> {
        self.event_params_map.get(event_name)
    }

    /// Declared signature for a custom `error`, or `None` when the name was
    /// never declared in (or inherited by / file-level-merged into) the
    /// current contract — callers then fall back to inferring canonical
    /// types from the revert-site argument expressions.
    pub(crate) fn error_signature(&self, error_name: &str) -> Option<&ErrorAbiSignature> {
        self.error_signature_map.get(error_name)
    }
}
