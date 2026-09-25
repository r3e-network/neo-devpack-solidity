use super::*;

fn normalize_using_target_type(expr: &Expression) -> String {
    fn normalize_type_string(raw: &str) -> String {
        let compact = raw
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .collect::<String>()
            .replace("payable", "");
        let lowered = compact.to_ascii_lowercase();
        match lowered.as_str() {
            "uint" => "uint256".to_string(),
            "int" => "int256".to_string(),
            "byte" => "bytes1".to_string(),
            other => other.to_string(),
        }
    }

    match expr {
        Expression::Type(_, ty) => normalize_type_string(&format!("{ty}")),
        _ => normalize_type_string(&format!("{expr}")),
    }
}

fn using_function_name(function: &UsingFunction) -> Option<String> {
    function
        .path
        .identifiers
        .last()
        .map(|identifier| identifier.name.clone())
}

fn using_function_library_name(function: &UsingFunction) -> Option<String> {
    if function.path.identifiers.len() < 2 {
        return None;
    }

    Some(
        function.path.identifiers[..function.path.identifiers.len() - 1]
            .iter()
            .map(|id| id.name.as_str())
            .collect::<Vec<_>>()
            .join("."),
    )
}

/// The operator symbol bound by a `using {f as <op>} for T` user-defined
/// operator directive (Solidity 0.8.19+).
fn user_defined_operator_symbol(op: &solang_parser::pt::UserDefinedOperator) -> &'static str {
    use solang_parser::pt::UserDefinedOperator;
    match op {
        UserDefinedOperator::BitwiseAnd => "&",
        UserDefinedOperator::BitwiseNot => "~",
        UserDefinedOperator::BitwiseOr => "|",
        UserDefinedOperator::BitwiseXor => "^",
        UserDefinedOperator::Add => "+",
        UserDefinedOperator::Divide => "/",
        UserDefinedOperator::Modulo => "%",
        UserDefinedOperator::Multiply => "*",
        UserDefinedOperator::Subtract | UserDefinedOperator::Negate => "-",
        UserDefinedOperator::Equal => "==",
        UserDefinedOperator::More => ">",
        UserDefinedOperator::MoreEqual => ">=",
        UserDefinedOperator::Less => "<",
        UserDefinedOperator::LessEqual => "<=",
        UserDefinedOperator::NotEqual => "!=",
    }
}

/// Convert a `solang_parser::pt::Using` directive into the IR-level state that
/// `ContractIR` tracks. Used by both `convert_contract` (for contract-scope
/// `using`) and `parse_source` (for file-scope `using`, Solidity 0.8.13+).
///
/// Returns a tuple `(directive, library_name, has_for_star, is_function_list)`
/// where `library_name` is present when the directive binds a library name
/// (either `using L for T` or `using { L.f } for T`).
fn convert_using_directive(using: &Using) -> (UsingDirectiveIR, Vec<String>, bool, bool) {
    let target_type = using.ty.as_ref().map(normalize_using_target_type);

    let mut library_names: Vec<String> = Vec::new();
    if let UsingList::Library(ref path) = using.list {
        let lib_name: String = path
            .identifiers
            .iter()
            .map(|id| id.name.as_str())
            .collect::<Vec<_>>()
            .join(".");
        library_names.push(lib_name);
    }
    if let UsingList::Functions(ref functions) = using.list {
        for function in functions {
            if let Some(lib_name) = using_function_library_name(function) {
                if !library_names.contains(&lib_name) {
                    library_names.push(lib_name);
                }
            }
        }
    }

    // `using X for *` — ty is None when the target is `*`
    let has_for_star = using.ty.is_none();
    let is_function_list = matches!(&using.list, UsingList::Functions(_));

    let function_names = match &using.list {
        UsingList::Functions(functions) => {
            let names: Vec<String> = functions.iter().filter_map(using_function_name).collect();
            Some(names)
        }
        _ => None,
    };

    let overloaded_operators: Vec<String> = match &using.list {
        UsingList::Functions(functions) => functions
            .iter()
            .filter_map(|f| {
                f.oper
                    .as_ref()
                    .map(|op| user_defined_operator_symbol(op).to_string())
            })
            .collect(),
        _ => Vec::new(),
    };

    (
        UsingDirectiveIR {
            target_type,
            function_names,
            overloaded_operators,
        },
        library_names,
        has_for_star,
        is_function_list,
    )
}

/// Merge a parsed `using` directive into an existing [`ContractIR`].
///
/// Used for Solidity 0.8.13+ file-level `using { L.f1, L.f2 } for T;` where
/// the directive is declared at the source unit (not contract) level and
/// applies to every contract in the file. Mirrors the bookkeeping that
/// `convert_contract` performs for contract-scope `using` directives.
pub(crate) fn apply_file_level_using(contract: &mut ContractIR, using: &Using) {
    let (directive, library_names, has_for_star, is_function_list) = convert_using_directive(using);

    for lib_name in library_names {
        if !contract.using_for_libraries.contains(&lib_name) {
            contract.using_for_libraries.push(lib_name);
        }
    }
    if has_for_star {
        contract.has_using_for_star = true;
    }
    if is_function_list {
        contract.has_using_function_list = true;
    }
    contract.using_directives.push(directive);
}

pub(crate) fn convert_contract(
    contract: ContractDefinition,
    comment_map: &HashMap<usize, NatspecDocIR>,
) -> ContractIR {
    let name = contract
        .name
        .as_ref()
        .map(|id| id.name.clone())
        .unwrap_or_else(|| "Contract".to_string());

    let kind = match contract.ty {
        ContractTy::Abstract(_) => ContractKind::AbstractContract,
        ContractTy::Contract(_) => ContractKind::Contract,
        ContractTy::Interface(_) => ContractKind::Interface,
        ContractTy::Library(_) => ContractKind::Library,
    };

    // Extract contract-level documentation
    let doc = find_preceding_doc(&contract.loc, comment_map);

    let bases = contract.base;

    let mut functions = Vec::new();
    let mut events = Vec::new();
    let mut contract_errors = Vec::new();
    let mut state_variables = Vec::new();
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut has_using_for_star = false;
    let mut has_using_function_list = false;
    let mut using_for_libraries: Vec<String> = Vec::new();
    let mut using_directives: Vec<UsingDirectiveIR> = Vec::new();
    let mut has_type_definitions = false;
    let mut type_aliases: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for part in contract.parts {
        match part {
            ContractPart::FunctionDefinition(def) => {
                functions.push(convert_function(*def, comment_map))
            }
            ContractPart::EventDefinition(def) => events.push(convert_event(*def)),
            ContractPart::ErrorDefinition(def) => contract_errors.push(convert_error(*def)),
            ContractPart::VariableDefinition(def) => {
                state_variables.push(convert_state_variable(*def))
            }
            ContractPart::StructDefinition(def) => structs.push(convert_struct(*def)),
            ContractPart::EnumDefinition(def) => enums.push(convert_enum(*def)),
            ContractPart::Using(using) => {
                let (directive, library_names, for_star, function_list) =
                    convert_using_directive(using.as_ref());
                for lib_name in library_names {
                    if !using_for_libraries.contains(&lib_name) {
                        using_for_libraries.push(lib_name);
                    }
                }
                if for_star {
                    has_using_for_star = true;
                }
                if function_list {
                    has_using_function_list = true;
                }
                using_directives.push(directive);
            }
            ContractPart::TypeDefinition(td) => {
                has_type_definitions = true;
                let underlying = format!("{}", td.ty);
                type_aliases.insert(td.name.name.clone(), underlying);
            }
            _ => {}
        }
    }

    // Qualify nested struct names with the containing contract / library so
    // they survive flattening when two different scopes both define a struct
    // with the same short name (e.g. Uniswap V4 declares both a file-level
    // `SwapParams` in PoolOperation.sol AND a `Pool.SwapParams` inside the
    // Pool library — these share the same suffix and would otherwise collide
    // in the structs list after library merge). The qualifier mirrors the
    // way Solidity callers reference these structs from outside the scope
    // (`Pool.SwapParams memory ...`), so external references continue to
    // resolve. Inside the contract / library itself, we rewrite bare
    // references to qualified form so the IR-side type lookup is unambiguous.
    let nested_struct_short_names: Vec<String> = structs.iter().map(|s| s.name.clone()).collect();
    if !nested_struct_short_names.is_empty() {
        // 1. Qualify the struct definitions.
        for s in &mut structs {
            s.name = format!("{}.{}", name, s.name);
            // Also rewrite intra-struct field references that point at
            // sibling structs (e.g. `struct A { B b; }` inside a library that
            // also defines `struct B`).
            for f in &mut s.fields {
                f.ty = rewrite_struct_type_references(&f.ty, &name, &nested_struct_short_names);
            }
        }
        // 2. Rewrite references in functions (parameters, returns, body
        //    variable definitions, function-pointer types) and state
        //    variables.
        for f in &mut functions {
            for p in &mut f.parameters {
                p.ty = rewrite_struct_type_references(&p.ty, &name, &nested_struct_short_names);
            }
            for p in &mut f.returns {
                p.ty = rewrite_struct_type_references(&p.ty, &name, &nested_struct_short_names);
            }
        }
        for sv in &mut state_variables {
            sv.ty = rewrite_struct_type_references(&sv.ty, &name, &nested_struct_short_names);
        }
    }

    ContractIR {
        name,
        kind,
        bases,
        functions,
        events,
        errors: contract_errors,
        state_variables,
        structs,
        enums,
        doc,
        has_using_for_star,
        has_using_function_list,
        using_for_libraries,
        using_directives,
        has_type_definitions,
        type_aliases,
        super_method_map: std::collections::HashMap::new(),
    }
}

/// Rewrite bare references to `nested` struct names inside a type string,
/// qualifying each match with `<owner>.<name>` so the IR-side struct lookup
/// can disambiguate from a same-suffixed struct defined in a different scope.
///
/// Heuristic: split on identifier boundaries (anything that's not
/// `[A-Za-z0-9_]`), then for each identifier component that matches a name in
/// `nested` AND isn't already preceded by a `.`, swap it for the qualified
/// form. Storage modifiers (`memory`, `calldata`, `storage`) and structural
/// punctuation (`[]`, `mapping(...)=>`) pass through unchanged.
fn rewrite_struct_type_references(ty: &str, owner: &str, nested: &[String]) -> String {
    if nested.is_empty() {
        return ty.to_string();
    }
    let bytes = ty.as_bytes();
    let mut out = String::with_capacity(ty.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_alphabetic() || c == '_' {
            // Read the full identifier.
            let start = i;
            while i < bytes.len() {
                let cc = bytes[i] as char;
                if cc.is_ascii_alphanumeric() || cc == '_' {
                    i += 1;
                } else {
                    break;
                }
            }
            let ident = &ty[start..i];
            // Is this identifier already qualified (preceded by `.`)? If so,
            // it's already pointing at a different scope — leave it alone.
            let already_qualified = start > 0 && ty.as_bytes()[start - 1] == b'.';
            if !already_qualified && nested.iter().any(|n| n == ident) {
                out.push_str(owner);
                out.push('.');
                out.push_str(ident);
            } else {
                out.push_str(ident);
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

pub(crate) fn convert_function(
    function: FunctionDefinition,
    comment_map: &HashMap<usize, NatspecDocIR>,
) -> FunctionIR {
    let name = function_name(&function);
    let mutability = extract_mutability(&function);
    let (visibility, explicit_visibility) = extract_visibility(&function);
    let doc = find_preceding_doc(&function.loc, comment_map);

    let mut is_virtual = false;
    let mut is_override = false;
    let mut base_or_modifiers: Vec<Base> = Vec::new();

    for attr in &function.attributes {
        match attr {
            FunctionAttribute::Virtual(_) => is_virtual = true,
            FunctionAttribute::Override(_, _) => is_override = true,
            FunctionAttribute::BaseOrModifier(_, base) => base_or_modifiers.push(base.clone()),
            _ => {}
        }
    }

    let parameters = convert_parameters(&function.params);
    let returns = convert_parameters(&function.returns);
    FunctionIR {
        name,
        ty: function.ty,
        parameters,
        returns,
        mutability,
        visibility,
        explicit_visibility,
        is_virtual,
        is_override,
        base_or_modifiers,
        body: function.body,
        doc,
        had_modifier_epilogue: false,
    }
}

fn function_name(function: &FunctionDefinition) -> String {
    match (&function.name, function.ty) {
        (Some(Identifier { name, .. }), _) => name.clone(),
        (None, FunctionTy::Constructor) => "constructor".to_string(),
        (None, FunctionTy::Fallback) => "fallback".to_string(),
        (None, FunctionTy::Receive) => "receive".to_string(),
        (None, FunctionTy::Modifier) => "modifier".to_string(),
        _ => "function".to_string(),
    }
}

fn extract_mutability(function: &FunctionDefinition) -> MutabilityKind {
    for attribute in &function.attributes {
        if let FunctionAttribute::Mutability(m) = attribute {
            return match m {
                Mutability::Pure(_) => MutabilityKind::Pure,
                Mutability::View(_) | Mutability::Constant(_) => MutabilityKind::View,
                Mutability::Payable(_) => MutabilityKind::Payable,
            };
        }
    }

    MutabilityKind::NonPayable
}

/// Returns the declared visibility and whether a specifier was present. An
/// absent specifier defaults to `Internal` (matching how an unguarded function
/// would lower) but reports `false` so validation can reject it for contract
/// functions, where Solidity 0.5.0+ requires an explicit visibility.
fn extract_visibility(function: &FunctionDefinition) -> (VisibilityKind, bool) {
    for attribute in &function.attributes {
        if let FunctionAttribute::Visibility(visibility) = attribute {
            let kind = match visibility {
                Visibility::External(_) => VisibilityKind::External,
                Visibility::Public(_) => VisibilityKind::Public,
                Visibility::Internal(_) => VisibilityKind::Internal,
                Visibility::Private(_) => VisibilityKind::Private,
            };
            return (kind, true);
        }
    }

    (VisibilityKind::Internal, false)
}

fn convert_parameters(params: &ParameterList) -> Vec<ParameterIR> {
    params
        .iter()
        .filter_map(|(_, param)| param.as_ref())
        .map(|param| ParameterIR {
            name: param.name.as_ref().map(|id| id.name.clone()),
            ty: format!("{}", param.ty),
            storage: param.storage.as_ref().map(storage_to_string),
        })
        .collect()
}

fn storage_to_string(storage: &StorageLocation) -> String {
    match storage {
        StorageLocation::Memory(_) => "memory",
        StorageLocation::Storage(_) => "storage",
        StorageLocation::Calldata(_) => "calldata",
        StorageLocation::Transient(_) => "transient",
    }
    .to_string()
}

fn convert_event(event: EventDefinition) -> EventIR {
    let name = event
        .name
        .as_ref()
        .map(|id| id.name.clone())
        .unwrap_or_else(|| "event".to_string());

    let parameters = event
        .fields
        .into_iter()
        .map(|param| EventParameterIR {
            name: param.name.map(|id| id.name),
            ty: format!("{}", param.ty),
            indexed: param.indexed,
        })
        .collect();

    EventIR {
        name,
        parameters,
        anonymous: event.anonymous,
    }
}

/// Convert a parsed custom `error` declaration into [`ErrorIR`], preserving
/// the declared parameter names and Solidity type strings in declaration
/// order so revert-site lowering can compute the EVM-canonical selector
/// from the DECLARED signature.
pub(crate) fn convert_error(def: solang_parser::pt::ErrorDefinition) -> ErrorIR {
    let name = def
        .name
        .as_ref()
        .map(|id| id.name.clone())
        .unwrap_or_else(|| "error".to_string());

    let parameters = def
        .fields
        .into_iter()
        .map(|field| ParameterIR {
            name: field.name.map(|id| id.name),
            ty: format!("{}", field.ty),
            storage: None,
        })
        .collect();

    ErrorIR { name, parameters }
}

fn convert_state_variable(def: VariableDefinition) -> StateVariableIR {
    let name = def.name.map(|id| id.name);
    let ty = format!("{}", def.ty);
    let initializer = def.initializer.clone();

    let mut visibility = None;
    let mut is_constant = false;
    let mut is_immutable = false;
    let mut is_transient = false;

    for attr in def.attrs {
        match attr {
            VariableAttribute::Visibility(vis) => {
                visibility = Some(
                    match vis {
                        Visibility::External(_) => "external",
                        Visibility::Public(_) => "public",
                        Visibility::Internal(_) => "internal",
                        Visibility::Private(_) => "private",
                    }
                    .to_string(),
                );
            }
            VariableAttribute::Constant(_) => {
                is_constant = true;
            }
            VariableAttribute::Immutable(_) => {
                is_immutable = true;
            }
            // `uint256 transient t;` (0.8.28+). Captured so the validator can
            // warn that NeoVM persists it (no EIP-1153 transient store).
            VariableAttribute::StorageLocation(StorageLocation::Transient(_)) => {
                is_transient = true;
            }
            _ => {}
        }
    }

    StateVariableIR {
        name,
        ty,
        is_constant,
        is_transient,
        is_immutable,
        visibility,
        has_initializer: initializer.is_some(),
        initializer,
    }
}

pub(crate) fn convert_struct(def: StructDefinition) -> StructIR {
    let name = def
        .name
        .as_ref()
        .map(|id| id.name.clone())
        .unwrap_or_else(|| "Struct".to_string());

    let fields = def
        .fields
        .into_iter()
        .filter_map(|field| {
            let field_name = field.name.map(|id| id.name)?;
            Some(StructFieldIR {
                name: field_name,
                ty: format!("{}", field.ty),
            })
        })
        .collect();

    StructIR { name, fields }
}

pub(crate) fn convert_enum(def: EnumDefinition) -> EnumIR {
    let name = def
        .name
        .as_ref()
        .map(|id| id.name.clone())
        .unwrap_or_else(|| "Enum".to_string());

    let values = def
        .values
        .into_iter()
        .filter_map(|value| value.map(|id| id.name))
        .collect();

    EnumIR { name, values }
}
