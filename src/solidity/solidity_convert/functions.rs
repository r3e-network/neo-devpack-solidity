use super::*;

pub(crate) fn convert_function(
    function: FunctionIR,
    struct_types: &[StructTypeMetadata],
    enum_types: &[EnumTypeMetadata],
    contract_types: &[String],
    has_explicit_on_nep17_payment: bool,
    type_aliases: &std::collections::HashMap<String, String>,
) -> FunctionMetadata {
    if matches!(function.ty, FunctionTy::Receive) {
        if has_explicit_on_nep17_payment {
            let name = "receive".to_string();
            let neo_name = name.clone();
            let selector = compute_function_selector(&name, &[]);

            return FunctionMetadata {
                neo_name,
                name,
                kind: FunctionKind::Regular,
                parameters: Vec::new(),
                return_parameters: Vec::new(),
                state_mutability: StateMutability::Payable,
                visibility: VisibilityKind::External,
                offset: 0,
                body: function.body,
                selector,
                is_virtual: function.is_virtual,
                is_override: function.is_override,
                documentation: function.doc.into(),
                had_modifier_epilogue: function.had_modifier_epilogue,
            };
        }

        let name = "onNEP17Payment".to_string();
        let neo_name = name.clone();
        let parameters = vec![
            ParameterMetadata {
                name: Some("from".to_string()),
                ty: "address".to_string(),
                neo_type: Some(NeoType::Address),
                storage: None,
            },
            ParameterMetadata {
                name: Some("amount".to_string()),
                ty: "uint256".to_string(),
                neo_type: Some(NeoType::Integer {
                    signed: false,
                    bits: 256,
                }),
                storage: None,
            },
            ParameterMetadata {
                name: Some("data".to_string()),
                ty: "Any".to_string(),
                neo_type: Some(NeoType::Any),
                storage: None,
            },
        ];

        let param_signatures: Vec<String> = parameters
            .iter()
            .map(|param| crate::utils::canonical_param_type(&param.ty))
            .collect();
        let selector = compute_function_selector(&name, &param_signatures);

        return FunctionMetadata {
            neo_name,
            name,
            kind: FunctionKind::Regular,
            parameters,
            return_parameters: Vec::new(),
            state_mutability: StateMutability::Payable,
            visibility: VisibilityKind::External,
            offset: 0,
            body: function.body,
            selector,
            is_virtual: false,
            is_override: false,
            documentation: function.doc.into(),
            had_modifier_epilogue: false,
        };
    }

    let parameters: Vec<ParameterMetadata> = function
        .parameters
        .into_iter()
        .map(|param| {
            convert_parameter(
                param,
                struct_types,
                enum_types,
                contract_types,
                type_aliases,
            )
        })
        .collect();

    let return_parameters = function
        .returns
        .into_iter()
        .map(|param| {
            convert_parameter(
                param,
                struct_types,
                enum_types,
                contract_types,
                type_aliases,
            )
        })
        .collect();

    // Build each parameter's EVM-ABI canonical type for the function selector
    // `keccak256(name(t1,t2,...))`. Prefer the RESOLVED `NeoType`, which expands
    // struct parameters to `(field0,field1,...)` tuples, renders enums as `uint8`
    // and always emits explicit integer widths (`uint256`, not `uint`) — matching
    // Ethereum tooling. The string-based `canonical_param_type` (which returns the
    // bare struct/enum name) is only a fallback for the rare unresolved type.
    let param_signatures: Vec<String> = parameters
        .iter()
        .map(|param| match &param.neo_type {
            Some(neo_type) => neo_type.canonical_abi_type(),
            None => crate::utils::canonical_param_type(&param.ty),
        })
        .collect();

    let name = function.name;
    let selector = compute_function_selector(&name, &param_signatures);

    FunctionMetadata {
        neo_name: name.clone(),
        name,
        kind: match function.ty {
            FunctionTy::Constructor => FunctionKind::Constructor,
            _ => FunctionKind::Regular,
        },
        parameters,
        return_parameters,
        state_mutability: match function.mutability {
            MutabilityKind::Pure => StateMutability::Pure,
            MutabilityKind::View => StateMutability::View,
            MutabilityKind::Payable => StateMutability::Payable,
            MutabilityKind::NonPayable => StateMutability::NonPayable,
        },
        visibility: function.visibility,
        offset: 0,
        body: function.body,
        selector,
        is_virtual: function.is_virtual,
        is_override: function.is_override,
        documentation: function.doc.into(),
        had_modifier_epilogue: function.had_modifier_epilogue,
    }
}

pub(crate) fn convert_parameter(
    param: ParameterIR,
    struct_types: &[StructTypeMetadata],
    enum_types: &[EnumTypeMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
) -> ParameterMetadata {
    let ty = param.ty;
    let neo_type = NeoType::from_solidity_with_aliases(
        &ty,
        struct_types,
        enum_types,
        contract_types,
        type_aliases,
    )
    .ok();
    ParameterMetadata {
        name: param.name,
        ty,
        neo_type,
        storage: param.storage,
    }
}
