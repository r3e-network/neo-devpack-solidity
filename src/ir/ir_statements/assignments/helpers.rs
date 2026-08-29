use super::*;

pub(crate) fn find_struct_field<'a>(
    value_type: &'a ValueType,
    field_name: &str,
) -> Option<&'a StructField> {
    match value_type {
        ValueType::Struct { fields, .. } => fields.iter().find(|field| field.name == field_name),
        _ => None,
    }
}
