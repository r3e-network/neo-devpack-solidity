use super::*;

pub(crate) fn literal_from_expression(expr: &Expression) -> Option<LiteralValue> {
    literal_from_expression_with_warning(expr, &mut |_| {})
}

/// A diagnostic when `value` is genuinely outside Neo N3's representable
/// 256-bit window.
///
/// `push_integer_bigint` encodes every integer in `[-2^255, 2^256-1]` as a
/// conformant 32-byte `PUSHINT256`: values in `[2^255, 2^256-1]` — most notably
/// `type(uint256).max` (max-approval / all-ones masks / sentinels) — are stored
/// as their two's-complement "negative-looking" face value, which is this
/// compiler's established `uint256` representation and is handled correctly by
/// the unsigned-aware limb routines and comparisons (see
/// `src/ir/ir_expressions/dispatch/binary.rs`). Those literals therefore need NO
/// warning: they are fully supported.
///
/// Only magnitudes *outside* `[-2^255, 2^256-1]` are unrepresentable as a NeoVM
/// integer. The emitter then falls back to coercing an oversized byte string via
/// `value + 0`, which a real node rejects the moment it is used as an integer.
/// That (rare, usually an overflowed constant expression) is the case worth
/// surfacing.
pub(crate) fn neovm_integer_limit_warning(value: &BigInt) -> Option<String> {
    let max_u256 = (BigInt::from(1) << 256u32) - BigInt::from(1); // 2^256 - 1
    let min_i256 = -(BigInt::from(1) << 255u32); // -2^255
    if *value > max_u256 || *value < min_i256 {
        Some(String::from(
            "integer literal is outside the representable NeoVM 256-bit range \
             [-2^255, 2^256-1]; a real node faults when it is used as an integer \
             — check for an overflowed constant expression or a value exceeding \
             uint256/int256",
        ))
    } else {
        None
    }
}

// `stacker::maybe_grow` wrapper — `literal_from_expression_with_warning`
// recurses on `Parenthesis(_)` (inner), which a pathological source like
// `1 + (((...(1)...)))` drives into a deep self-call. Stacker lets the
// compiler walk the chain without overflowing. See the sibling guards in
// `src/ir/expressions/dispatch/entry.rs` and `src/ir/statements/dispatch/statement.rs`.
pub(crate) fn literal_from_expression_with_warning<F>(
    expr: &Expression,
    on_warning: &mut F,
) -> Option<LiteralValue>
where
    F: FnMut(String),
{
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        literal_from_expression_with_warning_inner(expr, on_warning)
    })
}

pub(crate) fn literal_from_expression_with_warning_inner<F>(
    expr: &Expression,
    on_warning: &mut F,
) -> Option<LiteralValue>
where
    F: FnMut(String),
{
    match expr {
        Expression::BoolLiteral(_, value) => Some(LiteralValue::Boolean(*value)),
        Expression::NumberLiteral(_, integer, exp, unit) => {
            let mut value = parse_decimal_bigint(integer)?;
            let exponent = parse_signed_decimal_i32(exp)?;

            if exponent >= 0 {
                // `try_pow10` rejects exponents > `MAX_DECIMAL_EXPONENT`
                // (1024) to prevent compile-time OOM on pathological
                // sources like `1e2000000000`.
                value *= try_pow10(exponent as u32)?;
            } else {
                let divisor = try_pow10(exponent.unsigned_abs())?;
                if (&value % &divisor).is_zero() {
                    value /= divisor;
                } else {
                    return None;
                }
            }

            if let Some(unit) = unit.as_ref() {
                value *= unit_multiplier(unit)?;
            }

            if let Some(msg) = neovm_integer_limit_warning(&value) {
                on_warning(msg);
            }
            Some(LiteralValue::Integer(value))
        }
        Expression::HexNumberLiteral(_, value, unit) => {
            let mut number = parse_hex_bigint(value)?;
            if let Some(unit) = unit.as_ref() {
                number *= unit_multiplier(unit)?;
            }
            if let Some(msg) = neovm_integer_limit_warning(&number) {
                on_warning(msg);
            }
            Some(LiteralValue::Integer(number))
        }
        Expression::RationalNumberLiteral(_, integer, fraction, exp, unit) => {
            let int_part = parse_decimal_bigint(integer)?;
            let fraction_digits = sanitize_numeric_token(fraction);
            let frac_len = fraction_digits.len() as u32;
            let frac_part = if fraction_digits.trim().is_empty() {
                BigInt::zero()
            } else {
                BigInt::parse_bytes(fraction_digits.as_bytes(), 10)?
            };

            // Guard against pathological literals like `1.{10 MB of digits}`
            // or exponents > 10^1024 — see `MAX_DECIMAL_EXPONENT` guidance.
            let frac_pow = try_pow10(frac_len)?;
            let mut numerator = int_part * &frac_pow + frac_part;
            let mut denominator = frac_pow;

            let exponent = parse_signed_decimal_i32(exp)?;
            if exponent >= 0 {
                numerator *= try_pow10(exponent as u32)?;
            } else {
                denominator *= try_pow10(exponent.unsigned_abs())?;
            }

            if let Some(unit) = unit.as_ref() {
                numerator *= unit_multiplier(unit)?;
            }

            if denominator.is_zero() {
                return None;
            }

            if (&numerator % &denominator).is_zero() {
                Some(LiteralValue::Integer(numerator / denominator))
            } else {
                on_warning(format!(
                    "non-integer rational literal {integer}.{fraction} cannot be represented as an integer; fractional values are not supported on NeoVM"
                ));
                None
            }
        }
        Expression::StringLiteral(parts) => Some(LiteralValue::String(string_literal_bytes(parts))),
        Expression::HexLiteral(parts) => decode_hex_segments(parts).map(LiteralValue::ByteArray),
        Expression::AddressLiteral(_, value) => decode_hex_bytes(value).and_then(|mut bytes| {
            // Neo addresses are UInt160 (20 bytes). Reject malformed literals early.
            if bytes.len() != 20 {
                on_warning(format!(
                    "address literal has {} bytes, expected 20 (UInt160)",
                    bytes.len()
                ));
                return None;
            }
            // Neo smart contracts treat UInt160 values (script hashes) in little-endian byte
            // order on the VM stack. Solidity address literals are written in the canonical
            // big-endian hex form, so we reverse here to match Neo N3 conventions.
            bytes.reverse();
            Some(LiteralValue::Address(bytes))
        }),
        Expression::Parenthesis(_, inner) => {
            literal_from_expression_with_warning(inner, on_warning)
        }
        _ => None,
    }
}

pub(crate) fn address_bytes_le_from_expression(expr: &Expression) -> Option<Vec<u8>> {
    match expr {
        Expression::Parenthesis(_, inner) => address_bytes_le_from_expression(inner),
        Expression::AddressLiteral(_, value) => decode_hex_bytes(value).map(|mut bytes| {
            if bytes.len() > 20 {
                bytes.truncate(20);
            } else if bytes.len() < 20 {
                let mut padded = vec![0u8; 20 - bytes.len()];
                padded.extend_from_slice(&bytes);
                bytes = padded;
            }
            bytes.reverse();
            bytes
        }),
        Expression::HexNumberLiteral(_, value, unit) if unit.is_none() => {
            let raw = value.trim().trim_start_matches("0x");
            let mut hex: String = raw
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '_')
                .collect();
            if hex.is_empty() {
                return None;
            }

            if hex.len() > 40 {
                return None;
            }

            if hex.len() % 2 == 1 {
                hex.insert(0, '0');
            }

            if hex.len() < 40 {
                hex = format!("{hex:0>40}");
            }

            let mut bytes = hex_decode(&hex).ok()?;
            if bytes.len() != 20 {
                return None;
            }
            bytes.reverse();
            Some(bytes)
        }
        Expression::HexLiteral(parts) => {
            let mut bytes = decode_hex_segments(parts)?;
            if bytes.len() > 20 {
                bytes.truncate(20);
            } else if bytes.len() < 20 {
                let mut padded = vec![0u8; 20 - bytes.len()];
                padded.extend_from_slice(&bytes);
                bytes = padded;
            }
            bytes.reverse();
            Some(bytes)
        }
        _ => None,
    }
}

pub(crate) fn decode_hex_segments(parts: &[PtHexLiteral]) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for part in parts {
        let segment = part.hex.trim();
        let inner = segment
            .strip_prefix("hex")
            .and_then(|s| s.trim().strip_prefix('\"'))
            .and_then(|s| s.strip_suffix('\"'))
            .unwrap_or(segment);
        let cleaned: String = inner.chars().filter(|c| !c.is_whitespace()).collect();
        bytes.extend(hex_decode(&cleaned).ok()?);
    }
    Some(bytes)
}

pub(crate) fn decode_hex_bytes(value: &str) -> Option<Vec<u8>> {
    let cleaned = value.trim();
    if let Some(inner) = cleaned.strip_prefix("0x") {
        hex_decode(inner).ok()
    } else {
        hex_decode(cleaned).ok()
    }
}

pub(crate) fn parse_decimal_bigint(value: &str) -> Option<BigInt> {
    let sanitized: String = value.chars().filter(|c| *c != '_').collect();
    BigInt::parse_bytes(sanitized.as_bytes(), 10)
}

pub(crate) fn sanitize_numeric_token(value: &str) -> String {
    value.chars().filter(|c| *c != '_').collect()
}

pub(crate) fn parse_signed_decimal_i32(value: &str) -> Option<i32> {
    let sanitized = sanitize_numeric_token(value);
    if sanitized.trim().is_empty() {
        Some(0)
    } else {
        sanitized.parse::<i32>().ok()
    }
}

pub(crate) fn unit_multiplier(unit: &Identifier) -> Option<BigInt> {
    match unit.name.as_str() {
        // Solidity ether units
        "wei" => Some(BigInt::one()),
        "gwei" => Some(pow10(9)),
        "szabo" => Some(pow10(12)),
        "finney" => Some(pow10(15)),
        "ether" => Some(pow10(18)),

        // Solidity time units (Neo Runtime.GetTime is milliseconds; we normalize block.timestamp
        // elsewhere to seconds to preserve Solidity semantics.)
        "second" | "seconds" => Some(BigInt::one()),
        "minute" | "minutes" => Some(BigInt::from(60u64)),
        "hour" | "hours" => Some(BigInt::from(60u64 * 60)),
        "day" | "days" => Some(BigInt::from(60u64 * 60 * 24)),
        "week" | "weeks" => Some(BigInt::from(60u64 * 60 * 24 * 7)),
        "year" | "years" => Some(BigInt::from(60u64 * 60 * 24 * 365)),

        _ => None,
    }
}

pub(crate) fn is_ether_unit(name: &str) -> bool {
    matches!(name, "wei" | "gwei" | "szabo" | "finney" | "ether")
}

pub(crate) fn has_ether_unit(expr: &Expression) -> bool {
    match expr {
        Expression::NumberLiteral(_, _, _, Some(unit)) => is_ether_unit(&unit.name),
        Expression::HexNumberLiteral(_, _, Some(unit)) => is_ether_unit(&unit.name),
        Expression::RationalNumberLiteral(_, _, _, _, Some(unit)) => is_ether_unit(&unit.name),
        Expression::Parenthesis(_, inner) => has_ether_unit(inner),
        _ => false,
    }
}

/// Maximum `10^exp` we'll materialise for a Solidity numeric literal.
///
/// Solidity's `uint256.max` is ~1.1579 × 10^77, so any legal literal fits
/// under 10^78. We accept exponents up to `MAX_DECIMAL_EXPONENT = 1024`
/// (generous — covers every legitimate scientific-notation or rational
/// literal plus a wide safety margin for intermediate computations like
/// `1.0000000000000001e100`), and reject anything larger as unreasonable.
///
/// Without this cap, a pathological Solidity source like `uint x = 1e2_000_000_000;`
/// would call `BigInt::pow(10, 2_000_000_000)`, which attempts to allocate
/// ~830 MB of decimal digits and either OOMs or takes minutes of compile
/// time — a denial-of-service vector via arbitrary user input. The
/// `fuzz_target_1` corpus routinely discovers shapes that would hit this
/// without the guard; see docs/FUZZ.md for triage guidance.
pub(crate) const MAX_DECIMAL_EXPONENT: u32 = 1024;

pub(crate) fn pow10(exp: u32) -> BigInt {
    // Infallible variant: every call site passes a hardcoded ether-unit
    // exponent (9, 12, 15, 18), far below MAX_DECIMAL_EXPONENT, so
    // `try_pow10` cannot return None here. User-controlled exponents must
    // go through `try_pow10` instead. Safe under `panic = "abort"`.
    try_pow10(exp).expect("pow10 caller must pass a hardcoded exponent ≤ MAX_DECIMAL_EXPONENT")
}

/// Fallible variant of `pow10` for call sites that parse user-controlled
/// exponents. Returns `None` on exponents that would exceed
/// `MAX_DECIMAL_EXPONENT`, signalling an invalid Solidity literal rather
/// than a panic.
pub(crate) fn try_pow10(exp: u32) -> Option<BigInt> {
    if exp > MAX_DECIMAL_EXPONENT {
        return None;
    }
    let ten = BigInt::from(10u8);
    Some(ten.pow(exp))
}

pub(crate) fn parse_hex_bigint(value: &str) -> Option<BigInt> {
    let sanitized = value.trim_start_matches("0x");
    BigInt::parse_bytes(sanitized.as_bytes(), 16)
}

#[cfg(test)]
mod neovm_integer_limit_tests {
    use super::*;

    fn pow2(n: u32) -> BigInt {
        BigInt::from(1) << n
    }

    #[test]
    fn representable_window_does_not_warn() {
        for v in [
            BigInt::zero(),
            BigInt::from(-1),
            &pow2(255) - 1, // int256 max
            &pow2(256) - 1, // uint256 max — representable as two's-complement push
            -&pow2(255),    // int256 min
        ] {
            assert!(
                neovm_integer_limit_warning(&v).is_none(),
                "value within [-2^255, 2^256-1] must NOT warn: {v}"
            );
        }
    }

    #[test]
    fn outside_window_warns() {
        for v in [
            pow2(256), // 2^256, one past uint256 max
            &pow2(256) + 1,
            -(&pow2(255)) - 1, // one below int256 min
        ] {
            assert!(
                neovm_integer_limit_warning(&v).is_some(),
                "value outside [-2^255, 2^256-1] must warn: {v}"
            );
        }
    }
}
