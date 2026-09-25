use super::*;
use std::fs;

#[test]
fn single_file_compilation_resolves_relative_imports() {
    let temp = tempfile::tempdir().expect("tempdir");
    let base_path = temp.path().join("Base.sol");
    let derived_path = temp.path().join("Derived.sol");

    fs::write(
        &base_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.19;

        contract Base {
            function foo() public pure returns (uint256) {
                return 1;
            }
        }
        "#,
    )
    .expect("write Base.sol");

    fs::write(
        &derived_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.19;
        import "./Base.sol";

        contract Derived is Base {}
        "#,
    )
    .expect("write Derived.sol");

    let resolved =
        resolve_solidity_sources_with_imports(&derived_path, &[]).expect("resolve imports");
    assert_eq!(resolved.files.len(), 2, "expected Base + Derived sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("compilation with imports should succeed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Base"));
    assert!(names.contains(&"Derived"));
}

#[test]
fn single_file_compilation_resolves_imports_via_include_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lib_dir = temp.path().join("lib");
    fs::create_dir_all(&lib_dir).expect("create lib dir");

    let lib_path = lib_dir.join("Lib.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &lib_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.19;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    fs::write(
        &entry_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.19;
        import "Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[lib_dir])
        .expect("resolve imports via include path");
    assert_eq!(resolved.files.len(), 2, "expected Lib + Entry sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("compilation with include path imports should succeed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_accepts_import_aliasing_for_dependency_resolution() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lib_path = temp.path().join("Lib.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &lib_path,
        r#"
        pragma solidity ^0.8.19;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import { Lib as MathLib } from "./Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("alias import should resolve");
    assert_eq!(resolved.files.len(), 2, "expected Lib + Entry sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("compilation with aliased import syntax should succeed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_accepts_global_symbol_imports_for_dependency_resolution() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lib_path = temp.path().join("Lib.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &lib_path,
        r#"
        pragma solidity ^0.8.19;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import * as LibNS from "./Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("global-symbol import should resolve");
    assert_eq!(resolved.files.len(), 2, "expected Lib + Entry sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("compilation with wildcard import syntax should succeed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_aliased_symbol_binding() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lib_path = temp.path().join("Lib.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &lib_path,
        r#"
        pragma solidity ^0.8.19;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import { Lib as MathLib } from "./Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return MathLib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("alias import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("aliased symbol binding should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_wildcard_namespace_binding() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lib_path = temp.path().join("Lib.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &lib_path,
        r#"
        pragma solidity ^0.8.19;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import * as LibNS from "./Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return LibNS.Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("global-symbol import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("wildcard namespace binding should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_wildcard_namespace_type_cast_binding() {
    let temp = tempfile::tempdir().expect("tempdir");
    let iface_path = temp.path().join("IFoo.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &iface_path,
        r#"
        pragma solidity ^0.8.19;

        interface IFoo {
            function foo() external;
        }
        "#,
    )
    .expect("write IFoo.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import * as LibNS from "./IFoo.sol";

        contract Entry {
            function ping(address target) public {
                LibNS.IFoo(target).foo();
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("global-symbol import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("wildcard namespace type-cast binding should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_wildcard_namespace_selector_binding() {
    let temp = tempfile::tempdir().expect("tempdir");
    let iface_path = temp.path().join("IFoo.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &iface_path,
        r#"
        pragma solidity ^0.8.19;

        interface IFoo {
            function foo() external;
        }
        "#,
    )
    .expect("write IFoo.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import * as LibNS from "./IFoo.sol";

        contract Entry {
            function selectorOfFoo() public pure returns (bytes4) {
                return LibNS.IFoo.foo.selector;
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("global-symbol import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("wildcard namespace selector binding should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_aliased_encode_call_function_reference() {
    let temp = tempfile::tempdir().expect("tempdir");
    let iface_path = temp.path().join("IFoo.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &iface_path,
        r#"
        pragma solidity ^0.8.19;

        interface IFoo {
            function bar(uint256 x) external;
        }
        "#,
    )
    .expect("write IFoo.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import { IFoo as FooIface } from "./IFoo.sol";

        contract Entry {
            function payload() public pure returns (bytes memory) {
                return abi.encodeCall(FooIface.bar, (7));
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("alias import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("aliased encodeCall function reference should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn single_file_compilation_resolves_wildcard_namespace_encode_call_function_reference() {
    let temp = tempfile::tempdir().expect("tempdir");
    let iface_path = temp.path().join("IFoo.sol");
    let entry_path = temp.path().join("Entry.sol");

    fs::write(
        &iface_path,
        r#"
        pragma solidity ^0.8.19;

        interface IFoo {
            function bar(uint256 x) external;
        }
        "#,
    )
    .expect("write IFoo.sol");

    fs::write(
        &entry_path,
        r#"
        pragma solidity ^0.8.19;
        import * as FooNS from "./IFoo.sol";

        contract Entry {
            function payload() public pure returns (bytes memory) {
                return abi.encodeCall(FooNS.IFoo.bar, (7));
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("wildcard import should resolve");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("wildcard namespace encodeCall function reference should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn auto_discovers_node_modules_for_npm_style_imports() {
    // Regression: bare imports like `@openzeppelin/contracts/...` should
    // resolve against a sibling `node_modules/` without the user needing to
    // pass `--include-path` explicitly. solc / hardhat / foundry all do this
    // for npm-style imports; we mirror their behaviour by climbing parents of
    // the entry file looking for `node_modules`.
    let temp = tempfile::tempdir().expect("tempdir");

    let project_dir = temp.path().join("project");
    let contracts_dir = project_dir.join("contracts");
    fs::create_dir_all(&contracts_dir).expect("project layout");

    let node_modules = project_dir.join("node_modules");
    let pkg_dir = node_modules.join("@scope").join("pkg");
    fs::create_dir_all(&pkg_dir).expect("create pkg dir");
    fs::write(
        pkg_dir.join("Lib.sol"),
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;

        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    let entry_path = contracts_dir.join("Entry.sol");
    fs::write(
        &entry_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;
        import "@scope/pkg/Lib.sol";

        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let resolved = resolve_solidity_sources_with_imports(&entry_path, &[])
        .expect("node_modules import should resolve without explicit include-path");
    assert_eq!(resolved.files.len(), 2, "expected Lib + Entry sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("auto-discovered node_modules import should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn remap_flag_rewrites_import_prefix() {
    // Foundry-style remap: any import beginning with `@oz/` is rewritten to
    // start with the supplied replacement path. Mirrors solc's
    // `--allow-paths` + remappings feature and matches the Foundry default.
    let temp = tempfile::tempdir().expect("tempdir");
    let oz_layout = temp.path().join("oz_install");
    fs::create_dir_all(&oz_layout).expect("create oz dir");
    fs::write(
        oz_layout.join("Lib.sol"),
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;
        library Lib {
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                return a + b;
            }
        }
        "#,
    )
    .expect("write Lib.sol");

    let entry_path = temp.path().join("Entry.sol");
    fs::write(
        &entry_path,
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;
        import "@oz/Lib.sol";
        contract Entry {
            function sum(uint256 a, uint256 b) public pure returns (uint256) {
                return Lib.add(a, b);
            }
        }
        "#,
    )
    .expect("write Entry.sol");

    let remap =
        parse_remapping(&format!("@oz/={}/", oz_layout.display())).expect("parse remapping");
    let resolved = resolve_solidity_sources_with_options(&entry_path, &[], &[remap])
        .expect("remap should let bare-prefix import resolve");
    assert_eq!(resolved.files.len(), 2, "expected Lib + Entry sources");
    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("remapped imports should compile");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"Entry"));
}

#[test]
fn remappings_file_is_parsed_with_comments_and_blank_lines_ignored() {
    let temp = tempfile::tempdir().expect("tempdir");
    let target = temp.path().join("packages").join("foo");
    fs::create_dir_all(&target).expect("create dir");
    let remap_path = temp.path().join("remappings.txt");
    fs::write(
        &remap_path,
        format!(
            "# foundry-style remappings.txt\n\n@foo/={}/\n# trailing comment\n",
            target.display()
        ),
    )
    .expect("write remappings.txt");

    let remaps = load_remappings_file(&remap_path).expect("parse remappings");
    assert_eq!(remaps.len(), 1, "comments + blanks should be skipped");
    assert_eq!(remaps[0].prefix, "@foo/");
}

#[test]
fn cross_package_relative_imports_resolve_via_virtual_path() {
    // Regression: when a vendored file at `vendor/<pkg>/.../A.sol` does
    // `import "./B.sol"`, but `B.sol` was not vendored alongside it, the
    // resolver should fall back to resolving the relative import against the
    // file's virtual package path under any other include path that contains
    // a copy of the same package (e.g. an installed node_modules).
    let temp = tempfile::tempdir().expect("tempdir");

    // Layout 1: vendored copy that only contains A.sol (no B.sol next to it).
    let vendor_pkg = temp.path().join("vendor").join("@scope").join("pkg");
    fs::create_dir_all(&vendor_pkg).expect("vendor layout");
    fs::write(
        vendor_pkg.join("A.sol"),
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;
        import "./B.sol";

        contract A is B {}
        "#,
    )
    .expect("write A.sol");

    // Layout 2: fully populated install that has B.sol.
    let install_pkg = temp.path().join("install").join("@scope").join("pkg");
    fs::create_dir_all(&install_pkg).expect("install layout");
    fs::write(
        install_pkg.join("B.sol"),
        r#"
        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.20;

        contract B {
            function ping() public pure returns (uint256) { return 1; }
        }
        "#,
    )
    .expect("write B.sol");

    // The entry file is the vendored A.sol. The include path points at the
    // *install* root, which gives us a virtual layout of @scope/pkg/B.sol —
    // the resolver should bridge across.
    let resolved = resolve_solidity_sources_with_imports(
        &vendor_pkg.join("A.sol"),
        &[temp.path().join("install"), temp.path().join("vendor")],
    )
    .expect("virtual-path resolution should bridge to install copy");

    assert_eq!(resolved.files.len(), 2, "expected A + B sources");

    let artifacts = compile_contracts(&resolved.combined_source, false, 2)
        .expect("compilation across virtual-path-resolved import should succeed");
    let names: Vec<_> = artifacts.iter().map(|a| a.metadata.name.as_str()).collect();
    assert!(names.contains(&"A"));
    assert!(names.contains(&"B"));
}
