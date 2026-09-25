use super::*;

pub(crate) fn resolve_runtime_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "notify" => Some(BuiltinCall::RuntimeNotify),
        "checkWitness" => Some(BuiltinCall::RuntimeCheckWitness),
        "gasLeft" => Some(BuiltinCall::Syscall("System.Runtime.GasLeft".to_string())),
        "burnGas" => Some(BuiltinCall::Syscall("System.Runtime.BurnGas".to_string())),
        "log" => Some(BuiltinCall::Syscall("System.Runtime.Log".to_string())),
        "getTime" => Some(BuiltinCall::Syscall("System.Runtime.GetTime".to_string())),
        "getTrigger" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetTrigger".to_string(),
        )),
        "getInvocationCounter" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetInvocationCounter".to_string(),
        )),
        "getCurrentSigners" => Some(BuiltinCall::Syscall(
            "System.Runtime.CurrentSigners".to_string(),
        )),
        "getCallFlags" => Some(BuiltinCall::Syscall(
            "System.Contract.GetCallFlags".to_string(),
        )),
        "getScriptContainer" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetScriptContainer".to_string(),
        )),
        "loadScript" => Some(BuiltinCall::Syscall(
            "System.Runtime.LoadScript".to_string(),
        )),
        "getNetwork" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetNetwork".to_string(),
        )),
        "getPlatform" => Some(BuiltinCall::Syscall("System.Runtime.Platform".to_string())),
        "getAddressVersion" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetAddressVersion".to_string(),
        )),
        "getRandom" => Some(BuiltinCall::Syscall("System.Runtime.GetRandom".to_string())),
        "getExecutingScriptHash" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetExecutingScriptHash".to_string(),
        )),
        "getCallingScriptHash" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetCallingScriptHash".to_string(),
        )),
        "getEntryScriptHash" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetEntryScriptHash".to_string(),
        )),
        _ => None,
    }
}

pub(crate) fn resolve_storage_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "find" => Some(BuiltinCall::StorageFind),
        "put" => Some(BuiltinCall::StoragePut),
        "get" => Some(BuiltinCall::StorageGet),
        "remove" => Some(BuiltinCall::StorageDelete),
        "initializeContext" | "getContext" => Some(BuiltinCall::Syscall(
            "System.Storage.GetContext".to_string(),
        )),
        "getReadOnlyContext" => Some(BuiltinCall::Syscall(
            "System.Storage.GetReadOnlyContext".to_string(),
        )),
        "asReadOnly" => Some(BuiltinCall::Syscall(
            "System.Storage.AsReadOnly".to_string(),
        )),
        _ => None,
    }
}

pub(crate) fn resolve_neo_member(member: &str) -> Option<BuiltinCall> {
    match member {
        "verifySignature" => Some(BuiltinCall::VerifySignature),
        "callContract" => Some(BuiltinCall::ContractCall),
        "deployContract" => Some(BuiltinCall::DeployContract),
        "getNeoBalance" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "balanceOf".to_string(),
        }),
        "getGasBalance" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "balanceOf".to_string(),
        }),
        "getFeePerByte" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getFeePerByte".to_string(),
        }),
        "getGasPrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getFeePerByte".to_string(),
        }),
        "getStoragePrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getStoragePrice".to_string(),
        }),
        "getCommittee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCommittee".to_string(),
        }),
        "getValidators" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getNextBlockValidators".to_string(),
        }),
        "isCommittee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCommittee".to_string(),
        }),
        "isValidator" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getNextBlockValidators".to_string(),
        }),
        "getRandom" => Some(BuiltinCall::Syscall("System.Runtime.GetRandom".to_string())),
        "getBlockHeight" | "getCurrentBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "currentIndex".to_string(),
        }),
        "transactionExists" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransaction".to_string(),
        }),
        "getNetworkMagic" => Some(BuiltinCall::Syscall(
            "System.Runtime.GetNetwork".to_string(),
        )),
        "getBlockByIndex" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getBlock".to_string(),
        }),
        "getBlockTime" => Some(BuiltinCall::Syscall("System.Runtime.GetTime".to_string())),
        "getTransaction" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransaction".to_string(),
        }),
        "getTransactionHeight" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransactionHeight".to_string(),
        }),
        "verifyWithWitness" => Some(BuiltinCall::Syscall(
            "System.Runtime.CheckWitness".to_string(),
        )),
        "sha256Hash" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::CryptoLib,
            method: "sha256".to_string(),
        }),
        "ripemd160Hash" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::CryptoLib,
            method: "ripemd160".to_string(),
        }),
        _ => None,
    }
}
