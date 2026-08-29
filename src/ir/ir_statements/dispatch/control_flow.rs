use super::*;

pub(crate) fn lower_block_statement(
    statements: &[Statement],
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    ctx.enter_scope();
    let mut returned = false;
    for stmt in statements {
        if lower_statement(stmt, ctx, instructions) {
            returned = true;
            break;
        }
    }
    ctx.exit_scope();
    returned
}

pub(crate) fn lower_if_statement(
    condition: &Expression,
    then_stmt: &Statement,
    else_stmt: Option<&Statement>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let else_label = ctx.next_label();
    let end_label = if else_stmt.is_some() {
        ctx.next_label()
    } else {
        else_label
    };

    lower_expression(condition, ctx, instructions);
    instructions.push(Instruction::JumpIf { target: else_label });

    let then_returns = lower_statement(then_stmt, ctx, instructions);

    if let Some(else_stmt) = else_stmt {
        instructions.push(Instruction::Jump { target: end_label });
        instructions.push(Instruction::Label(else_label));
        let else_returns = lower_statement(else_stmt, ctx, instructions);
        instructions.push(Instruction::Label(end_label));
        then_returns && else_returns
    } else {
        instructions.push(Instruction::Label(else_label));
        false
    }
}

pub(crate) fn lower_while_statement(
    condition: &Expression,
    body: &Statement,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let start_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(start_label));
    lower_expression(condition, ctx, instructions);
    instructions.push(Instruction::JumpIf { target: end_label });
    ctx.push_loop(start_label, end_label);
    lower_statement(body, ctx, instructions);
    ctx.pop_loop();
    instructions.push(Instruction::Jump {
        target: start_label,
    });
    instructions.push(Instruction::Label(end_label));
    false
}

pub(crate) fn lower_do_while_statement(
    body: &Statement,
    condition: &Expression,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    let start_label = ctx.next_label();
    let condition_label = ctx.next_label();
    let end_label = ctx.next_label();

    // Task #114 — when the Solidity expander wraps a modifier-epilogue
    // function body in `do { body } while(false)`, push the `end_label` onto
    // the `modifier_break_stack` so `lower_return_statement` can jump past
    // any intervening user loops and land after the synthetic wrap (i.e.
    // just before the modifier epilogue statements run).
    //
    // The wrap is identified by the synthetic `Loc::Implicit` the expander
    // stamps on the condition (`modifiers/expand.rs`). Matching any
    // `while(false)` here would also capture a USER-written `do { … }
    // while(false)`, hijacking its `return` redirect so the value lands in the
    // wrong slot (it returned the value of a *later* `return` instead). The
    // parser only ever assigns real `Loc::File` spans to user source, so the
    // implicit marker distinguishes the two.
    let is_modifier_wrap = ctx.in_modifier_epilogue_scope()
        && matches!(
            condition,
            Expression::BoolLiteral(solang_parser::pt::Loc::Implicit, false)
        );

    instructions.push(Instruction::Label(start_label));
    ctx.push_loop(condition_label, end_label);
    if is_modifier_wrap {
        ctx.push_modifier_break_label(end_label);
    }
    lower_statement(body, ctx, instructions);
    if is_modifier_wrap {
        ctx.pop_modifier_break_label();
    }
    ctx.pop_loop();
    instructions.push(Instruction::Label(condition_label));
    lower_expression(condition, ctx, instructions);
    instructions.push(Instruction::JumpIf { target: end_label });
    instructions.push(Instruction::Jump {
        target: start_label,
    });
    instructions.push(Instruction::Label(end_label));
    false
}

pub(crate) fn lower_for_statement(
    init: Option<&Statement>,
    condition: Option<&Expression>,
    post: Option<&Expression>,
    body: Option<&Statement>,
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    ctx.enter_scope();
    if let Some(init_stmt) = init {
        lower_statement(init_stmt, ctx, instructions);
    }

    let condition_label = ctx.next_label();
    let post_label = ctx.next_label();
    let end_label = ctx.next_label();

    instructions.push(Instruction::Label(condition_label));

    if let Some(cond_expr) = condition {
        lower_expression(cond_expr, ctx, instructions);
        instructions.push(Instruction::JumpIf { target: end_label });
    }

    if let Some(body_stmt) = body {
        ctx.push_loop(post_label, end_label);
        lower_statement(body_stmt, ctx, instructions);
        ctx.pop_loop();
    }

    instructions.push(Instruction::Label(post_label));
    if let Some(post_expr) = post {
        if lower_expression(post_expr, ctx, instructions) {
            instructions.push(Instruction::Drop(ValueType::Any));
        }
    }

    instructions.push(Instruction::Jump {
        target: condition_label,
    });
    instructions.push(Instruction::Label(end_label));
    ctx.exit_scope();
    false
}

pub(crate) fn lower_break_statement(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(label) = ctx.break_target() {
        instructions.push(Instruction::Jump { target: label });
    }
    false
}

pub(crate) fn lower_continue_statement(
    ctx: &mut LoweringContext,
    instructions: &mut Vec<Instruction>,
) -> bool {
    if let Some(label) = ctx.continue_target() {
        instructions.push(Instruction::Jump { target: label });
    }
    false
}
