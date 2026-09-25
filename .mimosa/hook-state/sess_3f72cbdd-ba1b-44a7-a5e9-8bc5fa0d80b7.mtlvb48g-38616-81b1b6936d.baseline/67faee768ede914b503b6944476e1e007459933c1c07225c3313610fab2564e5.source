use neo_devpack_solidity::runtime::types::RuntimeValue;
use std::collections::HashMap;

#[test]
fn runtime_value_map_roundtrips_stack_item() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), RuntimeValue::UnsignedInteger(42));
    let value = RuntimeValue::Map(map);

    let stack_item = value.to_stack_item();
    let back = RuntimeValue::from_stack_item(&stack_item);

    if let RuntimeValue::Map(round) = back {
        assert_eq!(
            round.get("key"),
            Some(&RuntimeValue::UnsignedInteger(42)),
            "map value should round-trip through stack item"
        );
    } else {
        panic!("expected map after round-trip");
    }
}
