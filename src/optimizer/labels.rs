use super::*;

pub(crate) fn remove_trivial_jumps(block: &mut ir::BasicBlock) {
    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut iter = block.instructions.iter();
    while let Some(instr) = iter.next() {
        if let ir::Instruction::Jump { target } = instr {
            if let Some(ir::Instruction::Label(id)) = iter.clone().next() {
                if id == target {
                    // Skip the jump; fallthrough reaches the label
                    continue;
                }
            }
        }
        optimized.push(instr.clone());
    }
    block.instructions = optimized;
}

pub(crate) fn dedupe_labels(
    block: &mut ir::BasicBlock,
    remap: &mut std::collections::HashMap<usize, usize>,
) {
    let mut optimized = Vec::with_capacity(block.instructions.len());
    let mut last_label: Option<usize> = None;

    for instr in block.instructions.drain(..) {
        match instr {
            ir::Instruction::Label(id) => {
                if let Some(prev) = last_label {
                    remap.insert(id, prev);
                    continue;
                } else {
                    last_label = Some(id);
                    optimized.push(ir::Instruction::Label(id));
                }
            }
            other => {
                last_label = None;
                optimized.push(other);
            }
        }
    }

    block.instructions = optimized;
}

pub(crate) fn retarget_jumps(
    blocks: &mut [ir::BasicBlock],
    remap: &std::collections::HashMap<usize, usize>,
) {
    for block in blocks {
        for instr in &mut block.instructions {
            match instr {
                ir::Instruction::Jump { target } | ir::Instruction::JumpIf { target } => {
                    if let Some(canonical) = remap.get(target) {
                        *target = *canonical;
                    }
                }
                ir::Instruction::Try { catch_target } => {
                    if let Some(canonical) = remap.get(catch_target) {
                        *catch_target = *canonical;
                    }
                }
                ir::Instruction::EndTry { target } => {
                    if let Some(canonical) = remap.get(target) {
                        *target = *canonical;
                    }
                }
                _ => {}
            }
        }
    }
}
