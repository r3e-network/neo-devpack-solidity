use super::*;

// Syscall registry – the names are authoritative; IDs are derived via SHA-256.
//
// IMPORTANT: This list is aligned to Neo N3 `ApplicationEngine.Register(...)` syscall names.
pub static SYSCALLS: Lazy<HashMap<[u8; 4], SyscallSpec>> = Lazy::new(|| {
    let names = [
        // Storage
        "System.Storage.GetContext",
        "System.Storage.GetReadOnlyContext",
        "System.Storage.AsReadOnly",
        "System.Storage.Get",
        "System.Storage.Put",
        "System.Storage.Delete",
        "System.Storage.Find",
        // Iterator
        "System.Iterator.Next",
        "System.Iterator.Value",
        // Crypto
        "System.Crypto.CheckSig",
        "System.Crypto.CheckMultisig",
        // Contract
        "System.Contract.Call",
        "System.Contract.GetCallFlags",
        "System.Contract.CreateStandardAccount",
        "System.Contract.CreateMultisigAccount",
        // Runtime
        "System.Runtime.GetTrigger",
        "System.Runtime.Platform",
        "System.Runtime.GetNetwork",
        "System.Runtime.GetAddressVersion",
        "System.Runtime.GetTime",
        "System.Runtime.GetScriptContainer",
        "System.Runtime.GetExecutingScriptHash",
        "System.Runtime.GetCallingScriptHash",
        "System.Runtime.GetEntryScriptHash",
        "System.Runtime.LoadScript",
        "System.Runtime.CheckWitness",
        "System.Runtime.GetInvocationCounter",
        "System.Runtime.GetRandom",
        "System.Runtime.Log",
        "System.Runtime.Notify",
        "System.Runtime.GetNotifications",
        "System.Runtime.GasLeft",
        "System.Runtime.BurnGas",
        "System.Runtime.CurrentSigners",
        // Task #113 — Solidity `msg.value` host-injection slot. Neo N3 has
        // no native attached-value concept, so this syscall reads the
        // runtime-side override (`ExecutionOverrides::value` /
        // `NeoRuntime::override_value`). Returns 0 when no override set.
        "System.Runtime.GetMsgValue",
    ];

    let mut m = HashMap::new();
    for name in names {
        m.insert(
            interop_id_bytes(name),
            SyscallSpec {
                id: interop_id_bytes(name),
                name,
            },
        );
    }
    m
});

pub fn syscall_name(id: &[u8; 4]) -> Option<&'static str> {
    SYSCALLS.get(id).map(|s| s.name)
}
