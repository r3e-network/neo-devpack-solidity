use super::*;
use solang_parser::pt::{Type as PtType, VariableDeclaration};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Monotonic counter for synthetic modifier/constructor argument temporaries.
/// Only the per-temp UNIQUENESS matters — the name is a debug label and never
/// influences slot indices (those are assigned by allocation order), so the
/// global counter is reproducible-build-safe.
static ARG_TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A fresh, collision-free prefix for a modifier/constructor invocation's
/// argument temporaries, e.g. `__modarg_7` or `__basearg_7`.
pub(crate) fn next_arg_temp_prefix(kind: &str) -> String {
    format!(
        "__{kind}_{}",
        ARG_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

pub(crate) fn build_parameter_substitutions(
    params: &[ParameterIR],
    args: &[Expression],
) -> Result<std::collections::HashMap<String, Expression>, SolidityError> {
    if params.len() != args.len() {
        return Err(SolidityError::Analysis(format!(
            "modifier/constructor argument mismatch: expected {}, got {}",
            params.len(),
            args.len()
        )));
    }

    let mut map = std::collections::HashMap::new();
    for (param, arg) in params.iter().zip(args.iter()) {
        let Some(name) = param.name.as_ref() else {
            return Err(SolidityError::Analysis(
                "modifier/constructor parameter is missing an identifier".to_string(),
            ));
        };
        map.insert(name.clone(), arg.clone());
    }
    Ok(map)
}

/// Build parameter substitutions that evaluate each argument EXACTLY ONCE.
///
/// The plain [`build_parameter_substitutions`] clones the argument expression at
/// every parameter occurrence in the inlined modifier/constructor body, so a
/// side-effecting argument (`check(tick())`, `Base(side())`) runs once PER USE —
/// silently corrupting state (Solidity evaluates it exactly once at the
/// invocation point). Here, every NON-TRIVIAL argument is bound to a synthetic
/// temporary local declared at the top of the inlined body, and parameter
/// references are substituted with that temporary.
///
/// Returns `(substitutions, temp_decls)`. The caller must prepend `temp_decls`
/// to the rewritten body (see [`prepend_arg_temp_decls`]). Trivial arguments
/// (literals / plain variables — idempotent, side-effect free) are inlined
/// directly, as is any argument whose declared type is not a value type we can
/// synthesize a declaration for (reference/user types fall back to the prior
/// inline-clone behavior, preserving existing semantics).
pub(crate) fn build_parameter_substitutions_single_eval(
    params: &[ParameterIR],
    args: &[Expression],
    uniq_prefix: &str,
) -> Result<
    (
        std::collections::HashMap<String, Expression>,
        Vec<Statement>,
    ),
    SolidityError,
> {
    if params.len() != args.len() {
        return Err(SolidityError::Analysis(format!(
            "modifier/constructor argument mismatch: expected {}, got {}",
            params.len(),
            args.len()
        )));
    }

    let mut map = std::collections::HashMap::new();
    let mut decls = Vec::new();
    for (param, arg) in params.iter().zip(args.iter()) {
        let Some(name) = param.name.as_ref() else {
            return Err(SolidityError::Analysis(
                "modifier/constructor parameter is missing an identifier".to_string(),
            ));
        };

        // Trivial args are safe to inline at every use.
        if arg_is_trivial(arg) {
            map.insert(name.clone(), arg.clone());
            continue;
        }

        match value_type_expression_from_string(&param.ty) {
            Some(ty_expr) => {
                let temp_name = format!("{uniq_prefix}_{name}");
                decls.push(make_temp_decl(temp_name.clone(), ty_expr, arg.clone()));
                map.insert(
                    name.clone(),
                    Expression::Variable(Identifier {
                        loc: Loc::Implicit,
                        name: temp_name,
                    }),
                );
            }
            // Reference/user/array/mapping types: keep the prior inline-clone
            // behavior (single-eval not yet synthesized for these).
            None => {
                map.insert(name.clone(), arg.clone());
            }
        }
    }
    Ok((map, decls))
}

/// Prepend argument-temporary declarations to an inlined body so the arguments
/// are evaluated once, before any parameter use.
pub(crate) fn prepend_arg_temp_decls(decls: Vec<Statement>, body: Statement) -> Statement {
    if decls.is_empty() {
        return body;
    }
    match body {
        Statement::Block {
            loc,
            unchecked,
            statements,
        } => {
            let mut combined = decls;
            combined.extend(statements);
            Statement::Block {
                loc,
                unchecked,
                statements: combined,
            }
        }
        other => {
            let mut combined = decls;
            combined.push(other);
            Statement::Block {
                loc: Loc::Implicit,
                unchecked: false,
                statements: combined,
            }
        }
    }
}

/// An argument that is idempotent and side-effect free — safe to inline at every
/// parameter use without a temporary.
fn arg_is_trivial(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::BoolLiteral(..)
            | Expression::NumberLiteral(..)
            | Expression::RationalNumberLiteral(..)
            | Expression::HexNumberLiteral(..)
            | Expression::StringLiteral(..)
            | Expression::HexLiteral(..)
            | Expression::AddressLiteral(..)
            | Expression::Variable(..)
    )
}

fn make_temp_decl(name: String, ty: Expression, init: Expression) -> Statement {
    Statement::VariableDefinition(
        Loc::Implicit,
        VariableDeclaration {
            loc: Loc::Implicit,
            ty,
            storage: None,
            name: Some(Identifier {
                loc: Loc::Implicit,
                name,
            }),
        },
        Some(init),
    )
}

/// Reconstruct a solang `pt` type expression for an elementary VALUE type from
/// the `format!("{}", ty)` string stored on `ParameterIR` (`uint256`, `int8`,
/// `address`, `address payable`, `bool`, `bytesN`). Returns `None` for
/// reference types (`string`, dynamic `bytes`), user types, arrays, and
/// mappings — the caller then falls back to inlining the argument, which never
/// needs a synthesized declaration with a memory/storage location.
fn value_type_expression_from_string(ty: &str) -> Option<Expression> {
    let pt = match ty.trim() {
        "address" => PtType::Address,
        "address payable" => PtType::AddressPayable,
        "bool" => PtType::Bool,
        other => {
            if let Some(rest) = other.strip_prefix("uint") {
                PtType::Uint(parse_int_bits(rest)?)
            } else if let Some(rest) = other.strip_prefix("int") {
                PtType::Int(parse_int_bits(rest)?)
            } else if let Some(rest) = other.strip_prefix("bytes") {
                let n: u8 = rest.parse().ok()?;
                if n == 0 || n > 32 {
                    return None;
                }
                PtType::Bytes(n)
            } else {
                return None;
            }
        }
    };
    Some(Expression::Type(Loc::Implicit, pt))
}

/// Parse the bit width of a `uint`/`int` type suffix (`""` → 256). Rejects
/// non-multiples of 8 and out-of-range widths.
fn parse_int_bits(rest: &str) -> Option<u16> {
    let bits: u16 = if rest.is_empty() {
        256
    } else {
        rest.parse().ok()?
    };
    if bits == 0 || bits > 256 || !bits.is_multiple_of(8) {
        return None;
    }
    Some(bits)
}
