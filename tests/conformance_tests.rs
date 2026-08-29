//! Conformance Test Suite
//!
//! This test file runs the NeoVM conformance tests to validate
//! that the neo-devpack-solidity compiler produces correct bytecode.

mod conformance;

use conformance::{basic_test_vectors, ConformanceRunner};

#[test]
fn conformance_basic_vectors() {
    let vectors = basic_test_vectors();
    let runner = ConformanceRunner::new(vectors);
    let results = runner.run_all();

    let summary = ConformanceRunner::summary(&results);
    println!("\n{summary}");

    let mut failures = Vec::new();
    for result in &results {
        if !result.passed {
            failures.push(format!("  {} - {}", result.name, result.message));
        }
    }

    if !failures.is_empty() {
        println!("\nFailed tests:");
        for failure in &failures {
            println!("{failure}");
        }
    }

    assert!(
        summary.pass_rate >= 95.0,
        "Conformance pass rate too low: {:.1}% ({} failures)",
        summary.pass_rate,
        summary.failed
    );
}
