use super::*;

// `canonical_param_type` lives in `crate::utils`; import it rather than
// duplicating the body (which used to drift in subtle ways, e.g. when
// struct/enum special handling was added).

pub(crate) fn compute_function_selector(name: &str, param_types: &[String]) -> [u8; 4] {
    if name.eq_ignore_ascii_case("constructor") {
        return [0u8; 4];
    }

    let signature = if param_types.is_empty() {
        format!("{name}()")
    } else {
        format!("{name}({})", param_types.join(","))
    };

    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    let digest = hasher.finalize();
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&digest[..4]);
    selector
}
