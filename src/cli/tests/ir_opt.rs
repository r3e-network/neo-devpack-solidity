use super::*;
use neo_devpack_solidity::ir::{
    BasicBlock, Function, FunctionKind as IrFunctionKind, Instruction, Module, ValueType,
};
use neo_devpack_solidity::solidity::{
    ContractMetadata, FunctionKind, FunctionMetadata, NatspecDoc,
};

include!("ir_opt/control_flow.rs");
include!("ir_opt/bytecode_peephole.rs");
include!("ir_opt/neovm_simplify.rs");
