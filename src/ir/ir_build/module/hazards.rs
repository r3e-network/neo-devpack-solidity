use super::*;

#[derive(Default, Clone, Copy)]
pub(crate) struct Hazards {
    pub(crate) writes_state: bool,
    pub(crate) notifies: bool,
    pub(crate) unsafe_contract_call: bool,
    pub(crate) reads_state: bool,
    pub(crate) reads_environment: bool,
    pub(crate) contract_calls: bool,
}

impl Hazards {
    pub(crate) fn safe_violation(self) -> bool {
        self.writes_state || self.notifies || self.unsafe_contract_call
    }

    pub(crate) fn pure_violation(self) -> bool {
        self.safe_violation() || self.reads_state
    }

    pub(crate) fn merge_in(&mut self, other: Hazards) {
        self.writes_state |= other.writes_state;
        self.notifies |= other.notifies;
        self.unsafe_contract_call |= other.unsafe_contract_call;
        self.reads_state |= other.reads_state;
        self.reads_environment |= other.reads_environment;
        self.contract_calls |= other.contract_calls;
    }
}

pub(crate) fn direct_hazards(function: &Function) -> Hazards {
    let instrs: Vec<&Instruction> = function
        .basic_blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
        .collect();

    let mut hazards = Hazards::default();

    for (index, instr) in instrs.iter().enumerate() {
        match instr {
            Instruction::LoadState(_)
            | Instruction::LoadMappingElement { .. }
            | Instruction::LoadStructField { .. }
            | Instruction::LoadStructArrayElement { .. }
            | Instruction::LoadStructFieldMappingElement { .. }
            | Instruction::LoadStorageDynamic => {
                hazards.reads_state = true;
            }
            Instruction::StoreState(_)
            | Instruction::StoreMappingElement { .. }
            | Instruction::StoreArrayDeepCopy { .. }
            | Instruction::StoreStructField { .. }
            | Instruction::StoreStructArrayElement { .. }
            | Instruction::StoreStructFieldMappingElement { .. } => {
                hazards.writes_state = true;
            }
            Instruction::LoadRuntimeValue(_) => {
                hazards.reads_environment = true;
            }
            Instruction::EmitEvent { .. } | Instruction::EmitEventByName { .. } => {
                hazards.notifies = true;
            }
            Instruction::CallBuiltin { builtin, arg_count } => match builtin {
                BuiltinCall::StorageGet | BuiltinCall::StorageFind => {
                    hazards.reads_state = true;
                }
                BuiltinCall::StoragePut | BuiltinCall::StorageDelete => {
                    hazards.writes_state = true;
                }
                BuiltinCall::RuntimeNotify | BuiltinCall::NotifySerialized => {
                    hazards.notifies = true;
                }
                BuiltinCall::RuntimeCheckWitness => {
                    hazards.reads_environment = true;
                }
                BuiltinCall::DeployContract => {
                    hazards.writes_state = true;
                    hazards.contract_calls = true;
                }
                BuiltinCall::GetContract
                | BuiltinCall::GetContractScript
                | BuiltinCall::GetNeoAccountState => {
                    hazards.contract_calls = true;
                }
                BuiltinCall::NativeCall { contract, method } => {
                    if native_call_is_mutating(*contract, method) {
                        hazards.writes_state = true;
                    }
                    if !matches!(contract, NativeContract::CryptoLib | NativeContract::StdLib) {
                        hazards.contract_calls = true;
                    }
                }
                BuiltinCall::Syscall(name) => {
                    // Disallow syscalls that obviously mutate state or send notifications.
                    if matches!(
                        name.as_str(),
                        "System.Storage.Put" | "System.Storage.Delete"
                    ) {
                        hazards.writes_state = true;
                    }
                    if matches!(
                        name.as_str(),
                        "System.Runtime.Notify" | "System.Runtime.Log"
                    ) {
                        hazards.notifies = true;
                    }

                    // Storage syscalls read contract storage (including read-only contexts).
                    if matches!(
                        name.as_str(),
                        "System.Storage.Get"
                            | "System.Storage.Find"
                            | "System.Storage.GetContext"
                            | "System.Storage.GetReadOnlyContext"
                    ) {
                        hazards.reads_state = true;
                    }

                    // Runtime/environment syscalls.
                    if name.starts_with("System.Runtime.")
                        && !matches!(
                            name.as_str(),
                            "System.Runtime.Notify" | "System.Runtime.Log"
                        )
                    {
                        hazards.reads_environment = true;
                    }

                    // Iterators are produced by storage queries; treat them as state reads.
                    if matches!(
                        name.as_str(),
                        "System.Iterator.Next" | "System.Iterator.Value"
                    ) {
                        hazards.reads_state = true;
                    }

                    // Track contract calls that grant write/notify permissions so that safe
                    // Solidity methods cannot indirectly perform stateful cross-contract calls.
                    if name == "System.Contract.Call" {
                        hazards.contract_calls = true;
                    }
                    if name == "System.Contract.Call" && *arg_count == 4 {
                        let flags = match instrs.get(index.wrapping_sub(3)) {
                            Some(Instruction::PushLiteral(LiteralValue::Integer(value))) => {
                                parse_u8_literal(value)
                            }
                            _ => None,
                        };
                        if flags.is_none_or(call_flags_allow_write_or_notify) {
                            hazards.unsafe_contract_call = true;
                        }
                    }
                }
                BuiltinCall::ContractCall => {
                    // ContractCall uses CallFlags.All in the bytecode emitter.
                    hazards.contract_calls = true;
                    hazards.unsafe_contract_call = true;
                }
                BuiltinCall::ContractCallWithFlags => {
                    hazards.contract_calls = true;
                    let flags = match instrs.get(index.wrapping_sub(1)) {
                        Some(Instruction::PushLiteral(LiteralValue::Integer(value))) => {
                            parse_u8_literal(value)
                        }
                        _ => None,
                    };
                    if flags.is_none_or(call_flags_allow_write_or_notify) {
                        hazards.unsafe_contract_call = true;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    hazards
}

pub(crate) fn build_call_graph(functions: &[Function]) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for function in functions {
        let mut callees: Vec<String> = Vec::new();
        for instr in function
            .basic_blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
        {
            match instr {
                Instruction::CallFunction { name, .. } => callees.push(name.clone()),
                // Task #186 function pointers: taking a function's address
                // (`PushFunctionOffset`) makes it invocable through
                // `CallIndirect` (CALLA), which carries no static callee.
                // Conservatively treat the address-taken function as a callee
                // of the function that takes its address so its hazards
                // (storage writes, notifications, unsafe contract calls)
                // propagate transitively — otherwise a `view` method calling
                // a stateful function through a pointer is marked safe in the
                // manifest while mutating storage at runtime.
                Instruction::PushFunctionOffset { name } => callees.push(name.clone()),
                _ => {}
            }
        }
        graph.insert(function.name.clone(), callees);
    }
    graph
}

pub(crate) fn compute_transitive_hazards(
    name: &str,
    direct: &HashMap<String, Hazards>,
    graph: &HashMap<String, Vec<String>>,
    memo: &mut HashMap<String, Hazards>,
    visiting: &mut HashSet<String>,
) -> Hazards {
    if let Some(value) = memo.get(name).copied() {
        return value;
    }

    if visiting.contains(name) {
        return direct.get(name).copied().unwrap_or_default();
    }

    visiting.insert(name.to_string());

    let mut hazards = direct.get(name).copied().unwrap_or_default();
    if let Some(callees) = graph.get(name) {
        for callee in callees {
            let child = compute_transitive_hazards(callee, direct, graph, memo, visiting);
            hazards.merge_in(child);
        }
    }

    visiting.remove(name);
    memo.insert(name.to_string(), hazards);
    hazards
}
