//! Stage 3 — Sibling contract merge.
//!
//! When a primary contract references a sibling primary (via `new X()`,
//! `X(addr)` casts, interface casts, typed parameters/returns, state
//! variables, or low-level `.call` payloads), this stage merges the sibling's
//! relevant functions, modifiers, state variables, events, and constructors
//! into the referencing contract so the zero-hash self-dispatch table can
//! route calls at runtime (Task #83 and related tasks).

use super::*;

/// Merge sibling primary contracts into each referencing primary contract.
///
/// Mutates `primary` in place. Returns `Ok(())` on success or a Solidity
/// analysis error when merged state variables have conflicting types.
pub(crate) fn perform_sibling_merge(
    primary: &mut [ContractIR],
    pre_merge_contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Result<(), SolidityError> {
    // Task #83 — when a primary contract `A` runs `B b = new B(); b.foo();`
    // the compiler emits a 20-byte zero placeholder for `b` and lowers
    // `b.foo()` as `System.Contract.Call([0;20], "foo", flags, args)`. B's
    // compiled body is a separate artifact, so without help the call would
    // return `Null` and A's return value would silently go empty. Fix:
    // merge every sibling primary's public/external functions that A
    // references via `new X()` into A's own function table (name-preserving,
    // host-wins-on-collision); the runtime then routes the zero-hash call
    // through `self_method_offsets` — see the Task #83 branch in
    // `execution_impl_part2_contract_call.rs`.
    let sibling_fn_map: std::collections::HashMap<String, Vec<FunctionIR>> = primary
        .iter()
        .map(|c| {
            (
                c.name.clone(),
                c.functions
                    .iter()
                    .filter(|f| {
                        // Include abstract internal function declarations
                        // (body = None) alongside concrete externals.
                        // When sibling-merge pulls in an external body
                        // like `rawFulfillRandomWords` whose body calls
                        // an abstract sibling-internal function (e.g.
                        // VRFConsumerBaseV2's `fulfillRandomWords(uint256,
                        // uint256[])`), the host's IR-lowering pass would
                        // otherwise fail the overload lookup with
                        // "no overload of 'fulfillRandomWords' with 2
                        // argument(s)". Importing the abstract declaration
                        // satisfies the lookup; at runtime the call lands
                        // on an empty stub (RET-only) which is acceptable
                        // for the dead-code paths that typically include
                        // such helpers transitively.
                        let is_abstract_internal =
                            matches!(f.ty, FunctionTy::Function) && f.body.is_none();
                        // Task #126 — include Fallback (and Receive) alongside
                        // ordinary external/public named functions so that a
                        // primary contract whose only entrypoint is
                        // `fallback()` still contributes its dispatcher to
                        // the caller's merged function table when the caller
                        // invokes a method the callee doesn't declare.
                        //
                        // Without this, `try Target(t).nonExistentMethod()`
                        // (where TargetImpl only defines `fallback()`)
                        // would never be able to route through the zero-
                        // placeholder self-offsets path: the fallback entry
                        // simply wouldn't be in the merge set, and the
                        // runtime's unknown-method path would silently
                        // return Null rather than propagating the fallback's
                        // revert back to the caller's catch clause.
                        // Only merge external/public methods that have a
                        // BODY. A bodyless external/public method is an
                        // abstract virtual (an abstract contract's interface
                        // surface). Merging it into a host that merely
                        // *references* the abstract type — e.g. a
                        // `PriceOracle oracle;` / `InterestRateModel`-typed
                        // field (Compound v2 CToken/Comptroller) — has no
                        // dispatch value (there is no body to route to) and
                        // wrongly trips the host's "N unimplemented
                        // function(s) but not declared abstract" check
                        // (validate_abstract_contract). Calls through such a
                        // field are real cross-contract calls, not
                        // self-offsets dispatch. Same reasoning as the
                        // is_abstract_internal exclusion below.
                        let is_named_external = matches!(f.ty, FunctionTy::Function)
                            && matches!(
                                f.visibility,
                                VisibilityKind::External | VisibilityKind::Public
                            )
                            && f.body.is_some();
                        let is_fallback_like =
                            matches!(f.ty, FunctionTy::Fallback | FunctionTy::Receive);
                        // (We deliberately do NOT include
                        // `is_abstract_internal` here. Including abstract
                        // internal declarations would let merged bodies
                        // reference them, but it also fails the "all
                        // abstract methods implemented" validation since
                        // those functions don't have a body in the host.
                        // We handle the missing-overload case at
                        // IR-lowering time by emitting a runtime trap
                        // instead of a compile error.)
                        let _ = is_abstract_internal;
                        // Task #18/#20 — a merged PUBLIC body may CALL_L into
                        // its own contract's CONCRETE internal/private helper
                        // (e.g. `H.pub()` calls `H.helper()` which does
                        // `c += 10`, or `Base.f()` dispatches to an internal
                        // implementation). Those helpers are not entrypoints,
                        // so they were dropped from the merge and lowered to
                        // a PUSH0 placeholder — the SSTORE never ran and the
                        // callee's writes vanished. Merge body-bearing
                        // internals too so the CALL_L target is real. They
                        // still never reach the manifest ABI (export gates on
                        // Public|External) nor `self_method_offsets`; they are
                        // reachable only via the merged public method's
                        // internal CALL_L, matching EVM semantics. Abstract
                        // (bodyless) internals stay excluded (body.is_some()).
                        let is_concrete_internal = matches!(f.ty, FunctionTy::Function)
                            && f.body.is_some()
                            && matches!(
                                f.visibility,
                                VisibilityKind::Internal | VisibilityKind::Private
                            );
                        is_named_external || is_fallback_like || is_concrete_internal
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    // Companion map: every sibling's modifier definitions — INCLUDING
    // modifiers reachable through the sibling's inheritance chain. When
    // sibling functions are merged into a host below, any modifier they
    // apply (`function upgrade(...) public payable virtual onlyOwner`)
    // must still resolve in the host. The host's local `modifier_defs`
    // only sees ITS OWN modifier declarations — so without a parallel
    // merge pass, merged `upgrade`'s `onlyOwner` lookup fails with
    // "unresolved modifier 'onlyOwner' with 0 argument(s)". Repro: OZ
    // TransparentUpgradeableProxy / ProxyAdmin import cycle, where
    // ProxyAdmin's `onlyOwner` lives in its base contract Ownable.
    //
    // Because the sibling's own inheritance flattening hasn't happened
    // yet at this point in the pipeline, we walk each sibling's
    // linearized base chain ourselves and union their modifier
    // definitions per sibling, keyed by (name, arity), preferring
    // bodied modifiers over abstract declarations.
    let sibling_modifier_map: std::collections::HashMap<String, Vec<FunctionIR>> = primary
        .iter()
        .map(|c| {
            let mut seen: std::collections::HashMap<(String, usize), FunctionIR> =
                std::collections::HashMap::new();
            let mut visit = |contract: &ContractIR| {
                for f in &contract.functions {
                    if !matches!(f.ty, FunctionTy::Modifier) {
                        continue;
                    }
                    let key = (f.name.clone(), f.parameters.len());
                    match seen.get(&key) {
                        Some(existing) if existing.body.is_some() => {}
                        _ => {
                            seen.insert(key, f.clone());
                        }
                    }
                }
            };
            visit(c);
            // Try to walk the linearization. If it fails (shouldn't —
            // we already ran it during pre-merge analysis to detect
            // cycles), fall back to direct base inspection.
            if let Ok(chain) =
                contract_linearization_base_to_derived(&c.name, pre_merge_contract_map)
            {
                for ancestor_name in &chain {
                    if ancestor_name == &c.name {
                        continue;
                    }
                    if let Some(ancestor) = pre_merge_contract_map.get(ancestor_name) {
                        visit(ancestor);
                    }
                }
            }
            (c.name.clone(), seen.into_values().collect::<Vec<_>>())
        })
        .collect();
    // Task #197 — parallel state-variable map. When a sibling's external
    // function body references state variables (e.g. Mock.balanceOf
    // reading `_bal[a]`), merging only the FunctionIR leaves the
    // identifier unresolved in the caller's `state_index_map`, so the
    // variable lowering falls through to the `Integer(0)` placeholder
    // path (src/ir/expressions/variable.rs) and downstream opcodes like
    // SIZE/PICKITEM fault on the wrong StackItem type. The storage key
    // is derived from the state variable's name (see
    // `value_types.rs::compute_state_slot`), so merging Mock's `_bal`
    // into Client lets Client's compiled balanceOf read the same
    // storage slot Mock.mint wrote to.
    let sibling_state_map: std::collections::HashMap<String, Vec<StateVariableIR>> = primary
        .iter()
        .map(|c| (c.name.clone(), c.state_variables.clone()))
        .collect();
    // Task #23 — parallel event-declaration map. When a sibling's external
    // body is merged into a host (above), any `emit Foo(...)` in that body
    // resolves `Foo` against the HOST contract's event table during IR
    // lowering (`event_evm_signature`). If the host never declared `Foo`,
    // the emit fails the "no resolved declaration" guard and the whole
    // compile errors — even though the sibling that owns `doSimple`
    // declares `Foo` perfectly well. Mirror the sibling function/state/
    // modifier merges by also carrying each sibling's event declarations so
    // the merged bodies can resolve their own events against the host.
    //
    // The event set must include events the sibling INHERITS, not just its
    // directly-declared ones: e.g. `contract A is IERC20 { function approve()
    // { emit Approval(); } }` where `Approval` lives in `IERC20`. When a host
    // does `new A()` and A's `approve` is merged in, carrying only `A.events`
    // would drop the inherited `Approval` and the merged emit would fail "no
    // resolved declaration". We must walk the FULL base closure — including
    // INTERFACE bases, which `contract_linearization_base_to_derived`
    // deliberately omits (interfaces don't affect storage/ctor order) but
    // which are the usual home of event declarations (IERC20, IPool). BFS
    // every transitive base regardless of kind.
    let sibling_event_map: std::collections::HashMap<String, Vec<EventIR>> = primary
        .iter()
        .map(|c| {
            let mut seen: std::collections::HashMap<String, EventIR> =
                std::collections::HashMap::new();
            let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut queue: Vec<String> = vec![c.name.clone()];
            while let Some(name) = queue.pop() {
                if !visited.insert(name.clone()) {
                    continue;
                }
                if let Some(contract) = pre_merge_contract_map.get(&name) {
                    for e in &contract.events {
                        seen.entry(e.name.clone()).or_insert_with(|| e.clone());
                    }
                    for base in &contract.bases {
                        if let Some(base_name) = base_last_name(base) {
                            queue.push(base_name);
                        }
                    }
                }
            }
            (c.name.clone(), seen.into_values().collect::<Vec<_>>())
        })
        .collect();
    // Task #198 — parallel constructor map. For `new Child(x, y)` inside a
    // Parent contract, the compiled Child lives in a separate artifact, so
    // the Parent's runtime invocation of its own `_deploy` never runs the
    // Child constructor. Without executing the ctor body, Child's state
    // variables (`a`, `b`) stay at their zero defaults — and the follow-up
    // `c.a()` / `c.b()` cross-contract calls (routed through sibling-merge
    // self-offsets; Task #83) therefore observe zeros.
    //
    // Fix: expose each sibling's constructor as a regular, internal,
    // name-mangled function (`__ctor__<SiblingName>`) in the caller's
    // merged function table. The `new Child(x, y)` lowering then calls
    // `__ctor__Child(x, y)` in-line, running the ctor body against the
    // already-merged sibling state-variable slots (Task #197). The address
    // return value stays at the 20-byte zero placeholder that Task #83
    // already routes to self-offsets dispatch.
    let sibling_ctor_map: std::collections::HashMap<String, FunctionIR> = primary
        .iter()
        .filter_map(|c| {
            let ctor = c
                .functions
                .iter()
                .find(|f| matches!(f.ty, FunctionTy::Constructor))?;
            Some((c.name.clone(), ctor.clone()))
        })
        .collect();
    let primary_contract_map: std::collections::HashMap<String, ContractIR> = primary
        .iter()
        .map(|c| (c.name.clone(), c.clone()))
        .collect();
    let primary_names: std::collections::HashSet<String> =
        primary.iter().map(|c| c.name.clone()).collect();

    // Task #126 — a primary contract's `fallback()` acts as a universal
    // catch-all dispatcher: every unknown method name falls through to
    // it. For interface-cast routing `Target(t).someMethod()` where
    // `TargetImpl` has only `fallback()` (no named external methods),
    // we must still treat `TargetImpl` as a valid implementor of the
    // `Target` interface so the sibling-merge pass pulls its fallback
    // body into the caller's function table. This mirrors Solidity's
    // own runtime semantics: the ABI dispatcher routes unknown
    // selectors to `fallback()` when present.
    let primary_has_fallback: std::collections::HashSet<String> = primary
        .iter()
        .filter(|c| {
            c.functions
                .iter()
                .any(|f| matches!(f.ty, FunctionTy::Fallback))
        })
        .map(|c| c.name.clone())
        .collect();

    // Task #115 — collect interface kind names and their external method
    // sets. An expression like `I(t).getR()` in contract `C` (where `I`
    // is an interface declared in the same source unit) is a
    // cross-contract call routed through an `address`-typed receiver.
    // At runtime the 20-byte zero placeholder triggers self-offsets
    // dispatch (see `handle_contract_call` / Task #83 branch), so the
    // callee method must live in the caller's merged function table.
    // We match the interface to any sibling primary whose public/external
    // method set is a superset of the interface's method names, and
    // include those siblings in the sibling merge below.
    //
    // This mirrors the `new B()` / `B(addr)` / `B public b;` patterns
    // already handled — without this hook, interface-typed dispatch
    // silently returns `Null` (the `invoke_native_contract` fallback for
    // unknown-hash calls), which then blows up inside `r.a` /
    // `r.b` member accesses downstream.
    let interface_methods: std::collections::HashMap<String, std::collections::HashSet<String>> =
        pre_merge_contract_map
            .values()
            .filter(|c| matches!(c.kind, ContractKind::Interface))
            .map(|c| (c.name.clone(), public_external_method_names(c)))
            .collect();

    // Reverse map: interface name → list of primary contracts whose
    // method set covers the interface's method set. We pre-compute this
    // once so we don't re-walk the primary function tables per function
    // body.
    let primary_method_names: std::collections::HashMap<String, std::collections::HashSet<String>> =
        primary
            .iter()
            .map(|c| (c.name.clone(), public_external_method_names(c)))
            .collect();

    let interface_impls: std::collections::HashMap<String, Vec<String>> = interface_methods
        .iter()
        .map(|(iface_name, iface_method_set)| {
            let mut impls: Vec<String> = primary_method_names
                .iter()
                .filter_map(|(prim_name, prim_set)| {
                    // Task #126 — a primary with a `fallback()` catches
                    // any interface method that isn't explicitly declared,
                    // so it's always a valid implementor for sibling-
                    // merge purposes (at runtime the call routes through
                    // the merged `fallback` entry, which may itself
                    // revert — that revert is what we propagate to the
                    // caller's try/catch).
                    if iface_method_set.is_subset(prim_set)
                        || primary_has_fallback.contains(prim_name)
                    {
                        Some(prim_name.clone())
                    } else {
                        None
                    }
                })
                .collect();
            // Deterministic order → reproducible bytecode offsets.
            impls.sort();
            (iface_name.clone(), impls)
        })
        .collect();

    let interface_names: std::collections::HashSet<String> =
        interface_methods.keys().cloned().collect();

    // PERF: lazily-built memo of each sibling's DIRECT contract references,
    // shared across the whole per-primary loop below. The transitive
    // closure used to re-walk a sibling's full AST once per (primary, pop);
    // `collect_direct_sibling_contract_refs` depends only on tables that
    // are fixed across the loop, so each contract needs walking at most
    // once for the entire merge pass.
    let mut direct_sibling_refs_memo: std::collections::HashMap<
        String,
        std::collections::HashSet<String>,
    > = std::collections::HashMap::new();

    for contract in primary.iter_mut() {
        let mut referenced: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut iface_refs: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Task #194 — collect method names statically resolvable from
        // low-level-call payloads like
        // `addr.call(abi.encodeWithSelector(bytes4(keccak256("m(T)"))))`,
        // `addr.call(abi.encodeWithSignature("m(T)", …))`, or
        // `addr.call(abi.encodeCall(Iface.m, …))`. Previously the
        // sibling-merge pass only detected references through `new X()`,
        // `X(addr)` casts, interface casts, and typed params/returns/
        // state-vars. A low-level `.call()` with a constant selector is
        // semantically identical to a typed `X(addr).m(…)` — the
        // compiler routes it through the zero-placeholder
        // `self_method_offsets` dispatch (see Task #83) — but without
        // this scan the target method never lands in the merged table
        // and the call silently returns `Null`.
        let mut low_level_method_refs: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for function in &contract.functions {
            if let Some(body) = function.body.as_ref() {
                collect_new_contract_refs(body, &primary_names, &mut referenced);
                // Task #115 — interface casts `I(expr)` in statements.
                collect_interface_casts_stmt(body, &interface_names, &mut iface_refs);
                // Task #194 — low-level calls whose payload encodes a
                // statically resolvable method name.
                collect_low_level_call_method_refs_stmt(body, &mut low_level_method_refs);
            }
            // Task K4 — function params/returns typed as a sibling contract
            // (e.g. `function bounce() external returns (B) {...}`, or
            // `function xfer(C to, ...)`) mean the function is wired to
            // call into the sibling. Merge so self-call routing can see
            // the target method at runtime.
            for p in function.parameters.iter().chain(function.returns.iter()) {
                if primary_names.contains(&p.ty) {
                    referenced.insert(p.ty.clone());
                }
                // Task #115 — also scan for interface-typed parameters.
                if interface_names.contains(&p.ty) {
                    iface_refs.insert(p.ty.clone());
                }
            }
        }
        // Task K4 — also scan state-variable types and initializers.
        // `B public b;` means A is wired to call into B via the storage
        // slot without ever going through `new B()`. Without this hook,
        // K4 (cross-contract reentrancy) fails: `b.bounce()` routes
        // through `System.Contract.Call([0;20], "bounce", …)` which then
        // returns `Null` because B wasn't merged.
        for state in &contract.state_variables {
            if primary_names.contains(&state.ty) {
                referenced.insert(state.ty.clone());
            }
            if interface_names.contains(&state.ty) {
                iface_refs.insert(state.ty.clone());
            }
            if let Some(init) = state.initializer.as_ref() {
                collect_new_refs_expr(init, &primary_names, &mut referenced);
                collect_interface_casts_expr(init, &interface_names, &mut iface_refs);
                collect_low_level_call_method_refs_expr(init, &mut low_level_method_refs);
            }
        }
        // Task #115 — expand interface references to the primary contracts
        // that implement them. Multiple primaries may satisfy the same
        // interface; merge all of them so dispatch sees any signature.
        for iface in &iface_refs {
            if let Some(impls) = interface_impls.get(iface) {
                for prim in impls {
                    if prim != &contract.name {
                        referenced.insert(prim.clone());
                    }
                }
            }
        }
        // Task #194 — expand low-level-call method references to every
        // sibling primary that declares a method of that name. When
        // multiple siblings satisfy the same name (e.g. both X and Y
        // declare `foo()`), merge all of them so the dispatcher sees the
        // union; which one actually fires at runtime is decided by the
        // caller's address (handled in `handle_contract_call`). We skip
        // the host contract itself — its methods are already visible
        // through normal dispatch.
        if !low_level_method_refs.is_empty() {
            for (prim_name, prim_methods) in &primary_method_names {
                if prim_name == &contract.name {
                    continue;
                }
                if low_level_method_refs
                    .iter()
                    .any(|m| prim_methods.contains(m))
                {
                    referenced.insert(prim_name.clone());
                }
            }
        }
        // Task #206 — close over TRANSITIVE sibling references. The
        // zero-hash self-dispatch table is built from the caller
        // artifact's manifest only, so if `Client` references `Middle`
        // and `Middle` references `Target`, the merged `Client`
        // artifact must carry both `wrap` and `fail`. Without this
        // closure, the grandchild call silently falls through
        // `handle_contract_call`'s zero-hash branch and returns `Null`.
        //
        // PERF: the direct-reference set of each sibling is MEMOIZED in
        // `direct_sibling_refs_memo` (computed lazily, shared across the
        // whole per-primary outer loop). Previously every pop re-walked
        // the sibling's full statement/expression tree via
        // `collect_direct_sibling_contract_refs` — O(primaries × edges)
        // full-AST walks on multi-contract units; its inputs are all
        // fixed across the outer loop, so one walk per contract suffices.
        let mut transitive_queue: Vec<String> = referenced.iter().cloned().collect();
        while let Some(sibling_name) = transitive_queue.pop() {
            let Some(sibling_contract) = primary_contract_map.get(&sibling_name) else {
                continue;
            };
            let transitive_refs = direct_sibling_refs_memo
                .entry(sibling_name.clone())
                .or_insert_with(|| {
                    collect_direct_sibling_contract_refs(
                        sibling_contract,
                        &primary_names,
                        &interface_names,
                        &interface_impls,
                        &primary_method_names,
                    )
                });
            for transitive in transitive_refs.iter() {
                if transitive == &contract.name {
                    continue;
                }
                if referenced.insert(transitive.clone()) {
                    transitive_queue.push(transitive.clone());
                }
            }
        }
        referenced.remove(&contract.name);
        if referenced.is_empty() {
            continue;
        }
        let mut existing_sigs: std::collections::HashSet<(String, usize)> = contract
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.parameters.len()))
            .collect();
        // Deterministic order → reproducible bytecode offsets.
        let mut sibling_names: Vec<String> = referenced.into_iter().collect();
        sibling_names.sort();
        for sibling_name in &sibling_names {
            let Some(sibling_fns) = sibling_fn_map.get(sibling_name) else {
                continue;
            };
            // Caller names of the sibling functions actually merged below.
            let mut merged_sibling_fn_names: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for sibling_fn in sibling_fns {
                let sig = (sibling_fn.name.clone(), sibling_fn.parameters.len());
                if existing_sigs.insert(sig) {
                    merged_sibling_fn_names.insert(sibling_fn.name.clone());
                    contract.functions.push(sibling_fn.clone());
                }
            }
            // Audit 2026-08 (P0) — merge the sibling's `super_method_map`
            // entries for the bodies that were just copied. The sibling's
            // flattened bodies are copied verbatim, including their internal
            // `__super{N}_*` chain slots (via the concrete-internal filter
            // above), so a merged body containing `super.foo()` must find its
            // caller-keyed entry in the HOST's map. Without this, the
            // resolver's unqualified fallback
            // (`ir_context::ctx_overloads::super_method_name`) resolves
            // `super.foo()` inside a merged `foo` to `foo` itself and the
            // call recurses until the node faults with
            // `Call stack overflow (MaxInvocationStackSize)` — a silent
            // wrong-codegen for any 2+ level override chain reached through
            // a sibling-merged contract.
            //
            // Only entries whose caller function was actually merged are
            // copied (host-wins-on-collision: when the host defines its own
            // `foo`, the sibling's `foo` body was not merged, so its entries
            // must not leak). Host entries always win on key collision.
            if let Some(sibling_contract) = pre_merge_contract_map.get(sibling_name) {
                for (key, target) in sibling_contract.super_method_map.iter() {
                    let caller = key.split("::").next().unwrap_or(key);
                    if merged_sibling_fn_names.contains(caller) {
                        contract
                            .super_method_map
                            .entry(key.clone())
                            .or_insert_with(|| target.clone());
                    }
                }
            }
            // Pull in the sibling's modifier definitions so the merged
            // external bodies can still resolve their `onlyOwner` /
            // `onlyRole` / etc. references when the host's modifier-
            // expansion pass runs. Modifiers don't conflict on plain
            // function-signature equality (different `ty`), so we track
            // them in a small local set keyed on (name, arity).
            if let Some(sibling_modifiers) = sibling_modifier_map.get(sibling_name) {
                for sibling_mod in sibling_modifiers {
                    let already_present = contract.functions.iter().any(|existing| {
                        matches!(existing.ty, FunctionTy::Modifier)
                            && existing.name == sibling_mod.name
                            && existing.parameters.len() == sibling_mod.parameters.len()
                    });
                    if !already_present {
                        contract.functions.push(sibling_mod.clone());
                    }
                }
            }
            // Also pull in the sibling's `using` directives. Merged
            // external bodies may rely on contract-scope `using L for T;`
            // declarations declared in the sibling — e.g. Gnosis Safe
            // declares `using SafeMath for uint256;` and its
            // `execTransaction` body calls `gas.max(other)`. When
            // CompatibilityFallbackHandler triggers a sibling-merge of
            // Safe's external methods, the `execTransaction` body needs
            // its `using SafeMath` directive to remain in scope or the
            // IR-lowering pass reports "member-style call '...' requires
            // an explicit `using` directive". We dedup on
            // (target_type, function_names) so reinjected duplicates
            // don't grow the table.
            if let Some(sibling_contract) = pre_merge_contract_map.get(sibling_name) {
                for directive in &sibling_contract.using_directives {
                    if !contract.using_directives.iter().any(|existing| {
                        existing.target_type == directive.target_type
                            && existing.function_names == directive.function_names
                    }) {
                        contract.using_directives.push(directive.clone());
                    }
                }
                for lib_name in &sibling_contract.using_for_libraries {
                    if !contract.using_for_libraries.contains(lib_name) {
                        contract.using_for_libraries.push(lib_name.clone());
                    }
                }
                contract.has_using_for_star =
                    contract.has_using_for_star || sibling_contract.has_using_for_star;
                contract.has_using_function_list =
                    contract.has_using_function_list || sibling_contract.has_using_function_list;
            }
            // Task #23 — merge the sibling's event declarations so a merged
            // external body's `emit Foo(...)` resolves against the host's
            // event table. Dedup by event name; a host event of the same
            // name already covers the declaration (matching manifest
            // dedup), so we only append events the host doesn't declare.
            if let Some(sibling_events) = sibling_event_map.get(sibling_name) {
                for sibling_event in sibling_events {
                    if !contract.events.iter().any(|e| e.name == sibling_event.name) {
                        contract.events.push(sibling_event.clone());
                    }
                }
            }
        }

        // Task #197 — merge sibling state variables after their external
        // functions. Without this, a merged stateful method like
        // `Mock.balanceOf` → `return _bal[a]` would resolve `_bal`
        // against the caller's (Client's) `state_index_map`, find no
        // match, and fall through `variable.rs::lower_variable_expression`'s
        // final compatibility arm which pushes `Integer(0)` as a neutral
        // placeholder. Downstream SIZE/PICKITEM opcodes then fault on
        // the scalar, surfacing as "SIZE: unsupported type" at runtime.
        //
        // Storage-key derivation is name-based (see
        // `value_types.rs::compute_state_slot`), so merging Mock's
        // `_bal` into Client produces the same keccak-derived slot
        // that Mock.mint writes to — the cross-contract read lands on
        // the same storage entry. Host-wins-on-collision preserves any
        // state variable the caller already declares.
        let mut existing_state_names: std::collections::HashSet<String> = contract
            .state_variables
            .iter()
            .filter_map(|s| s.name.clone())
            .collect();
        for sibling_name in &sibling_names {
            let Some(sibling_states) = sibling_state_map.get(sibling_name) else {
                continue;
            };
            for sibling_state in sibling_states {
                if let Some(name) = sibling_state.name.as_ref() {
                    if existing_state_names.insert(name.clone()) {
                        contract.state_variables.push(sibling_state.clone());
                    } else if let Some(existing) = contract
                        .state_variables
                        .iter()
                        .find(|s| s.name.as_deref() == Some(name.as_str()))
                    {
                        // Storage-soundness guard — slots are derived
                        // from the BARE variable name (`sha256(name)`,
                        // see `storage_key::compute_state_slot`), so a
                        // host/sibling pair declaring the same name with
                        // DIFFERENT types would silently collapse two
                        // semantically distinct lvalues onto one slot
                        // (e.g. a sibling's `mapping(address=>uint256)
                        // _bal` aliasing the host's scalar `uint256
                        // _bal`), miscompiling both. Same-name SAME-type
                        // sharing stays allowed: that is the documented
                        // Task #197 design (merged sibling bodies must
                        // hit the same name-keyed slot).
                        if normalize_state_type_for_merge(&existing.ty)
                            != normalize_state_type_for_merge(&sibling_state.ty)
                        {
                            return Err(SolidityError::analysis(format!(
                                "state variable '{name}' is declared with conflicting types \
                                 across merged contracts: '{}' in '{}' vs '{}' in '{sibling_name}'. \
                                 Storage slots are derived from the bare variable name, so both \
                                 declarations would silently alias the same storage entry; \
                                 rename one of the variables.",
                                existing.ty, contract.name, sibling_state.ty
                            )));
                        }
                    }
                }
            }
        }

        // Task #198 — merge sibling constructors as name-mangled internal
        // regular functions so the caller's `new Child(args)` lowering can
        // invoke the ctor body in-line. Without this, `new Child(x, y)`
        // silently drops its args and the follow-up `c.a()` / `c.b()`
        // reads land on uninitialized storage (all zeros). Re-typing from
        // `Constructor` to `Function` prevents the caller's own `_deploy`
        // prologue from accidentally calling the merged entry at deploy
        // time (constructor_indices is populated by FunctionKind, so
        // Regular entries are skipped there).
        for sibling_name in &sibling_names {
            let Some(sibling_ctor) = sibling_ctor_map.get(sibling_name) else {
                continue;
            };
            let mangled_name = format!("__ctor__{sibling_name}");
            let sig = (mangled_name.clone(), sibling_ctor.parameters.len());
            if !existing_sigs.insert(sig) {
                continue;
            }
            let mut cloned = sibling_ctor.clone();
            cloned.name = mangled_name;
            cloned.ty = FunctionTy::Function;
            cloned.visibility = VisibilityKind::Internal;
            // Compiler-synthesized helper: its `internal` visibility is set
            // here by rule, so it must not trip the missing-visibility check
            // (a constructor carries no specifier, so the clone's flag is
            // `false`).
            cloned.explicit_visibility = true;
            // Base-constructor invocations (`base_or_modifiers`) were
            // already resolved by `apply_modifiers_and_base_constructors`
            // in the owning contract's pipeline; clear the residue so the
            // caller's modifier-application pass (which ran before this
            // merge) doesn't re-expand anything.
            cloned.base_or_modifiers.clear();
            contract.functions.push(cloned);
        }
    }

    Ok(())
}
