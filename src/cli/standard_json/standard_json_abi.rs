use super::*;

/// Map a resolved type to its EVM JSON-ABI `(type, components)`. A struct
/// becomes `"tuple"` (or `"tuple[]"`/`"tuple[N]"`) carrying a recursively-built
/// `components` array; enums already resolve to `uint8`. Falls back to the raw
/// declared type string when the type did not resolve. Ethereum tooling
/// (ethers/web3/Foundry) requires this shape — a bare struct NAME with no
/// `components` cannot be ABI-decoded.
fn abi_type_and_components(
    neo_type: &Option<crate::type_system::NeoType>,
    ty_fallback: &str,
) -> (String, Option<Vec<Value>>) {
    fn recur(nt: &crate::type_system::NeoType) -> (String, Option<Vec<Value>>) {
        match nt {
            crate::type_system::NeoType::Struct { fields, .. } => {
                let components = fields
                    .iter()
                    .map(|field| {
                        let (ty, comps) = recur(&field.ty);
                        let mut obj = serde_json::Map::new();
                        obj.insert("name".into(), Value::String(field.name.clone()));
                        obj.insert("type".into(), Value::String(ty.clone()));
                        obj.insert("internalType".into(), Value::String(ty));
                        if let Some(comps) = comps {
                            obj.insert("components".into(), Value::Array(comps));
                        }
                        Value::Object(obj)
                    })
                    .collect();
                ("tuple".to_string(), Some(components))
            }
            crate::type_system::NeoType::Array(inner, len) => {
                let (inner_ty, inner_comps) = recur(inner);
                let suffix = match len {
                    Some(n) => format!("[{n}]"),
                    None => "[]".to_string(),
                };
                (format!("{inner_ty}{suffix}"), inner_comps)
            }
            other => (other.canonical_abi_type(), None),
        }
    }
    match neo_type {
        Some(nt) => recur(nt),
        None => (ty_fallback.to_string(), None),
    }
}

/// Build one EVM JSON-ABI parameter object (`name`, `type`, `internalType`,
/// optional `components`/`indexed`).
fn abi_param_value(
    name: String,
    neo_type: &Option<crate::type_system::NeoType>,
    ty_fallback: &str,
    indexed: Option<bool>,
) -> Value {
    let (abi_type, components) = abi_type_and_components(neo_type, ty_fallback);
    let mut obj = serde_json::Map::new();
    obj.insert("name".into(), Value::String(name));
    obj.insert("type".into(), Value::String(abi_type));
    obj.insert(
        "internalType".into(),
        Value::String(ty_fallback.to_string()),
    );
    if let Some(components) = components {
        obj.insert("components".into(), Value::Array(components));
    }
    if let Some(indexed) = indexed {
        obj.insert("indexed".into(), Value::Bool(indexed));
    }
    Value::Object(obj)
}

pub(crate) fn build_standard_abi(metadata: &ContractMetadata) -> Vec<Value> {
    let mut abi_entries = Vec::new();

    for method in &metadata.methods {
        if matches!(method.kind, FunctionKind::Regular)
            && !matches!(
                method.visibility,
                VisibilityKind::Public | VisibilityKind::External
            )
        {
            continue;
        }
        let inputs: Vec<Value> = method
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                abi_param_value(
                    parameter
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("arg{index}")),
                    &parameter.neo_type,
                    &parameter.ty,
                    None,
                )
            })
            .collect();

        let outputs: Vec<Value> = method
            .return_parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                abi_param_value(
                    parameter
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("ret{index}")),
                    &parameter.neo_type,
                    &parameter.ty,
                    None,
                )
            })
            .collect();

        match method.kind {
            FunctionKind::Constructor => {
                abi_entries.push(json!({
                    "type": "constructor",
                    "inputs": inputs,
                    "stateMutability": state_mutability_label(method.state_mutability),
                }));
            }
            FunctionKind::Regular => {
                abi_entries.push(json!({
                    "type": "function",
                    "name": method.name,
                    "inputs": inputs,
                    "outputs": outputs,
                    "stateMutability": state_mutability_label(method.state_mutability),
                }));
            }
        }
    }

    for event in &metadata.events {
        let inputs: Vec<Value> = event
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                abi_param_value(
                    parameter
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("param{index}")),
                    &parameter.neo_type,
                    &parameter.ty,
                    Some(parameter.indexed),
                )
            })
            .collect();

        abi_entries.push(json!({
            "type": "event",
            "name": event.name,
            "inputs": inputs,
            "anonymous": event.anonymous,
        }));
    }

    abi_entries
}

pub(crate) fn build_method_identifiers(metadata: &ContractMetadata) -> Map<String, Value> {
    let mut identifiers = Map::new();

    for method in &metadata.methods {
        if matches!(method.kind, FunctionKind::Constructor) {
            continue;
        }

        if !matches!(
            method.visibility,
            VisibilityKind::Public | VisibilityKind::External
        ) {
            continue;
        }

        // Build the signature KEY with the same struct/enum-expanding
        // canonicalizer that produced `method.selector` (structs -> tuple,
        // enums -> uint8), so `keccak256(key)[..4] == selector`. The
        // split-on-whitespace fallback (struct name verbatim) would key the
        // map under `f(MyStruct)` while the value is the `f((uint256,bool))`
        // selector — a hash mismatch for any SDK that recomputes it.
        let param_signatures: Vec<String> = method
            .parameters
            .iter()
            .map(|param| match &param.neo_type {
                Some(neo_type) => neo_type.canonical_abi_type(),
                None => canonical_param_type(&param.ty),
            })
            .collect();
        let signature = if param_signatures.is_empty() {
            format!("{}()", method.name)
        } else {
            format!("{}({})", method.name, param_signatures.join(","))
        };

        identifiers.insert(
            signature,
            Value::String(hex_prefixed(&hex::encode(method.selector))),
        );
    }

    identifiers
}

pub(crate) fn build_neo_method_map(metadata: &ContractMetadata) -> Map<String, Value> {
    let mut map = Map::new();

    let exposed: Vec<_> = metadata
        .methods
        .iter()
        .filter(|method| {
            !matches!(method.kind, FunctionKind::Constructor)
                && matches!(
                    method.visibility,
                    VisibilityKind::Public | VisibilityKind::External
                )
        })
        .collect();

    // Report the MANIFEST-visible name, not the raw internal `neo_name`:
    // distinct-arity overloads keep their original Solidity name in the
    // manifest, so the methodMap must point callers at the name that is
    // actually invokable on-chain.
    let abi_name = crate::manifest::manifest_abi_method_name_fn(exposed.iter().copied());

    for method in &exposed {
        let param_signature: Vec<String> = method
            .parameters
            .iter()
            .map(|param| match &param.neo_type {
                Some(neo_type) => neo_type.canonical_abi_type(),
                None => canonical_param_type(&param.ty),
            })
            .collect();
        let signature = format!("{}({})", method.name, param_signature.join(","));
        map.insert(signature, Value::String(abi_name(method)));
    }

    map
}
