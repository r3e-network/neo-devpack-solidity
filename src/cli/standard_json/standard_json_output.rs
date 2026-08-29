use super::*;

pub(crate) fn build_compiled_contract_value(
    file_name: &str,
    artifact: &CompilationArtifacts,
    abi_entries: &[Value],
    settings: &Value,
    source_keccak: Option<&str>,
    nef_source: Option<&str>,
) -> Result<Value, String> {
    let raw_source = nef_source.unwrap_or(file_name);
    let (source_field, _) = clamp_nef_source_with_flag(raw_source);
    let script_hex = hex::encode(&artifact.bytecode);
    let bytecode_object = format!("0x{script_hex}");
    let metadata_blob = build_metadata_blob(
        &artifact.metadata.name,
        abi_entries,
        file_name,
        settings,
        source_keccak,
    );
    let method_identifiers = build_method_identifiers(&artifact.metadata);
    let neo_method_map = build_neo_method_map(&artifact.metadata);

    let storage_map = build_storage_map(&artifact.metadata);
    let manifest = artifact.manifest.clone();
    let tokens_json: Vec<Value> = artifact
        .tokens
        .iter()
        .map(|token| {
            let hash_be: Vec<u8> = token.hash.iter().rev().copied().collect();
            json!({
                "hash": format!("0x{}", hex::encode(hash_be)),
                "method": token.method.clone(),
                "paramcount": token.parameters_count,
                "hasreturnvalue": token.has_return_value,
                "callflags": token.call_flags,
            })
        })
        .collect();

    let nef_bytes = build_nef_with_tokens(
        &artifact.bytecode,
        COMPILER_ID,
        source_field.as_ref(),
        &artifact.tokens,
    )
    .map_err(|err| format!("failed to build NEF image for standard JSON output: {err}"))?;
    let checksum = if nef_bytes.len() >= 4 {
        hex::encode(&nef_bytes[nef_bytes.len() - 4..])
    } else {
        "00000000".to_string()
    };
    let nef_image = hex::encode(nef_bytes);

    Ok(json!({
        "abi": abi_entries,
        "metadata": metadata_blob,
        "evm": {
            "bytecode": {
                "object": bytecode_object,
                "opcodes": "",
                "sourceMap": "",
                "linkReferences": {}
            },
            "deployedBytecode": {
                "object": bytecode_object,
                "opcodes": "",
                "sourceMap": "",
                "linkReferences": {}
            },
            "methodIdentifiers": method_identifiers,
        },
        "neo": {
            "nef": {
                "magic": "NEF3",
                "compiler": COMPILER_ID,
                "source": source_field.as_ref(),
                "tokens": tokens_json,
                "script": script_hex,
                "image": nef_image,
                "checksum": checksum,
            },
            "manifest": manifest,
            "methodMap": neo_method_map,
            "storageMap": storage_map,
            "gasEstimates": {
                "creation": zero_gas_estimate_value(),
                "functions": Value::Object(Map::new())
            }
        }
    }))
}

pub(crate) fn build_metadata_blob(
    contract_name: &str,
    abi_entries: &[Value],
    file_name: &str,
    settings: &Value,
    keccak_hex: Option<&str>,
) -> String {
    let keccak_field = keccak_hex.map(hex_prefixed).unwrap_or_default();
    let metadata = json!({
        "compiler": {
            "name": COMPILER_ID,
            "version": super::compiler_version_string_4()
        },
        "language": "Solidity",
        "output": {
            "abi": abi_entries,
            "contractName": contract_name,
        },
        "settings": settings,
        "sources": {
            file_name: {
                "keccak256": keccak_field,
                "urls": []
            }
        },
        "version": 1
    });

    serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string())
}

pub(crate) fn build_storage_map(metadata: &ContractMetadata) -> Value {
    let mut entries = Map::new();
    // Constant state variables are inlined and do not occupy storage.
    for (slot, variable) in metadata
        .state_variables
        .iter()
        .filter(|var| !var.is_constant)
        .enumerate()
    {
        let name = variable
            .name
            .clone()
            .unwrap_or_else(|| format!("slot_{slot}"));
        entries.insert(
            name,
            json!({
                "slot": slot,
                "type": variable.ty,
                "description": variable.visibility.clone().unwrap_or_default(),
            }),
        );
    }
    Value::Object(entries)
}

pub(crate) fn zero_gas_estimate_value() -> Value {
    json!({
        "gas": "0",
        "systemFee": "0",
        "networkFee": "0",
    })
}

pub(crate) fn state_mutability_label(state: StateMutability) -> &'static str {
    match state {
        StateMutability::Pure => "pure",
        StateMutability::View => "view",
        StateMutability::Payable => "payable",
        StateMutability::NonPayable => "nonpayable",
    }
}

pub(crate) fn sanitize_contract_name(name: &str) -> Option<String> {
    let filtered: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if filtered.trim_matches('_').is_empty() {
        None
    } else {
        Some(filtered.trim_matches('_').to_string())
    }
}
