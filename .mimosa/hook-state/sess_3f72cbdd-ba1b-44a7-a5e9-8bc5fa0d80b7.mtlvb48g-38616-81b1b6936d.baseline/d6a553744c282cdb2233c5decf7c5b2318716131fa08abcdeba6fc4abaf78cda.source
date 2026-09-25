use super::*;

pub(crate) fn apply_modifiers_and_base_constructors(
    contract: &mut ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Result<(), SolidityError> {
    let contract_snapshot = contract.clone();
    let modifier_defs = build_modifier_definition_map(contract);

    for func in contract.functions.iter_mut() {
        let Some(body) = func.body.clone() else {
            continue;
        };

        if matches!(func.ty, FunctionTy::Modifier) {
            continue;
        }

        if matches!(func.ty, FunctionTy::Constructor) {
            let constructor_snapshot = func.clone();
            func.body = Some(apply_base_constructors_and_modifiers(
                &contract_snapshot,
                &constructor_snapshot,
                &modifier_defs,
                contract_map,
            )?);
            func.base_or_modifiers.clear();
            continue;
        }

        if func.base_or_modifiers.is_empty() {
            continue;
        }

        // Task #114 — threaded epilogue detection: if any applied modifier has
        // statements after its `_;` placeholder, record the flag on the
        // function so IR lowering can redirect returns through synthetic
        // slots + an end-of-function jump (so modifier tail statements still
        // run before the actual RET).
        let (expanded_body, had_epilogue) = apply_modifier_calls_to_body_with_epilogue(
            &body,
            &func.base_or_modifiers,
            &modifier_defs,
        )?;
        func.body = Some(expanded_body);
        if had_epilogue {
            func.had_modifier_epilogue = true;
        }
        func.base_or_modifiers.clear();
    }

    Ok(())
}
