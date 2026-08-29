use super::*;

pub(crate) fn compute_transitive_hazard_map(functions: &[Function]) -> HashMap<String, Hazards> {
    let mut direct = HashMap::new();
    for function in functions {
        direct.insert(function.name.clone(), direct_hazards(function));
    }

    let graph = build_call_graph(functions);
    let mut memo = HashMap::new();
    let mut visiting = HashSet::new();

    for name in direct.keys() {
        compute_transitive_hazards(name, &direct, &graph, &mut memo, &mut visiting);
    }

    memo
}

pub(crate) fn validate_safe_methods(
    metadata: &ContractMetadata,
    hazards: &HashMap<String, Hazards>,
) -> Vec<IrDiagnostic> {
    let mut errors = Vec::new();

    for method in &metadata.methods {
        if method.state_mutability != crate::solidity::StateMutability::View {
            continue;
        }

        let hazards = hazards.get(&method.neo_name).copied().unwrap_or_default();

        if !hazards.safe_violation() {
            continue;
        }

        if hazards.writes_state {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared view/pure but writes contract storage".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.notifies {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared view/pure but emits events/notifications".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.unsafe_contract_call {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared view/pure but performs non-readonly contract calls".to_string(),
                suggestion: None,
                code: None,
            });
        }
    }

    errors
}

pub(crate) fn validate_pure_methods(
    metadata: &ContractMetadata,
    hazards: &HashMap<String, Hazards>,
) -> Vec<IrDiagnostic> {
    let mut errors = Vec::new();

    for method in &metadata.methods {
        if method.state_mutability != crate::solidity::StateMutability::Pure {
            continue;
        }

        let hazards = hazards.get(&method.neo_name).copied().unwrap_or_default();

        if !hazards.pure_violation() {
            continue;
        }

        if hazards.writes_state {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared pure but writes contract storage".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.notifies {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared pure but emits events/notifications".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.reads_state {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared pure but reads contract storage".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.reads_environment {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared pure but reads the execution environment".to_string(),
                suggestion: None,
                code: None,
            });
        }

        if hazards.unsafe_contract_call {
            errors.push(IrDiagnostic {
                function_name: method.neo_name.clone(),
                message: "declared pure but performs non-readonly contract calls".to_string(),
                suggestion: None,
                code: None,
            });
        }
    }

    errors
}
