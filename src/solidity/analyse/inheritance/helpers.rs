use super::*;

pub(crate) fn base_last_name(base: &Base) -> Option<String> {
    base.name.identifiers.last().map(|id| id.name.clone())
}

pub(crate) fn function_ty_key(ty: FunctionTy) -> u8 {
    match ty {
        FunctionTy::Constructor => 0,
        FunctionTy::Function => 1,
        FunctionTy::Fallback => 2,
        FunctionTy::Receive => 3,
        FunctionTy::Modifier => 4,
    }
}
