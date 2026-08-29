use super::*;

pub(crate) fn build_defined_struct_types(
    structs: &[crate::solidity::StructMetadata],
    enums: &[EnumMetadata],
    contract_types: &[String],
    type_aliases: &std::collections::HashMap<String, String>,
) -> Vec<ValueType> {
    use crate::type_system::{
        EnumTypeMetadata, NeoType, StructFieldMetadata as TypeStructFieldMetadata, StructFieldType,
        StructTypeMetadata,
    };

    let struct_type_info: Vec<StructTypeMetadata> = structs
        .iter()
        .map(|struct_meta| StructTypeMetadata {
            name: struct_meta.name.clone(),
            fields: struct_meta
                .fields
                .iter()
                .map(|field| TypeStructFieldMetadata {
                    name: field.name.clone(),
                    ty: field.ty.clone(),
                })
                .collect(),
        })
        .collect();

    let enum_type_info: Vec<EnumTypeMetadata> = enums
        .iter()
        .map(|enum_meta| EnumTypeMetadata {
            name: enum_meta.name.clone(),
            variants: enum_meta.values.len(),
        })
        .collect();

    structs
        .iter()
        .map(|struct_meta| {
            let mut fields = Vec::new();
            for field in &struct_meta.fields {
                // Resolve user-defined value-type aliases (`type Slot0 is bytes32;`)
                // when typing struct fields. Without alias-aware resolution,
                // a field declared as `Slot0 slot0` becomes `NeoType::Any`
                // (because "Slot0" isn't a built-in or registered struct/enum),
                // which then propagates to ValueType::Any. Downstream IR
                // lowering can't bind member-style calls against an Any
                // receiver (e.g. `state.slot0.sqrtPriceX96()` from V4
                // PoolManager), and the using-directive check reports
                // "is not available for receiver type 'Any'".
                let field_type = NeoType::from_solidity_with_aliases(
                    &field.ty,
                    &struct_type_info,
                    &enum_type_info,
                    contract_types,
                    type_aliases,
                )
                .unwrap_or(NeoType::Any);

                fields.push(StructFieldType {
                    name: field.name.clone(),
                    ty: Box::new(field_type),
                });
            }

            ValueType::from_neotype(&NeoType::Struct {
                name: struct_meta.name.clone(),
                fields,
            })
        })
        .collect()
}
