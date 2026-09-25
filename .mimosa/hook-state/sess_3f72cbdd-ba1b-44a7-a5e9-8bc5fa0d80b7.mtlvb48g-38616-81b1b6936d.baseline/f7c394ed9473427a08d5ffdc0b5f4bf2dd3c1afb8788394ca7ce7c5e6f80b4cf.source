use super::*;

pub(crate) fn convert_event(
    event: EventIR,
    struct_types: &[StructTypeMetadata],
    enum_types: &[EnumTypeMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
) -> EventMetadata {
    let normalized = normalize_event_signature(&event.name, &event.parameters);
    EventMetadata {
        name: event.name,
        normalized_name: normalized,
        parameters: event
            .parameters
            .into_iter()
            .map(|param| {
                let ty = param.ty;
                let neo_type = NeoType::from_solidity_with_aliases(
                    &ty,
                    struct_types,
                    enum_types,
                    contract_types,
                    type_aliases,
                )
                .ok();

                EventParameter {
                    name: param.name,
                    ty,
                    indexed: param.indexed,
                    neo_type,
                }
            })
            .collect(),
        anonymous: event.anonymous,
    }
}

pub(crate) fn normalize_event_signature(
    name: &str,
    params: &[crate::frontend::EventParameterIR],
) -> String {
    let base = name.trim().to_ascii_lowercase();
    let param_types: Vec<String> = params
        .iter()
        .map(|param| crate::utils::canonical_param_type(&param.ty).to_ascii_lowercase())
        .collect();

    // `event Foo` with no parameters is valid Solidity syntax (treated as Foo()).
    if param_types.is_empty() {
        format!("{base}()")
    } else {
        // Preserve the declared order; Neo ABI dispatches events by name only,
        // so we use this signature only for de-duplication and validation.
        format!("{base}({})", param_types.join(","))
    }
}
