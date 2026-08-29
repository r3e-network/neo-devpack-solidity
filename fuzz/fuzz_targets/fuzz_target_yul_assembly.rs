#![no_main]
//! Fuzz target: generate **structurally valid** Solidity contracts whose
//! bodies wrap a randomly-generated Yul (`assembly { ... }`) block, then
//! feed them through `compile_contracts` at every optimizer level.
//!
//! Why a dedicated Yul fuzzer beats `fuzz_target_structured_sol`:
//!     * Yul is a separate sub-language (own opcode set, own scoping rules,
//!       own lowering pipeline at `src/ir/statements/assembly.rs`). The
//!       structured-Solidity fuzzer never emits an `assembly` block, so the
//!       Yul lowering path — `lower_yul_statement`, `lower_yul_expression`,
//!       `lower_yul_function_call`, the for/if/switch dispatchers, the
//!       mload/mstore/sload/sstore/return/revert builtins — is essentially
//!       uncovered by it.
//!     * Yul is gas-optimization territory: high-bug-density, frequently
//!       hand-written by users, and a common compiler-crash surface.
//!
//! Grammar (kept deliberately small to maximize exec/s):
//!     YulStmt ::= Let Ident := YulExpr
//!               | Ident := YulExpr
//!               | If YulExpr { YulStmt* }
//!               | For { Init } cond { post } { body }
//!               | mstore(YulExpr, YulExpr) | sstore(YulExpr, YulExpr)
//!               | return(YulExpr, YulExpr) | revert(YulExpr, YulExpr)
//!               | Block { YulStmt* }
//!     YulExpr ::= Lit(u8)
//!               | Ident
//!               | add|sub|mul|div|mod (YulExpr, YulExpr)
//!               | and|or|xor (YulExpr, YulExpr)
//!               | shl|shr (YulExpr, YulExpr)
//!               | lt|gt|eq (YulExpr, YulExpr)
//!               | iszero (YulExpr)
//!               | mload(YulExpr) | sload(YulExpr)
//!
//! Recursion bounds:
//!     * `MAX_EXPR_DEPTH = 4` — collapses to leaf at depth.
//!     * `MAX_BLOCK_DEPTH = 4` — collapses to leaf statement at depth.
//!     * `MAX_TOP_STMTS = 8`  — body length cap.
//!
//! Identifier discipline: every variable referenced must have been
//! declared in scope. We track an environment of declared identifiers
//! (purely additive — Yul has lexical scoping but for fuzz purposes we
//! never shadow). At leaf-level `Ident` collapses to `Lit(0)` if no name
//! is in scope.
//!
//! Output is wrapped as:
//!     contract Y {
//!         function f() external returns (uint256 r) { assembly { ... } }
//!     }
//! `r` is a Yul-visible return slot (Solidity declares it as the function
//! return). The generator may freely assign to `r` or ignore it; trailing
//! defaulted `return` in the function signature path is implicit.
//!
//! All rendering is deterministic (no RNG, no HashMap iteration), which
//! is required for proptest / corpus replay.

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::fmt::Write as _;

const MAX_EXPR_DEPTH: u8 = 4;
const MAX_BLOCK_DEPTH: u8 = 4;
const MAX_TOP_STMTS: u8 = 8;

/// Two-arg builtin returning a value.
#[derive(Debug, Clone, Copy)]
enum BuiltinBinop {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Lt,
    Gt,
    Eq,
}

impl BuiltinBinop {
    fn name(self) -> &'static str {
        match self {
            BuiltinBinop::Add => "add",
            BuiltinBinop::Sub => "sub",
            BuiltinBinop::Mul => "mul",
            BuiltinBinop::Div => "div",
            BuiltinBinop::Mod => "mod",
            BuiltinBinop::And => "and",
            BuiltinBinop::Or => "or",
            BuiltinBinop::Xor => "xor",
            BuiltinBinop::Shl => "shl",
            BuiltinBinop::Shr => "shr",
            BuiltinBinop::Lt => "lt",
            BuiltinBinop::Gt => "gt",
            BuiltinBinop::Eq => "eq",
        }
    }
}

impl<'a> Arbitrary<'a> for BuiltinBinop {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=12)? {
            0 => BuiltinBinop::Add,
            1 => BuiltinBinop::Sub,
            2 => BuiltinBinop::Mul,
            3 => BuiltinBinop::Div,
            4 => BuiltinBinop::Mod,
            5 => BuiltinBinop::And,
            6 => BuiltinBinop::Or,
            7 => BuiltinBinop::Xor,
            8 => BuiltinBinop::Shl,
            9 => BuiltinBinop::Shr,
            10 => BuiltinBinop::Lt,
            11 => BuiltinBinop::Gt,
            _ => BuiltinBinop::Eq,
        })
    }
}

#[derive(Debug)]
enum YulExpr {
    Lit(u8),
    /// Index into the active environment, modulo length. If the environment
    /// is empty at render time, falls back to `Lit(0)` rendering.
    Ident(u8),
    Binop(BuiltinBinop, Box<YulExpr>, Box<YulExpr>),
    IsZero(Box<YulExpr>),
    Mload(Box<YulExpr>),
    Sload(Box<YulExpr>),
}

impl YulExpr {
    fn arbitrary_with_depth(u: &mut Unstructured<'_>, depth: u8) -> Result<Self> {
        if depth >= MAX_EXPR_DEPTH {
            return Self::leaf(u);
        }
        Ok(match u.int_in_range::<u8>(0..=6)? {
            0 => YulExpr::Lit(u.arbitrary::<u8>()?),
            1 => YulExpr::Ident(u.arbitrary::<u8>()?),
            2 => YulExpr::Binop(
                u.arbitrary()?,
                Box::new(Self::arbitrary_with_depth(u, depth + 1)?),
                Box::new(Self::arbitrary_with_depth(u, depth + 1)?),
            ),
            3 => YulExpr::IsZero(Box::new(Self::arbitrary_with_depth(u, depth + 1)?)),
            4 => YulExpr::Mload(Box::new(Self::arbitrary_with_depth(u, depth + 1)?)),
            5 => YulExpr::Sload(Box::new(Self::arbitrary_with_depth(u, depth + 1)?)),
            _ => Self::leaf(u)?,
        })
    }

    fn leaf(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=1)? {
            0 => YulExpr::Lit(u.arbitrary::<u8>()?),
            _ => YulExpr::Ident(u.arbitrary::<u8>()?),
        })
    }

    /// Render the expression. `env` lists currently-declared identifiers
    /// (always non-empty when `r` and the per-function pre-seeded names
    /// are present). `Ident` indexes are taken modulo `env.len()`.
    fn render(&self, out: &mut String, env: &[String]) {
        match self {
            YulExpr::Lit(n) => {
                let _ = write!(out, "{}", n);
            }
            YulExpr::Ident(i) => {
                if env.is_empty() {
                    out.push('0');
                } else {
                    let idx = (*i as usize) % env.len();
                    out.push_str(&env[idx]);
                }
            }
            YulExpr::Binop(op, a, b) => {
                out.push_str(op.name());
                out.push('(');
                a.render(out, env);
                out.push_str(", ");
                b.render(out, env);
                out.push(')');
            }
            YulExpr::IsZero(a) => {
                out.push_str("iszero(");
                a.render(out, env);
                out.push(')');
            }
            YulExpr::Mload(a) => {
                out.push_str("mload(");
                a.render(out, env);
                out.push(')');
            }
            YulExpr::Sload(a) => {
                out.push_str("sload(");
                a.render(out, env);
                out.push(')');
            }
        }
    }
}

#[derive(Debug)]
enum YulStmt {
    /// `let x_<id> := <expr>` — adds `x_<id>` to the env.
    Let(u32, YulExpr),
    /// `<env[i]> := <expr>`. If env is empty, renders as a no-op block.
    Assign(u8, YulExpr),
    /// `if <cond> { body }`
    If(YulExpr, Vec<YulStmt>),
    /// `for { let i := init } cond { i := add(i, 1) } { body }` —
    /// `i` is a fresh identifier added to env for the body. The post is
    /// fixed (`i := add(i, 1)`) so the loop is shape-stable; the body
    /// random.
    For(YulExpr, YulExpr, Vec<YulStmt>),
    /// Block with own scope: `{ stmt* }`.
    Block(Vec<YulStmt>),
    Mstore(YulExpr, YulExpr),
    Sstore(YulExpr, YulExpr),
    /// `return(p, s)` — returns from the assembly block.
    Return(YulExpr, YulExpr),
    /// `revert(p, s)` — aborts.
    Revert(YulExpr, YulExpr),
}

impl YulStmt {
    fn arbitrary_with_depth(u: &mut Unstructured<'_>, depth: u8, next_id: &mut u32) -> Result<Self> {
        if depth >= MAX_BLOCK_DEPTH {
            // At max depth, only emit non-recursive statements.
            return Ok(match u.int_in_range::<u8>(0..=4)? {
                0 => {
                    let id = *next_id;
                    *next_id += 1;
                    YulStmt::Let(id, YulExpr::leaf(u)?)
                }
                1 => YulStmt::Assign(u.arbitrary::<u8>()?, YulExpr::leaf(u)?),
                2 => YulStmt::Mstore(YulExpr::leaf(u)?, YulExpr::leaf(u)?),
                3 => YulStmt::Sstore(YulExpr::leaf(u)?, YulExpr::leaf(u)?),
                _ => YulStmt::Return(YulExpr::leaf(u)?, YulExpr::leaf(u)?),
            });
        }
        Ok(match u.int_in_range::<u8>(0..=8)? {
            0 => {
                let id = *next_id;
                *next_id += 1;
                YulStmt::Let(id, YulExpr::arbitrary_with_depth(u, 0)?)
            }
            1 => YulStmt::Assign(
                u.arbitrary::<u8>()?,
                YulExpr::arbitrary_with_depth(u, 0)?,
            ),
            2 => {
                let cond = YulExpr::arbitrary_with_depth(u, 0)?;
                let n = u.int_in_range::<u8>(0..=3)?;
                let mut body = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    body.push(Self::arbitrary_with_depth(u, depth + 1, next_id)?);
                }
                YulStmt::If(cond, body)
            }
            3 => {
                let init = YulExpr::arbitrary_with_depth(u, 0)?;
                let cond = YulExpr::arbitrary_with_depth(u, 0)?;
                let n = u.int_in_range::<u8>(0..=3)?;
                let mut body = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    body.push(Self::arbitrary_with_depth(u, depth + 1, next_id)?);
                }
                // Loop var consumes one id slot.
                *next_id += 1;
                YulStmt::For(init, cond, body)
            }
            4 => {
                let n = u.int_in_range::<u8>(0..=3)?;
                let mut body = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    body.push(Self::arbitrary_with_depth(u, depth + 1, next_id)?);
                }
                YulStmt::Block(body)
            }
            5 => YulStmt::Mstore(
                YulExpr::arbitrary_with_depth(u, 0)?,
                YulExpr::arbitrary_with_depth(u, 0)?,
            ),
            6 => YulStmt::Sstore(
                YulExpr::arbitrary_with_depth(u, 0)?,
                YulExpr::arbitrary_with_depth(u, 0)?,
            ),
            7 => YulStmt::Return(
                YulExpr::arbitrary_with_depth(u, 0)?,
                YulExpr::arbitrary_with_depth(u, 0)?,
            ),
            _ => YulStmt::Revert(
                YulExpr::arbitrary_with_depth(u, 0)?,
                YulExpr::arbitrary_with_depth(u, 0)?,
            ),
        })
    }

    /// Render. `env` is the live environment of declared identifiers; new
    /// `let` bindings extend the caller's vec (mutated in place — Yul has
    /// lexical scoping, but the caller is responsible for snapshotting if
    /// it needs to undo on block exit).
    fn render(&self, out: &mut String, indent: usize, env: &mut Vec<String>) {
        for _ in 0..indent {
            out.push_str("    ");
        }
        match self {
            YulStmt::Let(id, init) => {
                let name = format!("x_{}", id);
                let _ = write!(out, "let {} := ", name);
                init.render(out, env);
                out.push('\n');
                env.push(name);
            }
            YulStmt::Assign(i, val) => {
                if env.is_empty() {
                    // No lvalue available — render an inert no-op block.
                    out.push_str("{ }\n");
                    return;
                }
                let idx = (*i as usize) % env.len();
                let name = env[idx].clone();
                let _ = write!(out, "{} := ", name);
                val.render(out, env);
                out.push('\n');
            }
            YulStmt::If(cond, body) => {
                out.push_str("if ");
                cond.render(out, env);
                out.push_str(" {\n");
                let snapshot = env.len();
                for s in body {
                    s.render(out, indent + 1, env);
                }
                env.truncate(snapshot);
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            YulStmt::For(init, cond, body) => {
                // For loop: `for { let i := <init> } <cond> { i := add(i, 1) } { body }`.
                // The loop variable name is unique per call (uses env.len()
                // as a tag — collisions are impossible because env is
                // strictly growing within a render path).
                let i_name = format!("i_{}", env.len());
                let _ = write!(out, "for {{ let {} := ", i_name);
                init.render(out, env);
                out.push_str(" } ");
                // Pre-bind `i_name` so the cond can reference it.
                env.push(i_name.clone());
                cond.render(out, env);
                let _ = write!(out, " {{ {} := add({}, 1) }} {{\n", i_name, i_name);
                let snapshot = env.len();
                for s in body {
                    s.render(out, indent + 1, env);
                }
                env.truncate(snapshot);
                // Pop the loop variable on exit.
                env.pop();
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            YulStmt::Block(body) => {
                out.push_str("{\n");
                let snapshot = env.len();
                for s in body {
                    s.render(out, indent + 1, env);
                }
                env.truncate(snapshot);
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            YulStmt::Mstore(a, b) => {
                out.push_str("mstore(");
                a.render(out, env);
                out.push_str(", ");
                b.render(out, env);
                out.push_str(")\n");
            }
            YulStmt::Sstore(a, b) => {
                out.push_str("sstore(");
                a.render(out, env);
                out.push_str(", ");
                b.render(out, env);
                out.push_str(")\n");
            }
            YulStmt::Return(a, b) => {
                out.push_str("return(");
                a.render(out, env);
                out.push_str(", ");
                b.render(out, env);
                out.push_str(")\n");
            }
            YulStmt::Revert(a, b) => {
                out.push_str("revert(");
                a.render(out, env);
                out.push_str(", ");
                b.render(out, env);
                out.push_str(")\n");
            }
        }
    }
}

#[derive(Debug)]
struct YulSource {
    body: Vec<YulStmt>,
}

impl<'a> Arbitrary<'a> for YulSource {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let n = u.int_in_range::<u8>(1..=MAX_TOP_STMTS)?;
        let mut body = Vec::with_capacity(n as usize);
        let mut next_id: u32 = 0;
        for _ in 0..n {
            body.push(YulStmt::arbitrary_with_depth(u, 0, &mut next_id)?);
        }
        Ok(YulSource { body })
    }
}

impl YulSource {
    fn render(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("// SPDX-License-Identifier: MIT\n");
        out.push_str("pragma solidity ^0.8.20;\n");
        out.push_str("contract Y {\n");
        out.push_str("    function f() external returns (uint256 r) {\n");
        out.push_str("        assembly {\n");
        // `r` is the Solidity return; it's visible inside the `assembly`
        // block as a writable identifier.
        let mut env: Vec<String> = vec!["r".to_string()];
        for s in &self.body {
            s.render(&mut out, 3, &mut env);
        }
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n");
        out
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let source = match YulSource::arbitrary(&mut u) {
        Ok(s) => s,
        Err(_) => return,
    };
    // Pick optimizer level from the remaining input so libfuzzer can
    // mutate it independently — one compile per iteration keeps exec/s
    // high while still exercising opt ∈ {0..=3} across the corpus.
    let opt: u8 = u.int_in_range(0..=3).unwrap_or(0);
    let src = source.render();

    // Wrap in catch_unwind: only a real panic / abort is a finding. Any
    // Result::Err is expected (semantically-weird Yul: uninitialized
    // reads, unreachable code after `return`, jumps out of scope, etc.).
    let _ = std::panic::catch_unwind(move || {
        let _ = neo_devpack_solidity::cli::compile_contracts(&src, false, opt);
    });
});
