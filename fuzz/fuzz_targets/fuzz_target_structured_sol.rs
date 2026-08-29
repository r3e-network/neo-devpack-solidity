#![no_main]
//! Fuzz target: generate **structurally valid** Solidity sources from a
//! small AST grammar via `arbitrary::Arbitrary`, then feed them through
//! `compile_contracts` at every optimizer level.
//!
//! Why this beats the raw-UTF-8 target (`fuzz_target_1`):
//! random bytes almost always die at the lexer / parser, so deeper layers
//! (IR builder, type checker, optimizer, NeoVM emitter) are barely
//! exercised. Inputs produced here are *always* lex-/parse-clean by
//! construction, so each iteration drives the compiler several layers
//! deeper, dramatically expanding effective coverage per run.
//!
//! Grammar (kept deliberately small):
//!     Source   ::= [Library] BaseContract* MainContract
//!     Contract ::= [abstract] contract C [is Base[, Mid]] { ... }
//!     Function ::= function f_i(Param*) [virtual|override] [returns (Type*)] { Stmt* }
//!     Stmt     ::= Block | If | Return | StateAdd | Emit | Require
//!     Expr     ::= Lit | Var | Binary | Unary | Compare | Ternary | UsingForCall
//!     Type     ::= uint256 | int256 | bool | address | bytes32
//!
//! Recursion depth is bounded (`MAX_DEPTH = 6`) so the rendered source
//! cannot exceed a small constant size. All rendering is deterministic
//! (no RNG, no HashMap iteration), which is required for proptest /
//! corpus replay.
//!
//! Inheritance / library bounds (anti-explosion):
//!     - At most 2 base contracts (depth 1) plus 1 mid contract (depth 2),
//!       i.e. inheritance chain length <= 3.
//!     - Main contract `is` clause picks 0..=2 of the available bases.
//!     - At most 1 library, with at most 2 functions.
//!     - At most 4 functions per contract (unchanged from baseline).

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::fmt::Write as _;

/// Maximum AST recursion depth. Anything past this collapses to a leaf,
/// preventing the rendered source from ballooning. Six is the spec cap;
/// deeper trees explode rendered-source size and tank exec/s without
/// adding new coverage paths.
const MAX_DEPTH: u8 = 6;

/// Maximum number of base contracts available for inheritance. The main
/// contract may pick 0..=MAX_BASES of these to inherit from. Capped low
/// to keep generated source bounded.
const MAX_BASES: u8 = 2;

/// Maximum functions per contract. Same as the original baseline; raising
/// this multiplies rendered-source size without commensurate coverage win.
const MAX_FUNCS_PER_CONTRACT: u8 = 4;

#[derive(Debug, Clone, Copy)]
enum SolType {
    Uint256,
    Int256,
    Bool,
    Address,
    Bytes32,
}

impl<'a> Arbitrary<'a> for SolType {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=4)? {
            0 => SolType::Uint256,
            1 => SolType::Int256,
            2 => SolType::Bool,
            3 => SolType::Address,
            _ => SolType::Bytes32,
        })
    }
}

impl SolType {
    fn name(self) -> &'static str {
        match self {
            SolType::Uint256 => "uint256",
            SolType::Int256 => "int256",
            SolType::Bool => "bool",
            SolType::Address => "address",
            SolType::Bytes32 => "bytes32",
        }
    }

    /// A literal value of this type. Always parses; type-checking may or
    /// may not pass — that's the point of fuzzing.
    fn literal(self, n: u64) -> String {
        match self {
            SolType::Uint256 | SolType::Int256 => format!("uint256({})", n & 0xff),
            SolType::Bool => {
                if n & 1 == 0 {
                    "false".into()
                } else {
                    "true".into()
                }
            }
            SolType::Address => format!("address(uint160({}))", n & 0xff),
            SolType::Bytes32 => format!("bytes32(uint256({}))", n & 0xff),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
}

impl<'a> Arbitrary<'a> for BinOp {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=7)? {
            0 => BinOp::Add,
            1 => BinOp::Sub,
            2 => BinOp::Mul,
            3 => BinOp::Div,
            4 => BinOp::Mod,
            5 => BinOp::BitAnd,
            6 => BinOp::BitOr,
            _ => BinOp::BitXor,
        })
    }
}

impl BinOp {
    fn sym(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum UnOp {
    Not,
    BitNot,
    Neg,
}

impl<'a> Arbitrary<'a> for UnOp {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=2)? {
            0 => UnOp::Not,
            1 => UnOp::BitNot,
            _ => UnOp::Neg,
        })
    }
}

impl UnOp {
    fn sym(self) -> &'static str {
        match self {
            UnOp::Not => "!",
            UnOp::BitNot => "~",
            UnOp::Neg => "-",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl<'a> Arbitrary<'a> for CmpOp {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=5)? {
            0 => CmpOp::Eq,
            1 => CmpOp::Ne,
            2 => CmpOp::Lt,
            3 => CmpOp::Le,
            4 => CmpOp::Gt,
            _ => CmpOp::Ge,
        })
    }
}

impl CmpOp {
    fn sym(self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
        }
    }
}

#[derive(Debug)]
enum Expr {
    LitInt(u64),
    LitBool(bool),
    State,                          // refers to `state`
    Param(u8),                      // refers to p_{n % param_count}
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `using L for uint256` member call: `(<inner>).add1()`. Only emitted
    /// when the contract has the `using_for` flag set; renderer always
    /// produces a uint256-typed receiver via `uint256(...)` cast.
    UsingForAdd1(Box<Expr>),
}

impl Expr {
    fn arbitrary_with_depth(u: &mut Unstructured<'_>, depth: u8, using_for: bool) -> Result<Self> {
        // Collapse to leaf at max depth.
        if depth >= MAX_DEPTH {
            return Self::leaf(u);
        }
        // Bias range up by one when using-for is enabled, so the new
        // production is reachable but not dominant.
        let max = if using_for { 9 } else { 8 };
        match u.int_in_range::<u8>(0..=max)? {
            0 => Ok(Expr::LitInt(u.arbitrary::<u64>()?)),
            1 => Ok(Expr::LitBool(u.arbitrary::<bool>()?)),
            2 => Ok(Expr::State),
            3 => Ok(Expr::Param(u.arbitrary::<u8>()?)),
            4 => Ok(Expr::Binary(
                u.arbitrary()?,
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
            )),
            5 => Ok(Expr::Unary(
                u.arbitrary()?,
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
            )),
            6 => Ok(Expr::Cmp(
                u.arbitrary()?,
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
            )),
            7 => Ok(Expr::Ternary(
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
                Box::new(Self::arbitrary_with_depth(u, depth + 1, using_for)?),
            )),
            8 => Self::leaf(u),
            _ => Ok(Expr::UsingForAdd1(Box::new(Self::arbitrary_with_depth(
                u,
                depth + 1,
                using_for,
            )?))),
        }
    }

    fn leaf(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(match u.int_in_range::<u8>(0..=3)? {
            0 => Expr::LitInt(u.arbitrary::<u64>()?),
            1 => Expr::LitBool(u.arbitrary::<bool>()?),
            2 => Expr::State,
            _ => Expr::Param(u.arbitrary::<u8>()?),
        })
    }

    /// Render with the given parameter count (so Param indices remain valid).
    fn render(&self, out: &mut String, param_count: u8) {
        match self {
            Expr::LitInt(n) => {
                let _ = write!(out, "uint256({})", n & 0xff);
            }
            Expr::LitBool(b) => {
                out.push_str(if *b { "true" } else { "false" });
            }
            Expr::State => out.push_str("state"),
            Expr::Param(i) => {
                if param_count == 0 {
                    out.push_str("uint256(0)");
                } else {
                    let _ = write!(out, "p_{}", i % param_count);
                }
            }
            Expr::Binary(op, a, b) => {
                out.push('(');
                a.render(out, param_count);
                let _ = write!(out, " {} ", op.sym());
                b.render(out, param_count);
                out.push(')');
            }
            Expr::Unary(op, a) => {
                out.push('(');
                out.push_str(op.sym());
                a.render(out, param_count);
                out.push(')');
            }
            Expr::Cmp(op, a, b) => {
                out.push('(');
                a.render(out, param_count);
                let _ = write!(out, " {} ", op.sym());
                b.render(out, param_count);
                out.push(')');
            }
            Expr::Ternary(c, t, e) => {
                out.push('(');
                c.render(out, param_count);
                out.push_str(" ? ");
                t.render(out, param_count);
                out.push_str(" : ");
                e.render(out, param_count);
                out.push(')');
            }
            Expr::UsingForAdd1(inner) => {
                // Force a uint256 receiver so the `using L for uint256` call
                // resolves cleanly. Inner expression is wrapped in a cast.
                out.push_str("(uint256(");
                inner.render(out, param_count);
                out.push_str(") & 0xff).add1()");
            }
        }
    }
}

/// Per-function caps to bound generation explosion of new statement kinds.
/// Mutated as the grammar emits the bounded variants.
#[derive(Debug, Default, Clone, Copy)]
struct StmtBudget {
    /// Remaining `revert("...")` allowance for the current function.
    revert_remaining: u8,
    /// Remaining try/catch allowance for the current function (typically 1).
    trycatch_remaining: u8,
    /// Remaining bounded `for`/`while` loop allowance (loop lowering coverage).
    loop_remaining: u8,
    /// Remaining local-variable-declaration allowance (slot-alloc coverage).
    local_remaining: u8,
    /// Remaining `mapping`/dynamic-array op allowance (storage-collection
    /// lowering coverage).
    collection_remaining: u8,
    /// Remaining devpack-framework call allowance (`Runtime.*` / `Storage.*`
    /// auto-resolution + syscall lowering coverage).
    devpack_remaining: u8,
    /// Monotonic id source for unique loop / local variable names so nested
    /// scopes never shadow (which neo-solc may reject).
    next_id: u16,
}

impl StmtBudget {
    fn fresh_id(&mut self) -> u16 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

/// Generate a small ASCII string literal of `1..=16` chars. All chars are
/// safe to embed in a Solidity double-quoted string (alphanumeric only).
fn arb_short_string(u: &mut Unstructured<'_>) -> Result<String> {
    let len = u.int_in_range::<u8>(1..=16)?;
    let mut s = String::with_capacity(len as usize);
    for _ in 0..len {
        let n = u.int_in_range::<u8>(0..=35)?;
        let c = if n < 10 {
            (b'0' + n) as char
        } else {
            (b'a' + (n - 10)) as char
        };
        s.push(c);
    }
    Ok(s)
}

#[derive(Debug)]
enum Stmt {
    Block(Vec<Stmt>),
    If(Expr, Vec<Stmt>),
    Return(Expr),
    StateAdd, // `state += 1;`
    Emit(Expr),
    Require(Expr),
    /// `revert("xxx");` — exercises the revert string ABI-encode path.
    RevertString(String),
    /// `require(cond, "xxx");` — exercises Error(string) revert envelope.
    RequireWithMsg(Expr, String),
    /// `try this.helper() returns (uint256 v) { state += v; } catch (bytes memory) { state = 0xfe; }`
    /// Exercises the canonical Panic/Error catch dispatch. We only emit the
    /// bytes-catch arm — Panic/Error arms need matching call shapes that
    /// would explode the grammar.
    TryCatch,
    /// `for (uint256 i_<id> = 0; i_<id> < <count>; i_<id>++) { body }`.
    /// Drives the loop-lowering codegen (init / cond / incr / back-edge JMP),
    /// a path the original grammar never reached. `count` is a small literal.
    ForLoop { id: u16, count: u8, body: Vec<Stmt> },
    /// `while (state < <count>) { state += 1; <body> }` — guaranteed to
    /// terminate (monotonic state vs literal bound). Exercises the `while`
    /// back-edge / pre-test loop shape distinctly from `for`.
    WhileLoop { count: u8, body: Vec<Stmt> },
    /// `uint256 v_<id> = uint256(<expr>) & 0xff; state += v_<id>;` — exercises
    /// local slot allocation + a read-back in the same scope.
    LocalDecl { id: u16, expr: Expr },
    /// `sm[uint256(<key>) & 0xff] = uint256(<val>) & 0xff;` — mapping store.
    MapStore { key: Expr, val: Expr },
    /// `state += sm[uint256(<key>) & 0xff];` — mapping load.
    MapLoad { key: Expr },
    /// `sa.push(uint256(<expr>) & 0xff);` — dynamic-array push.
    ArrPush { expr: Expr },
    /// `if (sa.length > 0) { state += sa[sa.length - 1]; sa.pop(); }` —
    /// guarded array index + pop (exercises bounds + length + pop lowering).
    ArrPopGuarded,
    /// A devpack-framework call. `Runtime.*` / `Storage.*` auto-resolve to Neo
    /// syscalls with no import, so emitting them here fuzzes the devpack
    /// resolution + lowering paths (`src/ir/context/builtins/resolve.rs`).
    /// `kind` selects the concrete call shape (see `render`).
    Devpack { kind: u8, expr: Expr, id: u16, msg: String },
}

impl Stmt {
    fn arbitrary_with_depth(
        u: &mut Unstructured<'_>,
        depth: u8,
        using_for: bool,
        budget: &mut StmtBudget,
    ) -> Result<Self> {
        if depth >= MAX_DEPTH {
            // Leaf-only at max depth.
            return Ok(match u.int_in_range::<u8>(0..=3)? {
                0 => Stmt::Return(Expr::leaf(u)?),
                1 => Stmt::StateAdd,
                2 => Stmt::Emit(Expr::leaf(u)?),
                _ => Stmt::Require(Expr::leaf(u)?),
            });
        }
        // Range 0..=13 covers the original kinds 0..=7 plus six new
        // budget-gated productions (loops, locals, mapping/array ops, devpack
        // calls), each falling back to a benign statement when its budget is
        // exhausted so probability mass stays sane.
        Ok(match u.int_in_range::<u8>(0..=13)? {
            0 => {
                // Block of 0..=2 inner statements.
                let n = u.int_in_range::<u8>(0..=2)?;
                let mut inner = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    inner.push(Self::arbitrary_with_depth(u, depth + 1, using_for, budget)?);
                }
                Stmt::Block(inner)
            }
            1 => {
                let cond = Expr::arbitrary_with_depth(u, depth + 1, using_for)?;
                let n = u.int_in_range::<u8>(0..=2)?;
                let mut body = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    body.push(Self::arbitrary_with_depth(u, depth + 1, using_for, budget)?);
                }
                Stmt::If(cond, body)
            }
            2 => Stmt::Return(Expr::arbitrary_with_depth(u, depth + 1, using_for)?),
            3 => Stmt::StateAdd,
            4 => Stmt::Emit(Expr::arbitrary_with_depth(u, depth + 1, using_for)?),
            5 => Stmt::Require(Expr::arbitrary_with_depth(u, depth + 1, using_for)?),
            6 => {
                if budget.revert_remaining > 0 {
                    budget.revert_remaining -= 1;
                    // ~50/50 split between RevertString and RequireWithMsg so
                    // both code paths get exercised.
                    if u.arbitrary::<bool>()? {
                        Stmt::RevertString(arb_short_string(u)?)
                    } else {
                        Stmt::RequireWithMsg(
                            Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                            arb_short_string(u)?,
                        )
                    }
                } else {
                    // Budget exhausted — fall back to a plain require.
                    Stmt::Require(Expr::arbitrary_with_depth(u, depth + 1, using_for)?)
                }
            }
            7 => {
                if budget.trycatch_remaining > 0 {
                    budget.trycatch_remaining -= 1;
                    Stmt::TryCatch
                } else {
                    Stmt::StateAdd
                }
            }
            8 => {
                if budget.loop_remaining > 0 {
                    budget.loop_remaining -= 1;
                    let id = budget.fresh_id();
                    let count = u.int_in_range::<u8>(0..=4)?;
                    let n = u.int_in_range::<u8>(0..=2)?;
                    let mut body = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        body.push(Self::arbitrary_with_depth(u, depth + 1, using_for, budget)?);
                    }
                    Stmt::ForLoop { id, count, body }
                } else {
                    Stmt::StateAdd
                }
            }
            9 => {
                if budget.loop_remaining > 0 {
                    budget.loop_remaining -= 1;
                    let count = u.int_in_range::<u8>(0..=4)?;
                    let n = u.int_in_range::<u8>(0..=2)?;
                    let mut body = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        body.push(Self::arbitrary_with_depth(u, depth + 1, using_for, budget)?);
                    }
                    Stmt::WhileLoop { count, body }
                } else {
                    Stmt::StateAdd
                }
            }
            10 => {
                if budget.local_remaining > 0 {
                    budget.local_remaining -= 1;
                    let id = budget.fresh_id();
                    let expr = Expr::arbitrary_with_depth(u, depth + 1, using_for)?;
                    Stmt::LocalDecl { id, expr }
                } else {
                    Stmt::StateAdd
                }
            }
            11 => {
                if budget.collection_remaining > 0 {
                    budget.collection_remaining -= 1;
                    if u.arbitrary::<bool>()? {
                        Stmt::MapStore {
                            key: Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                            val: Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                        }
                    } else {
                        Stmt::MapLoad {
                            key: Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                        }
                    }
                } else {
                    Stmt::StateAdd
                }
            }
            12 => {
                if budget.collection_remaining > 0 {
                    budget.collection_remaining -= 1;
                    if u.arbitrary::<bool>()? {
                        Stmt::ArrPush {
                            expr: Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                        }
                    } else {
                        Stmt::ArrPopGuarded
                    }
                } else {
                    Stmt::StateAdd
                }
            }
            _ => {
                if budget.devpack_remaining > 0 {
                    budget.devpack_remaining -= 1;
                    Stmt::Devpack {
                        kind: u.int_in_range::<u8>(0..=8)?,
                        expr: Expr::arbitrary_with_depth(u, depth + 1, using_for)?,
                        id: budget.fresh_id(),
                        msg: arb_short_string(u)?,
                    }
                } else {
                    Stmt::StateAdd
                }
            }
        })
    }

    /// `has_return_type` controls whether a `return` is emitted with an
    /// expression or as a bare `return;` (avoids "wrong return arity").
    fn render(&self, out: &mut String, indent: usize, param_count: u8, has_return_type: bool) {
        for _ in 0..indent {
            out.push_str("    ");
        }
        match self {
            Stmt::Block(inner) => {
                out.push_str("{\n");
                for s in inner {
                    s.render(out, indent + 1, param_count, has_return_type);
                }
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::If(cond, body) => {
                out.push_str("if (");
                cond.render(out, param_count);
                out.push_str(") {\n");
                for s in body {
                    s.render(out, indent + 1, param_count, has_return_type);
                }
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::Return(e) => {
                if has_return_type {
                    out.push_str("return ");
                    e.render(out, param_count);
                    out.push_str(";\n");
                } else {
                    out.push_str("return;\n");
                }
            }
            Stmt::StateAdd => {
                out.push_str("state += 1;\n");
            }
            Stmt::Emit(e) => {
                out.push_str("emit E(");
                e.render(out, param_count);
                out.push_str(");\n");
            }
            Stmt::Require(e) => {
                out.push_str("require(");
                e.render(out, param_count);
                out.push_str(");\n");
            }
            Stmt::RevertString(s) => {
                let _ = write!(out, "revert(\"{}\");\n", s);
            }
            Stmt::RequireWithMsg(e, s) => {
                out.push_str("require(");
                e.render(out, param_count);
                let _ = write!(out, ", \"{}\");\n", s);
            }
            Stmt::TryCatch => {
                // Calls the fixed `helper()` external method declared on the
                // main contract. The bytes-catch arm is sufficient to reach
                // the dispatcher; deeper Panic/Error arms would require
                // matching call shapes that would inflate the grammar.
                out.push_str("try this.helper() returns (uint256 _v) {\n");
                for _ in 0..(indent + 1) {
                    out.push_str("    ");
                }
                out.push_str("state += _v;\n");
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("} catch (bytes memory) {\n");
                for _ in 0..(indent + 1) {
                    out.push_str("    ");
                }
                out.push_str("state = 0xfe;\n");
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::ForLoop { id, count, body } => {
                let _ = write!(
                    out,
                    "for (uint256 i_{0} = 0; i_{0} < {1}; i_{0}++) {{\n",
                    id, count
                );
                for s in body {
                    s.render(out, indent + 1, param_count, has_return_type);
                }
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::WhileLoop { count, body } => {
                let _ = write!(out, "while (state < {}) {{\n", count);
                for _ in 0..(indent + 1) {
                    out.push_str("    ");
                }
                // Monotonic increment guarantees termination regardless of body.
                out.push_str("state += 1;\n");
                for s in body {
                    s.render(out, indent + 1, param_count, has_return_type);
                }
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::LocalDecl { id, expr } => {
                let _ = write!(out, "uint256 v_{} = uint256(", id);
                expr.render(out, param_count);
                let _ = write!(out, ") & 0xff;\n");
                for _ in 0..indent {
                    out.push_str("    ");
                }
                let _ = write!(out, "state += v_{};\n", id);
            }
            Stmt::MapStore { key, val } => {
                out.push_str("sm[uint256(");
                key.render(out, param_count);
                out.push_str(") & 0xff] = uint256(");
                val.render(out, param_count);
                out.push_str(") & 0xff;\n");
            }
            Stmt::MapLoad { key } => {
                out.push_str("state += sm[uint256(");
                key.render(out, param_count);
                out.push_str(") & 0xff];\n");
            }
            Stmt::ArrPush { expr } => {
                out.push_str("sa.push(uint256(");
                expr.render(out, param_count);
                out.push_str(") & 0xff);\n");
            }
            Stmt::ArrPopGuarded => {
                out.push_str("if (sa.length > 0) {\n");
                for _ in 0..(indent + 1) {
                    out.push_str("    ");
                }
                out.push_str("state += sa[sa.length - 1];\n");
                for _ in 0..(indent + 1) {
                    out.push_str("    ");
                }
                out.push_str("sa.pop();\n");
                for _ in 0..indent {
                    out.push_str("    ");
                }
                out.push_str("}\n");
            }
            Stmt::Devpack { kind, expr, id, msg } => match kind % 9 {
                0 => out.push_str("state += Runtime.getTime();\n"),
                1 => out.push_str("state += Runtime.gasLeft();\n"),
                2 => out.push_str("state += Runtime.getInvocationCounter();\n"),
                3 => out.push_str("state += Runtime.getRandom();\n"),
                4 => out.push_str("state += uint256(Runtime.getNetwork());\n"),
                5 => {
                    let _ = write!(out, "Runtime.log(\"{}\");\n", msg);
                }
                6 => out.push_str("if (Runtime.checkWitness(msg.sender)) { state += 1; }\n"),
                7 => {
                    out.push_str("Storage.put(abi.encodePacked(uint256(");
                    expr.render(out, param_count);
                    out.push_str(") & 0xff), abi.encodePacked(state));\n");
                }
                _ => {
                    let _ = write!(out, "bytes memory g_{} = Storage.get(abi.encodePacked(uint256(", id);
                    expr.render(out, param_count);
                    let _ = write!(out, ") & 0xff));\n");
                    for _ in 0..indent {
                        out.push_str("    ");
                    }
                    let _ = write!(out, "state += g_{}.length;\n", id);
                }
            },
        }
    }
}

#[derive(Debug)]
struct Function {
    params: Vec<SolType>,
    returns: Vec<SolType>,
    body: Vec<Stmt>,
}

impl Function {
    fn arbitrary_with(u: &mut Unstructured<'_>, using_for: bool) -> Result<Self> {
        let nparams = u.int_in_range::<u8>(0..=3)?;
        let mut params = Vec::with_capacity(nparams as usize);
        for _ in 0..nparams {
            params.push(SolType::arbitrary(u)?);
        }
        let nrets = u.int_in_range::<u8>(0..=2)?;
        let mut returns = Vec::with_capacity(nrets as usize);
        for _ in 0..nrets {
            returns.push(SolType::arbitrary(u)?);
        }
        let nstmts = u.int_in_range::<u8>(1..=8)?;
        let mut body = Vec::with_capacity(nstmts as usize);
        // Per-function budgets cap explosion of the new statement kinds.
        let mut budget = StmtBudget {
            revert_remaining: 2,
            trycatch_remaining: 1,
            loop_remaining: 2,
            local_remaining: 3,
            collection_remaining: 3,
            devpack_remaining: 3,
            next_id: 0,
        };
        for _ in 0..nstmts {
            body.push(Stmt::arbitrary_with_depth(u, 0, using_for, &mut budget)?);
        }
        Ok(Function { params, returns, body })
    }
}

/// Modifier flags that decorate a rendered function.
#[derive(Debug, Clone, Copy, Default)]
struct FnFlags {
    virtual_: bool,
    override_: bool,
    abstract_decl: bool, // abstract function: no body, must be virtual, contract is abstract
}

impl Function {
    fn render(&self, out: &mut String, idx: usize, flags: FnFlags) {
        let _ = write!(out, "    function f_{}(", idx);
        for (i, t) in self.params.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{} p_{}", t.name(), i);
        }
        out.push_str(") public");
        if flags.virtual_ {
            out.push_str(" virtual");
        }
        if flags.override_ {
            out.push_str(" override");
        }
        if !self.returns.is_empty() {
            out.push_str(" returns (");
            for (i, t) in self.returns.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(t.name());
            }
            out.push(')');
        }
        if flags.abstract_decl {
            // Body-less declaration; closing semicolon. Caller guarantees
            // virtual_ is set and the contract is abstract.
            out.push_str(";\n");
            return;
        }
        out.push_str(" {\n");
        let param_count = self.params.len() as u8;
        let has_ret = !self.returns.is_empty();
        for s in &self.body {
            s.render(out, 2, param_count, has_ret);
        }
        // If the function declares returns, append a defaulted return so the
        // function always type-checks past arity even when the random body
        // never returns. The deeper checks (type compatibility, etc.) still
        // get exercised on the user-emitted statements.
        if has_ret {
            out.push_str("        return (");
            for (i, t) in self.returns.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&t.literal(0));
            }
            out.push_str(");\n");
        }
        out.push_str("    }\n");
    }
}

/// One contract (possibly a base, mid, or main). The renderer is
/// parameterized over its name, optional `is`-clause, abstract flag, and
/// `using_for` flag.
#[derive(Debug)]
struct ContractBody {
    /// Whether this contract is `abstract`. If true, may include
    /// signature-only function declarations.
    is_abstract: bool,
    /// Functions implemented (with bodies). The first one may be marked
    /// virtual to let derived contracts override it.
    functions: Vec<Function>,
    /// If `is_abstract`, append this many signature-only declarations.
    /// Capped at 1 to bound size.
    abstract_decl: bool,
}

impl ContractBody {
    fn arbitrary_with(u: &mut Unstructured<'_>, using_for: bool) -> Result<Self> {
        let nfns = u.int_in_range::<u8>(1..=MAX_FUNCS_PER_CONTRACT)?;
        let mut functions = Vec::with_capacity(nfns as usize);
        for _ in 0..nfns {
            functions.push(Function::arbitrary_with(u, using_for)?);
        }
        // ~50% chance abstract; abstract_decl only emitted in that case.
        let is_abstract = u.arbitrary::<bool>()?;
        let abstract_decl = is_abstract && u.arbitrary::<bool>()?;
        Ok(ContractBody { is_abstract, functions, abstract_decl })
    }
}

/// A non-main "base" contract. We always make its first function `virtual`
/// with a fixed signature `function fbase_<id>() public virtual returns
/// (uint256)` so the main contract can override it cleanly.
#[derive(Debug)]
struct BaseContract {
    body: ContractBody,
    /// Optional parent index (for two-level inheritance, `Mid is Base`).
    parent: Option<usize>,
}

impl BaseContract {
    fn arbitrary_with(u: &mut Unstructured<'_>, using_for: bool) -> Result<Self> {
        let body = ContractBody::arbitrary_with(u, using_for)?;
        Ok(BaseContract { body, parent: None })
    }
}

/// Top-level source: optional library, 0..=2 base contracts, optional mid
/// contract that extends a base, then a main contract that may inherit
/// from any subset of the bases (size-capped) and may override the
/// well-known virtual hook.
#[derive(Debug)]
struct Source {
    use_library: bool,
    using_for: bool,
    bases: Vec<BaseContract>,
    /// If true, the main contract emits an `override` of `hook()` from the
    /// first base it inherits from. Otherwise no override.
    override_hook: bool,
    /// If true, the main contract is itself abstract.
    main_abstract: bool,
    main: ContractBody,
}

impl<'a> Arbitrary<'a> for Source {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let use_library = u.arbitrary::<bool>()?;
        let using_for = use_library && u.arbitrary::<bool>()?;
        let nbases = u.int_in_range::<u8>(0..=MAX_BASES)?;
        let mut bases = Vec::with_capacity(nbases as usize);
        for i in 0..nbases {
            let mut b = BaseContract::arbitrary_with(u, using_for)?;
            // With small probability, link this base to the previous one to
            // form a 2-deep chain (`Mid is Base`). Hard-cap so the total
            // chain depth never exceeds 2.
            if i > 0 && u.arbitrary::<bool>()? {
                b.parent = Some((i - 1) as usize);
            }
            bases.push(b);
        }
        let override_hook = !bases.is_empty() && u.arbitrary::<bool>()?;
        let main_abstract = u.arbitrary::<bool>()?;
        let main = ContractBody::arbitrary_with(u, using_for)?;
        Ok(Source {
            use_library,
            using_for,
            bases,
            override_hook,
            main_abstract,
            main,
        })
    }
}

impl Source {
    fn render(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("// SPDX-License-Identifier: MIT\n");
        out.push_str("pragma solidity ^0.8.20;\n");

        // Optional library (1-2 trivial functions). Always rendered when
        // requested so any `using L for uint256` reference resolves.
        if self.use_library {
            out.push_str("library L {\n");
            out.push_str("    function add1(uint256 x) internal pure returns (uint256) { return x + 1; }\n");
            out.push_str("    function add2(uint256 x) internal pure returns (uint256) { return x + 2; }\n");
            out.push_str("}\n");
        }

        // Base contracts. Each gets a synthetic `hook()` virtual so the
        // main contract can override cleanly when it chooses to.
        for (i, b) in self.bases.iter().enumerate() {
            if b.body.is_abstract {
                out.push_str("abstract ");
            }
            let _ = write!(out, "contract Base_{}", i);
            if let Some(p) = b.parent {
                let _ = write!(out, " is Base_{}", p);
            }
            out.push_str(" {\n");
            // Synthetic state + event so renderer of bodies can reference
            // them identically to the main contract. `sm`/`sa` back the
            // mapping / dynamic-array statement productions.
            out.push_str("    uint256 state;\n");
            out.push_str("    event E(uint256);\n");
            out.push_str("    mapping(uint256 => uint256) sm;\n");
            out.push_str("    uint256[] sa;\n");
            // Synthetic virtual hook with a fixed signature; main may
            // override it. If `parent` exists, mark hook as `override`
            // virtual (mid overrides base hook but stays virtual).
            if b.parent.is_some() {
                out.push_str("    function hook() public virtual override returns (uint256) { return uint256(0); }\n");
            } else {
                out.push_str("    function hook() public virtual returns (uint256) { return uint256(0); }\n");
            }
            // Synthetic helper for `try this.helper() ... catch ...` to
            // resolve. Marked virtual so the main contract can co-declare
            // it without a name clash. Mid overrides parent's helper.
            if b.parent.is_some() {
                out.push_str("    function helper() external pure virtual override returns (uint256) { return uint256(7); }\n");
            } else {
                out.push_str("    function helper() external pure virtual returns (uint256) { return uint256(7); }\n");
            }
            // Random body functions.
            for (idx, f) in b.body.functions.iter().enumerate() {
                f.render(&mut out, idx, FnFlags::default());
            }
            // Optional abstract declaration (signature only). Only valid
            // because contract is abstract.
            if b.body.is_abstract && b.body.abstract_decl {
                out.push_str("    function fa() external virtual returns (uint256);\n");
            }
            out.push_str("}\n");
        }

        // Main contract.
        if self.main_abstract {
            out.push_str("abstract ");
        }
        out.push_str("contract C");
        // Inherit from all generated bases (already capped at MAX_BASES,
        // and chain depth is capped at 2). Solidity needs the linearization
        // order: base classes first.
        if !self.bases.is_empty() {
            out.push_str(" is ");
            for i in 0..self.bases.len() {
                if i > 0 {
                    out.push_str(", ");
                }
                let _ = write!(out, "Base_{}", i);
            }
        }
        out.push_str(" {\n");
        // Note: when `bases` is non-empty these re-declare base state and may
        // produce a shadowing *compile error* — which is fine, the fuzzer only
        // treats a panic as a finding, and the shadow path is itself coverage.
        // The frequent no-base case compiles cleanly and drives the new
        // loop / mapping / array codegen end-to-end.
        out.push_str("    uint256 state;\n");
        out.push_str("    event E(uint256);\n");
        out.push_str("    mapping(uint256 => uint256) sm;\n");
        out.push_str("    uint256[] sa;\n");
        if self.using_for {
            out.push_str("    using L for uint256;\n");
        }
        // Synthetic helper for `try this.helper() ... catch ...` to resolve.
        // Must be declared `override(Base_0, Base_1, ...)` to satisfy
        // multi-inheritance when bases are present.
        if self.bases.is_empty() {
            out.push_str("    function helper() external pure returns (uint256) { return uint256(7); }\n");
        } else {
            out.push_str("    function helper() external pure override");
            if self.bases.len() > 1 {
                out.push_str("(");
                for i in 0..self.bases.len() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    let _ = write!(out, "Base_{}", i);
                }
                out.push_str(")");
            }
            out.push_str(" returns (uint256) { return uint256(7); }\n");
        }
        // Override the `hook()` virtual (matching signature) when the
        // grammar requested it. The override list must include every base
        // that declares `hook()`, which is all of them when we inherit
        // from >= 1 base.
        if self.override_hook && !self.bases.is_empty() {
            out.push_str("    function hook() public override");
            if self.bases.len() > 1 {
                out.push_str("(");
                for i in 0..self.bases.len() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    let _ = write!(out, "Base_{}", i);
                }
                out.push_str(")");
            }
            out.push_str(" returns (uint256) { return uint256(1); }\n");
        }
        // Random body functions.
        for (idx, f) in self.main.functions.iter().enumerate() {
            f.render(&mut out, idx, FnFlags::default());
        }
        // Optional abstract declaration (signature only). Only valid
        // because contract is abstract.
        if self.main_abstract && self.main.abstract_decl {
            out.push_str("    function fa() external virtual returns (uint256);\n");
        }
        out.push_str("}\n");
        out
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let source = match Source::arbitrary(&mut u) {
        Ok(s) => s,
        Err(_) => return,
    };
    // Pick optimizer level from the remaining input so libfuzzer can mutate
    // it independently — one compile per iteration keeps exec/s high while
    // still exercising opt ∈ {0..=3} across the corpus.
    let opt: u8 = u.int_in_range(0..=3).unwrap_or(0);
    let src = source.render();

    // Wrap in catch_unwind: the only signal we care about is a real
    // panic / abort. Any Result::Err is expected for ill-typed but
    // parser-clean programs.
    let _ = std::panic::catch_unwind(move || {
        let _ = neo_devpack_solidity::cli::compile_contracts(&src, false, opt);
    });
});
