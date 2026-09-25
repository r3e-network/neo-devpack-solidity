#[test]
fn call_options_gas_is_ignored_for_external_calls() {
    let source = r#"
    pragma solidity ^0.8.19;

    interface IFoo {
        function foo() external;
    }

    contract GasOptionHarness {
        function run(address target) public {
            IFoo(target).foo{gas: 123}();
        }
    }
    "#;

    let metadata = analyse_source(source).expect("analysis failed");
    let _module = ir::Module::from_contract(&metadata).expect("IR lowering failed");
}
