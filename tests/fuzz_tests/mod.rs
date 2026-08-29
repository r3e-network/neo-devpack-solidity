//! Comprehensive Fuzz Tests for Neo DevPack for Solidity
//!
//! Split into submodules for maintainability. See `tests/fuzz_tests/` for
//! per-category sources.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_match)]
#![allow(clippy::partialeq_to_none)]
#![allow(clippy::err_expect)]
#![allow(clippy::implicit_clone)]
#![allow(clippy::len_zero)]
#![allow(clippy::manual_contains)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::manual_repeat_n)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::needless_return)]
#![allow(clippy::op_ref)]
#![allow(clippy::type_complexity)]
#![allow(clippy::useless_format)]
#![allow(clippy::useless_vec)]

mod abi_roundtrip_props;

mod arithmetic_helpers_props;

mod arithmetic_props;

mod baseline_tests;

mod batches_100_105;

mod batches_106_110;

mod batches_111_115;

mod batches_116_120;

mod batches_18_30;

mod batches_31_45;

mod batches_46_64;

mod batches_66_80;

mod batches_81_90;

mod batches_91_100;

mod common;

mod compile_runtime_roundtrip;

mod compiler_props;

mod conditional_jumps;

mod constant_immutable_gap_props;

mod constructor_lifecycle_props;

mod contract_upgrade_props;

mod convergence_props;

mod custom_error_envelope_props;

mod determinism_props;

mod devpack_props;

mod diagnostic_stability_props;

mod differential;

mod disasm_stability_props;

mod erc1155_proxy_props;

mod examples_smoke_props;

mod fallback_receive_props;

mod in_contract_array_return_props;

mod library_deployment_props;

mod modifier_rewrite_props;

mod multi_source_compile_props;

mod native_contract_props;

mod native_resolver_props;

mod openzeppelin_patterns_props;

mod optimizer_props;

mod pathological_corpus_smoke;

mod performance_regression;

mod reentrancy_props;

mod stdlib_native_props;

mod storage_iterator_stress;

mod storage_props;

mod storage_state_machine;

mod task107_catch_panic_tests;

mod uint256_conformance;
