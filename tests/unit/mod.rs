//! Unit test suite for compiler components.
//!
//! This module is organized into logical test categories:
//! - lexer_tests: Tokenization and lexical analysis
//! - parser_tests: Parsing and AST construction
//! - semantic_tests: Semantic analysis and validation
//! - codegen_tests: Code generation and bytecode emission
//! - runtime_tests: Runtime execution and memory management
//! - integration_tests: End-to-end compilation workflows
//! - ir_tests: IR (Intermediate Representation) lowering and analysis
//! - storage_key_tests: Storage key computation and derivation

pub mod codegen_tests;
pub mod integration_tests;
pub mod ir_tests;
pub mod lexer_tests;
pub mod parser_tests;
pub mod runtime_tests;
pub mod semantic_tests;
pub mod storage_key_tests;
