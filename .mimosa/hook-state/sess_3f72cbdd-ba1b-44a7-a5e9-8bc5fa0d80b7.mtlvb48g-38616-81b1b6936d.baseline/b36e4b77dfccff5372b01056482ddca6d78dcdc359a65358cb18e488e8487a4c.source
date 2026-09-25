use super::*;

pub(crate) fn try_lower_selector_member_access(
    inner: &Expression,
    member: &Identifier,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> Option<bool> {
    pub(crate) fn selector_from_signature(signature: &str) -> [u8; 4] {
        let mut hasher = Keccak256::new();
        hasher.update(signature.as_bytes());
        let digest = hasher.finalize();
        [digest[0], digest[1], digest[2], digest[3]]
    }

    pub(crate) fn fallback_selector_for_name(name: &str) -> Option<[u8; 4]> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        Some(selector_from_signature(&format!("{name}()")))
    }

    pub(crate) fn resolve_contract_type_name(
        expr: &Expression,
        ctx: &LoweringContext,
    ) -> Option<String> {
        match expr {
            Expression::Variable(type_name) if ctx.is_contract_type_name(&type_name.name) => {
                Some(type_name.name.clone())
            }
            Expression::MemberAccess(_, namespace_expr, type_name)
                if matches!(
                    namespace_expr.as_ref(),
                    Expression::Variable(namespace_id)
                        if !ctx.param_index_map.contains_key(&namespace_id.name)
                            && ctx.resolve_local(&namespace_id.name).is_none()
                            && !ctx.state_index_map.contains_key(&namespace_id.name)
                            && !ctx.is_contract_type_name(&namespace_id.name)
                ) && ctx.is_contract_type_name(&type_name.name) =>
            {
                Some(type_name.name.clone())
            }
            _ => None,
        }
    }

    if member.name != "selector" {
        return None;
    }

    // Solidity exposes function selectors via `TypeName.method.selector` (bytes4).
    // This is commonly used for interface receivers and low-level call encoding.
    //
    // The AST shape is: `MemberAccess(MemberAccess(Variable(TypeName), method), selector)`.
    if let Expression::MemberAccess(_, target_inner, target_method) = inner {
        if let Some(type_name) = resolve_contract_type_name(target_inner.as_ref(), ctx) {
            if let Some(selectors) = ctx.type_method_selectors(&type_name, &target_method.name) {
                if selectors.len() == 1 {
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        selectors[0].to_vec(),
                    )));
                    return Some(true);
                }

                ctx.record_error(format!(
                    "ambiguous selector '{}.{}': {} overload(s)",
                    type_name,
                    target_method.name,
                    selectors.len()
                ));
                return Some(false);
            }

            // Compatibility fallback: custom errors and non-interface helper types
            // may expose `.selector` without being part of the function selector
            // registry. Use a deterministic best-effort selector hash.
            if let Some(selector) = fallback_selector_for_name(&target_method.name) {
                instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                    selector.to_vec(),
                )));
                return Some(true);
            }
        }

        // `this.method.selector` — resolve against the current contract's
        // method registry so the selector matches the Ethereum keccak-4 form
        // that Solidity code observes (e.g., `bytes4(keccak256("foo(uint256)"))`).
        //
        // Without this branch the fallback below would produce a
        // parameterless `"name()"` hash, which for `this.foo.selector`
        // with `foo(uint256)` collapses to `0xc2985578` — silently
        // mismatched against any caller that encodes arguments via the
        // canonical selector.
        if matches!(
            target_inner.as_ref(),
            Expression::Variable(id) if id.name == "this"
        ) {
            let current_contract = ctx.current_contract_name().to_string();
            if let Some(selectors) =
                ctx.type_method_selectors(&current_contract, &target_method.name)
            {
                if selectors.len() == 1 {
                    instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                        selectors[0].to_vec(),
                    )));
                    return Some(true);
                }

                ctx.record_error(format!(
                    "ambiguous selector 'this.{}': {} overload(s)",
                    target_method.name,
                    selectors.len()
                ));
                return Some(false);
            }
        }

        // `this.method.selector` and other instance-style selector expressions.
        if let Some(selector) = fallback_selector_for_name(&target_method.name) {
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                selector.to_vec(),
            )));
            return Some(true);
        }
    }

    // `ErrorName.selector` style custom-error selectors. The selector MUST hash
    // the parametrized canonical signature `Name(t1,t2,...)` (matching solc and
    // `revert ErrorName(args)`), not the bare `Name()` — otherwise a declared
    // error with parameters gets a different 4-byte selector than the one its own
    // revert emits and than off-chain decoders expect.
    if let Expression::Variable(name) = inner {
        let declared_params: Option<Vec<String>> = ctx
            .error_signature(&name.name)
            .map(|sig| sig.param_types.clone());
        if let Some(param_types) = declared_params {
            let canonical: Vec<String> = param_types
                .iter()
                .map(|ty| declared_error_param_canonical(ty, ctx))
                .collect();
            let selector =
                selector_from_signature(&format!("{}({})", name.name, canonical.join(",")));
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                selector.to_vec(),
            )));
            return Some(true);
        }
        if let Some(selector) = fallback_selector_for_name(&name.name) {
            instructions.push(Instruction::PushLiteral(LiteralValue::ByteArray(
                selector.to_vec(),
            )));
            return Some(true);
        }
    }

    ctx.record_error("unsupported selector expression");
    Some(false)
}
