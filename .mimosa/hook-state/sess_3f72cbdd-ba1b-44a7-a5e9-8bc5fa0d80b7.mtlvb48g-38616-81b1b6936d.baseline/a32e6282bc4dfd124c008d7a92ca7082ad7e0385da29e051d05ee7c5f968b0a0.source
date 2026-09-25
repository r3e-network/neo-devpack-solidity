use super::*;

impl NeoRuntime {
    /// Override block height for the next execution.
    pub fn override_block_height(&mut self, height: u64) {
        self.execution_context.override_block_height(height);
    }

    /// Override timestamp for the next execution.
    pub fn override_timestamp(&mut self, timestamp: u64) {
        self.execution_context.override_timestamp(timestamp);
    }

    /// Override caller script hash for the next execution.
    pub fn override_caller_account(&mut self, account: &str) -> Result<(), RuntimeError> {
        self.execution_context.override_caller_account(account)
    }

    /// Task #113 — Override Solidity `msg.value` for the next execution.
    ///
    /// Neo N3 has no native attached-value concept, so compiled
    /// `MsgValue` reads default to 0. This override lets harnesses drive
    /// `msg.value` to a specific u64 for payable-fallback / NEP-17
    /// `onPayment` test scenarios. Drained after the next execution
    /// (mirrors `override_timestamp` / `override_block_height`).
    pub fn override_value(&mut self, value: u64) {
        self.execution_context.override_value(value);
    }

    /// Execute bytecode with optional metadata overrides applied to this invocation.
    pub fn execute_with_overrides(
        &mut self,
        bytecode: &[u8],
        input: &[u8],
        overrides: &ExecutionOverrides,
    ) -> Result<ExecutionResult, RuntimeError> {
        if let Some(height) = overrides.block_height {
            self.override_block_height(height);
        }

        if let Some(timestamp) = overrides.timestamp {
            self.override_timestamp(timestamp);
        }

        if let Some(account) = overrides.caller_account.as_deref() {
            if let Err(err) = self.override_caller_account(account) {
                self.execution_context.clear_pending_overrides();
                return Err(err);
            }
        }

        // Task #113 — thread msg.value through. No validation needed (any
        // u64 is a legal injected value); unlike `caller_account` this
        // slot can't fail construction, so we simply set it after the
        // potentially-failing account override above.
        if let Some(value) = overrides.value {
            self.override_value(value);
        }

        self.execute(bytecode, input)
    }

    /// Create new runtime instance
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        Ok(Self {
            execution_context: execution::ExecutionContext::new(&config)?,
            state_manager: state::StateManager::new(&config)?,
            storage_manager: storage::StorageManager::new(&config)?,
            vm_bridge: bridge::VMBridge::new(&config)?,
            gas_tracker: execution::GasTracker::new(config.gas_limit),
            deploy_triggered: false,
        })
    }

    fn capture_metadata(&self) -> ExecutionMetadata {
        let block_height = self.execution_context.block_height();
        let timestamp = self.execution_context.timestamp();
        let caller_account = self.execution_context.caller_account().map(|bytes| {
            if bytes.len() == 20 {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(bytes);
                crate::neo::format_uint160_hex_be(&arr)
            } else {
                format!("0x{}", hex::encode(bytes))
            }
        });

        ExecutionMetadata {
            block_height,
            timestamp,
            caller_account,
        }
    }

    /// Execute bytecode with given input
    pub fn execute(
        &mut self,
        bytecode: &[u8],
        input: &[u8],
    ) -> Result<ExecutionResult, RuntimeError> {
        // Initialize execution context
        self.execution_context.initialize(bytecode, input)?;

        // Reset gas tracker
        self.gas_tracker.reset(self.execution_context.gas_limit());

        // Execute bytecode through VM bridge
        let result = self.vm_bridge.execute(
            &mut self.execution_context,
            &mut self.state_manager,
            &mut self.storage_manager,
            &mut self.gas_tracker,
        )?;

        let mut result = result;
        result.metadata = self.capture_metadata();
        Ok(result)
    }

    /// Execute bytecode with a NEF method token table.
    ///
    /// This is required when the script contains the `CALLT` opcode (0x37).
    pub fn execute_with_tokens(
        &mut self,
        bytecode: &[u8],
        input: &[u8],
        tokens: &[crate::neo::MethodToken],
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execution_context
            .initialize_with_tokens(bytecode, input, tokens)?;

        self.gas_tracker.reset(self.execution_context.gas_limit());

        let result = self.vm_bridge.execute(
            &mut self.execution_context,
            &mut self.state_manager,
            &mut self.storage_manager,
            &mut self.gas_tracker,
        )?;

        let mut result = result;
        result.metadata = self.capture_metadata();
        Ok(result)
    }

    /// Call specific function with arguments
    pub fn call_function(
        &mut self,
        bytecode: &[u8],
        function_name: &str,
        args: &[types::StackItem],
    ) -> Result<ExecutionResult, RuntimeError> {
        // Prepare function call
        let call_data = self.prepare_function_call(function_name, args)?;

        // Execute with call data
        self.execute(bytecode, &call_data)
    }

    /// Invoke a named method on a compiled Neo DevPack for Solidity contract.
    ///
    /// Unlike `call_function`, which cannot reach anything past the first
    /// public method of a multi-method contract, this helper (Task #19):
    ///
    /// 1. Runs `_deploy(null, false)` once per `NeoRuntime` instance so that
    ///    state-variable initializers populate storage before any user call
    ///    observes the contract (Task #47: state-var initializers did not
    ///    run under the `execute`/`call_function` paths — this subsumes it).
    /// 2. Looks up the target method's offset from
    ///    `manifest.abi.methods[].offset` (emitted by the compiler at
    ///    `src/cli/bytecode/bytecode_core.rs:120`) and starts execution at
    ///    that offset rather than always at `bytecode[0]`.
    /// 3. Pushes `args` in reverse onto the NeoVM evaluation stack so that
    ///    the `INITSLOT locals, args` prologue emitted by each compiled
    ///    method (opcode 0x57, handler at `instruction/slots.rs:4`) pops
    ///    them into the method's argument slots in declaration order.
    ///
    /// The persisted storage lives in `storage_manager`; `bridge::execute`
    /// flushes the `storage_overlay` via `storage.commit(..)` on halt, so
    /// subsequent `call_method` invocations on the same `NeoRuntime`
    /// observe deploy-time writes.
    ///
    /// `tokens` is the NEF method-token table carried alongside the
    /// bytecode (see `CompilationArtifacts::tokens`); pass an empty slice
    /// for scripts that do not use `CALLT`.
    pub fn call_method(
        &mut self,
        bytecode: &[u8],
        tokens: &[crate::neo::MethodToken],
        manifest: &serde_json::Value,
        method_name: &str,
        args: &[types::StackItem],
    ) -> Result<ExecutionResult, RuntimeError> {
        // Backwards-compat path (Task #19 shape): no ctor args → `data=Null`.
        // For non-parameterised constructors the `_deploy` prologue never
        // touches `data`, so `Null` is fine. Parameterised constructors that
        // need ctor args must use `call_method_with_deploy_args` (Task #81).
        self.call_method_with_deploy_args(bytecode, tokens, manifest, method_name, args, None)
    }

    /// Task #81: Invoke a named method like `call_method`, but pass
    /// `deploy_args` through the auto-fired `_deploy(data, update)` trigger
    /// as the `data` parameter.
    ///
    /// `deploy_args` wires constructor arguments for contracts whose
    /// Solidity `constructor(...)` takes parameters. When
    /// `deploy_args = Some(&[..])`, the slice is wrapped in a
    /// `StackItem::Array` and passed as `data`; the compiled `_deploy`
    /// prologue then decodes it (`jsonDeserialize`/`deserialize` fallbacks
    /// throw on Array inputs, so the original Array flows through unchanged
    /// to `PICKITEM` / `ArrayGet` for constructor-argument extraction, see
    /// `src/ir/ir_deploy.rs:112`).
    ///
    /// When `deploy_args = None`, behavior matches pre-Task #81
    /// `call_method`: `data = Null`, suitable for no-arg constructors.
    ///
    /// `update` is always `Boolean(false)` here (this is a first-deployment
    /// trigger, not a contract update).
    pub fn call_method_with_deploy_args(
        &mut self,
        bytecode: &[u8],
        tokens: &[crate::neo::MethodToken],
        manifest: &serde_json::Value,
        method_name: &str,
        args: &[types::StackItem],
        deploy_args: Option<&[types::StackItem]>,
    ) -> Result<ExecutionResult, RuntimeError> {
        // Resolve method offsets from the manifest. Real Neo N3 nodes key
        // method dispatch on `(name, parameter count)` — distinct-arity
        // overloads legally share one manifest name (e.g. native
        // ContractManagement `deploy` with 2 and 3 parameters) — so prefer
        // the entry whose declared arity matches `args.len()` before falling
        // back to the first name match.
        let target_offset = find_manifest_method(manifest, method_name, args.len())
            .and_then(|m| m.get("offset"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| RuntimeError::ExecutionError {
                message: format!(
                    "call_method: manifest.abi.methods has no entry named '{method_name}'"
                ),
            })?;

        // Task #70: Build the self-method dispatch table from the manifest so
        // that `handle_contract_call` can route `this.someFn()` self external
        // calls to compiled method offsets.
        let self_method_table = build_self_method_table(manifest);
        // S6 follow-up — install the manifest's call-permissions so the runtime
        // gates non-self `System.Contract.Call` targets the way Neo N3 does
        // on-chain. Set once here (it persists across the deploy-prologue and
        // user-method `initialize` calls, neither of which resets it), so the
        // deploy constructor is covered too.
        self.execution_context.set_manifest_permissions(manifest);

        // Task #176 — re-arm `pending_caller_account` from the sticky slot
        // so that a single `override_caller_account` call (made BEFORE the
        // first `call_method`) persists across every subsequent
        // `call_method` invocation on this `NeoRuntime`. The `initialize`
        // call inside `invoke_at_offset_with_input` drains
        // `pending_caller_account` into `caller_account`, so without this
        // re-arm the second `call_method` would observe `pending = None`
        // and `initialize` would wipe `caller_account` to `None`, which the
        // `GetCallingScriptHash` syscall handler then falls back to
        // `default_account_bytes`. That breaks mapping-by-sender invariants
        // in multi-call vault-style harnesses (deposit then withdraw, where
        // `balances[msg.sender] += amt` on deposit writes the slot keyed by
        // alice and the subsequent `balances[msg.sender] >= amt` require on
        // withdraw reads the slot keyed by default — which is 0, so the
        // require revert-msg literal "insufficient" bubbles up even though
        // alice's deposit just persisted). Only a later
        // `override_caller_account` (which updates the sticky slot) or a
        // `clear_pending_overrides` (which nulls it) changes this. The
        // non-`call_method` path (`execute_with_overrides`) does not
        // consult the sticky slot, preserving the per-invocation reset
        // semantic pinned by `tests::test_execute_with_overrides_apply`.
        if self.execution_context.pending_caller_account.is_none() {
            if let Some(sticky) = self.execution_context.sticky_caller_account.clone() {
                self.execution_context.pending_caller_account = Some(sticky);
            }
        }

        // Trigger `_deploy(data, false)` once per runtime instance so that
        // state-variable initializers run before the user's method sees
        // storage. This mirrors Neo-N3 deploy-time semantics. The `data`
        // parameter becomes an Array of constructor args when the caller
        // supplied `deploy_args`, matching how Neo-Express / N3 SDKs wrap
        // ctor inputs.
        if !self.deploy_triggered {
            if let Some(deploy_offset) = lookup_manifest_method_offset(manifest, "_deploy") {
                // Task #105 — snapshot any pending metadata overrides BEFORE
                // the `_deploy` prologue consumes them. The override API
                // (`override_timestamp` / `override_block_height` /
                // `override_caller_account`) is advertised as "for the next
                // execution", where "execution" from the caller's viewpoint
                // is the user-method invocation. Letting `_deploy` drain the
                // override makes every parameterised-constructor test's
                // first user call fall back to `default_timestamp`, which
                // breaks timestamp-sensitive harnesses (batch43 S4 filed
                // this as Task #105). We save the three override slots here,
                // run `_deploy`, then restore them so the subsequent
                // `invoke_at_offset` call sees exactly the override the
                // caller asked for.
                let saved_pending_timestamp = self.execution_context.pending_timestamp;
                let saved_pending_block_height = self.execution_context.pending_block_height;
                let saved_pending_caller_account =
                    self.execution_context.pending_caller_account.clone();
                // Task #113 (Strategy A revision) — `pending_msg_value` is
                // intentionally NOT snapshotted/restored across `_deploy`.
                // A `payable constructor` must observe `msg.value` set via
                // `override_value(N)`, so the value is allowed to drain into
                // the deploy-prologue invocation and the constructor reads
                // it through the normal `MsgValue` syscall path. The
                // remaining override slots (timestamp / block_height /
                // caller_account) keep their snapshot/restore semantics
                // because state-variable initializers and `_deploy` itself
                // do not read those, and the caller's intent for those
                // overrides is the user-facing method invocation.
                //
                // Side-effect: a user-facing method invoked in the same
                // `call_method` call that triggered `_deploy` will observe
                // `msg.value == 0` unless the caller calls `override_value`
                // again before subsequent `call_method` invocations. This
                // matches Neo N3 semantics where the deploy trigger and
                // user invocations are separate transactions.

                // Reverse-push convention (see `invoke_at_offset`): args[0]
                // is popped first off the VM stack and becomes arg0 in the
                // callee's frame. `_deploy(data, update)` has `data` at
                // slot 0 and `update` at slot 1 (see
                // `src/ir/ir_deploy.rs:103,115`), so the slice here is
                // `[data, update]`.
                let data_item = match deploy_args {
                    Some(inputs) => types::StackItem::array(inputs.to_vec()),
                    None => types::StackItem::Null,
                };
                let deploy_call = [data_item, types::StackItem::Boolean(false)];
                let deploy_result = self.invoke_at_offset(
                    bytecode,
                    tokens,
                    deploy_offset,
                    &deploy_call,
                    self_method_table.clone(),
                )?;
                if !deploy_result.success {
                    return Ok(deploy_result);
                }

                // Restore the snapshotted pending overrides so the user
                // method sees them. `initialize_with_tokens` inside
                // `invoke_at_offset` called `initialize`, which `take()`s
                // each `pending_*` slot — so the context fields are now
                // `None`. Putting the snapshot back re-arms the override
                // for exactly the next `invoke_at_offset` call below.
                self.execution_context.pending_timestamp = saved_pending_timestamp;
                self.execution_context.pending_block_height = saved_pending_block_height;
                self.execution_context.pending_caller_account = saved_pending_caller_account;
                // `pending_msg_value` deliberately not restored — see the
                // snapshot-site comment above (Strategy A).
            }
            self.deploy_triggered = true;
        }

        // Task #112 — synthesise EVM-style `msg.data` for the invoked method
        // so Solidity source that reads `msg.data`, `msg.data.length`, or
        // `msg.sig` observes a coherent `selector || abi.encode(args)` payload
        // during the user-method execution. On Neo N3 the script container's
        // `Script` slot (which `RuntimeValue::MsgData` reads) would otherwise
        // reflect the raw NEF bytecode — exactly what broke
        // `batch47_w4_msg_data_length_and_selector_via_call_method`.
        //
        // The signature used for the selector is derived from
        // `manifest.abi.methods[<name>].parameters[].type` (Neo manifest
        // types mapped back to Solidity canonical names). Where a method
        // takes no parameters the payload collapses to the 4-byte selector
        // alone, matching `batch26` H3 which pins this exact shape via
        // `execute_with_overrides`.
        let msg_data = build_user_method_msg_data(manifest, method_name, args);
        self.invoke_at_offset_with_input(
            bytecode,
            tokens,
            target_offset,
            args,
            self_method_table,
            &msg_data,
        )
    }

    /// Initialize the VM at a specific bytecode offset with `args` pre-pushed
    /// onto the evaluation stack, then run until halt. Shared between
    /// `call_method` and its deploy-prologue sub-invocation.
    ///
    /// NeoVM INITSLOT pops arg_count items in order — the first pop becomes
    /// `arg0`. Therefore the caller pushes arguments in reverse so that the
    /// first argument ends up on top of the stack.
    ///
    /// The `_deploy` prologue runs with an empty `input_data` — it never
    /// observes `msg.data` and the Solidity spec leaves constructor `msg.data`
    /// unspecified anyway. User-method invocations call
    /// `invoke_at_offset_with_input` to inject a synthesised
    /// `selector || abi.encode(args)` payload (Task #112).
    fn invoke_at_offset(
        &mut self,
        bytecode: &[u8],
        tokens: &[crate::neo::MethodToken],
        offset: u32,
        args: &[types::StackItem],
        self_method_table: Vec<(String, u32, u16)>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.invoke_at_offset_with_input(bytecode, tokens, offset, args, self_method_table, &[])
    }

    /// Like `invoke_at_offset` but with caller-supplied `input_data` (the
    /// bytes observed by Solidity as `msg.data` for the duration of the
    /// invocation). Callers targeting a user method should pass
    /// `selector(method_name) || abi.encode(args)`; the `_deploy` prologue
    /// passes an empty slice.
    fn invoke_at_offset_with_input(
        &mut self,
        bytecode: &[u8],
        tokens: &[crate::neo::MethodToken],
        offset: u32,
        args: &[types::StackItem],
        self_method_table: Vec<(String, u32, u16)>,
        input_data: &[u8],
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execution_context
            .initialize_with_tokens(bytecode, input_data, tokens)?;
        self.execution_context
            .set_self_method_table(self_method_table);
        self.execution_context.instruction_pointer = offset;
        for arg in args.iter().rev() {
            self.execution_context.push_stack(arg.clone())?;
        }

        self.gas_tracker.reset(self.execution_context.gas_limit());

        let result = self.vm_bridge.execute(
            &mut self.execution_context,
            &mut self.state_manager,
            &mut self.storage_manager,
            &mut self.gas_tracker,
        )?;

        let mut result = result;
        result.metadata = self.capture_metadata();
        Ok(result)
    }
}

/// Resolve `manifest.abi.methods[<name>].offset` as a `u32`.
fn lookup_manifest_method_offset(manifest: &serde_json::Value, name: &str) -> Option<u32> {
    manifest
        .get("abi")?
        .get("methods")?
        .as_array()?
        .iter()
        .find(|m| m.get("name").and_then(serde_json::Value::as_str) == Some(name))?
        .get("offset")?
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
}

/// Locate a `manifest.abi.methods[]` entry by `(name, parameter count)`,
/// mirroring real Neo N3 method resolution (ContractAbi.GetMethod). Falls
/// back to the first name-only match so legacy manifests with unique names
/// keep resolving even when the caller's arg count differs from the
/// declaration.
fn find_manifest_method<'a>(
    manifest: &'a serde_json::Value,
    name: &str,
    arg_count: usize,
) -> Option<&'a serde_json::Value> {
    let methods = manifest.get("abi")?.get("methods")?.as_array()?;
    methods
        .iter()
        .find(|m| {
            m.get("name").and_then(serde_json::Value::as_str) == Some(name)
                && m.get("parameters")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|params| params.len() == arg_count)
        })
        .or_else(|| {
            methods
                .iter()
                .find(|m| m.get("name").and_then(serde_json::Value::as_str) == Some(name))
        })
}

/// Task #70: Build a `(method_name, offset, arg_count)` dispatch table from
/// `manifest.abi.methods[]`, used by `handle_contract_call` to route
/// `this.someFn()` self external calls.
fn build_self_method_table(manifest: &serde_json::Value) -> Vec<(String, u32, u16)> {
    let Some(methods) = manifest
        .get("abi")
        .and_then(|a| a.get("methods"))
        .and_then(|m| m.as_array())
    else {
        return Vec::new();
    };
    methods
        .iter()
        .filter_map(|m| {
            let name = m
                .get("name")
                .and_then(serde_json::Value::as_str)?
                .to_string();
            let offset = u32::try_from(m.get("offset")?.as_u64()?).ok()?;
            let arg_count = m
                .get("parameters")
                .and_then(serde_json::Value::as_array)
                .map(|p| u16::try_from(p.len()).unwrap_or(u16::MAX))
                .unwrap_or(0);
            Some((name, offset, arg_count))
        })
        .collect()
}

/// Task #112 — synthesise the raw calldata that a Solidity user method
/// should observe as `msg.data` during a `call_method` invocation.
///
/// EVM shape: `selector(4) || abi.encode(args)`, where `selector =
/// keccak256("name(type1,type2,...)")[0..4]`. Each parameter type is
/// recovered from `manifest.abi.methods[<name>].parameters[].type`
/// (Neo manifest names) and mapped back to the Solidity canonical form
/// Solidity itself would hash (`Integer` → `uint256`, `Hash160` → `address`,
/// `Hash256` → `bytes32`, `ByteArray` → `bytes`, `Boolean` → `bool`,
/// `String` → `string`, `Array` → `bytes[]` as a best-effort fallback).
///
/// For zero-arg methods the payload collapses to the 4-byte selector alone
/// — `batch26` H3 and `batch47` W4 both pin that canonical shape.
/// When argument bodies are non-trivial the encoding delegates to the same
/// `abi_pad32_be` / `abi_dynamic_tail_bytes` helpers used by the runtime
/// `abiencode` handler (EVM spec head+tail layout for dynamic arguments).
fn build_user_method_msg_data(
    manifest: &serde_json::Value,
    method_name: &str,
    args: &[types::StackItem],
) -> Vec<u8> {
    use sha3::{Digest, Keccak256};

    // Locate the manifest method entry to recover the canonical parameter
    // type list. If the method isn't in the manifest (shouldn't happen since
    // `call_method_with_deploy_args` already resolved an offset above but
    // defensive-fallback anyway), we still compute a `name()` selector.
    let method_entry = find_manifest_method(manifest, method_name, args.len());

    let param_types: Vec<String> = method_entry
        .and_then(|m| m.get("parameters"))
        .and_then(|p| p.as_array())
        .map(|params| {
            params
                .iter()
                .map(|p| {
                    p.get("type")
                        .and_then(serde_json::Value::as_str)
                        .map(manifest_type_to_canonical_solidity)
                        .unwrap_or_else(|| "bytes".to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    let signature = format!("{}({})", method_name, param_types.join(","));
    let hash = Keccak256::digest(signature.as_bytes());
    let mut out = Vec::with_capacity(4 + args.len() * 32);
    out.extend_from_slice(&hash[..4]);

    // Empty-args fast path: matches `batch26_h3` / `batch47_w4` (selector
    // only, `msg.data.length == 4`).
    if args.is_empty() {
        return out;
    }

    // Non-empty args: delegate to the same EVM head+tail encoder used by
    // the runtime `abiencode` handler. We reconstruct the static/dynamic
    // classification inline — `ExecutionContext::{abi_is_dynamic,
    // abi_pad32_be, abi_dynamic_tail_bytes}` are private (`fn` not `pub`),
    // so we replicate their short bodies here.
    let is_dynamic: Vec<bool> = args
        .iter()
        .map(stack_item_is_dynamic_for_selector)
        .collect();
    let any_dynamic = is_dynamic.iter().any(|b| *b);

    if !any_dynamic {
        for item in args.iter() {
            out.extend_from_slice(&stack_item_pad32_be(item));
        }
        return out;
    }

    let head_len = args.len() * 32;
    let mut tails: Vec<Vec<u8>> = Vec::with_capacity(args.len());
    let mut running_offset: u64 = head_len as u64;
    let mut head: Vec<u8> = Vec::with_capacity(head_len);
    for (i, item) in args.iter().enumerate() {
        if is_dynamic[i] {
            let tail = stack_item_dynamic_tail_bytes(item);
            let mut offset_slot = [0u8; 32];
            offset_slot[24..].copy_from_slice(&running_offset.to_be_bytes());
            head.extend_from_slice(&offset_slot);
            running_offset += tail.len() as u64;
            tails.push(tail);
        } else {
            head.extend_from_slice(&stack_item_pad32_be(item));
        }
    }
    out.extend_from_slice(&head);
    for tail in &tails {
        out.extend_from_slice(tail);
    }
    out
}

/// Map a Neo manifest parameter `"type"` back to the Solidity canonical name
/// that Solidity itself would hash into a function selector.
///
/// The frontend (`src/cli/cli_parts/cli_manifest/build.rs`) emits
/// `"Integer"`/`"Boolean"`/`"String"`/`"Hash160"`/`"Hash256"`/`"ByteArray"`
/// /`"Array"`/`"Map"`/`"Any"`; the inverse mapping here collapses uintN /
/// intN / uint16 etc. to `uint256` because that information is lost on the
/// manifest side. This matches how Solidity hashes selectors (it always
/// uses the 256-bit canonical form for int/uint arguments regardless of
/// the declared width in the source).
fn manifest_type_to_canonical_solidity(manifest_type: &str) -> String {
    match manifest_type {
        "Integer" => "uint256".to_string(),
        "Boolean" => "bool".to_string(),
        "String" => "string".to_string(),
        "Hash160" => "address".to_string(),
        "Hash256" => "bytes32".to_string(),
        "ByteArray" => "bytes".to_string(),
        "Array" => "bytes[]".to_string(),
        "Map" => "bytes".to_string(),
        "Any" | "Void" => "bytes".to_string(),
        _ => "bytes".to_string(),
    }
}

/// Content-based dynamic/static classification for a [`types::StackItem`].
/// Mirrors `ExecutionContext::abi_is_dynamic` in
/// `src/runtime/execution/execution_impl_part2_native/stdlib.rs`.
fn stack_item_is_dynamic_for_selector(item: &types::StackItem) -> bool {
    match item {
        types::StackItem::Integer(_)
        | types::StackItem::UnsignedInteger(_)
        | types::StackItem::Boolean(_)
        | types::StackItem::Null => false,
        types::StackItem::ByteArray { data: bytes, .. } => {
            let len = bytes.borrow().len();
            !matches!(len, 16 | 20 | 32)
        }
        types::StackItem::Array(_) | types::StackItem::Map(_) => true,
    }
}

/// Left-padded 32-byte BE encoding of a [`types::StackItem`]. Mirrors
/// `ExecutionContext::abi_pad32_be` (short-form: address/bytes32/small
/// integer). Cross-reference `src/runtime/execution/execution_impl_part2_native/stdlib.rs`.
fn stack_item_pad32_be(item: &types::StackItem) -> [u8; 32] {
    let mut slot = [0u8; 32];
    match item {
        types::StackItem::Integer(v) => {
            if *v >= 0 {
                slot[24..].copy_from_slice(&(*v as u64).to_be_bytes());
            } else {
                slot.fill(0xff);
                slot[24..].copy_from_slice(&(*v as u64).to_be_bytes());
            }
        }
        types::StackItem::UnsignedInteger(v) => {
            slot[24..].copy_from_slice(&v.to_be_bytes());
        }
        types::StackItem::Boolean(b) => {
            slot[31] = if *b { 1 } else { 0 };
        }
        types::StackItem::ByteArray { data: bytes, .. } => {
            let b = bytes.borrow();
            if b.len() >= 32 {
                if b[20..32].iter().all(|byte| *byte == 0) {
                    let mut prefix = [0u8; 20];
                    prefix.copy_from_slice(&b[..20]);
                    prefix.reverse();
                    slot[12..32].copy_from_slice(&prefix);
                } else {
                    slot.copy_from_slice(&b[..32]);
                }
            } else if b.len() == 20 {
                let mut rev = b.clone();
                rev.reverse();
                slot[12..32].copy_from_slice(&rev);
            } else if b.len() == 16 {
                let mut rev = b.clone();
                rev.reverse();
                let start = 32 - rev.len();
                slot[start..].copy_from_slice(&rev);
            } else {
                let start = 32 - b.len();
                slot[start..].copy_from_slice(&b);
            }
        }
        types::StackItem::Null | types::StackItem::Array(_) | types::StackItem::Map(_) => {}
    }
    slot
}

/// Emit the EVM-canonical tail-section for a dynamic [`types::StackItem`]:
/// 32-byte BE length prefix + raw content padded to the next 32-byte
/// boundary. Mirrors `ExecutionContext::abi_dynamic_tail_bytes`.
fn stack_item_dynamic_tail_bytes(item: &types::StackItem) -> Vec<u8> {
    match item {
        types::StackItem::ByteArray { data: bytes, .. } => {
            let content = bytes.borrow().clone();
            let len = content.len();
            let padded_len = len.div_ceil(32) * 32;
            let mut out = Vec::with_capacity(32 + padded_len);
            let mut len_slot = [0u8; 32];
            len_slot[24..].copy_from_slice(&(len as u64).to_be_bytes());
            out.extend_from_slice(&len_slot);
            out.extend_from_slice(&content);
            out.resize(32 + padded_len, 0);
            out
        }
        types::StackItem::Array(arr) => {
            // Task #121 mirror — selector-side `T[]` encoding: 32-byte BE
            // length prefix followed by N × 32-byte BE-padded element slots.
            // Keeps `build_user_method_msg_data` consistent with the runtime
            // `abiencode` handler for dynamic-array args.
            let elements = arr.borrow().clone();
            let n = elements.len();
            let mut out = Vec::with_capacity(32 + n * 32);
            let mut len_slot = [0u8; 32];
            len_slot[24..].copy_from_slice(&(n as u64).to_be_bytes());
            out.extend_from_slice(&len_slot);
            for el in elements.iter() {
                out.extend_from_slice(&stack_item_pad32_be(el));
            }
            out
        }
        _ => vec![0u8; 32],
    }
}
