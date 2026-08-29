use super::*;

pub(crate) fn analyze_contract_calls(
    function: &ir::Function,
    ir_module: &ir::Module,
) -> Vec<ContractCallRequirement> {
    const MAX_STATE_VARIANTS: usize = 16;

    let mut has_contract_calls = false;
    for block in &function.basic_blocks {
        for instr in &block.instructions {
            if let ir::Instruction::CallBuiltin { builtin, .. } = instr {
                let is_contract_call = match builtin {
                    ir::BuiltinCall::ContractCall | ir::BuiltinCall::ContractCallWithFlags => true,
                    ir::BuiltinCall::Syscall(name) if name == "System.Contract.Call" => true,
                    _ => false,
                };
                if is_contract_call {
                    has_contract_calls = true;
                    break;
                }
            }
        }
    }

    if !has_contract_calls {
        return Vec::new();
    }

    fn possible_contracts(value: &AbstractValue) -> Option<Vec<Option<String>>> {
        let atoms = value.atoms()?;
        let mut contracts = Vec::new();
        for atom in atoms {
            match atom {
                AbstractAtom::Literal(lit) => contracts.push(descriptor_from_literal(lit)),
                AbstractAtom::ExecutingScriptHash => contracts.push(None),
            }
        }
        Some(contracts)
    }

    fn possible_methods(value: &AbstractValue) -> Option<Vec<Option<String>>> {
        let atoms = value.atoms()?;
        let mut methods = Vec::new();
        for atom in atoms {
            match atom {
                AbstractAtom::Literal(lit) => methods.push(method_name_from_literal(lit)),
                AbstractAtom::ExecutingScriptHash => methods.push(None),
            }
        }
        Some(methods)
    }

    let instructions: Vec<&ir::Instruction> = function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    if instructions.is_empty() {
        return Vec::new();
    }

    let mut label_positions = std::collections::HashMap::new();
    for (index, instr) in instructions.iter().enumerate() {
        if let ir::Instruction::Label(label) = instr {
            label_positions.insert(*label, index);
        }
    }

    fn enqueue_state(
        states: &mut [Vec<AbstractState>],
        worklist: &mut std::collections::VecDeque<(usize, AbstractState)>,
        index: usize,
        state: AbstractState,
    ) {
        if states[index].contains(&state) {
            return;
        }

        if states[index].len() < MAX_STATE_VARIANTS {
            states[index].push(state.clone());
            worklist.push_back((index, state));
            return;
        }

        let mut merged = states[index][0].clone();
        for existing in states[index].iter().skip(1) {
            merged.merge_from(existing);
        }
        merged.merge_from(&state);

        if states[index].len() == 1 && states[index][0] == merged {
            return;
        }

        states[index].clear();
        states[index].push(merged.clone());
        worklist.push_back((index, merged));
    }

    let mut requirements = Vec::new();
    let mut states: Vec<Vec<AbstractState>> = vec![Vec::new(); instructions.len()];
    let initial = AbstractState::new(function.local_count);
    states[0].push(initial.clone());
    let mut worklist = std::collections::VecDeque::from([(0usize, initial)]);

    while let Some((index, state_in)) = worklist.pop_front() {
        let instr = instructions[index];

        if let ir::Instruction::CallBuiltin { builtin, arg_count } = instr {
            let (contract_from_end, method_from_end, expected_args) = match builtin {
                ir::BuiltinCall::ContractCall => (3usize, 2usize, 3usize),
                ir::BuiltinCall::ContractCallWithFlags => (4usize, 3usize, 4usize),
                ir::BuiltinCall::Syscall(name) if name == "System.Contract.Call" => {
                    (1usize, 2usize, 4usize)
                }
                _ => (0usize, 0usize, 0usize),
            };

            if expected_args > 0
                && *arg_count == expected_args
                && state_in.stack.len() >= expected_args
            {
                let stack_len = state_in.stack.len();
                let contract_value = &state_in.stack[stack_len - contract_from_end];
                let method_value = &state_in.stack[stack_len - method_from_end];

                let contracts = possible_contracts(contract_value);
                let methods = possible_methods(method_value);

                match (contracts, methods) {
                    (Some(contract_options), Some(method_options)) => {
                        let mut emitted = false;
                        for contract in contract_options {
                            for method in &method_options {
                                if contract.is_none() {
                                    continue;
                                }
                                requirements.push(ContractCallRequirement {
                                    contract: contract.clone(),
                                    method: method.clone(),
                                });
                                emitted = true;
                            }
                        }
                        if !emitted {
                            // self-call only
                        }
                    }
                    (Some(contract_options), None) => {
                        for contract in contract_options {
                            if contract.is_none() {
                                continue;
                            }
                            requirements.push(ContractCallRequirement {
                                contract,
                                method: None,
                            });
                        }
                    }
                    (None, Some(method_options)) => {
                        for method in method_options {
                            requirements.push(ContractCallRequirement {
                                contract: None,
                                method,
                            });
                        }
                    }
                    (None, None) => requirements.push(ContractCallRequirement {
                        contract: None,
                        method: None,
                    }),
                }
            } else if expected_args > 0 {
                requirements.push(ContractCallRequirement {
                    contract: None,
                    method: None,
                });
            }
        }

        let mut state_out = state_in;
        if apply_instruction(&mut state_out, instr, ir_module).is_err() {
            state_out.stack.clear();
            for slot in &mut state_out.locals {
                *slot = AbstractValue::Unknown;
            }
        }

        let mut successors = Vec::new();
        match instr {
            ir::Instruction::Jump { target } => {
                if let Some(position) = label_positions.get(target) {
                    successors.push(*position);
                }
            }
            ir::Instruction::JumpIf { target } => {
                if let Some(position) = label_positions.get(target) {
                    successors.push(*position);
                }
                if index + 1 < instructions.len() {
                    successors.push(index + 1);
                }
            }
            ir::Instruction::Try { catch_target } => {
                if index + 1 < instructions.len() {
                    successors.push(index + 1);
                }
                // Exception edge into the catch handler. The eval stack at
                // catch entry is NOT the stack at TRY time: NeoVM unwinds the
                // eval stack and pushes only the exception value. Seed the
                // catch state with the locals as of TRY entry (locals persist
                // across the unwind) and a single Unknown stack slot for the
                // exception object, so contract calls inside catch bodies are
                // analysed instead of being silently skipped (which dropped
                // their manifest permissions and FAULTed on-chain).
                if let Some(position) = label_positions.get(catch_target) {
                    let mut catch_state = state_out.clone();
                    catch_state.stack.clear();
                    catch_state.stack.push(AbstractValue::Unknown);
                    enqueue_state(&mut states, &mut worklist, *position, catch_state);
                }
            }
            ir::Instruction::EndTry { target } => {
                if let Some(position) = label_positions.get(target) {
                    successors.push(*position);
                }
            }
            ir::Instruction::Return
            | ir::Instruction::ReturnVoid
            | ir::Instruction::ReturnDefault(_)
            | ir::Instruction::Abort
            | ir::Instruction::AbortMsg
            | ir::Instruction::Throw => {}
            _ => {
                if index + 1 < instructions.len() {
                    successors.push(index + 1);
                }
            }
        }

        for successor in successors {
            enqueue_state(&mut states, &mut worklist, successor, state_out.clone());
        }
    }

    // Defense in depth: if any contract-call site was never reached by the
    // dataflow walk (e.g. control flow this analysis does not model yet),
    // degrade to an explicit wildcard requirement rather than silently
    // omitting the permission — under-permission FAULTs on-chain, while
    // over-permission is merely broad.
    for (index, instr) in instructions.iter().enumerate() {
        if !states[index].is_empty() {
            continue;
        }
        if let ir::Instruction::CallBuiltin { builtin, .. } = instr {
            let is_contract_call = match builtin {
                ir::BuiltinCall::ContractCall | ir::BuiltinCall::ContractCallWithFlags => true,
                ir::BuiltinCall::Syscall(name) if name == "System.Contract.Call" => true,
                _ => false,
            };
            if is_contract_call {
                requirements.push(ContractCallRequirement {
                    contract: None,
                    method: None,
                });
            }
        }
    }

    requirements
}
