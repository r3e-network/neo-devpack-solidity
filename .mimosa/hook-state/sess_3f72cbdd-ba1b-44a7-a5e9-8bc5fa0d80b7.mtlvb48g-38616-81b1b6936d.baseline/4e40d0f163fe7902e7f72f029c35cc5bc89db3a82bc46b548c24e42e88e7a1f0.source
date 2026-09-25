use super::*;

pub(crate) fn find_named_struct_type(value_type: &ValueType, name: &str) -> Option<ValueType> {
    pub(crate) fn last_segment(qualified: &str) -> &str {
        qualified
            .rsplit_once('.')
            .map_or(qualified, |(_, last)| last)
    }
    let lookup_short = last_segment(name);
    match value_type {
        ValueType::Struct {
            name: struct_name,
            fields,
        } => {
            // Accept either the exact qualified match or a match against
            // the last `.`-separated segment. `frontend_convert` qualifies
            // nested struct names with their containing scope (e.g. the
            // library `Pool`'s nested `struct SwapParams` becomes
            // `"Pool.SwapParams"`), but constructor calls and bare type
            // references inside the same scope still use the short name.
            // The exact-match branch wins when both qualified and short
            // forms are searched in turn, so callers that pass a qualified
            // name still disambiguate against duplicates.
            if struct_name.eq_ignore_ascii_case(name)
                || last_segment(struct_name).eq_ignore_ascii_case(lookup_short)
            {
                return Some(value_type.clone());
            }

            fields
                .iter()
                .find_map(|field| find_named_struct_type(&field.ty, name))
        }
        ValueType::Array(inner) => find_named_struct_type(inner, name),
        ValueType::Mapping { key, value } => {
            find_named_struct_type(key, name).or_else(|| find_named_struct_type(value, name))
        }
        _ => None,
    }
}
