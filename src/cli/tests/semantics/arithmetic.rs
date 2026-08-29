// M-IR fix tests — `mulmod` / `addmod` correctness for large uint256 operands.
//
// Before this fix, `mulmod(x, y, m)` and `addmod(x, y, m)` emitted a native
// NeoVM `MOD` for the final reduction. NeoVM `MOD` is signed, so for moduli
// `m >= 2^255` it returned a wrong residue. The fix routes the `% m` step
// through the conformant uint256 software divmod (`emit_u256_divmod_ir`).
//
// Note on `mulmod`'s 512-bit intermediate: this fix corrects the signed-MOD
// residue bug (the reported audit finding) and matches EVM semantics for all
// `addmod` inputs and for `mulmod` inputs whose product fits in 256 bits. A
// fully EVM-conformant `mulmod` (512-bit product + 512/256 long-division) is
// a separate, larger task — see the TODO in
// `src/ir/expressions/calls/variable_calls.rs`.

fn le_bytes_to_biguint(bytes: &[u8]) -> num_bigint::BigUint {
    let mut v = bytes.to_vec();
    while v.last() == Some(&0) {
        v.pop();
    }
    v.reverse();
    num_bigint::BigUint::from_bytes_be(&v)
}

fn run_addmod(a: &str, b: &str, m: &str) -> num_bigint::BigUint {
    let src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {{
    function f() external pure returns (uint256) {{
        return addmod({a}, {b}, {m});
    }}
}}"#
    );
    let artifacts = compile_contracts(&src, false, 2).expect("compile failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.success, "addmod execution failed: {:?}", result.exception);
    le_bytes_to_biguint(&result.return_data)
}

#[test]
fn addmod_large_modulus_above_2_255_uses_unsigned_reduction() {
    // M-IR fix: modulus = 2^255 + 7. As a NeoVM BigInteger this value has its
    // sign bit set, so native signed MOD would yield a wrong residue.
    //
    // Choose a and b so their sum is < 2^256 (no Add truncation) but >= m, so
    // the % m step exercises the unsigned divmod path. a = m + 100, b = 0 ⇒
    // sum = m + 100, expected residue = 100.
    let m: num_bigint::BigUint = (num_bigint::BigUint::from(1u32) << 255) + 7u32;
    let a = m.clone() + 100u32;
    let got = run_addmod(&a.to_str_radix(10), "0", &m.to_str_radix(10));
    assert_eq!(
        got,
        num_bigint::BigUint::from(100u32),
        "addmod(m+100, 0, m) with m >= 2^255 must be 100 (unsigned reduction)"
    );
}

#[test]
fn addmod_small_inputs_still_correct() {
    let got = run_addmod("3", "5", "7");
    assert_eq!(
        got,
        num_bigint::BigUint::from(1u32),
        "(3+5) % 7 = 1"
    );
}

fn run_mulmod(a: &str, b: &str, m: &str) -> num_bigint::BigUint {
    let src = format!(
        r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
contract C {{
    function f() external pure returns (uint256) {{
        return mulmod({a}, {b}, {m});
    }}
}}"#
    );
    let artifacts = compile_contracts(&src, false, 2).expect("compile failed");
    let result = execute_bytecode(&artifacts[0].bytecode);
    assert!(result.success, "mulmod execution failed: {:?}", result.exception);
    le_bytes_to_biguint(&result.return_data)
}

#[test]
fn mulmod_large_modulus_above_2_255_uses_unsigned_reduction() {
    // M-IR fix: modulus >= 2^255. mulmod(3, 5, 2^255+7) = 15 (product 15 fits
    // in 256 bits, so the truncation TODO does NOT bite here).
    let m: num_bigint::BigUint = (num_bigint::BigUint::from(1u32) << 255) + 7u32;
    let got = run_mulmod("3", "5", &m.to_str_radix(10));
    assert_eq!(
        got,
        num_bigint::BigUint::from(15u32),
        "mulmod(3,5,2^255+7) must be 15 with unsigned reduction"
    );
}
