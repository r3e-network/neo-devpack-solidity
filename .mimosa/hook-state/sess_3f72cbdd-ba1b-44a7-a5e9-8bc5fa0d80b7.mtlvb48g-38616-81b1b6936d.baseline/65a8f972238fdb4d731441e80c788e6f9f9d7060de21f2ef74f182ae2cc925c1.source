/// Normalize a state-variable type string for the sibling-merge collision
/// check so that equivalent spellings compare equal: whitespace is dropped
/// and the bare `uint`/`int` aliases expand to their canonical 256-bit
/// forms (`uint256[3]` == `uint [3]` == `uint256 [ 3 ]`,
/// `mapping(address=>uint)` == `mapping(address => uint256)`).
pub(crate) fn normalize_state_type_for_merge(ty: &str) -> String {
    let mut out = String::with_capacity(ty.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if word.is_empty() {
            return;
        }
        match word.as_str() {
            "uint" => out.push_str("uint256"),
            "int" => out.push_str("int256"),
            other => out.push_str(other),
        }
        word.clear();
    };
    for ch in ty.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '$' {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
            if !ch.is_whitespace() {
                out.push(ch);
            }
        }
    }
    flush(&mut word, &mut out);
    out
}
