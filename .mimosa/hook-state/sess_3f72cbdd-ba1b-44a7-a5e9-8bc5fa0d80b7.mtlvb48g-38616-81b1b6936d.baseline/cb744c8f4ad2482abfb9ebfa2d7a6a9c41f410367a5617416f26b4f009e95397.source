//! Stack item types for execution context.
//!
//! StackItem represents values that can be pushed onto the evaluation stack
//! during NeoVM execution.

use serde::{Deserialize, Serialize};

/// Distinguishes ByteString (0x28) from Buffer (0x30) within `StackItem::ByteArray`.
/// NeoVM's EQUAL is type-strict: equal bytes with different type tags are NOT equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteArrayType {
    ByteString, // 0x28 — immutable, CONVERT output, ABI params, storage reads
    Buffer,     // 0x30 — mutable, NEWBUFFER, CAT, SUBSTR, LEFT, RIGHT output
}

/// Stack item in execution context
#[derive(Debug, Clone)]
pub enum StackItem {
    Integer(i64),
    UnsignedInteger(u64),
    ByteArray {
        data: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
        type_tag: ByteArrayType,
    },
    Array(std::rc::Rc<std::cell::RefCell<Vec<StackItem>>>),
    Map(std::rc::Rc<std::cell::RefCell<std::collections::HashMap<Vec<u8>, StackItem>>>),
    Boolean(bool),
    Null,
}

impl StackItem {
    /// Create a ByteString-tagged ByteArray (0x28). This is the common case
    /// for most byte values (ABI parameters, storage reads, literals, etc.).
    pub fn byte_array(bytes: Vec<u8>) -> Self {
        Self::ByteArray {
            data: std::rc::Rc::new(std::cell::RefCell::new(bytes)),
            type_tag: ByteArrayType::ByteString,
        }
    }

    /// Create a Buffer-tagged ByteArray (0x30). Use for mutable buffer
    /// results (NEWBUFFER, CAT, SUBSTR, LEFT, RIGHT).
    pub fn buffer(bytes: Vec<u8>) -> Self {
        Self::ByteArray {
            data: std::rc::Rc::new(std::cell::RefCell::new(bytes)),
            type_tag: ByteArrayType::Buffer,
        }
    }

    /// Read the byte content of any ByteArray regardless of type tag.
    pub fn as_bytes(&self) -> Option<std::cell::Ref<'_, Vec<u8>>> {
        match self {
            Self::ByteArray { data, .. } => Some(data.borrow()),
            _ => None,
        }
    }

    /// Check if this is a Buffer-tagged ByteArray.
    pub fn is_buffer(&self) -> bool {
        matches!(
            self,
            Self::ByteArray {
                type_tag: ByteArrayType::Buffer,
                ..
            }
        )
    }

    /// Check if this is a ByteString-tagged ByteArray.
    pub fn is_byte_string(&self) -> bool {
        matches!(
            self,
            Self::ByteArray {
                type_tag: ByteArrayType::ByteString,
                ..
            }
        )
    }

    pub fn array(items: Vec<StackItem>) -> Self {
        Self::Array(std::rc::Rc::new(std::cell::RefCell::new(items)))
    }

    pub fn map(map: std::collections::HashMap<Vec<u8>, StackItem>) -> Self {
        Self::Map(std::rc::Rc::new(std::cell::RefCell::new(map)))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum StackItemSerde {
    Integer(i64),
    UnsignedInteger(u64),
    ByteArray(Vec<u8>),
    Array(Vec<StackItemSerde>),
    Map(Vec<(Vec<u8>, StackItemSerde)>),
    Boolean(bool),
    Null,
}

impl StackItemSerde {
    fn from_item(item: &StackItem) -> Self {
        match item {
            StackItem::Integer(value) => StackItemSerde::Integer(*value),
            // Canonicalise `UnsignedInteger(v)` to `Integer(v as i64)` whenever
            // it fits, so the `StdLib.serialize` JSON envelope (used for storage
            // slot derivation via `keccak256(serialize(key) || base)`) does not
            // diverge between callers that pass a numeric argument as
            // `Integer(0i64)` vs `UnsignedInteger(0u64)`. Compiled IR only ever
            // produces `Integer`-shaped values for integer locals/storage reads,
            // so without this collapse `pushArr(uint256 v)` writes under
            // `serialize(Integer(len))` while `getArr(uint256 i)` reads from
            // `serialize(UnsignedInteger(i))` — surfaced by the storage
            // state-machine proptest as `getArr(0)` returning `0` for a
            // freshly-pushed value. Wide values (`v > i64::MAX`) keep the
            // `UnsignedInteger` envelope; callers pushing those are already
            // outside the i64 narrow path that any IR-emitted integer would
            // produce, so a divergence there would be visible at the arithmetic
            // level too — not silently in slot derivation.
            StackItem::UnsignedInteger(value) => {
                if *value <= i64::MAX as u64 {
                    StackItemSerde::Integer(*value as i64)
                } else {
                    StackItemSerde::UnsignedInteger(*value)
                }
            }
            StackItem::ByteArray { data, .. } => StackItemSerde::ByteArray(data.borrow().clone()),
            StackItem::Array(items) => StackItemSerde::Array(
                items
                    .borrow()
                    .iter()
                    .map(StackItemSerde::from_item)
                    .collect(),
            ),
            StackItem::Map(map) => StackItemSerde::Map(
                map.borrow()
                    .iter()
                    .map(|(k, v)| (k.clone(), StackItemSerde::from_item(v)))
                    .collect(),
            ),
            StackItem::Boolean(value) => StackItemSerde::Boolean(*value),
            StackItem::Null => StackItemSerde::Null,
        }
    }

    fn into_item(self) -> StackItem {
        match self {
            StackItemSerde::Integer(value) => StackItem::Integer(value),
            StackItemSerde::UnsignedInteger(value) => StackItem::UnsignedInteger(value),
            StackItemSerde::ByteArray(bytes) => StackItem::byte_array(bytes),
            StackItemSerde::Array(items) => {
                let converted = items.into_iter().map(StackItemSerde::into_item).collect();
                StackItem::array(converted)
            }
            StackItemSerde::Map(entries) => {
                let mut map = std::collections::HashMap::new();
                for (key, value) in entries {
                    map.insert(key, value.into_item());
                }
                StackItem::map(map)
            }
            StackItemSerde::Boolean(value) => StackItem::Boolean(value),
            StackItemSerde::Null => StackItem::Null,
        }
    }
}

impl serde::Serialize for StackItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        StackItemSerde::from_item(self).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for StackItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StackItemSerde::deserialize(deserializer).map(StackItemSerde::into_item)
    }
}

impl PartialEq for StackItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StackItem::Integer(a), StackItem::Integer(b)) => a == b,
            (StackItem::UnsignedInteger(a), StackItem::UnsignedInteger(b)) => a == b,
            (StackItem::Boolean(a), StackItem::Boolean(b)) => a == b,
            (StackItem::Null, StackItem::Null) => true,
            (
                StackItem::ByteArray {
                    data: a,
                    type_tag: ta,
                },
                StackItem::ByteArray {
                    data: b,
                    type_tag: tb,
                },
            ) => ta == tb && a.borrow().eq(&*b.borrow()),
            (StackItem::Array(a), StackItem::Array(b)) => a.borrow().eq(&*b.borrow()),
            (StackItem::Map(a), StackItem::Map(b)) => a.borrow().eq(&*b.borrow()),
            _ => false,
        }
    }
}
