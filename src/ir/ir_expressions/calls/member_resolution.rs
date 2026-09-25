use super::*;

pub(crate) fn format_builtin_member_list(members: &[&str]) -> String {
    const MAX_SHOWN: usize = 12;
    if members.len() <= MAX_SHOWN {
        return members.join(", ");
    }

    format!(
        "{} … (+{} more)",
        members[..MAX_SHOWN].join(", "),
        members.len() - MAX_SHOWN
    )
}

pub(crate) fn resolve_static_library_base(
    inner: &Expression,
    ctx: &LoweringContext,
) -> Option<String> {
    match inner {
        Expression::Variable(lib_id)
            if !ctx.param_index_map.contains_key(&lib_id.name)
                && ctx.resolve_local(&lib_id.name).is_none()
                && !ctx.state_index_map.contains_key(&lib_id.name) =>
        {
            Some(lib_id.name.clone())
        }
        Expression::MemberAccess(_, namespace_expr, imported_symbol)
            if matches!(
                namespace_expr.as_ref(),
                Expression::Variable(namespace_id)
                    if !ctx.param_index_map.contains_key(&namespace_id.name)
                        && ctx.resolve_local(&namespace_id.name).is_none()
                        && !ctx.state_index_map.contains_key(&namespace_id.name)
                        && !ctx.is_contract_type_name(&namespace_id.name)
            ) && ctx.is_contract_type_name(&imported_symbol.name) =>
        {
            Some(imported_symbol.name.clone())
        }
        _ => None,
    }
}

pub(crate) fn resolve_contract_type_name(
    inner: &Expression,
    ctx: &LoweringContext,
) -> Option<String> {
    match inner {
        Expression::Variable(type_id) if ctx.is_contract_type_name(&type_id.name) => {
            Some(type_id.name.clone())
        }
        Expression::MemberAccess(_, namespace_expr, type_id)
            if matches!(
                namespace_expr.as_ref(),
                Expression::Variable(namespace_id)
                    if !ctx.param_index_map.contains_key(&namespace_id.name)
                        && ctx.resolve_local(&namespace_id.name).is_none()
                        && !ctx.state_index_map.contains_key(&namespace_id.name)
                        && !ctx.is_contract_type_name(&namespace_id.name)
            ) && ctx.is_contract_type_name(&type_id.name) =>
        {
            Some(type_id.name.clone())
        }
        _ => None,
    }
}

pub(crate) fn native_contract_from_constant(
    base: &str,
    constant: &str,
) -> Option<NativeContract> {
    if !matches!(base, "NativeCalls" | "NativeContracts") {
        return None;
    }

    match constant {
        "NEO_CONTRACT" => Some(NativeContract::Neo),
        "GAS_CONTRACT" => Some(NativeContract::Gas),
        "CONTRACT_MANAGEMENT" => Some(NativeContract::ContractManagement),
        "POLICY_CONTRACT" => Some(NativeContract::Policy),
        "ORACLE_CONTRACT" => Some(NativeContract::Oracle),
        "ROLE_MANAGEMENT" => Some(NativeContract::RoleManagement),
        "NOTARY_CONTRACT" => Some(NativeContract::Notary),
        "TREASURY_CONTRACT" => Some(NativeContract::Treasury),
        "LEDGER_CONTRACT" => Some(NativeContract::Ledger),
        "CRYPTO_LIB" => Some(NativeContract::CryptoLib),
        "STD_LIB" => Some(NativeContract::StdLib),
        _ => None,
    }
}
