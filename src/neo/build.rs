use super::constants::{
    MAX_CALL_FLAGS, MAX_METHOD_TOKENS, MAX_TOKEN_METHOD_LENGTH, NEF_MAGIC, NEF_SOURCE_MAX_BYTES,
};
use super::encoding::{
    calculate_checksum, write_fixed_string, write_varbytes, write_varint, write_varstring,
};
use super::method_token::MethodToken;
use super::source::clamp_nef_source;

/// Build a NEF (Neo Executable Format) file from raw NeoVM bytecode.
///
/// The implementation follows the Neo N3 specification:
///
/// - Magic header `NEF3`
/// - Compiler identifier (fixed 64 bytes, UTF-8, padded with zeros)
/// - Source URL (varstring, max 256 bytes)
/// - Reserved byte (must be 0)
/// - Method token table (varint count + entries, max 128 entries)
/// - Reserved 2 bytes (must be 0)
/// - Script payload (varbytes)
/// - Checksum (first four bytes of double SHA256 over all previous bytes)
///
/// # Arguments
/// * `script` - The NeoVM bytecode
/// * `compiler` - Compiler identifier string (max 64 bytes)
///
/// # Returns
/// Complete NEF file as byte vector
pub fn build_nef(script: &[u8], compiler: &str) -> Result<Vec<u8>, String> {
    build_nef_with_tokens(script, compiler, "", &[])
}

/// Build a NEF file with method tokens for cross-contract calls.
///
/// # Arguments
/// * `script` - The NeoVM bytecode
/// * `compiler` - Compiler identifier string (max 64 bytes)
/// * `source` - Optional source URL (max 256 bytes)
/// * `tokens` - Array of method tokens for cross-contract calls
///
/// # Returns
/// Complete NEF file as byte vector
pub fn build_nef_with_tokens(
    script: &[u8],
    compiler: &str,
    source: &str,
    tokens: &[MethodToken],
) -> Result<Vec<u8>, String> {
    if script.is_empty() {
        return Err("NEF script payload cannot be empty".to_string());
    }
    let source = clamp_nef_source(source);
    if tokens.len() > MAX_METHOD_TOKENS {
        return Err(format!(
            "NEF method token table exceeds {MAX_METHOD_TOKENS} entries"
        ));
    }

    for token in tokens {
        let method_len = token.method.len();
        if method_len > MAX_TOKEN_METHOD_LENGTH {
            return Err(format!(
                "method token '{}' exceeds {MAX_TOKEN_METHOD_LENGTH} bytes",
                token.method
            ));
        }
        if token.method.starts_with('_') {
            return Err(format!(
                "method token '{}' must not start with '_'",
                token.method
            ));
        }
        if token.call_flags & !MAX_CALL_FLAGS != 0 {
            return Err(format!(
                "method token '{}' has invalid call flags {:#x}",
                token.method, token.call_flags
            ));
        }
    }

    // Rough capacity hint: header (magic + compiler + empty source + reserves) plus script and tokens.
    let token_size: usize = tokens.iter().map(|t| 20 + t.method.len() + 10).sum();
    let mut buffer = Vec::with_capacity(80 + token_size + script.len());

    // Magic (4 bytes)
    buffer.extend_from_slice(b"NEF3");

    // Compiler identifier (64 bytes, zero padded)
    write_fixed_string(&mut buffer, compiler, 64);

    // Source URL (varstring, max 256 bytes)
    write_varstring(&mut buffer, &source);

    // Reserved byte must be zero
    buffer.push(0u8);

    // Method token table
    write_varint(&mut buffer, tokens.len() as u64);
    for token in tokens {
        token.serialize(&mut buffer)?;
    }

    // Reserved bytes (2 bytes, must be 0)
    buffer.extend_from_slice(&[0u8; 2]);

    // Script payload (length-prefixed)
    write_varbytes(&mut buffer, script);

    // Checksum (first 4 bytes of double SHA-256)
    let checksum = calculate_checksum(&buffer);
    buffer.extend_from_slice(&checksum.to_le_bytes());

    Ok(buffer)
}

/// Result of parsing a NEF file via [`parse_nef`].
///
/// Fields mirror the inputs to [`build_nef_with_tokens`], so a `ParsedNef`
/// can be fed straight back into the builder for round-trip tests.
#[derive(Debug, Clone)]
pub struct ParsedNef {
    /// Compiler identifier (the fixed 64-byte field, trimmed of trailing NULs).
    pub compiler: String,
    /// Source URL / identifier (varstring, UTF-8).
    pub source: String,
    /// Method token table.
    pub tokens: Vec<MethodToken>,
    /// NeoVM script payload.
    pub script: Vec<u8>,
}

/// Parse a Neo N3 NEF3 file produced by [`build_nef_with_tokens`].
///
/// Validates, in order:
/// * the 4-byte `NEF3` magic,
/// * the trailing `sha256(sha256(prefix))[..4]` checksum,
/// * that every varint length prefix (source, token count, method name, script)
///   is consistent with the remaining buffer,
/// * the single reserved byte (post-source) and two reserved bytes (pre-script)
///   are all zero,
/// * method-token invariants matching those enforced by the builder
///   (`method.len() <= MAX_TOKEN_METHOD_LENGTH`, `!method.starts_with('_')`,
///   `call_flags & !MAX_CALL_FLAGS == 0`, `tokens.len() <= MAX_METHOD_TOKENS`).
///
/// Returns [`Err`] with a human-readable diagnostic on any violation.
pub fn parse_nef(bytes: &[u8]) -> Result<ParsedNef, String> {
    // Minimum size: 4 (magic) + 64 (compiler) + 1 (varint=0 source) + 1 (reserved)
    // + 1 (varint=0 tokens) + 2 (reserved) + 1 (varint=0 script) + 4 (checksum).
    const MIN_NEF_SIZE: usize = 4 + 64 + 1 + 1 + 1 + 2 + 1 + 4;
    if bytes.len() < MIN_NEF_SIZE {
        return Err(format!(
            "NEF buffer too small: got {} bytes, need at least {}",
            bytes.len(),
            MIN_NEF_SIZE
        ));
    }

    // --- checksum first (over everything except the trailing 4 bytes) -------
    let (prefix, trailer) = bytes.split_at(bytes.len() - 4);
    let expected = calculate_checksum(prefix).to_le_bytes();
    if trailer != expected {
        return Err(format!(
            "NEF checksum mismatch: stored={trailer:02x?} expected={expected:02x?}"
        ));
    }

    let mut cursor = Cursor::new(prefix);

    // --- magic --------------------------------------------------------------
    let magic = cursor.take(4)?;
    if magic != NEF_MAGIC {
        return Err(format!(
            "NEF magic mismatch: got {magic:02x?}, expected {NEF_MAGIC:02x?}"
        ));
    }

    // --- compiler (64 byte fixed field, zero padded) ------------------------
    let compiler_bytes = cursor.take(64)?;
    // Strip trailing NULs written by `write_fixed_string`.
    let trimmed_end = compiler_bytes
        .iter()
        .rposition(|&b| b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    let compiler = std::str::from_utf8(&compiler_bytes[..trimmed_end])
        .map_err(|e| format!("NEF compiler field is not valid UTF-8: {e}"))?
        .to_string();

    // --- source (varstring) -------------------------------------------------
    let source_len = cursor.read_varint()? as usize;
    if source_len > NEF_SOURCE_MAX_BYTES {
        return Err(format!(
            "NEF source length {source_len} exceeds maximum {NEF_SOURCE_MAX_BYTES}"
        ));
    }
    let source_bytes = cursor.take(source_len)?;
    let source = std::str::from_utf8(source_bytes)
        .map_err(|e| format!("NEF source is not valid UTF-8: {e}"))?
        .to_string();

    // --- reserved byte ------------------------------------------------------
    let reserved1 = cursor.take(1)?[0];
    if reserved1 != 0 {
        return Err(format!(
            "NEF reserved byte after source must be 0, got {reserved1:#x}"
        ));
    }

    // --- method tokens ------------------------------------------------------
    let token_count = cursor.read_varint()? as usize;
    if token_count > MAX_METHOD_TOKENS {
        return Err(format!(
            "NEF method token count {token_count} exceeds maximum {MAX_METHOD_TOKENS}"
        ));
    }
    let mut tokens = Vec::with_capacity(token_count);
    for i in 0..token_count {
        // 20-byte hash
        let hash_bytes = cursor.take(20)?;
        let mut hash = [0u8; 20];
        hash.copy_from_slice(hash_bytes);

        // method name (varbytes -> UTF-8 string, max 32 bytes)
        let method_len = cursor.read_varint()? as usize;
        if method_len > MAX_TOKEN_METHOD_LENGTH {
            return Err(format!(
                "NEF token[{i}] method length {method_len} exceeds maximum {MAX_TOKEN_METHOD_LENGTH}"
            ));
        }
        let method_bytes = cursor.take(method_len)?;
        let method = std::str::from_utf8(method_bytes)
            .map_err(|e| format!("NEF token[{i}] method name is not valid UTF-8: {e}"))?
            .to_string();
        if method.starts_with('_') {
            return Err(format!(
                "NEF token[{i}] method name '{method}' must not start with '_'"
            ));
        }

        // parameters_count (u16 LE)
        let params_bytes = cursor.take(2)?;
        let parameters_count = u16::from_le_bytes([params_bytes[0], params_bytes[1]]);

        // has_return_value (u8 bool)
        let has_return_value = cursor.take(1)?[0] != 0;

        // call_flags (u8)
        let call_flags = cursor.take(1)?[0];
        if call_flags & !MAX_CALL_FLAGS != 0 {
            return Err(format!(
                "NEF token[{i}] has invalid call flags {call_flags:#x}"
            ));
        }

        tokens.push(MethodToken {
            hash,
            method,
            parameters_count,
            has_return_value,
            call_flags,
        });
    }

    // --- reserved 2 bytes ---------------------------------------------------
    let reserved2 = cursor.take(2)?;
    if reserved2 != [0u8, 0u8] {
        return Err(format!(
            "NEF reserved bytes before script must be [0,0], got {reserved2:02x?}"
        ));
    }

    // --- script (varbytes) --------------------------------------------------
    let script_len = cursor.read_varint()? as usize;
    let script = cursor.take(script_len)?.to_vec();
    if script.is_empty() {
        return Err("NEF script payload cannot be empty".to_string());
    }

    // Must have consumed exactly the prefix (no trailing garbage before the checksum).
    if !cursor.is_empty() {
        return Err(format!(
            "NEF has {} unexpected trailing bytes before checksum",
            cursor.remaining()
        ));
    }

    Ok(ParsedNef {
        compiler,
        source,
        tokens,
        script,
    })
}

/// Byte-oriented cursor used by [`parse_nef`].
///
/// Intentionally tiny (no dependency on `std::io::Cursor`) so every error path
/// returns our `String` diagnostics rather than `io::Error`.
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        // `self.pos + n` can overflow `usize` for an adversarial varint length
        // near `usize::MAX`, wrapping to a small value that passes the bound and
        // then panics on the slice. Compare against the remaining bytes instead
        // so the check can never wrap.
        if n > self.buf.len() - self.pos {
            return Err(format!(
                "NEF truncated: tried to read {} bytes at offset {}, only {} available",
                n,
                self.pos,
                self.buf.len().saturating_sub(self.pos)
            ));
        }
        let slice = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Read a Neo-style varint (matching [`super::encoding::write_varint`]).
    fn read_varint(&mut self) -> Result<u64, String> {
        let prefix = self.take(1)?[0];
        match prefix {
            0xFD => {
                let b = self.take(2)?;
                Ok(u16::from_le_bytes([b[0], b[1]]) as u64)
            }
            0xFE => {
                let b = self.take(4)?;
                Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as u64)
            }
            0xFF => {
                let b = self.take(8)?;
                Ok(u64::from_le_bytes([
                    b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                ]))
            }
            n => Ok(n as u64),
        }
    }
}
