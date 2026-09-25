use super::*;

pub(crate) fn build_parameter_index_map(metadata: &FunctionMetadata) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (index, param) in metadata.parameters.iter().enumerate() {
        if let Some(name) = &param.name {
            map.insert(name.clone(), index);
        }
    }
    map
}
