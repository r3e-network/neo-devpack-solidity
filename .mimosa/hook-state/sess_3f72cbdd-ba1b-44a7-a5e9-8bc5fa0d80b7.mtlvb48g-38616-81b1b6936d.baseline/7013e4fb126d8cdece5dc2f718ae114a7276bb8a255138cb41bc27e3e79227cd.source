use super::*;

pub(crate) fn convert_contract(
    contract: ContractIR,
    inherited_events: &[EventMetadata],
    contract_types: &[String],
    selector_registry: std::sync::Arc<SelectorRegistry>,
) -> ContractMetadata {
    let structs: Vec<StructMetadata> = contract.structs.into_iter().map(convert_struct).collect();
    let enums: Vec<EnumMetadata> = contract.enums.into_iter().map(convert_enum).collect();

    let struct_type_info = structs_to_type_metadata(&structs);
    let enum_type_info = enums_to_type_metadata(&enums);

    // If the contract already defines an explicit `onNEP17Payment`, do not
    // remap Solidity `receive()` into the Neo NEP-17 callback entrypoint.
    //
    // This keeps the contract compilable when authors include both `receive()`
    // (for Solidity source compatibility) and `onNEP17Payment` (for Neo).
    //
    // M-FE3 fix — the detection is now case-INSENSITIVE (`eq_ignore_ascii_case`)
    // to match the rest of the NEP-pattern validation
    // (`erc_nep_patterns.rs:175`). A user-written `function onnep17payment(...)`
    // or `onNEP17PAYMENT(...)` must still be recognized as the explicit
    // callback; otherwise the compiler would remap `receive()` into a SECOND
    // synthetic `onNEP17Payment`, producing a manifest-level name collision
    // (Neo dispatches by name).
    let has_explicit_on_nep17_payment = contract.functions.iter().any(|function| {
        !matches!(function.ty, FunctionTy::Receive)
            && function.name.eq_ignore_ascii_case("onnep17payment")
    });

    // A contract-level `function` declared without a visibility specifier is a
    // hard error in Solidity 0.5.0+. Capture the offenders now, while the
    // original `explicit_visibility` flag is available, so validation can
    // surface them instead of letting the defaulted `internal` visibility drop
    // them from the ABI. (Constructors/fallback/receive carry no visibility by
    // rule, and free functions were marked explicit at parse time.)
    let functions_missing_visibility: Vec<String> = contract
        .functions
        .iter()
        .filter(|function| {
            matches!(function.ty, FunctionTy::Function)
                && !function.explicit_visibility
                // Defensive: compiler-synthesized helpers use a reserved `__`
                // mangling prefix (e.g. `__ctor__X`, `__super_X`) and are not
                // user-authored, so they can never be a missing-visibility
                // mistake even if a future synthetic site forgets the flag.
                && !function.name.starts_with("__")
        })
        .map(|function| function.name.clone())
        .collect();

    let mut methods: Vec<FunctionMetadata> = contract
        .functions
        .into_iter()
        .filter(|function| {
            matches!(
                function.ty,
                FunctionTy::Function
                    | FunctionTy::Constructor
                    | FunctionTy::Fallback
                    | FunctionTy::Receive
            )
        })
        .map(|function| {
            convert_function(
                function,
                &struct_type_info,
                &enum_type_info,
                contract_types,
                has_explicit_on_nep17_payment,
                &contract.type_aliases,
            )
        })
        .collect();

    // Dedup ONLY the synthetic `onNEP17Payment(address,uint256,Any)`
    // entries produced by `convert_function`'s receive→onNEP17Payment
    // remap. The flatten + sibling-merge passes can surface multiple
    // `receive()` declarations from independent inheritance paths
    // (concrete repro: OZ GovernorTimelockControlUpgradeable inheriting
    // both GovernorUpgradeable and a separate interface that also exposes
    // a payment receiver) — each one converts to the same synthetic
    // entry and the duplicate-signature validation then rejects the
    // contract.
    //
    // We deliberately do NOT do a general dedup here. Real user errors
    // like `function foo(uint256) public {} function foo(uint256) public {}`
    // must still hit the `validate_contract` "duplicate function signature"
    // path so the user sees the error. The receive-conversion path is
    // identifiable by the exact parameter signature `(address, uint256, Any)`
    // and the payable mutability that `convert_function` sets — neither
    // is what a user-written `onNEP17Payment` typically looks like at
    // both declaration sites simultaneously.
    {
        let mut seen_synthetic_payment = false;
        methods.retain(|m| {
            if m.neo_name != "onNEP17Payment" {
                return true;
            }
            // Match the exact synthetic shape `convert_function` emits.
            let is_synthetic_signature = m.parameters.len() == 3
                && m.parameters
                    .iter()
                    .map(|p| p.ty.as_str())
                    .eq(["address", "uint256", "Any"]);
            if !is_synthetic_signature {
                return true;
            }
            if seen_synthetic_payment {
                false
            } else {
                seen_synthetic_payment = true;
                true
            }
        });
    }

    // Mangle Neo entrypoint names for overloaded Solidity functions.
    // Neo ABI dispatches by method name and parameter count, so overloaded
    // functions must have unique Neo-visible names to avoid collisions during
    // code generation and manifest export. We preserve the original Solidity
    // name in `FunctionMetadata::name` for selector/ABI purposes.
    use std::collections::HashMap;
    let mut overloads: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, method) in methods.iter().enumerate() {
        if matches!(method.kind, FunctionKind::Regular) {
            overloads.entry(method.name.clone()).or_default().push(idx);
        }
    }
    for (name, indices) in overloads {
        if indices.len() > 1 {
            for index in indices {
                let method = &mut methods[index];
                let param_signatures: Vec<String> = method
                    .parameters
                    .iter()
                    .map(|param| crate::utils::canonical_param_type(&param.ty))
                    .collect();
                method.neo_name = if param_signatures.is_empty() {
                    format!("{name}()")
                } else {
                    format!("{name}({})", param_signatures.join(","))
                };
            }
        }
    }

    use std::collections::BTreeMap;
    let mut event_map: BTreeMap<String, EventMetadata> = BTreeMap::new();
    for event in inherited_events {
        event_map
            .entry(event.normalized_name.clone())
            .or_insert_with(|| event.clone());
    }
    for event in contract.events.into_iter().map(|event| {
        convert_event(
            event,
            &struct_type_info,
            &enum_type_info,
            contract_types,
            &contract.type_aliases,
        )
    }) {
        event_map.insert(event.normalized_name.clone(), event);
    }
    let events: Vec<EventMetadata> = event_map.into_values().collect();

    // Carry declared custom errors through to metadata so IR lowering can
    // resolve revert selectors from the DECLARED signature (not the types
    // inferred from revert-site argument expressions).
    let errors: Vec<ErrorMetadata> = contract
        .errors
        .iter()
        .map(|err| ErrorMetadata {
            name: err.name.clone(),
            parameters: err
                .parameters
                .iter()
                .map(|param| ErrorParameterMetadata {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                })
                .collect(),
        })
        .collect();

    let state_variables: Vec<StateVariableMetadata> = contract
        .state_variables
        .into_iter()
        .map(|var| {
            convert_state_variable(
                var,
                &struct_type_info,
                &enum_type_info,
                contract_types,
                &contract.type_aliases,
            )
        })
        .collect();

    // Synthesize public state variable getters to match Solidity ABI behavior.
    synthesize_public_getters(&mut methods, &state_variables);

    ContractMetadata {
        name: contract.name,
        is_abstract: matches!(contract.kind, ContractKind::AbstractContract),
        is_interface: matches!(contract.kind, ContractKind::Interface),
        is_library: matches!(contract.kind, ContractKind::Library),
        methods,
        events,
        errors,
        uses_storage: state_variables.iter().any(|state| !state.is_constant),
        state_variables,
        structs,
        enums,
        contract_types: contract_types.to_vec(),
        selector_registry,
        documentation: contract.doc.into(),
        has_using_for_star: contract.has_using_for_star,
        has_using_function_list: contract.has_using_function_list,
        using_for_libraries: contract.using_for_libraries.clone(),
        using_directives: contract
            .using_directives
            .iter()
            .map(|directive| {
                // Resolve user-defined value-type aliases at directive-recording
                // time. The IR-lowering pass renders the receiver as its
                // underlying type (`type Currency is address;` → "address"),
                // so a library-form `using CurrencyLibrary for Currency global;`
                // would otherwise have an unmatched target "currency" while
                // every receiver is "address" — `currency.balanceOf(addr)`
                // would then fail with "not available for receiver type
                // 'Address' under the current `using` directives".
                //
                // BUT: function-list-form directives like
                // `using {add as +, sub as -} for BalanceDelta global;` MUST
                // keep the alias target. The function list scope-restricts
                // the directive to *alias-typed* values only — it should not
                // capture every `int256` receiver, which is what would happen
                // if we resolved `BalanceDelta` → `int256` here. (Repro:
                // Uniswap V4 BalanceDelta.sol — `using {add, sub}` for
                // BalanceDelta would otherwise capture every `int256.toInt128()`
                // call inside `add()`'s own body and reject it because
                // `toInt128` isn't in the operator list.)
                //
                // Library form gets the resolution; function-list form does
                // not. `frontend_convert::normalize_using_target_type` already
                // lowercased the target string and the alias map is keyed on
                // the source spelling, so we look up case-insensitively.
                let resolved_target = directive.target_type.as_ref().map(|t| {
                    if directive.function_names.is_some() {
                        return t.clone();
                    }
                    let stripped = t.trim();
                    // (a) User-defined value type alias: `type Currency is address;`
                    //     resolves `using L for Currency;` to `using L for address;`.
                    if let Some(underlying) = contract
                        .type_aliases
                        .iter()
                        .find(|(alias, _)| alias.eq_ignore_ascii_case(stripped))
                        .map(|(_, underlying)| underlying.clone())
                    {
                        return underlying;
                    }
                    // (b) Interface or contract type: `using GPv2SafeERC20 for IERC20;`
                    //     The IR-lowering pass treats every interface/contract
                    //     handle as `address`, so the directive's effective
                    //     receiver type is `address` too. Without this
                    //     resolution, OZ-style code in Aave AToken
                    //     (`IERC20(_underlyingAsset).safeTransfer(to, amt)`)
                    //     fails with "member-style call '...' is not available
                    //     for receiver type 'Address' under the current
                    //     `using` directives".
                    if contract_types
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(stripped))
                    {
                        return "address".to_string();
                    }
                    t.clone()
                });
                UsingDirectiveMetadata {
                    target_type: resolved_target,
                    function_names: directive.function_names.clone(),
                    overloaded_operators: directive.overloaded_operators.clone(),
                }
            })
            .collect(),
        has_type_definitions: contract.has_type_definitions,
        type_aliases: contract.type_aliases,
        flatten_warnings: Vec::new(),
        functions_missing_visibility,
        super_method_map: contract.super_method_map,
    }
}

pub(crate) fn synthesize_public_getters(
    methods: &mut Vec<FunctionMetadata>,
    state_variables: &[StateVariableMetadata],
) {
    // Pre-compute the (name, arity) signatures that the host already
    // exposes via a real function declaration. After inheritance flattening,
    // a derived contract may have inherited a state variable AND an
    // interface that declares the same getter as `function POOL() external view returns (IPool);`
    // — synthesizing another getter on top would produce a duplicate ABI
    // entry which `validate_contract` then rejects with
    // "duplicate function signature 'POOL()'". This dedup is keyed on
    // (name, arity) instead of full canonical signature because Solidity's
    // auto-generated getter has a fixed parameter shape that depends only
    // on the state variable's type, and the manifest layer treats name as
    // the primary identity anyway. Library state variables (still tracked
    // as `internal` after `normalize_library_for_neo`) never produce a
    // synthesized getter, so they don't enter the existing-signature set.
    let mut existing_signatures: std::collections::HashSet<(String, usize)> = methods
        .iter()
        .filter(|m| {
            matches!(m.kind, FunctionKind::Regular)
                && matches!(
                    m.visibility,
                    VisibilityKind::External | VisibilityKind::Public
                )
        })
        .map(|m| (m.name.clone(), m.parameters.len()))
        .collect();

    // Also dedup against state variables we've already synthesized a getter
    // for. Diamond inheritance can surface the same `public` state var via
    // multiple paths through `flatten_contract_inheritance` — the
    // `state_variables.extend(ancestor.state_variables.clone())` loop is
    // not deduped — so without this guard each path's clone produces its
    // own getter and the duplicate-signature check fires.
    let mut emitted_getters: std::collections::HashSet<(String, usize)> =
        std::collections::HashSet::new();

    for state in state_variables {
        if state
            .visibility
            .as_deref()
            .map(|v| v.eq_ignore_ascii_case("public"))
            != Some(true)
        {
            continue;
        }

        let name = match state.name.as_deref() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let neotype = match state.neo_type.as_ref() {
            Some(neo) => neo.clone(),
            None => continue,
        };

        let (parameters, return_parameters, expr) = getter_signature_from_neotype(&name, &neotype);
        let arity = parameters.len();
        let dedup_key = (name.clone(), arity);
        if existing_signatures.contains(&dedup_key) || !emitted_getters.insert(dedup_key.clone()) {
            continue;
        }
        let param_signatures: Vec<String> = parameters
            .iter()
            .map(|param| crate::utils::canonical_param_type(&param.ty))
            .collect();
        let selector = compute_function_selector(&name, &param_signatures);

        methods.push(FunctionMetadata {
            name: name.clone(),
            neo_name: name,
            kind: FunctionKind::Regular,
            parameters,
            return_parameters,
            state_mutability: StateMutability::View,
            visibility: VisibilityKind::Public,
            offset: 0,
            body: Some(Statement::Return(Default::default(), Some(expr))),
            selector,
            is_virtual: false,
            is_override: false,
            documentation: NatspecDoc::default(),
            had_modifier_epilogue: false,
        });
        existing_signatures.insert(dedup_key);
    }
}

pub(crate) fn convert_state_variable(
    var: StateVariableIR,
    struct_types: &[StructTypeMetadata],
    enum_types: &[EnumTypeMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
) -> StateVariableMetadata {
    let ty = var.ty;
    let neo_type = NeoType::from_solidity_with_aliases(
        &ty,
        struct_types,
        enum_types,
        contract_types,
        type_aliases,
    )
    .ok();
    let initializer = var.initializer;
    StateVariableMetadata {
        name: var.name,
        ty,
        is_constant: var.is_constant,
        is_immutable: var.is_immutable,
        is_transient: var.is_transient,
        visibility: var.visibility,
        neo_type,
        has_initializer: initializer.is_some(),
        initializer,
    }
}
