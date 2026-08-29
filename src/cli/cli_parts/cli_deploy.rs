use super::*;

pub(crate) fn ensure_deploy_stub(metadata: &mut ContractMetadata) -> Result<(), CompileError> {
    const DEPLOY_NAME: &str = "_deploy";

    if metadata.methods.iter().any(|m| m.name == DEPLOY_NAME) {
        return Ok(());
    }

    let constructors: Vec<&FunctionMetadata> = metadata
        .methods
        .iter()
        .filter(|function| matches!(function.kind, FunctionKind::Constructor))
        .collect();

    if constructors.len() > 1 {
        return Err(CompileError::Message(
            "multiple constructors are not supported".to_string(),
        ));
    }

    // Parameterised constructors are supported via `_deploy(data, update)`.
    // When the Solidity constructor requires arguments, `data` is treated as an array
    // of constructor args. Tooling commonly passes this as a JSON-encoded array string
    // (e.g., `[7]`); the injected deploy prologue attempts StdLib.jsonDeserialize, then
    // StdLib.deserialize (for StdLib.serialize(...) bytes), and finally falls back to
    // using `data` as-is if native calls throw.

    let deploy_params = vec![
        ParameterMetadata {
            name: Some("data".to_string()),
            ty: "Any".to_string(),
            neo_type: Some(NeoType::Any),
            storage: None,
        },
        ParameterMetadata {
            name: Some("update".to_string()),
            ty: "bool".to_string(),
            neo_type: Some(NeoType::Boolean),
            storage: None,
        },
    ];

    let signature = format!(
        "{}_({},{})",
        DEPLOY_NAME, deploy_params[0].ty, deploy_params[1].ty
    );
    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&digest[..4]);

    metadata.methods.push(FunctionMetadata {
        name: DEPLOY_NAME.to_string(),
        neo_name: DEPLOY_NAME.to_string(),
        kind: FunctionKind::Regular,
        parameters: deploy_params,
        return_parameters: vec![],
        state_mutability: StateMutability::NonPayable,
        visibility: VisibilityKind::Public,
        offset: 0,
        body: None,
        selector,
        is_virtual: false,
        is_override: false,
        documentation: NatspecDoc::default(),
        had_modifier_epilogue: false,
    });

    Ok(())
}
