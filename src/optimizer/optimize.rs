use super::*;

/// Drop instructions that follow a terminator (Return / Jump / Abort / Throw /
/// EndTry) up to the next `Label`. Task #88 — also drop labels that are no
/// longer reachable (no incoming Jump/JumpIf, preceded only by a terminator)
/// so a now-statically-dead arm (e.g. `if (true) return 1; return s;` at L2+)
/// can have its tail instructions — including `LoadState`/storage reads —
/// elided instead of falling through a dead fall-through at the label.
fn prune_after_terminator(
    block: &mut ir::BasicBlock,
    live_labels: &std::collections::HashSet<usize>,
) {
    let mut trimmed = Vec::with_capacity(block.instructions.len());
    let mut terminated = false;

    for instr in block.instructions.drain(..) {
        if terminated {
            if let ir::Instruction::Label(id) = instr {
                if live_labels.contains(&id) {
                    trimmed.push(ir::Instruction::Label(id));
                    terminated = false;
                }
                // Unreferenced label after a terminator → drop it; keep terminated=true
                // so the tail of the dead arm (LoadState, return, ...) continues to be elided.
            }
            continue;
        }

        match instr {
            ir::Instruction::Return
            | ir::Instruction::ReturnVoid
            | ir::Instruction::ReturnDefault(_)
            | ir::Instruction::Jump { .. }
            | ir::Instruction::Abort
            | ir::Instruction::AbortMsg
            | ir::Instruction::Throw
            | ir::Instruction::EndTry { .. } => {
                trimmed.push(instr);
                terminated = true;
            }
            other => trimmed.push(other),
        }
    }

    block.instructions = trimmed;
}

/// Collect every label id referenced by a `Jump`, `JumpIf`, `Try`, or `EndTry`
/// across every block of the function.
fn collect_live_labels(function: &ir::Function) -> std::collections::HashSet<usize> {
    let mut live = std::collections::HashSet::new();
    for block in &function.basic_blocks {
        for instr in &block.instructions {
            match instr {
                ir::Instruction::Jump { target } | ir::Instruction::JumpIf { target } => {
                    live.insert(*target);
                }
                ir::Instruction::Try { catch_target } => {
                    live.insert(*catch_target);
                }
                ir::Instruction::EndTry { target } => {
                    live.insert(*target);
                }
                _ => {}
            }
        }
    }
    live
}

/// IR optimization hook. Currently performs simple control-flow cleanup to drop
/// instructions that appear after a terminal return in a basic block.
pub(crate) fn optimize_ir(mut module: ir::Module, optimizer_level: u8) -> ir::Module {
    if optimizer_level == 0 {
        return module;
    }

    let enable_neovm_specific = optimizer_level >= 3;
    let enable_constant_folding = optimizer_level >= 2;

    for function in &mut module.functions {
        let mut label_remap = std::collections::HashMap::new();

        // First-pass: prune unreachable tails against the original live-label set.
        let live_labels = collect_live_labels(function);
        for block in &mut function.basic_blocks {
            prune_after_terminator(block, &live_labels);
        }

        if enable_constant_folding {
            // Task #88 — fold `PushLit(const); JumpIf(t)` into Jump or no-op so
            // a now-dead arm's tail (incl. LoadState) can be elided. Re-run
            // the label-reachability pruner with a fresh live-label set to
            // observe the newly-unreferenced target.
            for block in &mut function.basic_blocks {
                fold_constant_binary_ops(block);
            }
            let live_labels = collect_live_labels(function);
            for block in &mut function.basic_blocks {
                prune_after_terminator(block, &live_labels);
            }
        }

        if enable_neovm_specific {
            for block in &mut function.basic_blocks {
                dedupe_labels(block, &mut label_remap);
                remove_trivial_jumps(block);

                // NeoVM-specific optimizations at O3
                neovm_peephole_optimize(block);
                neovm_simplify_identity_ops(block);
                neovm_bool_optimize(block);

                // Run peephole again to catch newly exposed patterns
                neovm_peephole_optimize(block);
            }
        }

        if enable_neovm_specific && !label_remap.is_empty() {
            retarget_jumps(&mut function.basic_blocks, &label_remap);
        }
    }

    module
}
