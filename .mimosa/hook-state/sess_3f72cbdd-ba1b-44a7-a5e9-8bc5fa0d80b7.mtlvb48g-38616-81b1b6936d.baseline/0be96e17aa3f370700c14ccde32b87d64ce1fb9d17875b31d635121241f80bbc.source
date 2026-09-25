use super::*;
use crate::solidity::rewrite_statement;

pub(crate) fn flatten_contract_inheritance(
    contract: ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Result<(ContractIR, Vec<String>), SolidityError> {
    let order = contract_linearization_base_to_derived(&contract.name, contract_map)?;

    let mut functions: Vec<FunctionIR> = Vec::new();
    let mut function_index: std::collections::HashMap<(u8, String, Vec<String>), (String, usize)> =
        std::collections::HashMap::new();

    let mut events: Vec<EventIR> = Vec::new();
    let mut errors: Vec<ErrorIR> = Vec::new();
    let mut state_variables: Vec<StateVariableIR> = Vec::new();

    let mut structs: Vec<StructIR> = Vec::new();
    let mut struct_index: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    let mut enums: Vec<EnumIR> = Vec::new();
    let mut enum_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    let mut warnings: Vec<String> = Vec::new();

    // Merge user-defined value type aliases from the inheritance chain.
    let mut type_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Maps `"{caller_fn}::{method}"` → renamed super-method name for
    // `super.method()` resolution. Keyed on the caller so that nested
    // super-chains (A→B→C where each overrides `foo` and calls `super.foo()`)
    // resolve to the next-older base body rather than recursing into self.
    //
    // Backward compat: a plain `"{method}"` key is also written for the
    // top-of-chain lookup so callers that haven't been updated to the
    // qualified form still work.
    let mut super_method_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Resolve private-state-variable shadowing. Solidity allows a derived
    // contract to redeclare a `private` state variable with the SAME NAME as
    // an ancestor (typically a different type) — each contract scope keeps
    // its own storage slot and inherited methods continue to read the
    // ancestor's slot. Our flattener collapses both into a single
    // `state_variables` list, which would then look up `_name` by name and
    // pick whichever entry the index map keeps last. The OZ Governor / EIP712
    // pair triggers this exact pattern: Governor's `string private _name`
    // shadows EIP712's `ShortString private immutable _name`, and EIP712's
    // inherited `_EIP712Name` body then reads Governor's String `_name`
    // instead of its own bytes32 one — surfacing as
    // "cannot bind receiver type 'String' to the library function first
    // parameter" for `_name.toStringWithFallback(...)`.
    //
    // Fix: build a per-ancestor rename map for state vars that conflict with
    // a later (more-derived) declaration, rename the ancestor's storage slot
    // to `<Ancestor>__<name>`, and rewrite identifier references inside the
    // ancestor's own functions to match. The derived contract's own state
    // variable keeps the original name, so the derived's bodies continue to
    // read the right slot.
    //
    // We compute the rename plan upfront by walking the linearization in
    // derived-first order (reverse of base-first emission order) and marking
    // any ancestor's state var whose name is already claimed by a more-
    // derived ancestor as needing a rename.
    let mut state_var_renames: std::collections::HashMap<
        String,
        std::collections::HashMap<String, String>,
    > = std::collections::HashMap::new();
    {
        let mut claimed_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Walk derived-first so the *most-derived* declaration wins on
        // collisions. The contract being flattened is the leaf, so its own
        // state vars never get renamed.
        for ancestor_name in order.iter().rev() {
            let Some(ancestor) = contract_map.get(ancestor_name) else {
                continue;
            };
            let mut renames: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for sv in &ancestor.state_variables {
                let Some(name) = sv.name.as_ref() else {
                    continue;
                };
                if claimed_names.contains(name) {
                    let renamed = format!("__{ancestor_name}__{name}");
                    renames.insert(name.clone(), renamed);
                } else {
                    claimed_names.insert(name.clone());
                }
            }
            if !renames.is_empty() {
                state_var_renames.insert(ancestor_name.clone(), renames);
            }
        }
    }

    for ancestor_name in &order {
        let Some(ancestor) = contract_map.get(ancestor_name) else {
            continue;
        };

        let ancestor_is_interface = matches!(ancestor.kind, ContractKind::Interface);

        // Preserve Solidity storage layout order: base state variables first, then derived.
        let ancestor_renames = state_var_renames.get(ancestor_name);
        if let Some(renames) = ancestor_renames {
            // Rename ancestor's state vars per the rename plan.
            for sv in &ancestor.state_variables {
                let mut sv_clone = sv.clone();
                if let Some(name) = sv_clone.name.as_ref() {
                    if let Some(new_name) = renames.get(name) {
                        sv_clone.name = Some(new_name.clone());
                    }
                }
                state_variables.push(sv_clone);
            }
        } else {
            state_variables.extend(ancestor.state_variables.clone());
        }

        // Merge user-defined value type aliases (base-first, derived wins on conflict).
        for (alias_name, underlying) in &ancestor.type_aliases {
            type_aliases
                .entry(alias_name.clone())
                .or_insert_with(|| underlying.clone());
        }

        for s in &ancestor.structs {
            if let Some(idx) = struct_index.get(&s.name).copied() {
                structs[idx] = s.clone();
            } else {
                struct_index.insert(s.name.clone(), structs.len());
                structs.push(s.clone());
            }
        }

        for e in &ancestor.enums {
            if let Some(idx) = enum_index.get(&e.name).copied() {
                enums[idx] = e.clone();
            } else {
                enum_index.insert(e.name.clone(), enums.len());
                enums.push(e.clone());
            }
        }

        // Events are additive in Solidity; duplicates are later de-duplicated by manifest builder.
        events.extend(ancestor.events.clone());

        // Custom errors are inherited too; later (more-derived) declarations
        // shadow same-named base declarations.
        for err in &ancestor.errors {
            if let Some(existing) = errors.iter_mut().find(|e| e.name == err.name) {
                *existing = err.clone();
            } else {
                errors.push(err.clone());
            }
        }

        // Build a substitution map for state-var rewrites in this ancestor's
        // function bodies. The map renames identifier references like
        // `_name` to the renamed slot (`__EIP712___name`) so the inherited
        // bodies still read THEIR original storage when flattened into a
        // derived contract that shadowed the same name. Empty when no
        // collisions occurred for this ancestor.
        let renames_for_this_ancestor = ancestor_renames.cloned().unwrap_or_default();
        let body_subs: std::collections::HashMap<String, Expression> = renames_for_this_ancestor
            .iter()
            .map(|(orig, renamed)| {
                (
                    orig.clone(),
                    Expression::Variable(Identifier {
                        loc: Loc::Implicit,
                        name: renamed.clone(),
                    }),
                )
            })
            .collect();

        // Helper: clone a function and rewrite its body to apply ancestor
        // state-var renames. The rewrite is identifier-substitution at the
        // Expression level — `rewrite_expression` (from the modifier-
        // expansion subsystem) does exactly what we need.
        let rewrite_func = |f: &FunctionIR| -> FunctionIR {
            if body_subs.is_empty() {
                return f.clone();
            }
            let mut cloned = f.clone();
            if let Some(body) = cloned.body.as_ref() {
                cloned.body = Some(rewrite_statement(body, &body_subs, None));
            }
            cloned
        };

        for func in &ancestor.functions {
            // When flattening, keep only the most-derived constructor to avoid name collisions.
            if matches!(func.ty, FunctionTy::Constructor) && ancestor.name != contract.name {
                continue;
            }

            let key = (
                function_ty_key(func.ty),
                func.name.clone(),
                func.parameters
                    .iter()
                    .map(|param| crate::utils::canonical_param_type(&param.ty))
                    .collect::<Vec<_>>(),
            );
            match function_index.get(&key) {
                Some((origin, _)) if origin == &ancestor.name => {
                    // Duplicate definition within the same contract; preserve it so validation can
                    // emit a proper DUPLICATE_SIGNATURE diagnostic.
                    functions.push(rewrite_func(func));
                }
                Some((base_origin, idx)) => {
                    let idx = *idx;
                    let base_func_is_virtual = functions[idx].is_virtual;
                    let base_func_has_body = functions[idx].body.is_some();
                    let base_origin = base_origin.clone();

                    // Determine if the base contract is an interface.
                    let base_is_interface = contract_map
                        .get(&base_origin)
                        .map(|c| matches!(c.kind, ContractKind::Interface))
                        .unwrap_or(false);

                    // Virtual/override enforcement:
                    // - Base function must be `virtual` (or from an interface, which is implicitly virtual)
                    // - Derived function must be marked `override`
                    if !base_is_interface && !base_func_is_virtual {
                        warnings.push(format!(
                            "function '{}' in '{}' overrides '{}::{}' which is not marked 'virtual'",
                            func.name, ancestor_name, base_origin, func.name
                        ));
                    }

                    if !ancestor_is_interface && !func.is_override {
                        warnings.push(format!(
                            "function '{}' in '{}' overrides a base function but is not marked 'override'",
                            func.name, ancestor_name
                        ));
                    }

                    // Preserve the base method as a renamed internal function so that
                    // `super.method()` can resolve to it during IR lowering.
                    // Only preserve if the base function has a body (skip interface stubs).
                    //
                    // Task #85 fix — in multi-level inheritance (A→B→C) where
                    // each level overrides `foo` and calls `super.foo()`, the
                    // old logic collapsed every preserved body into a single
                    // `__super_foo` slot, so `super.foo()` in the preserved
                    // B-body resolved back to `__super_foo` (itself) and
                    // recursed until call-stack overflow. Fix: each ancestor
                    // body in the chain gets a unique `__super{N}_{name}`
                    // slot (N=1 for the immediate base, N=2 for the
                    // grandbase, …) and the caller-keyed `super_method_map`
                    // routes each frame to the NEXT-older slot.
                    if base_func_has_body {
                        // Shift any existing chain entries up one level:
                        // `__super{N}_foo` → `__super{N+1}_foo`. Process
                        // highest-level first to avoid collisions. Also
                        // rewrite the super-map entries they own.
                        let mut levels: Vec<usize> = functions
                            .iter()
                            .filter_map(|f| parse_super_level(&f.name, &func.name))
                            .collect();
                        levels.sort_unstable_by(|a, b| b.cmp(a));
                        for old_level in levels {
                            let old_name = super_level_name(old_level, &func.name);
                            let new_name = super_level_name(old_level + 1, &func.name);
                            if let Some(pos) = functions.iter().position(|f| f.name == old_name) {
                                functions[pos].name = new_name.clone();
                            }
                            // Shift the caller-keyed map entry whose KEY names
                            // the renamed function (i.e. what `super.foo()`
                            // inside that body should resolve to).
                            let old_key = format!("{}::{}", old_name, func.name);
                            let new_key = format!("{}::{}", new_name, func.name);
                            if let Some(target) = super_method_map.remove(&old_key) {
                                // The target also shifts up one level.
                                let new_target = match parse_super_level(&target, &func.name) {
                                    Some(l) => super_level_name(l + 1, &func.name),
                                    None => target,
                                };
                                super_method_map.insert(new_key, new_target);
                            }
                        }

                        let super_name = super_level_name(1, &func.name);
                        let mut super_func = functions[idx].clone();
                        super_func.name = super_name.clone();
                        super_func.visibility = VisibilityKind::Internal;
                        super_func.is_virtual = false;
                        super_func.is_override = false;

                        functions.push(super_func);

                        // The incoming derived body (about to replace
                        // `functions[idx]`) calls `super.{name}` → the just-
                        // preserved level-1 slot. The displaced base body
                        // (now renamed to level-2, if a chain already
                        // existed) calls `super.{name}` → level-2's
                        // predecessor, which the shift above already
                        // installed as `__super{N+1}_…`.
                        super_method_map
                            .insert(format!("{}::{}", func.name, func.name), super_name.clone());
                        // Also shift the unqualified top-of-chain pointer so
                        // the previously-preserved __super_foo now points to
                        // the level-2 slot (A's body), and register the newly
                        // preserved level-1 slot (B's body) as pointing to
                        // that level-2 slot too — i.e. B's super.foo → A's.
                        let prev_l1_key = format!("{}::{}", super_name, func.name);
                        if functions
                            .iter()
                            .any(|f| f.name == super_level_name(2, &func.name))
                        {
                            super_method_map.insert(prev_l1_key, super_level_name(2, &func.name));
                        }
                        // Plain key retained for backward-compat callers
                        // (expression lowerer falls back to it).
                        super_method_map.insert(func.name.clone(), super_name);
                    }

                    // Don't clobber a bodied override with a bodyless one.
                    // The linearization can put an interface declaration
                    // (bodyless) AFTER an abstract-contract override (with
                    // body) — e.g. Chainlink FunctionsCoordinator inherits
                    // OCR2Base (body) AND IFunctionsCoordinator (interface,
                    // bodyless) where the interface position comes later in
                    // the MRO. A blind replacement here loses the OCR2Base
                    // body and surfaces as "declares a return type but has
                    // no implementation".
                    if func.body.is_none() && base_func_has_body {
                        function_index.insert(key, (ancestor.name.clone(), idx));
                        continue;
                    }
                    functions[idx] = rewrite_func(func);
                    function_index.insert(key, (ancestor.name.clone(), idx));
                }
                None => {
                    let idx = functions.len();
                    functions.push(rewrite_func(func));
                    function_index.insert(key, (ancestor.name.clone(), idx));
                }
            }
        }
    }

    // Merge interface events from the full inheritance tree.
    events.extend(collect_interface_events(&contract, contract_map));

    // Contracts may reference structs/enums declared on inherited interfaces, but interface types
    // are excluded from the storage linearization order. Merge them explicitly so type parsing
    // (NeoType inference) and IR lowering can recognize `Interface.StructName` and unqualified
    // `StructName` references used in contract bodies and ABI signatures.
    if matches!(
        contract.kind,
        ContractKind::Contract | ContractKind::AbstractContract
    ) {
        let (iface_structs, iface_enums) = collect_interface_types(&contract, contract_map);
        for s in iface_structs {
            if struct_index.contains_key(&s.name) {
                continue;
            }
            struct_index.insert(s.name.clone(), structs.len());
            structs.push(s);
        }
        for e in iface_enums {
            if enum_index.contains_key(&e.name) {
                continue;
            }
            enum_index.insert(e.name.clone(), enums.len());
            enums.push(e);
        }
    }

    // Merge ancestor `using` directives. Inherited function bodies are
    // already in `functions` above, and those bodies may rely on
    // member-style call resolution that depends on a base-class-scope
    // `using` directive — e.g. Aave IncentivizedERC20 declares
    // `using SafeCast for uint256;` then calls `amount.toUint128()` inside
    // a `transfer` body inherited by AToken. Without this merge, the
    // descendant's `using_directives` table is missing the ancestor's
    // entries and the IR lowering pass reports "member-style call '...'
    // requires an explicit `using` directive".
    //
    // Base-first walk preserves derived-wins-on-conflict (we already added
    // the derived's own directives last when we cloned the snapshot above —
    // the dedup check below ensures we don't add a duplicate from a base
    // that already matches one of derived's own).
    let mut merged_using_directives = contract.using_directives.clone();
    let mut merged_using_for_libraries = contract.using_for_libraries.clone();
    let mut merged_has_using_for_star = contract.has_using_for_star;
    let mut merged_has_using_function_list = contract.has_using_function_list;
    for ancestor_name in &order {
        if ancestor_name == &contract.name {
            continue;
        }
        let Some(ancestor) = contract_map.get(ancestor_name) else {
            continue;
        };
        for directive in &ancestor.using_directives {
            if !merged_using_directives.iter().any(|existing| {
                existing.target_type == directive.target_type
                    && existing.function_names == directive.function_names
            }) {
                merged_using_directives.push(directive.clone());
            }
        }
        for lib_name in &ancestor.using_for_libraries {
            if !merged_using_for_libraries.contains(lib_name) {
                merged_using_for_libraries.push(lib_name.clone());
            }
        }
        merged_has_using_for_star = merged_has_using_for_star || ancestor.has_using_for_star;
        merged_has_using_function_list =
            merged_has_using_function_list || ancestor.has_using_function_list;
    }

    Ok((
        ContractIR {
            name: contract.name,
            kind: contract.kind,
            bases: contract.bases,
            functions,
            events,
            errors,
            state_variables,
            structs,
            enums,
            doc: contract.doc,
            has_using_for_star: merged_has_using_for_star,
            has_using_function_list: merged_has_using_function_list,
            using_for_libraries: merged_using_for_libraries,
            using_directives: merged_using_directives,
            has_type_definitions: contract.has_type_definitions,
            type_aliases,
            super_method_map,
        },
        warnings,
    ))
}

pub(crate) fn inheritance_contract_chain(
    contract: &ContractIR,
    contract_map: &std::collections::HashMap<String, ContractIR>,
) -> Result<Vec<String>, SolidityError> {
    contract_linearization_base_to_derived(&contract.name, contract_map)
}

/// Returns the synthesized name for the N-th preserved super body of `method_name`.
/// Level 1 uses the legacy `__super_{name}` name for backward compatibility.
pub(crate) fn super_level_name(level: usize, method_name: &str) -> String {
    if level == 1 {
        format!("__super_{method_name}")
    } else {
        format!("__super{level}_{method_name}")
    }
}

/// Parses the chain depth `N` out of a synthesized super-body name for
/// `method_name`. Returns `None` if `name` is not a super-body for that method.
pub(crate) fn parse_super_level(name: &str, method_name: &str) -> Option<usize> {
    let legacy = format!("__super_{method_name}");
    if name == legacy {
        return Some(1);
    }
    // `__super{N}_{method_name}` where N >= 2.
    let rest = name.strip_prefix("__super")?;
    let (digits, tail) = {
        let end = rest.find('_').unwrap_or(rest.len());
        (&rest[..end], &rest[end..])
    };
    if digits.is_empty() {
        return None;
    }
    let level: usize = digits.parse().ok()?;
    let expected_tail = format!("_{method_name}");
    if tail == expected_tail {
        Some(level)
    } else {
        None
    }
}
