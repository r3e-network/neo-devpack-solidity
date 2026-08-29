use super::*;

pub(crate) fn build_method_name_counts(
    metadata: &ContractMetadata,
) -> std::collections::HashMap<String, usize> {
    use std::collections::HashMap;

    // Track method names to detect conflicts with public state variable getters.
    let mut method_name_counts: HashMap<String, usize> = HashMap::new();
    for function in &metadata.methods {
        if matches!(function.kind, FunctionKind::Constructor) {
            continue;
        }
        *method_name_counts.entry(function.name.clone()).or_insert(0) += 1;
    }

    method_name_counts
}
