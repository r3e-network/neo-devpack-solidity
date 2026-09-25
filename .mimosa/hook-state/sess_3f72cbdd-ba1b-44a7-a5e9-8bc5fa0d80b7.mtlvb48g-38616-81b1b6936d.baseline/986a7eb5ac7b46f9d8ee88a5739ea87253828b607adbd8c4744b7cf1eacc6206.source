use super::*;

pub(crate) struct LoopLabels {
    pub(crate) continue_label: usize,
    pub(crate) break_label: usize,
}

pub(crate) struct MappingAccess<'a> {
    pub(crate) state_index: usize,
    pub(crate) key_expressions: Vec<&'a Expression>,
    pub(crate) key_types: Vec<ValueType>,
    pub(crate) value_type: ValueType,
}

#[derive(Clone)]
pub(crate) struct StorageReference {
    pub(crate) state_index: usize,
    pub(crate) key_expressions: Vec<Expression>,
    pub(crate) key_types: Vec<ValueType>,
    pub(crate) value_type: ValueType,
    pub(crate) field_path: Vec<StorageReferenceField>,
    // Task #82: inner-mapping key chain applied AFTER `field_path`.
    // Populated when the reference walks through a struct-field mapping such as
    // `slots[k].balances[a]`.
    pub(crate) trailing_key_expressions: Vec<Expression>,
    pub(crate) trailing_key_types: Vec<ValueType>,
}

#[derive(Clone)]
pub(crate) struct StorageReferenceField {
    pub(crate) key: [u8; 32],
    pub(crate) ty: ValueType,
}

impl MappingAccess<'_> {
    pub(crate) fn to_storage_reference(&self) -> StorageReference {
        StorageReference {
            state_index: self.state_index,
            key_expressions: self
                .key_expressions
                .iter()
                .map(|expr| (*expr).clone())
                .collect(),
            key_types: self.key_types.clone(),
            value_type: self.value_type.clone(),
            field_path: Vec::new(),
            trailing_key_expressions: Vec::new(),
            trailing_key_types: Vec::new(),
        }
    }
}
