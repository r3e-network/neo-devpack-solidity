use super::constants::MAX_TOKEN_METHOD_LENGTH;
use super::encoding::write_varbytes;

/// Method token for cross-contract calls in NEF format.
///
/// Method tokens are used to optimize calls to other contracts by caching
/// the target contract hash and method information in the NEF header.
#[derive(Debug, Clone)]
pub struct MethodToken {
    /// Target contract hash (20 bytes, Script Hash)
    pub hash: [u8; 20],
    /// Method name to call
    pub method: String,
    /// Number of parameters the method accepts
    pub parameters_count: u16,
    /// Whether the method returns a value
    pub has_return_value: bool,
    /// Call flags (Neo N3 `CallFlags` bitmask; `All` = 0x0F)
    pub call_flags: u8,
}

impl MethodToken {
    /// Create a new method token
    pub fn new(hash: [u8; 20], method: &str, params: u16, has_return: bool, flags: u8) -> Self {
        Self {
            hash,
            method: method.to_string(),
            parameters_count: params,
            has_return_value: has_return,
            call_flags: flags,
        }
    }

    /// Get the contract hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash)
    }

    /// Check if call flags allow state changes
    pub fn allows_state_changes(&self) -> bool {
        self.call_flags & 0x01 != 0
    }

    /// Serialize the method token to bytes.
    ///
    /// M-BC3 fix — returns `Err` (rather than `assert!`-panicking) when the
    /// method name exceeds the Neo N3 NEF3 spec cap (`MAX_TOKEN_METHOD_LENGTH`
    /// = 32 bytes). `build_nef_with_tokens` validates up-front, but
    /// `serialize` is `pub(super)` and any future direct caller would have
    /// panicked instead of returning a clean error.
    pub(super) fn serialize(&self, buffer: &mut Vec<u8>) -> Result<(), String> {
        // Contract hash (20 bytes)
        buffer.extend_from_slice(&self.hash);

        // Method name (length-prefixed string, max 32 bytes per Neo spec)
        let bytes = self.method.as_bytes();
        if bytes.len() > MAX_TOKEN_METHOD_LENGTH {
            return Err(format!(
                "method token '{}' exceeds {MAX_TOKEN_METHOD_LENGTH} bytes (Neo N3 NEF3 cap)",
                self.method
            ));
        }
        write_varbytes(buffer, bytes);

        // Parameters count (2 bytes, little-endian)
        buffer.extend_from_slice(&self.parameters_count.to_le_bytes());

        // Has return value (1 byte)
        buffer.push(if self.has_return_value { 1 } else { 0 });

        // Call flags (1 byte)
        buffer.push(self.call_flags);

        Ok(())
    }
}
