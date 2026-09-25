use super::*;

/// Lower `Neo.*` member calls (from the `Neo.sol` DevPack library).
///
/// These operations mirror the `NativeCalls.*` equivalents in
/// `member_nativecalls/neo.rs` but use `"Neo."` in diagnostic messages.
/// The shared ECPoint→address conversion helpers live in `member_nativecalls`
/// so both call paths produce identical bytecode.
pub(crate) fn try_lower_neo_member_builtin(
    base: &Identifier,
    member: &Identifier,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if base.name != "Neo" {
        return None;
    }

    match member.name.as_str() {
        "isCommittee" => {
            if args.len() != 1 {
                ctx.record_error(format!(
                    "Neo.isCommittee requires 1 argument(s), got {}",
                    args.len()
                ));
                return Some(false);
            }
            // Lower the account argument, then emit the shared ECPoint membership check.
            if !lower_expression(&args[0], ctx, instructions) {
                return Some(false);
            }
            emit_ecpoint_membership_check(ctx, "getCommittee", "neo_is_committee", instructions);
            Some(true)
        }
        "getCommittee" => {
            if !args.is_empty() {
                ctx.record_error(format!(
                    "Neo.getCommittee requires 0 argument(s), got {}",
                    args.len()
                ));
                return Some(false);
            }
            emit_ecpoint_to_address_conversion(ctx, "getCommittee", "neo_committee", instructions);
            Some(true)
        }
        "getValidators" => {
            if !args.is_empty() {
                ctx.record_error(format!(
                    "Neo.getValidators requires 0 argument(s), got {}",
                    args.len()
                ));
                return Some(false);
            }
            emit_ecpoint_to_address_conversion(
                ctx,
                "getNextBlockValidators",
                "neo_validator",
                instructions,
            );
            Some(true)
        }
        "isValidator" => {
            if args.len() != 1 {
                ctx.record_error(format!(
                    "Neo.isValidator requires 1 argument(s), got {}",
                    args.len()
                ));
                return Some(false);
            }
            if !lower_expression(&args[0], ctx, instructions) {
                return Some(false);
            }
            emit_ecpoint_membership_check(
                ctx,
                "getNextBlockValidators",
                "neo_is_validator",
                instructions,
            );
            Some(true)
        }
        _ => None,
    }
}
