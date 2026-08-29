use super::*;

/// Syscall gas cost table based on Neo N3 fee schedule.
/// Note: Some syscalls use approximate costs; see Neo N3 documentation for exact values.
pub fn syscall_gas_table() -> std::collections::HashMap<[u8; 4], u64> {
    use std::collections::HashMap;
    let mut m = HashMap::new();
    for (_, spec) in SYSCALLS.iter() {
        let gas = match spec.name {
            // Storage
            "System.Storage.GetContext"
            | "System.Storage.GetReadOnlyContext"
            | "System.Storage.AsReadOnly" => 1,
            "System.Storage.Get" => 100,
            "System.Storage.Put" => 1_000,
            "System.Storage.Delete" => 100,
            "System.Storage.Find" => 100,
            // Iterators
            "System.Iterator.Next" | "System.Iterator.Value" => 1,
            // Runtime
            "System.Runtime.Platform"
            | "System.Runtime.GetTrigger"
            | "System.Runtime.GetScriptContainer"
            | "System.Runtime.GetNetwork"
            | "System.Runtime.GetAddressVersion"
            | "System.Runtime.GetTime"
            | "System.Runtime.GasLeft"
            | "System.Runtime.GetInvocationCounter"
            | "System.Runtime.GetCallingScriptHash"
            | "System.Runtime.GetEntryScriptHash"
            | "System.Runtime.GetExecutingScriptHash"
            | "System.Runtime.GetNotifications"
            | "System.Runtime.BurnGas"
            | "System.Runtime.CurrentSigners"
            | "System.Runtime.GetMsgValue"
            | "System.Runtime.LoadScript" => 1,
            "System.Runtime.CheckWitness" => 200,
            "System.Runtime.Log" | "System.Runtime.Notify" => 1,
            "System.Runtime.GetRandom" => 50,
            // Crypto
            "System.Crypto.CheckSig" | "System.Crypto.CheckMultisig" => 1_000,
            // Contract / native calls
            "System.Contract.Call" | "System.Contract.GetCallFlags" => 10,
            "System.Contract.CreateStandardAccount" | "System.Contract.CreateMultisigAccount" => 10,
            _ => 1,
        };
        m.insert(spec.id, gas);
    }
    m
}
