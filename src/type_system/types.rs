use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldMetadata {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructTypeMetadata {
    pub name: String,
    pub fields: Vec<StructFieldMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumTypeMetadata {
    pub name: String,
    pub variants: usize,
}

/// Neo type representation for compiled contracts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeoType {
    Integer {
        signed: bool,
        bits: u16,
    },
    Boolean,
    String,
    Address,
    ByteArray {
        fixed_len: Option<u16>,
    },
    /// A Solidity array. The second field carries the FIXED length for `T[N]`
    /// (preserved so the ABI signature / selector is `T[N]`, not `T[]`); `None`
    /// is a dynamic `T[]`.
    Array(Box<NeoType>, Option<usize>),
    Mapping {
        key: Box<NeoType>,
        value: Box<NeoType>,
    },
    Struct {
        name: String,
        fields: Vec<StructFieldType>,
    },
    Any,
}

impl NeoType {
    /// Create a uint256 type
    pub fn uint256() -> Self {
        Self::Integer {
            signed: false,
            bits: 256,
        }
    }

    /// Create an int256 type
    pub fn int256() -> Self {
        Self::Integer {
            signed: true,
            bits: 256,
        }
    }

    /// Create a fixed-size byte array
    pub fn bytes_fixed(len: u16) -> Self {
        Self::ByteArray {
            fixed_len: Some(len),
        }
    }

    /// Create a dynamic byte array
    pub fn bytes_dynamic() -> Self {
        Self::ByteArray { fixed_len: None }
    }

    /// Check if this is a value type (stored by value)
    pub fn is_value_type(&self) -> bool {
        matches!(self, Self::Integer { .. } | Self::Boolean | Self::Address)
    }

    /// Check if this is a reference type (stored by reference)
    pub fn is_reference_type(&self) -> bool {
        matches!(
            self,
            Self::Array(..) | Self::Mapping { .. } | Self::Struct { .. }
        )
    }

    /// Get the storage size in bytes (for value types)
    pub fn storage_size(&self) -> Option<usize> {
        match self {
            Self::Integer { bits, .. } => Some((*bits as usize).div_ceil(8)),
            Self::Boolean => Some(1),
            Self::Address => Some(20),
            Self::ByteArray {
                fixed_len: Some(len),
            } => Some(*len as usize),
            _ => None,
        }
    }

    /// Get the type name as a string
    pub fn type_name(&self) -> String {
        match self {
            Self::Integer { signed: true, bits } => format!("int{bits}"),
            Self::Integer {
                signed: false,
                bits,
            } => format!("uint{bits}"),
            Self::Boolean => "bool".to_string(),
            Self::String => "string".to_string(),
            Self::Address => "address".to_string(),
            Self::ByteArray { fixed_len: Some(n) } => format!("bytes{n}"),
            Self::ByteArray { fixed_len: None } => "bytes".to_string(),
            Self::Array(inner, Some(n)) => format!("{}[{}]", inner.type_name(), n),
            Self::Array(inner, None) => format!("{}[]", inner.type_name()),
            Self::Mapping { key, value } => {
                format!("mapping({} => {})", key.type_name(), value.type_name())
            }
            Self::Struct { name, .. } => name.clone(),
            Self::Any => "any".to_string(),
        }
    }

    /// EVM-ABI canonical type spelling used for function-selector and event
    /// `topic0` hashing (`keccak256(name(t1,t2,...))`). Differs from
    /// [`Self::type_name`] in that STRUCTS expand to `(field0,field1,...)`
    /// tuples and the integer width is always explicit (`uint256`, not `uint`).
    /// Enums are already resolved to `uint8` (`Integer { bits: 8 }`) during type
    /// resolution, so they canonicalize correctly here. Using the declared
    /// struct/enum NAME instead (the old behavior) produced selectors and event
    /// topics that disagree with Ethereum tooling for struct/enum parameters.
    pub fn canonical_abi_type(&self) -> String {
        match self {
            Self::Integer { signed: true, bits } => format!("int{bits}"),
            Self::Integer {
                signed: false,
                bits,
            } => format!("uint{bits}"),
            Self::Boolean => "bool".to_string(),
            Self::String => "string".to_string(),
            Self::Address => "address".to_string(),
            Self::ByteArray { fixed_len: Some(n) } => format!("bytes{n}"),
            Self::ByteArray { fixed_len: None } => "bytes".to_string(),
            Self::Array(inner, Some(n)) => format!("{}[{}]", inner.canonical_abi_type(), n),
            Self::Array(inner, None) => format!("{}[]", inner.canonical_abi_type()),
            Self::Struct { fields, .. } => {
                let parts: Vec<String> = fields.iter().map(|f| f.ty.canonical_abi_type()).collect();
                format!("({})", parts.join(","))
            }
            // Mappings never appear in a function/event signature; keep a stable
            // spelling rather than panicking on malformed input.
            Self::Mapping { key, value } => format!(
                "mapping({} => {})",
                key.canonical_abi_type(),
                value.canonical_abi_type()
            ),
            // The devpack placeholder `type Any is bytes` shows as "Any" in the
            // manifest (see `type_name`), but for ABI SELECTORS and event topic
            // hashes it must canonicalize to its underlying type `bytes` — "any"
            // is not a valid Solidity ABI type, so a literal "any" produces a
            // selector no Ethereum/Solidity tool can reproduce.
            Self::Any => "bytes".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldType {
    pub name: String,
    pub ty: Box<NeoType>,
}

#[derive(Debug, Error)]
pub enum TypeParseError {
    #[error("unsupported Solidity type '{0}'")]
    Unsupported(String),
    #[error("fixed-point types are not supported on NeoVM: '{0}'")]
    FixedPoint(String),
}
