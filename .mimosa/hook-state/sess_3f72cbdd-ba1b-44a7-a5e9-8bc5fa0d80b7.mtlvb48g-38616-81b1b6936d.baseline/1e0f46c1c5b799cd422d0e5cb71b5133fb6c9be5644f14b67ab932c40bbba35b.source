//! Neo — Neo N3 native contract neo operations
//!
//! Extracted from member_nativecalls.rs for maintainability.

#![allow(non_snake_case)]

use super::*;

pub(crate) fn lower_native_getCommittee(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getCommittee", args, 0) {
        return Some(false);
    }

    emit_ecpoint_to_address_conversion(ctx, "getCommittee", "committee", instructions);
    Some(true)
}

pub(crate) fn lower_native_isCommittee(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "isCommittee", args, 1) {
        return Some(false);
    }

    // Neo native contract exposes committee members as ECPoint public keys.
    // Lower the account argument, then run the ECPoint→address membership check.
    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    emit_ecpoint_membership_check(ctx, "getCommittee", "is_committee", instructions);
    Some(true)
}

pub(crate) fn lower_native_getNextBlockValidators(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "getNextBlockValidators", args, 0) {
        return Some(false);
    }

    emit_ecpoint_to_address_conversion(ctx, "getNextBlockValidators", "validator", instructions);
    Some(true)
}

pub(crate) fn lower_native_isValidator(
    ctx: &mut LoweringContext,
    args: &[Expression],
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if let Some(false) = check_arg_count(ctx, "NativeCalls", "isValidator", args, 1) {
        return Some(false);
    }

    if !lower_expression(&args[0], ctx, instructions) {
        return Some(false);
    }
    emit_ecpoint_membership_check(ctx, "getNextBlockValidators", "is_validator", instructions);
    Some(true)
}
