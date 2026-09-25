use super::*;

/// Natspec documentation extracted from source comments.
#[derive(Debug, Clone, Default)]
pub struct NatspecDocIR {
    /// @title - Contract title
    pub title: Option<String>,
    /// @author - Author information
    pub author: Option<String>,
    /// @notice - User-facing description
    pub notice: Option<String>,
    /// @dev - Developer-facing notes
    pub dev: Option<String>,
    /// @param name description
    pub params: Vec<(String, String)>,
    /// @return descriptions
    pub returns: Vec<String>,
    /// @custom:tag value pairs
    pub custom: Vec<(String, String)>,
}

/// Representation of a Solidity contract.
#[derive(Debug, Clone)]
pub struct ContractIR {
    pub name: String,
    pub kind: ContractKind,
    /// Inheritance specifiers (`contract X is A, B(...) { ... }`).
    pub bases: Vec<Base>,
    pub functions: Vec<FunctionIR>,
    pub events: Vec<EventIR>,
    /// Declared custom `error` definitions (contract-level plus file-level
    /// merged in by `parse_source`). Used to resolve EVM custom-error
    /// selectors from the DECLARED parameter types rather than the types
    /// inferred from `revert`-site argument expressions.
    pub errors: Vec<ErrorIR>,
    pub state_variables: Vec<StateVariableIR>,
    pub structs: Vec<StructIR>,
    pub enums: Vec<EnumIR>,
    /// Natspec documentation for this contract
    pub doc: NatspecDocIR,
    /// Whether this contract contains `using X for Y` directives.
    ///
    /// The compiler merges library functions into the contract wholesale,
    /// so basic `using LibName for Type` works implicitly. This flag is
    /// set when advanced forms (`using X for *`, `using { f, g } for Y`)
    /// are present so that diagnostics can be emitted.
    pub has_using_for_star: bool,
    pub has_using_function_list: bool,
    /// Library names referenced by `using X for Y` directives.
    ///
    /// The compiler merges all non-builtin library functions into the contract
    /// wholesale, so `using LibName for Type` member-call syntax (e.g. `x.add(y)`)
    /// resolves to `LibName.add(x, y)` automatically. This list is kept for
    /// diagnostic purposes.
    pub using_for_libraries: Vec<String>,
    /// Parsed `using` directives with enough structure for type-aware lowering.
    pub using_directives: Vec<UsingDirectiveIR>,
    /// Whether this contract contains `type X is Y` definitions.
    pub has_type_definitions: bool,
    /// User-defined value type aliases (`type X is Y`).
    ///
    /// Maps the user-defined type name to its underlying Solidity type string.
    /// During type resolution, `X` is transparently replaced by `Y`.
    /// `X.wrap(v)` and `X.unwrap(v)` compile to no-ops.
    pub type_aliases: std::collections::HashMap<String, String>,
    /// Mapping from original method name to the renamed super-method name.
    ///
    /// When inheritance flattening detects an override, the base version of the
    /// function is preserved as `__super_{methodName}` and this map records the
    /// relationship so that `super.method()` can be resolved during IR lowering.
    pub super_method_map: std::collections::HashMap<String, String>,
}

/// Parsed `using` directive (`using <list> for <type | *>`).
#[derive(Debug, Clone)]
pub struct UsingDirectiveIR {
    /// `None` means wildcard target (`for *`), otherwise normalized target type.
    pub target_type: Option<String>,
    /// Function-name allowlist for `using {f, g} for T`.
    ///
    /// `None` means library-form directive (`using Lib for T`) where all compatible
    /// library functions are available.
    pub function_names: Option<Vec<String>>,
    /// Operator symbols bound by a user-defined-operator directive
    /// (`using {add as +, eq as ==} for T global`, Solidity 0.8.19+). Empty for
    /// ordinary member-style `using`. neo-solc does NOT yet dispatch operator
    /// overloads (the operand's user-defined-value-type identity is erased to
    /// its underlying type before operator lowering), so a diagnostic is
    /// surfaced from these — see `validate_using_directives`.
    pub overloaded_operators: Vec<String>,
}

/// Classification of contract kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    Contract,
    AbstractContract,
    Interface,
    Library,
}

/// Representation of a Solidity function or constructor.
#[derive(Debug, Clone)]
pub struct FunctionIR {
    pub name: String,
    pub ty: FunctionTy,
    pub parameters: Vec<ParameterIR>,
    pub returns: Vec<ParameterIR>,
    pub mutability: MutabilityKind,
    pub visibility: VisibilityKind,
    /// Whether the source declared an explicit visibility specifier. When
    /// `false`, `visibility` holds the defaulted value (`Internal`). Solidity
    /// 0.5.0+ makes an omitted visibility on a contract function a hard error,
    /// so this lets validation reject it rather than silently demoting the
    /// function to `internal` (which drops it from the ABI). Synthetic /
    /// file-level functions set this to `true` because their visibility is
    /// fixed by rule, not by a missing specifier.
    pub explicit_visibility: bool,
    /// Whether this function is marked `virtual`.
    pub is_virtual: bool,
    /// Whether this function is marked `override`.
    pub is_override: bool,
    /// Modifier applications and constructor base invocations.
    pub base_or_modifiers: Vec<Base>,
    pub body: Option<Statement>,
    /// Natspec documentation for this function
    pub doc: NatspecDocIR,
    /// Task #114 — set during modifier expansion when at least one applied
    /// modifier has an epilogue (statements after the `_;` placeholder).
    /// Signals the IR lowerer to redirect `return expr;` inside the expanded
    /// body to synthetic return slots + a jump past the epilogue, so
    /// modifier tail statements like `locked = 0;` still run before the
    /// function returns.
    pub had_modifier_epilogue: bool,
}

/// Function mutability classification based on Solidity state mutability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutabilityKind {
    Pure,
    View,
    Payable,
    NonPayable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityKind {
    External,
    Public,
    Internal,
    Private,
}

/// Representation of a Solidity parameter.
#[derive(Debug, Clone)]
pub struct ParameterIR {
    pub name: Option<String>,
    pub ty: String,
    pub storage: Option<String>,
}

/// Representation of a Solidity event.
#[derive(Debug, Clone)]
pub struct EventIR {
    pub name: String,
    pub parameters: Vec<EventParameterIR>,
    /// `true` when the event was declared with the `anonymous` keyword.
    /// Anonymous events suppress the `keccak256(signature)` topic0 slot so
    /// they can carry up to 4 indexed topics (vs. 3 for non-anonymous).
    pub anonymous: bool,
}

/// Representation of a Solidity event parameter.
#[derive(Debug, Clone)]
pub struct EventParameterIR {
    pub name: Option<String>,
    pub ty: String,
    pub indexed: bool,
}

/// Representation of a Solidity custom `error` declaration.
#[derive(Debug, Clone)]
pub struct ErrorIR {
    pub name: String,
    /// Declared parameters in declaration order (`storage` is always `None`
    /// — error parameters cannot carry a data location).
    pub parameters: Vec<ParameterIR>,
}

/// Representation of a state variable.
#[derive(Debug, Clone)]
pub struct StateVariableIR {
    pub name: Option<String>,
    pub ty: String,
    pub is_constant: bool,
    pub is_immutable: bool,
    /// `transient` data location (Solidity 0.8.28+, EIP-1153). NeoVM has no
    /// transient store — the variable compiles to REGULAR persistent storage,
    /// so `validate_state_variables` surfaces a loud warning.
    pub is_transient: bool,
    pub visibility: Option<String>,
    pub has_initializer: bool,
    pub initializer: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct StructIR {
    pub name: String,
    pub fields: Vec<StructFieldIR>,
}

#[derive(Debug, Clone)]
pub struct StructFieldIR {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct EnumIR {
    pub name: String,
    pub values: Vec<String>,
}
