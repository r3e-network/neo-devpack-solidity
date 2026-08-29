// Neo N3 NEF format constants
// These constants define the limits and constraints for the Neo Executable Format.

/// Maximum byte length for the NEF `source` field (URL or identifier).
pub(super) const MAX_SOURCE_LENGTH: usize = 256;

/// Maximum number of method tokens allowed in a single NEF file.
/// Method tokens enable optimized cross-contract calls. A real Neo N3 node
/// hard-caps this at 128 during NefFile deserialization
/// (`Tokens = reader.ReadSerializableArray<MethodToken>(128)` in NefFile.cs),
/// so a NEF with more than 128 tokens is rejected on deploy. Match the node.
pub const MAX_METHOD_TOKENS: usize = 128;

/// Maximum byte length of a method name stored in a NEF method token.
/// Method names exceeding this limit will be truncated.
pub const MAX_TOKEN_METHOD_LENGTH: usize = 32;

/// Bitmask of valid CallFlags values (`CallFlags.All`).
/// Used to validate call flags in method tokens.
pub const MAX_CALL_FLAGS: u8 = 0x0F;

/// Publicly exposed maximum byte length for the NEF `source` field.
pub const NEF_SOURCE_MAX_BYTES: usize = MAX_SOURCE_LENGTH;

/// NEF file magic bytes ("NEF3")
pub const NEF_MAGIC: &[u8] = b"NEF3";

/// NEF header size in bytes
pub const NEF_HEADER_SIZE: usize = 64;

/// Maximum script size in NEF (512 KB)
pub const MAX_SCRIPT_SIZE: usize = 512 * 1024;
