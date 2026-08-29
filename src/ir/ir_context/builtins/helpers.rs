pub(crate) fn is_low_level_evm_member(name: &str) -> bool {
    matches!(name, "call" | "delegatecall" | "staticcall" | "callcode")
}
