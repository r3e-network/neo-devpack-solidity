use super::*;

pub(crate) fn solidity_type_from_neotype(neotype: &NeoType) -> String {
    match neotype {
        NeoType::Integer { signed, bits } => {
            if *signed {
                format!("int{bits}")
            } else {
                format!("uint{bits}")
            }
        }
        NeoType::Boolean => "bool".to_string(),
        NeoType::String => "string".to_string(),
        NeoType::Address => "address".to_string(),
        NeoType::ByteArray { fixed_len } => match fixed_len {
            Some(len) => format!("bytes{len}"),
            None => "bytes".to_string(),
        },
        NeoType::Array(inner, Some(n)) => {
            format!("{}[{}]", solidity_type_from_neotype(inner.as_ref()), n)
        }
        NeoType::Array(inner, None) => format!("{}[]", solidity_type_from_neotype(inner.as_ref())),
        NeoType::Mapping { key, value } => format!(
            "mapping({}=>{})",
            solidity_type_from_neotype(key.as_ref()),
            solidity_type_from_neotype(value.as_ref())
        ),
        NeoType::Struct { name, .. } => name.clone(),
        NeoType::Any => "Any".to_string(),
    }
}

pub(crate) fn getter_signature_from_neotype(
    var_name: &str,
    neotype: &NeoType,
) -> (Vec<ParameterMetadata>, Vec<ParameterMetadata>, Expression) {
    let mut parameters = Vec::new();
    let mut cursor = neotype;
    let mut depth = 0usize;

    while let NeoType::Mapping { key, value } = cursor {
        let key_type = key.as_ref().clone();
        let key_ty_str = solidity_type_from_neotype(&key_type);
        parameters.push(ParameterMetadata {
            name: Some(format!("arg{depth}")),
            ty: key_ty_str,
            neo_type: Some(key_type),
            storage: None,
        });
        cursor = value.as_ref();
        depth += 1;
    }

    while let NeoType::Array(inner, _) = cursor {
        parameters.push(ParameterMetadata {
            name: Some(format!("arg{depth}")),
            ty: "uint256".to_string(),
            neo_type: Some(NeoType::Integer {
                signed: false,
                bits: 256,
            }),
            storage: None,
        });
        cursor = inner.as_ref();
        depth += 1;
    }

    let mut access_expr = Expression::Variable(Identifier {
        loc: Default::default(),
        name: var_name.to_string(),
    });

    for param in &parameters {
        let param_name = param.name.clone().unwrap_or_else(|| "arg".to_string());
        let index_expr = Expression::Variable(Identifier {
            loc: Default::default(),
            name: param_name,
        });
        access_expr = Expression::ArraySubscript(
            Default::default(),
            Box::new(access_expr),
            Some(Box::new(index_expr)),
        );
    }

    match cursor {
        NeoType::Struct { fields, .. } => {
            let mut ret_params = Vec::new();
            let mut tuple_entries = Vec::new();
            for field in fields {
                let field_type = field.ty.as_ref().clone();
                // Solidity's auto-generated public getter for a struct state
                // variable OMITS mapping and array members — there is no ABI
                // representation for returning them as part of the struct tuple.
                // `bytes`/`string` are NOT `NeoType::Array`, so they remain
                // included, matching solc. Without this filter the getter emits
                // a return type referencing a mapping/array member, producing an
                // invalid ABI / manifest.
                if matches!(field_type, NeoType::Mapping { .. } | NeoType::Array(..)) {
                    continue;
                }
                let field_ty_str = solidity_type_from_neotype(&field_type);
                ret_params.push(ParameterMetadata {
                    name: Some(field.name.clone()),
                    ty: field_ty_str,
                    neo_type: Some(field_type),
                    storage: None,
                });
                let field_expr = Expression::MemberAccess(
                    Default::default(),
                    Box::new(access_expr.clone()),
                    Identifier {
                        loc: Default::default(),
                        name: field.name.clone(),
                    },
                );
                tuple_entries.push((
                    Default::default(),
                    Some(Parameter {
                        loc: Default::default(),
                        annotation: None,
                        ty: field_expr,
                        storage: None,
                        name: None,
                    }),
                ));
            }
            let tuple_expr = Expression::List(Default::default(), tuple_entries);
            (parameters, ret_params, tuple_expr)
        }
        _ => {
            let ret_ty_str = solidity_type_from_neotype(cursor);
            let ret_params = vec![ParameterMetadata {
                name: None,
                ty: ret_ty_str,
                neo_type: Some(cursor.clone()),
                storage: None,
            }];
            (parameters, ret_params, access_expr)
        }
    }
}
