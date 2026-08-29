use super::*;

pub(crate) fn validate_events(metadata: &ContractMetadata, diagnostics: &mut Vec<Diagnostic>) {
    use std::collections::{BTreeSet, HashMap};

    let mut event_groups: HashMap<String, BTreeSet<String>> = HashMap::new();

    for event in &metadata.events {
        event_groups
            .entry(event.name.trim().to_ascii_lowercase())
            .or_default()
            .insert(event.normalized_name.clone());

        if event.parameters.len() > 16 {
            diagnostics.push(
                Diagnostic::warning(format!(
                    "event '{}' has {} parameters which exceeds Neo ABI limits",
                    event.name,
                    event.parameters.len()
                ))
                .with_code("EVENT_PARAM_LIMIT"),
            );
        }
    }

    // Neo N3 ABI dispatches events by name only, so event overloading is not supported.
    // Reject multiple distinct signatures that share the same event name.
    for (event_name, signatures) in event_groups {
        if signatures.len() > 1 {
            let mut sigs: Vec<_> = signatures.into_iter().collect();
            sigs.sort();
            diagnostics.push(Diagnostic::error(format!(
                "event '{event_name}' is overloaded ({} distinct signatures): {}. Neo N3 requires unique event names",
                sigs.len(),
                sigs.join(", ")
            )));
        }
    }
}
