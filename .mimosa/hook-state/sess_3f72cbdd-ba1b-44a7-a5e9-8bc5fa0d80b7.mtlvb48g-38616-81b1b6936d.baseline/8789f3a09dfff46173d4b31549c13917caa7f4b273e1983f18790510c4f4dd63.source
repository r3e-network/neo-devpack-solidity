use super::*;

pub(crate) fn convert_struct(ir: StructIR) -> StructMetadata {
    let fields = ir
        .fields
        .into_iter()
        .map(|field| StructFieldMetadata {
            name: field.name,
            ty: field.ty,
        })
        .collect();

    StructMetadata {
        name: ir.name,
        fields,
    }
}

pub(crate) fn convert_enum(ir: EnumIR) -> EnumMetadata {
    EnumMetadata {
        name: ir.name,
        values: ir.values,
    }
}

pub(crate) fn structs_to_type_metadata(structs: &[StructMetadata]) -> Vec<StructTypeMetadata> {
    structs
        .iter()
        .map(|s| StructTypeMetadata {
            name: s.name.clone(),
            fields: s
                .fields
                .iter()
                .map(|f| NeoStructFieldMetadata {
                    name: f.name.clone(),
                    ty: f.ty.clone(),
                })
                .collect(),
        })
        .collect()
}

pub(crate) fn enums_to_type_metadata(enums: &[EnumMetadata]) -> Vec<EnumTypeMetadata> {
    enums
        .iter()
        .map(|enum_meta| EnumTypeMetadata {
            name: enum_meta.name.clone(),
            variants: enum_meta.values.len(),
        })
        .collect()
}
