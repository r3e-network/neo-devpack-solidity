use super::*;

pub(crate) fn try_lower_storage_array_helpers(
    func: &Expression,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    // Storage array helpers: support `arr.push(value)` and `arr.pop()`.
    if let Some(result) = try_lower_storage_reference_array_helpers(func, args, ctx, instructions) {
        return Some(result);
    }

    if let Some(result) = try_lower_state_array_helpers(func, args, ctx, instructions) {
        return Some(result);
    }

    None
}
