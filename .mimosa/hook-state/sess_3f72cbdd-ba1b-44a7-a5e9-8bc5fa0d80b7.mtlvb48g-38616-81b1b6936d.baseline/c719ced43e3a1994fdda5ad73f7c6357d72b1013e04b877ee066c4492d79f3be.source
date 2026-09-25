//! Regression tests for gap `hasrole` — uncallable builtin intrinsics.
//!
//! Background: `resolve_syscalls_member` used to contain a dead
//! `"hasRole" => None` arm, and `devpack/libraries/Runtime.sol` declared a
//! `hasRole(address, bytes32)` helper that (a) could never be called —
//! builtin devpack libraries are compiler intrinsics whose Solidity bodies
//! are not compiled, and `hasRole` had no intrinsic lowering — and (b) was a
//! security footgun: its body silently ignored `role` and fell back to
//! `checkWitness(account)`.
//!
//! Real Neo N3 has no generic role-membership check: the native
//! RoleManagement contract exposes only `getDesignatedByRole(role, index)`
//! (returning the ECPoint node list designated at a block height) and
//! `designateAsRole`. The official C# devpack (neo-devpack-dotnet) likewise
//! provides no `HasRole`. The faithful fix is therefore removal: `hasRole`
//! is not part of the intrinsic surface, calls fail with the standard
//! targeted "unsupported builtin library call" diagnostic, and the devpack
//! sources no longer advertise it.
//!
//! The probe tests in this file also pin the general invariant the original
//! bug violated: every member advertised by the builtin intrinsic whitelist
//! (`builtin_intrinsic_surface()` in src/ir/context/builtins/resolve.rs)
//! must actually lower. The older resolver probes in
//! tests/fuzz_tests/native_resolver_props.rs walk hand-copied member lists,
//! so a whitelist entry added without a lowering was never flagged; the
//! sweep below iterates the real whitelist instead, so
//! whitelist-without-lowering can never silently reappear.

use neo_devpack_solidity::cli::compile_contracts;
use neo_devpack_solidity::ir::builtin_intrinsic_surface;

/// The diagnostic emitted by member_calls.rs when a call targets a builtin
/// library base but no intrinsic lowering matched the member.
fn unsupported_builtin_msg(base: &str, member: &str) -> String {
    format!("unsupported builtin library call '{base}.{member}'")
}

fn compile_err_text(src: &str) -> Option<String> {
    match compile_contracts(src, false, 2) {
        Ok(_) => None,
        Err(e) => Some(format!("{e:?}")),
    }
}

// ---------------------------------------------------------------------------
// 1. hasRole is cleanly unsupported (not an uncallable advertised intrinsic).
// ---------------------------------------------------------------------------

#[test]
fn syscalls_hasrole_is_clean_unsupported_builtin_error() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(bytes1 role, bytes memory pk) external view returns (bool) {
        return Syscalls.hasRole(role, pk);
    }
}"#;
    let err = compile_err_text(src)
        .expect("Syscalls.hasRole must be a compile error (Neo N3 has no hasRole intrinsic)");
    assert!(
        err.contains(&unsupported_builtin_msg("Syscalls", "hasRole")),
        "Syscalls.hasRole must fail with the targeted unsupported-builtin \
         diagnostic; got: {err}"
    );
    // The diagnostic's supported-intrinsics list must NOT advertise hasRole.
    assert!(
        !err.contains("hasRole,") && !err.contains(", hasRole"),
        "the supported-intrinsics list must not advertise 'hasRole'; got: {err}"
    );
}

#[test]
fn runtime_hasrole_is_clean_unsupported_builtin_error() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function f(address a, bytes32 role) external view returns (bool) {
        return Runtime.hasRole(a, role);
    }
}"#;
    let err = compile_err_text(src)
        .expect("Runtime.hasRole must be a compile error (removed devpack surface)");
    assert!(
        err.contains(&unsupported_builtin_msg("Runtime", "hasRole")),
        "Runtime.hasRole must fail with the targeted unsupported-builtin \
         diagnostic; got: {err}"
    );
}

/// The supported alternative — RoleManagement.getDesignatedByRole — must
/// keep lowering through both the Syscalls and NativeCalls namespaces.
#[test]
fn get_designated_by_role_still_lowers() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract C {
    function viaSyscalls(bytes1 role, uint256 index) external view returns (bytes[] memory) {
        return Syscalls.getDesignatedByRole(role, index);
    }
    function viaNativeCalls(bytes1 role, uint256 index) external view returns (bytes[] memory) {
        return NativeCalls.getDesignatedByRole(role, index);
    }
}"#;
    if let Some(err) = compile_err_text(src) {
        panic!("getDesignatedByRole must keep compiling: {err}");
    }
}

/// User-level `hasRole` methods (OpenZeppelin AccessControl pattern) are
/// ordinary contract functions and must be unaffected by the intrinsic
/// pruning.
#[test]
fn user_defined_hasrole_methods_are_unaffected() {
    let src = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;
contract AccessControl {
    mapping(bytes32 => mapping(address => bool)) private _roles;
    function hasRole(bytes32 role, address account) public view returns (bool) {
        return _roles[role][account];
    }
    function grantRole(bytes32 role, address account) external {
        _roles[role][account] = true;
    }
    function check(bytes32 role) external view returns (bool) {
        return hasRole(role, msg.sender);
    }
}"#;
    if let Some(err) = compile_err_text(src) {
        panic!("user-defined hasRole must keep compiling: {err}");
    }
}

// ---------------------------------------------------------------------------
// 2. Devpack sources must not advertise an uncallable hasRole.
// ---------------------------------------------------------------------------

#[test]
fn devpack_sources_do_not_declare_hasrole() {
    for rel in [
        "devpack/libraries/Runtime.sol",
        "devpack/contracts/Syscalls.sol",
        "devpack/contracts/NativeCalls.sol",
    ] {
        let path = format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(
            !text.contains("function hasRole"),
            "{rel} must not declare `function hasRole` — builtin devpack \
             libraries are compiler intrinsics; a declared-but-unlowered \
             member is an uncallable surface (and the removed implementation \
             ignored `role`, silently degrading to checkWitness)"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Whitelist ⊆ lowering sweep: every member advertised by
//    builtin_intrinsic_surface() must actually lower. A member counts as
//    lowerable when at least one probe arity does NOT produce the
//    "unsupported builtin library call" diagnostic for it (other targeted
//    diagnostics — e.g. arg-count or arg-type errors — are fine: they prove
//    a lowering exists and rejected the probe's shape, not that the member
//    is uncallable).
// ---------------------------------------------------------------------------

fn probe_source(base: &str, member: &str, arity: usize) -> String {
    let params = (0..arity)
        .map(|i| format!("bytes memory a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let args = (0..arity)
        .map(|i| format!("a{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "// SPDX-License-Identifier: MIT\n\
         pragma solidity ^0.8.19;\n\
         contract Probe {{\n\
             function probe({params}) public {{\n\
                 {base}.{member}({args});\n\
             }}\n\
         }}"
    )
}

#[test]
fn every_whitelisted_builtin_member_lowers() {
    let mut uncallable: Vec<String> = Vec::new();

    for (base, members) in builtin_intrinsic_surface() {
        for member in members {
            let mut lowered = false;
            let mut last_err = String::new();
            for arity in 0..=4usize {
                let src = probe_source(base, member, arity);
                match compile_err_text(&src) {
                    None => {
                        lowered = true;
                        break;
                    }
                    Some(err) => {
                        if !err.contains(&unsupported_builtin_msg(base, member)) {
                            // A targeted diagnostic (arg-count/type) — the
                            // member has a lowering that rejected this probe
                            // shape; that satisfies the invariant.
                            lowered = true;
                            break;
                        }
                        last_err = err;
                    }
                }
            }
            if !lowered {
                uncallable.push(format!("{base}.{member} (last error: {last_err})"));
            }
        }
    }

    assert!(
        uncallable.is_empty(),
        "builtin whitelist members without a lowering (uncallable intrinsics \
         — remove them from builtin_library_supported_members in \
         src/ir/context/builtins/resolve.rs or implement their lowering):\n{}",
        uncallable.join("\n")
    );
}

/// Duplicate whitelist entries are harmless to resolution but duplicate the
/// member in the "supported intrinsics" diagnostic; keep the lists clean.
#[test]
fn builtin_whitelists_have_no_duplicate_members() {
    for (base, members) in builtin_intrinsic_surface() {
        let mut seen = std::collections::HashSet::new();
        let dupes: Vec<&str> = members
            .iter()
            .copied()
            .filter(|m| !seen.insert(*m))
            .collect();
        assert!(
            dupes.is_empty(),
            "duplicate members in the '{base}' builtin whitelist: {dupes:?}"
        );
    }
}
