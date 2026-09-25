//! Semantic validation extracted from Solidity metadata.

use crate::frontend::VisibilityKind;
use crate::solidity::{
    ContractMetadata, Diagnostic, DiagnosticSeverity, FunctionMetadata, ParameterMetadata,
    StateVariableMetadata,
};

pub fn build_semantic_model(metadata: &ContractMetadata) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    for function in &metadata.methods {
        check_function(function, &mut diagnostics);
    }

    for state in &metadata.state_variables {
        if let Some(diag) = check_state_variable(state) {
            diagnostics.push(diag);
        }
    }

    let has_error = diagnostics
        .iter()
        .any(|diag| matches!(diag.severity, DiagnosticSeverity::Error));

    if has_error {
        Err(diagnostics)
    } else {
        Ok(())
    }
}

fn check_function(function: &FunctionMetadata, diagnostics: &mut Vec<Diagnostic>) {
    let allow_unsupported_internal_types = !matches!(
        function.visibility,
        VisibilityKind::Public | VisibilityKind::External
    );

    for param in &function.parameters {
        if let Some(diag) = check_parameter(
            param,
            FunctionSide::Parameter,
            &function.name,
            allow_unsupported_internal_types,
        ) {
            diagnostics.push(diag);
        }
    }

    for param in &function.return_parameters {
        if let Some(diag) = check_parameter(
            param,
            FunctionSide::Return,
            &function.name,
            allow_unsupported_internal_types,
        ) {
            diagnostics.push(diag);
        }
    }
}

fn check_state_variable(state: &StateVariableMetadata) -> Option<Diagnostic> {
    match &state.neo_type {
        Some(_) => None,
        None => Some(
            Diagnostic::error(format!(
                "state variable '{}' has unsupported type '{}'",
                state.name.as_deref().unwrap_or("<unnamed>"),
                state.ty
            ))
            .with_code("UNSUPPORTED_STATE_TYPE"),
        ),
    }
}

enum FunctionSide {
    Parameter,
    Return,
}

fn check_parameter(
    param: &ParameterMetadata,
    side: FunctionSide,
    function_name: &str,
    allow_unsupported_internal_types: bool,
) -> Option<Diagnostic> {
    match &param.neo_type {
        Some(_) => None,
        None if allow_unsupported_internal_types => None,
        // Task #94 — tolerate parenthesised tuple return/parameter types whose
        // components are all individually supported. The frontend loses the
        // structured tuple shape (solang flattens the top-level tuple into
        // sibling ParameterIR entries), but nested tuples still arrive as a
        // raw `(T1, T2, ...)` string here. Downstream IR lowering treats the
        // slot as `NeoType::Any` and routes it through `abiEncode`, which
        // produces the EVM-canonical flat head layout the spec calls for.
        None if is_supported_tuple_type(&param.ty) => None,
        // Gap `nep11` — the devpack NeoVM iterator handle (`Syscalls.Iterator`).
        // The builtin helper libraries are never struct-merged into user
        // contracts, so the type never resolves to a `NeoType::Struct`; treat
        // the opaque handle as `Any` so NEP-11 `tokensOf`/`tokens` can declare
        // it (manifest returntype `InteropInterface`).
        None if is_devpack_iterator_type(&param.ty) => None,
        None => Some(match side {
            FunctionSide::Parameter => Diagnostic::error(format!(
                "function '{}' parameter '{}' uses unsupported type '{}'",
                function_name,
                param
                    .name
                    .clone()
                    .unwrap_or_else(|| "<unnamed>".to_string()),
                param.ty
            ))
            .with_code("UNSUPPORTED_PARAMETER_TYPE"),
            FunctionSide::Return => Diagnostic::error(format!(
                "function '{}' return type '{}' is unsupported",
                function_name, param.ty
            ))
            .with_code("UNSUPPORTED_RETURN_TYPE"),
        }),
    }
}

/// Gap `nep11` — true iff `ty` names the devpack NeoVM iterator handle type
/// (`Syscalls.Iterator` / `Storage.Iterator` / bare `Iterator`). Mirrors
/// `is_devpack_iterator_type` in `validate/contract/return_types.rs`.
fn is_devpack_iterator_type(ty: &str) -> bool {
    let lowered = ty.trim().to_ascii_lowercase();
    matches!(
        lowered.as_str(),
        "iterator" | "syscalls.iterator" | "storage.iterator"
    )
}

/// Task #94 — recognise `(T1, T2, ...)` tuple-shaped type strings with all
/// inner components individually supported by the scalar/array/bytes/mapping
/// rules. Recursive on nested parens.
fn is_supported_tuple_type(ty: &str) -> bool {
    let trimmed = ty.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'(' || bytes[bytes.len() - 1] != b')' {
        return false;
    }
    let mut depth: i32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 && i != bytes.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return false;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let parts = split_tuple_components_for_params(inner);
    !parts.is_empty() && parts.iter().all(|p| is_supported_leaf_or_tuple(p))
}

fn split_tuple_components_for_params(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let bytes = body.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(body[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < body.len() {
        let last = body[start..].trim();
        if !last.is_empty() {
            parts.push(last.to_string());
        }
    }
    parts
}

fn is_supported_leaf_or_tuple(ty: &str) -> bool {
    let trimmed = ty.trim();
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        return is_supported_tuple_type(trimmed);
    }
    let lowered = trimmed.to_ascii_lowercase();
    lowered.starts_with("uint")
        || lowered.starts_with("int")
        || lowered == "bool"
        || lowered == "string"
        || lowered == "address"
        || lowered == "bytes"
        || lowered == "bytearray"
        || lowered.starts_with("bytes")
        || lowered.ends_with("[]")
        || lowered.starts_with("mapping")
}
