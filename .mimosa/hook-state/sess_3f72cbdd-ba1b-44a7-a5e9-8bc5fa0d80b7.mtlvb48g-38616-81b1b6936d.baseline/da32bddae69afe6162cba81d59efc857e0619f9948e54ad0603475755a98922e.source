use super::*;

impl StateVariable {
    pub(crate) fn from_metadata(symbol: &StateVariableMetadata) -> Self {
        let storage_key = symbol
            .name
            .as_deref()
            .map(|name| compute_state_slot(name).to_vec())
            .unwrap_or_default();
        Self {
            name: symbol.name.clone(),
            ty: ValueType::from_neotype(
                &symbol
                    .neo_type
                    .clone()
                    .unwrap_or(crate::type_system::NeoType::Any),
            ),
            is_constant: symbol.is_constant,
            is_immutable: symbol.is_immutable,
            storage_key,
        }
    }
}

impl Event {
    pub(crate) fn from_metadata(event: &crate::solidity::EventMetadata) -> Self {
        Self {
            name: event.name.clone(),
        }
    }
}

impl ValueType {
    pub(crate) fn from_neotype(neotype: &crate::type_system::NeoType) -> Self {
        match neotype {
            crate::type_system::NeoType::Integer { signed, bits } => ValueType::Integer {
                signed: *signed,
                bits: *bits,
            },
            crate::type_system::NeoType::Boolean => ValueType::Boolean,
            crate::type_system::NeoType::String => ValueType::String,
            crate::type_system::NeoType::Address => ValueType::Address,
            crate::type_system::NeoType::ByteArray { fixed_len } => ValueType::ByteArray {
                fixed_len: *fixed_len,
            },
            crate::type_system::NeoType::Array(element, _) => {
                ValueType::Array(Box::new(ValueType::from_neotype(element)))
            }
            crate::type_system::NeoType::Mapping { key, value } => ValueType::Mapping {
                key: Box::new(ValueType::from_neotype(key.as_ref())),
                value: Box::new(ValueType::from_neotype(value.as_ref())),
            },
            crate::type_system::NeoType::Struct { name, fields } => ValueType::Struct {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|field| StructField {
                        name: field.name.clone(),
                        ty: ValueType::from_neotype(field.ty.as_ref()),
                        key: compute_state_slot(&format!("{}::{}", name, field.name)),
                    })
                    .collect(),
            },
            crate::type_system::NeoType::Any => ValueType::Any,
        }
    }

    pub(crate) fn from_parameter(param: &ParameterMetadata) -> Self {
        match &param.neo_type {
            Some(neo_type) => ValueType::from_neotype(neo_type),
            None => ValueType::Any,
        }
    }
}
