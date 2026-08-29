use super::*;

/// Structured diagnostic emitted during IR lowering.
///
/// Captures the originating function name, a human-readable message, and an
/// optional actionable suggestion so that CLI consumers can render richer
/// error output than a bare string.
#[derive(Debug, Clone, serde::Serialize)]
pub struct IrDiagnostic {
    pub function_name: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub code: Option<String>,
}

impl IrDiagnostic {
    /// Format as a human-readable error string.
    pub fn display(&self) -> String {
        let mut out = format!("function '{}': {}", self.function_name, self.message);
        if let Some(ref suggestion) = self.suggestion {
            out.push_str(&format!("\n  help: {suggestion}"));
        }
        out
    }
}

pub(crate) struct LoweringContext<'a> {
    pub(crate) function_name: String,
    /// Name of the contract that owns this lowering context.
    ///
    /// Used by member-access selector resolution to look up `this.method.selector`
    /// expressions against the current contract's method table so they can be
    /// lowered to their Ethereum keccak-4 selectors (matching how the AST shape
    /// `MemberAccess(MemberAccess(Variable("this"), method), selector)` is
    /// expected to behave in Solidity).
    pub(crate) current_contract_name: String,
    pub(crate) current_function_selector: [u8; 4],
    pub(crate) is_safe: bool,
    /// Task #64 — whether this function is directly callable from the host
    /// (External or Public visibility). When true, multi-value returns are
    /// lowered through `abiEncode` so the main-frame RET emits EVM-canonical
    /// BE-packed bytes rather than a `StackItem::Array` that would leak as
    /// serde_json at `stack_item_to_bytes`. Internal/Private functions keep
    /// the Array shape so callers can destructure via `ArrayGet`.
    pub(crate) is_externally_callable: bool,
    pub(crate) param_index_map: HashMap<String, usize>,
    pub(crate) param_types: &'a [ValueType],
    pub(crate) return_slots: Vec<Option<usize>>,
    pub(crate) return_types: Vec<ValueType>,
    /// Task #185 — original Solidity type string for each declared return
    /// parameter (e.g. `uint[3][2]`). Used by `lower_return_statement` to
    /// detect nested fixed-size arrays (`T[N1][N2]...[Nk]`) so the return
    /// path can emit flat EVM-canonical static encoding instead of the
    /// dynamic-array offset+length wrapper. ValueType alone cannot carry
    /// fixed-size info today (adding `fixed_size: Option<u64>` to
    /// `ValueType::Array` would cascade through 55+ call sites), so the
    /// fixed-shape hint rides alongside as a string.
    pub(crate) return_type_strings: Vec<String>,
    pub(crate) state_variables: &'a [StateVariableMetadata],
    pub(crate) state_index_map: &'a HashMap<String, usize>,
    pub(crate) state_types: &'a [ValueType],
    /// Canonical struct type definitions available to the compilation unit.
    ///
    /// This enables resolving user-defined structs even when they are only used
    /// in local variables (i.e., not present in state/param/return types).
    pub(crate) defined_struct_types: &'a [ValueType],
    /// Compile-time bound `N` for every fixed-size array struct field
    /// (`struct S { uint256[3] arr; }`), keyed by `(struct_name, field_name)`.
    ///
    /// `ValueType::Array` collapses `T[N]` and `T[]` into one variant, but the
    /// storage layout differs: fixed-size fields never maintain a length slot,
    /// so the struct-field array bounds guard (storage-soundness fix, see
    /// `lower_array_subscript_expression`) must use the declared `N` instead
    /// of loading a length that would always read 0.
    pub(crate) struct_fixed_array_bounds: &'a HashMap<(String, String), u64>,
    pub(crate) event_index_map: &'a HashMap<String, usize>,
    pub(crate) event_signature_map: &'a HashMap<String, Vec<ManifestType>>,
    pub(crate) event_params_map: &'a HashMap<String, EventSignature>,
    /// Declared custom `error` signatures keyed by error name. Used by the
    /// revert/require lowering to compute EVM custom-error selectors from
    /// the DECLARED parameter types and to reorder named arguments into
    /// declaration order.
    pub(crate) error_signature_map: &'a HashMap<String, ErrorAbiSignature>,
    pub(crate) enum_variant_map: &'a HashMap<String, HashMap<String, u64>>,
    pub(crate) contract_types: &'a HashSet<String>,
    pub(crate) selector_registry: &'a SelectorRegistry,
    pub(crate) function_names: &'a HashSet<String>,
    pub(crate) function_overloads: &'a FunctionOverloadTable,
    /// Every observed first-parameter type for each overload key
    /// (name, arg_count).
    ///
    /// Used to enforce Solidity-style receiver compatibility for `using for`
    /// member calls (`x.f(...)` lowers to `f(x, ...)`). Solidity allows
    /// overloading by parameter type — `toInt128(int256)` and
    /// `toInt128(uint256)` share the same `(name, arity)` key — so the
    /// bucket stores ALL observed first-parameter types and the receiver
    /// check accepts a match against ANY of them.
    pub(crate) function_first_param_types: &'a HashMap<(String, usize), Vec<ValueType>>,
    /// Task #191 — first return-parameter type for each overload key
    /// (name, arg_count). Used by `infer_type_from_expression` to resolve
    /// member access on a FunctionCall result (e.g. `makeCounter().value`
    /// or `c.inc().value`), so the struct-field-access lowering can pick
    /// the right field index instead of falling through to the drop-and-
    /// push-zero compatibility branch. Only the FIRST return type is
    /// recorded because member access on a multi-return function call is
    /// not valid Solidity — `(a, b) = f()` uses tuple destructuring.
    pub(crate) function_return_types: &'a HashMap<(String, usize), ValueType>,
    /// Normalized target types from parsed `using` directives.
    ///
    /// `None` means wildcard target (`for *`).
    pub(crate) using_target_types: &'a [Option<String>],
    /// Function-list constraints from `using {f, g} for T`.
    ///
    /// Map key is normalized function name (lowercase) and values are allowed
    /// target types (`None` means wildcard).
    pub(crate) using_function_list_targets: &'a HashMap<String, Vec<Option<String>>>,
    /// Target scopes that have at least one `using { ... } for T` directive.
    pub(crate) using_function_list_scope_targets: &'a [Option<String>],
    /// Ordered parameter names for each function overload, keyed by (name, arg_count).
    /// Used to reorder named function call arguments into positional order.
    pub(crate) function_param_names: &'a HashMap<(String, usize), Vec<String>>,
    /// Functions that return void (empty return_parameters). Used to avoid
    /// emitting DROP after calling a void internal function as a statement.
    pub(crate) void_functions: &'a HashSet<String>,
    /// Bug-hunt #9 — same-contract functions that are externally callable
    /// (Public/External), keyed by (name, arity). Their body ABI-encodes a
    /// single dynamic-array/`bytes`/`string` return (the external calling
    /// convention), so a PLAIN internal call to one must decode that blob back
    /// into the value instead of binding the raw ByteString.
    pub(crate) externally_callable_functions: &'a HashSet<(String, usize)>,
    /// Mapping from original method name to renamed super-method name.
    /// Used to resolve `super.method()` calls during IR lowering.
    pub(crate) super_method_map: &'a HashMap<String, String>,
    /// Task #91 — library functions whose first parameter is `T storage`,
    /// keyed by (name, arg_count). `member_calls.rs` inlines the body at
    /// the call site rather than emitting `CallFunction`, so storage writes
    /// (`d.x = v`) hit the caller's slot instead of a materialised copy.
    pub(crate) library_storage_bodies: &'a HashMap<(String, usize), LibraryStorageBody>,
    /// Task #196 — zero-arg internal functions that trivially return a
    /// storage pointer to a state variable (body is `return <state_var>;`
    /// and the return parameter is declared `T storage`). When a call site
    /// references `foo()` whose result feeds into a storage operation
    /// (`foo().push(v)`, `foo().length`, `foo()[i] = v`), the resolver
    /// unwraps the call into the backing `Variable(state_var)` so the
    /// downstream storage-reference machinery can alias the actual slot
    /// instead of the raw `LoadState` value (which for an array state
    /// variable is the LENGTH, not the backing Array — see
    /// `emit_coerce_storage_value` for `ValueType::Array`). Extends Task
    /// #117's local-binding alias fix (`uint[] storage a = arr;`) across
    /// the function-return boundary.
    pub(crate) storage_pointer_returning_fns: &'a HashMap<String, String>,
    /// Task #91 — stack of (inline-return slot, end-label). When set,
    /// `lower_return_statement` redirects `return expr;` to store into
    /// `slot` and jump to `end_label` instead of emitting a raw `Return`
    /// that would exit the caller.
    pub(crate) inline_return_stack: Vec<(Option<usize>, usize)>,
    /// Task #114 — modifier-epilogue return redirect. When set by
    /// `function.rs` for a function whose body was wrapped by at least one
    /// modifier with an epilogue (statements after `_;`), every
    /// `Statement::Return(expr)` in the body must store into `slots` (one
    /// per declared return parameter, already allocated at function
    /// prologue) and jump to the INNERMOST modifier-wrap break label (see
    /// `modifier_break_stack`) — or to `end_label` as a fallback when the
    /// wrap hasn't been entered yet. The modifier-wrap break label lands
    /// inside the Solidity expansion BETWEEN the inlined body and the
    /// modifier epilogue, so tail statements (`locked = 0;` and friends)
    /// still run before the actual RET. Distinct from the library-inline
    /// single-slot mechanism (`inline_return_stack`): modifier returns must
    /// carry multi-value tuples when the function declares `returns (T, U)`.
    pub(crate) modifier_return_redirect: Option<(Vec<Option<usize>>, usize)>,
    /// Task #114 — stack of break labels belonging to the synthetic
    /// `do { body } while(false)` wrappers emitted by
    /// `apply_modifier_calls_to_body_with_epilogue`. Unlike the normal
    /// `loop_stack`, this tracks ONLY modifier-wrap scopes so a `return`
    /// inside a user loop jumps past the user loop and into the OUTERMOST
    /// modifier wrap's epilogue chain. Innermost wrap sits at the top.
    pub(crate) modifier_break_stack: Vec<usize>,
    /// Number of enclosing try/catch CATCH bodies currently being lowered. A
    /// `return` lowered while this is > 0 sits inside an active NeoVM TRY frame
    /// (state = Catch), so it must emit one `ENDTRY` per active frame to pop the
    /// try-stack before the RET — otherwise the reference C# NeoVM faults at RET
    /// ("There is still try in the stack"). Incremented around catch-body
    /// lowering in `try_catch.rs`; the TRY-body and success-handler run with the
    /// frame already popped (the success-path `EndTry`), so they are excluded.
    pub(crate) catch_frame_depth: usize,
    pub(crate) local_index_map: HashMap<String, Vec<usize>>,
    pub(crate) local_types: HashMap<usize, ValueType>,
    pub(crate) scope_stack: Vec<Vec<String>>,
    pub(crate) storage_aliases: HashMap<String, StorageReference>,
    pub(crate) call_data_locals: HashMap<usize, String>,
    pub(crate) local_count: u16,
    /// Lazily-allocated pool of scratch local slots reused by the inline
    /// software uint256 routines (add/sub/mul over 128-bit limbs). The routines
    /// consume their scratch transiently and leave their result on the stack, so
    /// every uint256 arith site in a function can share one pool — avoiding a
    /// per-site allocation that would blow past NeoVM's local-slot limit.
    pub(crate) u256_scratch: Vec<usize>,
    /// Depth-indexed scratch-local pool for the nested-dynamic ABI
    /// encoder/decoder (`emit_abi_dynamic_nested_array_tail` /
    /// `emit_abi_decode_nested_array_tail_runtime`). `abi_nested_scratch[d]`
    /// holds the reusable locals for nesting depth `d`: distinct depths never
    /// alias (an inner `string[][]` element is encoded while the outer array's
    /// locals are live), but every call site at the same depth shares one
    /// block — so a function with many `abi.encode(string[])` calls does not
    /// allocate a fresh batch of slots per site and blow NeoVM's 255 limit.
    pub(crate) abi_nested_scratch: Vec<Vec<usize>>,
    pub(crate) label_counter: usize,
    pub(crate) loop_stack: Vec<LoopLabels>,
    /// State variable indices currently being inlined (constant resolution).
    /// Used to break infinite recursion when a constant's initializer
    /// transitively references itself through a cross-contract alias.
    pub(crate) resolving_constants: Vec<usize>,
    pub(crate) errors: Vec<IrDiagnostic>,
    pub(crate) warnings: Vec<crate::solidity::Diagnostic>,
    /// Task #30: nested `unchecked { }` depth. When > 0, binary-arithmetic
    /// lowerings skip the Solidity-0.8.x checked overflow guard emission.
    /// Counter semantics (vs. a bool) correctly handle nested blocks:
    /// ```text
    /// unchecked { unchecked { a + b; } }  // depth 2 inside; guard still skipped
    /// ```
    pub(crate) unchecked_depth: usize,
    /// Task #186 — function-pointer parameter/local bindings keyed by the
    /// binding name. Populated by `Function::from_metadata_with_warnings` for
    /// parameters whose declared Solidity type starts with `function` (the
    /// only source of function-pointer values currently supported). Consumed
    /// by `try_lower_variable_call` to emit a `CallIndirect` through `CALLA`
    /// instead of the legacy "drop args, push 0" compatibility fallback.
    pub(crate) function_pointer_bindings: HashMap<String, FunctionPointerBinding>,
}

impl<'a> LoweringContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        function_name: &str,
        current_contract_name: &str,
        current_function_selector: [u8; 4],
        is_safe: bool,
        is_externally_callable: bool,
        param_index_map: HashMap<String, usize>,
        param_types: &'a [ValueType],
        state_variables: &'a [StateVariableMetadata],
        state_index_map: &'a HashMap<String, usize>,
        state_types: &'a [ValueType],
        defined_struct_types: &'a [ValueType],
        struct_fixed_array_bounds: &'a HashMap<(String, String), u64>,
        event_index_map: &'a HashMap<String, usize>,
        event_signature_map: &'a HashMap<String, Vec<ManifestType>>,
        event_params_map: &'a HashMap<String, EventSignature>,
        error_signature_map: &'a HashMap<String, ErrorAbiSignature>,
        enum_variant_map: &'a HashMap<String, HashMap<String, u64>>,
        contract_types: &'a HashSet<String>,
        selector_registry: &'a SelectorRegistry,
        function_names: &'a HashSet<String>,
        function_overloads: &'a FunctionOverloadTable,
        function_first_param_types: &'a HashMap<(String, usize), Vec<ValueType>>,
        function_return_types: &'a HashMap<(String, usize), ValueType>,
        using_target_types: &'a [Option<String>],
        using_function_list_targets: &'a HashMap<String, Vec<Option<String>>>,
        using_function_list_scope_targets: &'a [Option<String>],
        function_param_names: &'a HashMap<(String, usize), Vec<String>>,
        void_functions: &'a HashSet<String>,
        externally_callable_functions: &'a HashSet<(String, usize)>,
        super_method_map: &'a HashMap<String, String>,
        library_storage_bodies: &'a HashMap<(String, usize), LibraryStorageBody>,
        storage_pointer_returning_fns: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            function_name: function_name.to_string(),
            current_contract_name: current_contract_name.to_string(),
            current_function_selector,
            is_safe,
            is_externally_callable,
            param_index_map,
            param_types,
            return_slots: Vec::new(),
            return_types: Vec::new(),
            return_type_strings: Vec::new(),
            state_variables,
            state_index_map,
            state_types,
            defined_struct_types,
            struct_fixed_array_bounds,
            event_index_map,
            event_signature_map,
            event_params_map,
            error_signature_map,
            enum_variant_map,
            contract_types,
            selector_registry,
            function_names,
            function_overloads,
            function_first_param_types,
            function_return_types,
            using_target_types,
            using_function_list_targets,
            using_function_list_scope_targets,
            function_param_names,
            void_functions,
            externally_callable_functions,
            super_method_map,
            library_storage_bodies,
            storage_pointer_returning_fns,
            inline_return_stack: Vec::new(),
            modifier_return_redirect: None,
            modifier_break_stack: Vec::new(),
            catch_frame_depth: 0,
            local_index_map: HashMap::new(),
            local_types: HashMap::new(),
            scope_stack: vec![Vec::new()],
            storage_aliases: HashMap::new(),
            call_data_locals: HashMap::new(),
            local_count: 0,
            u256_scratch: Vec::new(),
            abi_nested_scratch: Vec::new(),
            label_counter: 0,
            loop_stack: Vec::new(),
            resolving_constants: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            unchecked_depth: 0,
            function_pointer_bindings: HashMap::new(),
        }
    }

    /// Task #186 — register a function-pointer binding for a parameter or
    /// local. `name` is the Solidity source name; `arg_count` and `has_return`
    /// are derived from the declared `function(...)` type.
    pub(crate) fn register_function_pointer_binding(
        &mut self,
        name: &str,
        arg_count: usize,
        has_return: bool,
    ) {
        self.function_pointer_bindings.insert(
            name.to_string(),
            FunctionPointerBinding {
                arg_count,
                has_return,
            },
        );
    }

    /// Task #186 — look up a function-pointer binding for an identifier.
    pub(crate) fn function_pointer_binding(&self, name: &str) -> Option<&FunctionPointerBinding> {
        self.function_pointer_bindings.get(name)
    }

    /// Returns `true` if currently lowering inside an `unchecked { ... }` block.
    /// Used by `lower_binary_expr` to suppress the Solidity 0.8.x checked
    /// overflow guard emission for Add/Sub/Mul.
    pub(crate) fn in_unchecked_block(&self) -> bool {
        self.unchecked_depth > 0
    }

    /// Increment unchecked depth when entering `unchecked { ... }`.
    pub(crate) fn enter_unchecked_block(&mut self) {
        self.unchecked_depth = self.unchecked_depth.saturating_add(1);
    }

    /// Decrement unchecked depth when leaving an `unchecked { ... }` block.
    pub(crate) fn exit_unchecked_block(&mut self) {
        self.unchecked_depth = self.unchecked_depth.saturating_sub(1);
    }

    pub(crate) fn current_function_selector(&self) -> [u8; 4] {
        self.current_function_selector
    }

    /// Returns the name of the contract that owns this lowering context.
    ///
    /// Used to resolve `this.method.selector` expressions against the current
    /// contract's method registry when the inner expression is
    /// `Expression::Variable("this")`.
    pub(crate) fn current_contract_name(&self) -> &str {
        &self.current_contract_name
    }

    pub(crate) fn set_return_info(&mut self, slots: Vec<Option<usize>>, types: Vec<ValueType>) {
        self.return_slots = slots;
        self.return_types = types;
    }

    /// Task #185 — record the original Solidity type strings for each
    /// declared return parameter. Called once per function during
    /// `Function::from_metadata_with_warnings`. The strings retain static
    /// fixed-size-array dimensions (e.g. `uint[3][2]`) that `ValueType`
    /// doesn't preserve, enabling `lower_return_statement` to pick the
    /// flat EVM-canonical encoding for nested fixed-size array returns.
    pub(crate) fn set_return_type_strings(&mut self, type_strings: Vec<String>) {
        self.return_type_strings = type_strings;
    }

    pub(crate) fn return_type_strings(&self) -> &[String] {
        &self.return_type_strings
    }

    /// Task #91 — temporarily hide caller parameters that collide with an
    /// inlined library parameter name (`setX(uint256 v) { d.store(v); }` would
    /// otherwise resolve `v` inside the body to the caller's
    /// `LoadParameter(0)`). Returns the previous entry for later restoration.
    pub(crate) fn hide_param_binding(&mut self, name: &str) -> Option<usize> {
        self.param_index_map.remove(name)
    }

    pub(crate) fn restore_param_binding(&mut self, name: String, index: Option<usize>) {
        if let Some(idx) = index {
            self.param_index_map.insert(name, idx);
        }
    }

    /// Task #91 — push/pop an inline-return redirect; see
    /// `inline_return_stack` docs and `lower_return_statement`.
    pub(crate) fn push_inline_return(&mut self, slot: Option<usize>, end_label: usize) {
        self.inline_return_stack.push((slot, end_label));
    }

    pub(crate) fn pop_inline_return(&mut self) {
        self.inline_return_stack.pop();
    }

    /// Return the current inline-return target, if any.
    pub(crate) fn inline_return_target(&self) -> Option<(Option<usize>, usize)> {
        self.inline_return_stack.last().copied()
    }

    /// Task #114 — activate the modifier-epilogue return redirect. `slots`
    /// is a per-return-parameter `LoadLocal` index (one per declared return),
    /// and `end_label` is emitted after the expanded body. Inside
    /// `lower_return_statement`, a Return expression stores into the slots
    /// (in declaration order for multi-return) and jumps to `end_label`,
    /// bypassing the raw RET that would otherwise skip the modifier
    /// epilogue (e.g. `locked = 0;` after `_;`).
    pub(crate) fn set_modifier_return_redirect(
        &mut self,
        slots: Vec<Option<usize>>,
        end_label: usize,
    ) {
        self.modifier_return_redirect = Some((slots, end_label));
    }

    pub(crate) fn clear_modifier_return_redirect(&mut self) {
        self.modifier_return_redirect = None;
    }

    /// Return the current modifier-return redirect (slots, end_label) if set.
    /// The slots Vec mirrors `return_slots` (one entry per declared return).
    pub(crate) fn modifier_return_target(&self) -> Option<(Vec<Option<usize>>, usize)> {
        self.modifier_return_redirect.clone()
    }

    /// Task #114 — push a modifier-wrap break label (from the synthetic
    /// `do { body } while(false)` emitted by the Solidity expander). Used by
    /// `lower_do_while_statement` when `had_modifier_epilogue` is active AND
    /// the condition is the constant `false` we inject — see
    /// `src/solidity/analyse/modifiers/expand.rs`.
    pub(crate) fn push_modifier_break_label(&mut self, label: usize) {
        self.modifier_break_stack.push(label);
    }

    pub(crate) fn pop_modifier_break_label(&mut self) {
        self.modifier_break_stack.pop();
    }

    /// Return the innermost modifier-wrap break label, or `None` if we are
    /// not currently inside a modifier-epilogue scope. Used by
    /// `lower_return_statement` to pick the correct jump target when
    /// redirecting `return expr;` past user loops.
    pub(crate) fn innermost_modifier_break_label(&self) -> Option<usize> {
        self.modifier_break_stack.last().copied()
    }

    /// Enter a try/catch CATCH body — see [`Self::catch_frame_depth`].
    pub(crate) fn enter_catch_frame(&mut self) {
        self.catch_frame_depth += 1;
    }

    /// Leave a try/catch CATCH body.
    pub(crate) fn exit_catch_frame(&mut self) {
        self.catch_frame_depth = self.catch_frame_depth.saturating_sub(1);
    }

    /// Number of active TRY frames a `return` must pop (it is lexically inside
    /// this many enclosing catch bodies).
    pub(crate) fn active_catch_frames(&self) -> usize {
        self.catch_frame_depth
    }

    /// Returns `true` iff this function was flagged with
    /// `had_modifier_epilogue` at the Solidity analyse layer. Mirrors the
    /// `modifier_return_redirect` being Some — that redirect is only set by
    /// `function.rs` when the flag was true.
    pub(crate) fn in_modifier_epilogue_scope(&self) -> bool {
        self.modifier_return_redirect.is_some()
    }

    pub(crate) fn return_slots(&self) -> &[Option<usize>] {
        &self.return_slots
    }

    pub(crate) fn return_types(&self) -> &[ValueType] {
        &self.return_types
    }

    pub(crate) fn is_externally_callable(&self) -> bool {
        self.is_externally_callable
    }

    /// Bug-hunt #9 — true when `name`/`arity` names a same-contract function
    /// that is externally callable (Public/External) and therefore ABI-encodes
    /// a single dynamic return value in its body.
    pub(crate) fn is_externally_callable_fn(&self, name: &str, arity: usize) -> bool {
        self.externally_callable_functions
            .iter()
            .any(|(n, a)| n == name && *a == arity)
    }

    pub(crate) fn record_error(&mut self, message: impl Into<String>) {
        self.errors.push(IrDiagnostic {
            function_name: self.function_name.clone(),
            message: message.into(),
            suggestion: None,
            code: None,
        });
    }

    pub(crate) fn record_error_with_suggestion(
        &mut self,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.errors.push(IrDiagnostic {
            function_name: self.function_name.clone(),
            message: message.into(),
            suggestion: Some(suggestion.into()),
            code: None,
        });
    }

    pub(crate) fn record_warning(&mut self, message: impl Into<String>) {
        self.warnings
            .push(crate::solidity::Diagnostic::warning(message));
    }

    pub(crate) fn record_warning_with_suggestion(
        &mut self,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.warnings
            .push(crate::solidity::Diagnostic::warning(message).with_suggestion(suggestion));
    }

    pub(crate) fn set_call_data_local(&mut self, local_index: usize, method: String) {
        self.call_data_locals.insert(local_index, method);
    }

    pub(crate) fn clear_call_data_local(&mut self, local_index: usize) {
        self.call_data_locals.remove(&local_index);
    }

    pub(crate) fn call_data_method_for_local(&self, local_index: usize) -> Option<&str> {
        self.call_data_locals
            .get(&local_index)
            .map(|method| method.as_str())
    }

    pub(crate) fn is_contract_type_name(&self, name: &str) -> bool {
        self.contract_types.contains(name)
    }

    /// Returns `true` if the given state variable index is currently being
    /// resolved (constant inlining). Used to break infinite recursion when a
    /// constant's initializer transitively references itself.
    pub(crate) fn is_resolving_constant(&self, index: usize) -> bool {
        self.resolving_constants.contains(&index)
    }

    pub(crate) fn push_resolving_constant(&mut self, index: usize) {
        self.resolving_constants.push(index);
    }

    pub(crate) fn pop_resolving_constant(&mut self) {
        self.resolving_constants.pop();
    }

    pub(crate) fn type_method_selectors(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&Vec<[u8; 4]>> {
        self.selector_registry
            .type_method_selectors
            .get(type_name)
            .and_then(|methods| methods.get(method_name))
    }

    pub(crate) fn is_interface_type_name(&self, name: &str) -> bool {
        self.selector_registry.interface_types.contains(name)
    }

    pub(crate) fn interface_id_for_type(&self, type_name: &str) -> Option<[u8; 4]> {
        let methods = self
            .selector_registry
            .type_method_selectors
            .get(type_name)?;
        let mut selectors: HashSet<[u8; 4]> = HashSet::new();
        for overloads in methods.values() {
            for selector in overloads {
                selectors.insert(*selector);
            }
        }

        let mut interface_id = [0u8; 4];
        for selector in selectors {
            for (idx, byte) in selector.iter().enumerate() {
                interface_id[idx] ^= byte;
            }
        }
        Some(interface_id)
    }

    pub(crate) fn local_type(&self, index: usize) -> Option<&ValueType> {
        self.local_types.get(&index)
    }
}

impl<'a> DiagnosticContext for LoweringContext<'a> {
    // The trait exposes only `record_error` (its sole consumer,
    // member_nativecalls, is generic over `DiagnosticContext` and reports
    // errors). Direct callers use the richer inherent methods on
    // `LoweringContext` (record_warning_with_suggestion, etc.), which win
    // method resolution over any trait method anyway.
    fn record_error(&mut self, message: impl Into<String>) {
        self.errors.push(IrDiagnostic {
            function_name: self.function_name.clone(),
            message: message.into(),
            suggestion: None,
            code: None,
        });
    }
}
