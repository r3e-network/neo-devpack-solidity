use super::*;

/// Value type for runtime operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum RuntimeValue {
    #[default]
    Null,
    Boolean(bool),
    Integer(i64),
    UnsignedInteger(u64),
    ByteString(Vec<u8>),
    Array(Vec<RuntimeValue>),
    Map(std::collections::HashMap<String, RuntimeValue>),
}

impl From<bool> for RuntimeValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<i64> for RuntimeValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<u64> for RuntimeValue {
    fn from(u: u64) -> Self {
        Self::UnsignedInteger(u)
    }
}

impl From<Vec<u8>> for RuntimeValue {
    fn from(bytes: Vec<u8>) -> Self {
        Self::ByteString(bytes)
    }
}

impl From<&[u8]> for RuntimeValue {
    fn from(bytes: &[u8]) -> Self {
        Self::ByteString(bytes.to_vec())
    }
}

impl From<String> for RuntimeValue {
    fn from(s: String) -> Self {
        Self::ByteString(s.into_bytes())
    }
}

impl From<&str> for RuntimeValue {
    fn from(s: &str) -> Self {
        Self::ByteString(s.as_bytes().to_vec())
    }
}

impl RuntimeValue {
    /// Convert to bytes representation
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            RuntimeValue::Null => vec![0],
            RuntimeValue::Boolean(b) => vec![if *b { 1 } else { 0 }],
            RuntimeValue::Integer(i) => i.to_le_bytes().to_vec(),
            RuntimeValue::UnsignedInteger(u) => u.to_le_bytes().to_vec(),
            RuntimeValue::ByteString(bytes) => bytes.clone(),
            RuntimeValue::Array(_) => {
                // Serialize complex types as JSON for interoperability
                serde_json::to_vec(self).unwrap_or_default()
            }
            RuntimeValue::Map(_) => {
                // Serialize complex types as JSON for interoperability
                serde_json::to_vec(self).unwrap_or_default()
            }
        }
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return RuntimeValue::Null;
        }

        if bytes.len() == 8 {
            if let Ok(array) = bytes.try_into() {
                return RuntimeValue::UnsignedInteger(u64::from_le_bytes(array));
            }
        }

        RuntimeValue::ByteString(bytes.to_vec())
    }

    /// Check if value is truthy
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Null => false,
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Integer(i) => *i != 0,
            RuntimeValue::UnsignedInteger(u) => *u != 0,
            RuntimeValue::ByteString(bytes) => !bytes.is_empty() && bytes.iter().any(|&b| b != 0),
            RuntimeValue::Array(arr) => !arr.is_empty(),
            RuntimeValue::Map(map) => !map.is_empty(),
        }
    }

    /// Get type name
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Null => "null",
            RuntimeValue::Boolean(_) => "boolean",
            RuntimeValue::Integer(_) => "integer",
            RuntimeValue::UnsignedInteger(_) => "unsigned_integer",
            RuntimeValue::ByteString(_) => "byte_string",
            RuntimeValue::Array(_) => "array",
            RuntimeValue::Map(_) => "map",
        }
    }

    /// Convert to stack item
    pub fn to_stack_item(&self) -> StackItem {
        match self {
            RuntimeValue::Null => StackItem::Null,
            RuntimeValue::Boolean(b) => StackItem::Boolean(*b),
            RuntimeValue::Integer(i) => StackItem::Integer(*i),
            RuntimeValue::UnsignedInteger(u) => StackItem::UnsignedInteger(*u),
            RuntimeValue::ByteString(bytes) => StackItem::byte_array(bytes.clone()),
            RuntimeValue::Array(values) => {
                StackItem::array(values.iter().map(|v| v.to_stack_item()).collect())
            }
            RuntimeValue::Map(entries) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in entries {
                    map.insert(k.clone().into_bytes(), v.to_stack_item());
                }
                StackItem::map(map)
            }
        }
    }

    /// Create from stack item
    pub fn from_stack_item(item: &StackItem) -> Self {
        match item {
            StackItem::Null => RuntimeValue::Null,
            StackItem::Boolean(b) => RuntimeValue::Boolean(*b),
            StackItem::Integer(i) => RuntimeValue::Integer(*i),
            StackItem::UnsignedInteger(u) => RuntimeValue::UnsignedInteger(*u),
            StackItem::ByteArray { data: bytes, .. } => {
                RuntimeValue::ByteString(bytes.borrow().clone())
            }
            StackItem::Array(items) => RuntimeValue::Array(
                items
                    .borrow()
                    .iter()
                    .map(RuntimeValue::from_stack_item)
                    .collect(),
            ),
            StackItem::Map(map) => {
                let mut converted = std::collections::HashMap::new();
                for (k, v) in map.borrow().iter() {
                    let key = String::from_utf8_lossy(k).to_string();
                    converted.insert(key, RuntimeValue::from_stack_item(v));
                }
                RuntimeValue::Map(converted)
            }
        }
    }
}
