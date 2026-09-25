//! Native contract call lowering — dispatch + shared helpers.
//!
//! This module handles lowering of Solidity `NativeCalls.*` member calls
//! to NeoVM bytecode. The actual lowering logic for each native contract
//! operation lives in domain-specific sub-modules.
//!
//! Layout:
//! - mod.rs     — dispatch, structs, constants, shared helpers (this file)
//! - neo.rs     — NEO governance: getCommittee, isCommittee, getNextBlockValidators, isValidator
//! - utility.rs — identification: isNativeContract, getNativeContractName, getAllNativeContracts, getNativeContractManifest
//! - contract_state.rs — ContractManagement: getContractById
//! - network_config.rs — Policy/Oracle: getNetworkConfiguration
//! - call_ops.rs — generic call wrappers: estimateNativeCallGas, batchNativeCalls, safeNativeCall

#![allow(non_snake_case)]

pub(crate) use super::*;

mod call_ops;
mod contract_state;
mod neo;
mod network_config;
mod utility;

use crate::ir::ir_context::DiagnosticContext;
use call_ops::*;
use contract_state::*;
use neo::*;
use network_config::*;
use utility::*;

// Native Contract Call Handling
//
// This module handles lowering of Solidity native contract calls to NeoVM bytecode.
// See the source code for full implementation of NeoToken, GasToken, ContractManagement,
// PolicyContract, OracleContract, RoleManagement, Notary, Treasury, LedgerContract,
// CryptoLib, and StdLib native contract operations.
//
// Note: This file is large (~1300 lines) due to the complexity of native contract handling.
// Consider splitting by contract type in future refactoring.

pub(crate) struct NativeContractDescriptor {
    hash: [u8; 20],
    name: &'static str,
}

/// Record the standard "`<prefix>.<name>` requires N argument(s), got M"
/// diagnostic when the actual arg count disagrees, and return `Some(false)`
/// so callers can early-return. The function returns `Some(false)` (not
/// `None` / `Option<bool>`) so it composes with the existing
/// `return Some(false)` shape used by every native-call lowering site
/// without changing the outer match signature.
///
/// `caller_prefix` is appended as `"<caller_prefix>.<name>"` in the diagnostic
/// (e.g. `"NativeCalls"` → `"NativeCalls.getCommittee requires …"`,
///        `"Neo"` → `"Neo.isCommittee requires …"`).
pub(crate) fn check_arg_count(
    ctx: &mut impl DiagnosticContext,
    caller_prefix: &str,
    name: &str,
    args: &[Expression],
    expected: usize,
) -> Option<bool> {
    if args.len() != expected {
        ctx.record_error(format!(
            "{caller_prefix}.{name} requires {expected} argument(s), got {}",
            args.len()
        ));
        return Some(false);
    }
    None
}

pub(crate) const NATIVE_CONTRACTS: [NativeContractDescriptor; 11] = [
    NativeContractDescriptor {
        hash: [
            0xf5, 0x63, 0xea, 0x40, 0xbc, 0x28, 0x3d, 0x4d, 0x0e, 0x05, 0xc4, 0x8e, 0xa3, 0x05,
            0xb3, 0xf2, 0xa0, 0x73, 0x40, 0xef,
        ],
        name: "NeoToken",
    },
    NativeContractDescriptor {
        hash: [
            0xcf, 0x76, 0xe2, 0x8b, 0xd0, 0x06, 0x2c, 0x4a, 0x47, 0x8e, 0xe3, 0x55, 0x61, 0x01,
            0x13, 0x19, 0xf3, 0xcf, 0xa4, 0xd2,
        ],
        name: "GasToken",
    },
    NativeContractDescriptor {
        hash: [
            0xfd, 0xa3, 0xfa, 0x43, 0x46, 0xea, 0x53, 0x2a, 0x25, 0x8f, 0xc4, 0x97, 0xdd, 0xad,
            0xdb, 0x64, 0x37, 0xc9, 0xfd, 0xff,
        ],
        name: "ContractManagement",
    },
    NativeContractDescriptor {
        hash: [
            0x7b, 0xc6, 0x81, 0xc0, 0xa1, 0xf7, 0x1d, 0x54, 0x34, 0x57, 0xb6, 0x8b, 0xba, 0x8d,
            0x5f, 0x9f, 0xdd, 0x4e, 0x5e, 0xcc,
        ],
        name: "PolicyContract",
    },
    NativeContractDescriptor {
        hash: [
            0x58, 0x87, 0x17, 0x11, 0x7e, 0x0a, 0xa8, 0x10, 0x72, 0xaf, 0xab, 0x71, 0xd2, 0xdd,
            0x89, 0xfe, 0x7c, 0x4b, 0x92, 0xfe,
        ],
        name: "OracleContract",
    },
    NativeContractDescriptor {
        hash: [
            0xe2, 0x95, 0xe3, 0x91, 0x54, 0x4c, 0x17, 0x8a, 0xd9, 0x4f, 0x03, 0xec, 0x4d, 0xcd,
            0xff, 0x78, 0x53, 0x4e, 0xcf, 0x49,
        ],
        name: "RoleManagement",
    },
    NativeContractDescriptor {
        hash: [
            0x3b, 0xec, 0x35, 0x31, 0x11, 0x9b, 0xba, 0xd7, 0x6d, 0xd0, 0x44, 0x92, 0x0b, 0x0d,
            0xe6, 0xc3, 0x19, 0x4f, 0xe1, 0xc1,
        ],
        name: "Notary",
    },
    NativeContractDescriptor {
        hash: [
            0xc1, 0x3a, 0x56, 0xc9, 0x83, 0x53, 0xa7, 0xea, 0x6a, 0x32, 0x4d, 0x9a, 0x83, 0x5d,
            0x1b, 0x5b, 0xf2, 0x26, 0x63, 0x15,
        ],
        name: "Treasury",
    },
    NativeContractDescriptor {
        hash: [
            0xbe, 0xf2, 0x04, 0x31, 0x40, 0x36, 0x2a, 0x77, 0xc1, 0x50, 0x99, 0xc7, 0xe6, 0x4c,
            0x12, 0xf7, 0x00, 0xb6, 0x65, 0xda,
        ],
        name: "LedgerContract",
    },
    NativeContractDescriptor {
        hash: [
            0x1b, 0xf5, 0x75, 0xab, 0x11, 0x89, 0x68, 0x84, 0x13, 0x61, 0x0a, 0x35, 0xa1, 0x28,
            0x86, 0xcd, 0xe0, 0xb6, 0x6c, 0x72,
        ],
        name: "CryptoLib",
    },
    NativeContractDescriptor {
        hash: [
            0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25, 0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1,
            0x44, 0x0d, 0xd8, 0x6f, 0xce, 0xac,
        ],
        name: "StdLib",
    },
];

pub(crate) fn emit_throw_with_message(instructions: &mut Vec<Instruction>, message: &str) {
    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        message.as_bytes().to_vec(),
    )));
    instructions.push(Instruction::Throw);
}

/// Emit instructions to call a Neo native method returning ECPoint[] and convert
/// each public key to a UInt160 address via `System.Contract.CreateStandardAccount`.
/// On exit, leaves `address[]` (ValueType::Address) on the evaluation stack.
pub(crate) fn emit_ecpoint_to_address_conversion(
    ctx: &mut LoweringContext,
    native_method: &str,
    slot_prefix: &str,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let keys_slot = ctx.allocate_local(
        format!("__{slot_prefix}_keys_{tmp_id}"),
        Some(ValueType::Any),
    );
    let addrs_slot = ctx.allocate_local(
        format!("__{slot_prefix}_addrs_{tmp_id}"),
        Some(ValueType::Array(Box::new(ValueType::Address))),
    );
    let index_slot = ctx.allocate_local(
        format!("__{slot_prefix}_index_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: native_method.to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::StoreLocal(keys_slot));

    instructions.push(Instruction::LoadLocal(keys_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::NewArray {
        element_type: ValueType::Address,
    });
    instructions.push(Instruction::StoreLocal(addrs_slot));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(index_slot));

    let loop_label = ctx.next_label();
    let done_label = ctx.next_label();
    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::LoadLocal(keys_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });

    // addrs[index] = CreateStandardAccount(keys[index])
    instructions.push(Instruction::LoadLocal(addrs_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::LoadLocal(keys_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::Syscall("System.Contract.CreateStandardAccount".to_string()),
        arg_count: 1,
    });
    instructions.push(Instruction::ArraySet);

    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(index_slot));
    instructions.push(Instruction::Jump { target: loop_label });

    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::LoadLocal(addrs_slot));
}

/// Emit instructions for a membership check: test whether an account (already
/// on the evaluation stack as a UInt160 address) belongs to the set of ECPoint
/// keys returned by a Neo native method, after converting each key via
/// `System.Contract.CreateStandardAccount`.
///
/// Caller is responsible for lowering the account argument and pushing it onto
/// the stack *before* calling this function.
///
/// On exit, leaves `bool` on the evaluation stack.
pub(crate) fn emit_ecpoint_membership_check(
    ctx: &mut LoweringContext,
    native_method: &str,
    slot_prefix: &str,
    instructions: &mut Vec<Instruction>,
) {
    let tmp_id = ctx.next_label();
    let account_slot = ctx.allocate_local(
        format!("__{slot_prefix}_account_{tmp_id}"),
        Some(ValueType::Address),
    );
    let keys_slot = ctx.allocate_local(
        format!("__{slot_prefix}_keys_{tmp_id}"),
        Some(ValueType::Any),
    );
    let index_slot = ctx.allocate_local(
        format!("__{slot_prefix}_index_{tmp_id}"),
        Some(ValueType::Integer {
            signed: false,
            bits: 256,
        }),
    );

    instructions.push(Instruction::StoreLocal(account_slot));

    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: native_method.to_string(),
        },
        arg_count: 0,
    });
    instructions.push(Instruction::StoreLocal(keys_slot));

    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::zero(),
    )));
    instructions.push(Instruction::StoreLocal(index_slot));

    let loop_label = ctx.next_label();
    let advance_label = ctx.next_label();
    let done_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(loop_label));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::LoadLocal(keys_slot));
    instructions.push(Instruction::GetSize);
    instructions.push(Instruction::BinaryOp(BinaryOperator::Lt));
    instructions.push(Instruction::JumpIf { target: done_label });

    instructions.push(Instruction::LoadLocal(keys_slot));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::ArrayGet);
    instructions.push(Instruction::CallBuiltin {
        builtin: BuiltinCall::Syscall("System.Contract.CreateStandardAccount".to_string()),
        arg_count: 1,
    });
    instructions.push(Instruction::LoadLocal(account_slot));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
    instructions.push(Instruction::JumpIf {
        target: advance_label,
    });

    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
    instructions.push(Instruction::Jump { target: end_label });

    instructions.push(Instruction::Label(advance_label));
    instructions.push(Instruction::LoadLocal(index_slot));
    instructions.push(Instruction::PushLiteral(LiteralValue::Integer(
        BigInt::one(),
    )));
    instructions.push(Instruction::BinaryOp(BinaryOperator::Add));
    instructions.push(Instruction::StoreLocal(index_slot));
    instructions.push(Instruction::Jump { target: loop_label });

    instructions.push(Instruction::Label(done_label));
    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
    instructions.push(Instruction::Label(end_label));
}

pub(crate) fn emit_is_native_contract_check(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    contract_slot: usize,
) {
    let done_label = ctx.next_label();

    for contract in NATIVE_CONTRACTS.iter() {
        let next_label = ctx.next_label();
        instructions.push(Instruction::LoadLocal(contract_slot));
        instructions.push(Instruction::PushLiteral(LiteralValue::Address(
            contract.hash.to_vec(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
        instructions.push(Instruction::JumpIf { target: next_label });
        instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(true)));
        instructions.push(Instruction::Jump { target: done_label });
        instructions.push(Instruction::Label(next_label));
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::Boolean(false)));
    instructions.push(Instruction::Label(done_label));
}

pub(crate) fn emit_native_contract_name(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
    contract_slot: usize,
) {
    let done_label = ctx.next_label();

    for contract in NATIVE_CONTRACTS.iter() {
        let next_label = ctx.next_label();
        instructions.push(Instruction::LoadLocal(contract_slot));
        instructions.push(Instruction::PushLiteral(LiteralValue::Address(
            contract.hash.to_vec(),
        )));
        instructions.push(Instruction::BinaryOp(BinaryOperator::Eq));
        instructions.push(Instruction::JumpIf { target: next_label });
        instructions.push(Instruction::PushLiteral(LiteralValue::String(
            contract.name.as_bytes().to_vec(),
        )));
        instructions.push(Instruction::Jump { target: done_label });
        instructions.push(Instruction::Label(next_label));
    }

    instructions.push(Instruction::PushLiteral(LiteralValue::String(
        b"Unknown".to_vec(),
    )));
    instructions.push(Instruction::Label(done_label));
}

pub(crate) fn try_lower_nativecalls_member_builtin(
    base: &Identifier,
    member: &Identifier,
    args: &[Expression],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    if base.name != "NativeCalls" {
        return None;
    }

    match member.name.as_str() {
        "getCommittee" => lower_native_getCommittee(ctx, args, instructions),
        "isCommittee" => lower_native_isCommittee(ctx, args, instructions),
        "getNextBlockValidators" => lower_native_getNextBlockValidators(ctx, args, instructions),
        "isValidator" => lower_native_isValidator(ctx, args, instructions),
        "isNativeContract" => lower_native_isNativeContract(ctx, args, instructions),
        "getNativeContractName" => lower_native_getNativeContractName(ctx, args, instructions),
        "getAllNativeContracts" => lower_native_getAllNativeContracts(ctx, args, instructions),
        "getNativeContractManifest" => {
            lower_native_getNativeContractManifest(ctx, args, instructions)
        }
        "getContractById" => lower_native_getContractById(ctx, args, instructions),
        "getNetworkConfiguration" => lower_native_getNetworkConfiguration(ctx, args, instructions),
        "estimateNativeCallGas" => lower_native_estimateNativeCallGas(ctx, args, instructions),
        "batchNativeCalls" => lower_native_batchNativeCalls(ctx, args, instructions),
        "safeNativeCall" => lower_native_safeNativeCall(ctx, args, instructions),
        _ => None,
    }
}
