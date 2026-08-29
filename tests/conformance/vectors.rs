//! Built-in conformance test vectors.
//!
//! Provides predefined test vectors for basic compiler validation.

use super::infrastructure::TestVector;

/// Built-in test vectors for basic validation
pub fn basic_test_vectors() -> Vec<TestVector> {
    vec![
        TestVector {
            name: "simple_return".to_string(),
            description: "Simple function that returns a constant".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract SimpleReturn {
                    function run() public pure returns (uint256) {
                        return 42;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "arithmetic_add".to_string(),
            description: "Basic addition operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ArithmeticAdd {
                    function run() public pure returns (uint256) {
                        return 10 + 32;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "arithmetic_mul".to_string(),
            description: "Basic multiplication operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ArithmeticMul {
                    function run() public pure returns (uint256) {
                        return 6 * 7;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "boolean_true".to_string(),
            description: "Boolean true returns 1".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BooleanTrue {
                    function run() public pure returns (uint256) {
                        bool b = true;
                        if (b) return 1;
                        return 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        TestVector {
            name: "boolean_false".to_string(),
            description: "Boolean false returns 0".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BooleanFalse {
                    function run() public pure returns (uint256) {
                        bool b = false;
                        if (b) return 1;
                        return 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(0),
            expected_success: true,
        },
        // ========== Arithmetic extensions ==========
        TestVector {
            name: "arithmetic_sub".to_string(),
            description: "Subtraction operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ArithmeticSub {
                    function run() public pure returns (uint256) {
                        return 100 - 58;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "arithmetic_div".to_string(),
            description: "Integer division".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ArithmeticDiv {
                    function run() public pure returns (uint256) {
                        return 84 / 2;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "arithmetic_mod".to_string(),
            description: "Modulo operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ArithmeticMod {
                    function run() public pure returns (uint256) {
                        return 47 % 5;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(2),
            expected_success: true,
        },
        // ========== Conditionals ==========
        TestVector {
            name: "if_else_true_branch".to_string(),
            description: "If-else takes true branch".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract IfElseTrue {
                    function run() public pure returns (uint256) {
                        uint256 x = 10;
                        if (x > 5) {
                            return 1;
                        } else {
                            return 0;
                        }
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        TestVector {
            name: "ternary_operator".to_string(),
            description: "Ternary conditional expression".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract Ternary {
                    function run() public pure returns (uint256) {
                        uint256 x = 3;
                        return x > 2 ? 42 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== Loops ==========
        TestVector {
            name: "for_loop_sum".to_string(),
            description: "For loop summing 1..10".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ForLoop {
                    function run() public pure returns (uint256) {
                        uint256 sum = 0;
                        for (uint256 i = 1; i <= 10; i++) {
                            sum += i;
                        }
                        return sum;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(55),
            expected_success: true,
        },
        TestVector {
            name: "while_loop".to_string(),
            description: "While loop counting down".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract WhileLoop {
                    function run() public pure returns (uint256) {
                        uint256 n = 10;
                        uint256 count = 0;
                        while (n > 0) {
                            n -= 1;
                            count += 1;
                        }
                        return count;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(10),
            expected_success: true,
        },
        TestVector {
            name: "do_while_loop".to_string(),
            description: "Do-while executes at least once".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract DoWhile {
                    function run() public pure returns (uint256) {
                        uint256 x = 0;
                        do {
                            x += 7;
                        } while (x < 5);
                        return x;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(7),
            expected_success: true,
        },
        // ========== Internal function calls ==========
        TestVector {
            name: "internal_call".to_string(),
            description: "Internal function call and return".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract InternalCall {
                    function double(uint256 x) internal pure returns (uint256) {
                        return x * 2;
                    }
                    function run() public pure returns (uint256) {
                        return double(21);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "nested_calls".to_string(),
            description: "Nested internal function calls".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract NestedCalls {
                    function add(uint256 a, uint256 b) internal pure returns (uint256) {
                        return a + b;
                    }
                    function triple(uint256 x) internal pure returns (uint256) {
                        return add(x, add(x, x));
                    }
                    function run() public pure returns (uint256) {
                        return triple(14);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== Bitwise operations ==========
        TestVector {
            name: "bitwise_and".to_string(),
            description: "Bitwise AND operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BitwiseAnd {
                    function run() public pure returns (uint256) {
                        return 0xFF & 0x0F;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(15),
            expected_success: true,
        },
        TestVector {
            name: "bitwise_or".to_string(),
            description: "Bitwise OR operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BitwiseOr {
                    function run() public pure returns (uint256) {
                        return 0x30 | 0x0A;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(58),
            expected_success: true,
        },
        TestVector {
            name: "bitwise_shift_left".to_string(),
            description: "Left shift operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ShiftLeft {
                    function run() public pure returns (uint256) {
                        return 1 << 8;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(256),
            expected_success: true,
        },
        // ========== Type conversions ==========
        TestVector {
            name: "uint8_to_uint256".to_string(),
            description: "Widen uint8 to uint256".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract Widen {
                    function run() public pure returns (uint256) {
                        uint8 small = 42;
                        return uint256(small);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "uint256_to_uint8".to_string(),
            description: "Narrow uint256 to uint8".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract Narrow {
                    function run() public pure returns (uint256) {
                        uint256 big = 42;
                        uint8 small = uint8(big);
                        return uint256(small);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "signed_cast".to_string(),
            description: "Cast int256 to uint256".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract SignedCast {
                    function run() public pure returns (uint256) {
                        int256 s = 42;
                        return uint256(s);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== Comparison operators ==========
        TestVector {
            name: "comparison_lt".to_string(),
            description: "Less-than comparison".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract CompLt {
                    function run() public pure returns (uint256) {
                        return 3 < 5 ? 1 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        TestVector {
            name: "comparison_eq".to_string(),
            description: "Equality comparison".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract CompEq {
                    function run() public pure returns (uint256) {
                        uint256 a = 42;
                        uint256 b = 42;
                        return a == b ? 1 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        // ========== Logical operators ==========
        TestVector {
            name: "logical_and".to_string(),
            description: "Logical AND short-circuit".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract LogicalAnd {
                    function run() public pure returns (uint256) {
                        bool a = true;
                        bool b = true;
                        return (a && b) ? 1 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        TestVector {
            name: "logical_or".to_string(),
            description: "Logical OR short-circuit".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract LogicalOr {
                    function run() public pure returns (uint256) {
                        bool a = false;
                        bool b = true;
                        return (a || b) ? 1 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        TestVector {
            name: "logical_not".to_string(),
            description: "Logical NOT operator".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract LogicalNot {
                    function run() public pure returns (uint256) {
                        bool a = false;
                        return (!a) ? 1 : 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        // ========== Local variables ==========
        TestVector {
            name: "local_variable_reassign".to_string(),
            description: "Local variable reassignment".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract LocalReassign {
                    function run() public pure returns (uint256) {
                        uint256 x = 10;
                        x = x + 20;
                        x = x + 12;
                        return x;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "nested_expression".to_string(),
            description: "Complex nested arithmetic".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract NestedExpr {
                    function run() public pure returns (uint256) {
                        return (2 + 3) * (4 + 4) + 2;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== Additional bitwise ==========
        TestVector {
            name: "bitwise_xor".to_string(),
            description: "Bitwise XOR operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BitwiseXor {
                    function run() public pure returns (uint256) {
                        return 0xFF ^ 0xF0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(15),
            expected_success: true,
        },
        TestVector {
            name: "bitwise_shift_right".to_string(),
            description: "Right shift operation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ShiftRight {
                    function run() public pure returns (uint256) {
                        return 256 >> 4;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(16),
            expected_success: true,
        },
        // ========== Compound assignment ==========
        TestVector {
            name: "compound_assignment".to_string(),
            description: "Compound assignment operators".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract CompoundAssign {
                    function run() public pure returns (uint256) {
                        uint256 x = 10;
                        x += 5;
                        x -= 3;
                        x *= 4;
                        return x;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(48),
            expected_success: true,
        },
        // ========== Break / continue ==========
        TestVector {
            name: "break_in_loop".to_string(),
            description: "Break exits loop early".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract BreakLoop {
                    function run() public pure returns (uint256) {
                        uint256 sum = 0;
                        for (uint256 i = 1; i <= 100; i++) {
                            if (i > 5) break;
                            sum += i;
                        }
                        return sum;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(15),
            expected_success: true,
        },
        // ========== Structs ==========
        // Known limitation: struct support may not be fully implemented
        TestVector {
            name: "struct_field_read".to_string(),
            description: "Define a struct, create instance, read a field".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract StructTest {
                    struct Point {
                        uint256 x;
                        uint256 y;
                    }
                    function run() public pure returns (uint256) {
                        Point memory p = Point(10, 32);
                        return p.x + p.y;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== Enums ==========
        // Known limitation: enum support may not be fully implemented
        TestVector {
            name: "enum_value".to_string(),
            description: "Define an enum and return its integer representation".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract EnumTest {
                    enum Status { Pending, Active, Closed }
                    function run() public pure returns (uint256) {
                        Status s = Status.Active;
                        return uint256(s);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        // ========== Fixed-size arrays ==========
        // Known limitation: fixed-size array support may not be fully implemented
        TestVector {
            name: "fixed_array_sum".to_string(),
            description: "Create a fixed array [1,2,3] and return the sum".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract FixedArrayTest {
                    function run() public pure returns (uint256) {
                        uint256[3] memory arr = [uint256(1), uint256(2), uint256(3)];
                        return arr[0] + arr[1] + arr[2];
                    }
                }
            "#
            .to_string(),
            expected_return: Some(6),
            expected_success: true,
        },
        // ========== Dynamic arrays ==========
        // Known limitation: dynamic array and .push() may not be fully implemented
        TestVector {
            name: "dynamic_array_length".to_string(),
            description: "Create a dynamic array, push values, return length".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract DynamicArrayTest {
                    function run() public returns (uint256) {
                        uint256[] memory arr = new uint256[](3);
                        arr[0] = 10;
                        arr[1] = 20;
                        arr[2] = 30;
                        return arr.length;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(3),
            expected_success: true,
        },
        // ========== Events ==========
        // Known limitation: event emission may not be fully implemented
        TestVector {
            name: "event_emit".to_string(),
            description: "Emit an event and verify execution succeeds".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract EventTest {
                    event ValueSet(uint256 value);
                    function run() public returns (uint256) {
                        emit ValueSet(42);
                        return 1;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(1),
            expected_success: true,
        },
        // ========== Modifiers ==========
        // Known limitation: modifier support may not be fully implemented
        TestVector {
            name: "modifier_pass".to_string(),
            description: "Modifier with require that passes".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ModifierPassTest {
                    modifier onlyPositive(uint256 x) {
                        require(x > 0);
                        _;
                    }
                    function guarded(uint256 x) internal pure onlyPositive(x) returns (uint256) {
                        return x * 2;
                    }
                    function run() public pure returns (uint256) {
                        return guarded(21);
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // Known limitation: modifier revert path may not be fully implemented
        TestVector {
            name: "modifier_fail".to_string(),
            description: "Modifier with require that reverts".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract ModifierFailTest {
                    modifier onlyPositive(uint256 x) {
                        require(x > 0);
                        _;
                    }
                    function guarded(uint256 x) internal pure onlyPositive(x) returns (uint256) {
                        return x * 2;
                    }
                    function run() public pure returns (uint256) {
                        return guarded(0);
                    }
                }
            "#
            .to_string(),
            expected_return: None,
            expected_success: false,
        },
        // ========== require / revert ==========
        TestVector {
            name: "require_true".to_string(),
            description: "require(true) passes execution".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract RequireTrue {
                    function run() public pure returns (uint256) {
                        require(true);
                        return 42;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        TestVector {
            name: "require_false".to_string(),
            description: "require(false) reverts execution".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract RequireFalse {
                    function run() public pure returns (uint256) {
                        require(false);
                        return 42;
                    }
                }
            "#
            .to_string(),
            expected_return: None,
            expected_success: false,
        },
        // ========== Multiple return values ==========
        // Known limitation: multiple return values not yet supported — local slot allocation
        // fails with "Local index 2 out of bounds" at runtime
        TestVector {
            name: "multiple_returns".to_string(),
            description: "Function returning multiple values, verify first".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract MultiReturn {
                    function getPair() internal pure returns (uint256, bool) {
                        return (42, true);
                    }
                    function run() public pure returns (uint256) {
                        (uint256 val, bool flag) = getPair();
                        if (flag) {
                            return val;
                        }
                        return 0;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
        // ========== String literals ==========
        // Known limitation: string/bytes support may not be fully implemented
        TestVector {
            name: "string_length".to_string(),
            description: "Return the byte-length of a string literal".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract StringLen {
                    function run() public pure returns (uint256) {
                        bytes memory s = bytes("hello");
                        return s.length;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(5),
            expected_success: true,
        },
        // ========== Ternary operator (false branch) ==========
        TestVector {
            name: "ternary_false_branch".to_string(),
            description: "Ternary takes false branch when condition is false".to_string(),
            source: r#"
                pragma solidity ^0.8.20;
                contract TernaryFalse {
                    function run() public pure returns (uint256) {
                        uint256 x = 1;
                        return x > 2 ? 99 : 42;
                    }
                }
            "#
            .to_string(),
            expected_return: Some(42),
            expected_success: true,
        },
    ]
}
