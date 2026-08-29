#[test]
fn deploy_stub_is_injected_for_constructor() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract WithConstructor {
        uint256 public value;

        constructor() {
            value = 1;
        }
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");

    let deploy = metadata
        .methods
        .iter()
        .find(|method| method.name == "_deploy")
        .expect("_deploy should exist");

    assert_eq!(
        deploy.parameters.len(),
        2,
        "expected (data, update) parameters"
    );
    assert!(deploy.return_parameters.is_empty());
}

#[test]
fn constructor_with_parameters_is_supported() {
    let source = r#"
    pragma solidity ^0.8.19;

    contract NeedsArgs {
        uint256 public value;

        constructor(uint256 initialValue) {
            value = initialValue;
        }
    }
    "#;

    let mut metadata = analyse_source(source).expect("analysis failed");
    ensure_deploy_stub(&mut metadata).expect("deploy stub");

    assert!(
        metadata.methods.iter().any(|m| m.name == "_deploy"),
        "_deploy should be injected for parameterised constructor"
    );
}

