use super::*;

pub(crate) fn resolve_native_calls_member(member: &str) -> Option<BuiltinCall> {
    match member {
        // ========== NEO native contract ==========
        "neoTotalSupply" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "totalSupply".to_string(),
        }),
        "neoBalanceOf" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "balanceOf".to_string(),
        }),
        "neoTransfer" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "transfer".to_string(),
        }),
        "neoDecimals" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "decimals".to_string(),
        }),
        "neoSymbol" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "symbol".to_string(),
        }),
        // NOTE: Neo's native NEO/GAS (`FungibleToken`) contracts expose only the
        // NEP-17 methods (symbol/decimals/totalSupply/balanceOf/transfer); there
        // is no callable `name` method (the contract name lives in the manifest,
        // not the ABI). A `neoName`/`gasName` → native `name` mapping would emit a
        // `System.Contract.Call` that faults with "method not found" on a real
        // node, so it is intentionally NOT registered here.
        "vote" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "vote".to_string(),
        }),
        "getCandidates" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCandidates".to_string(),
        }),
        "registerCandidate" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "registerCandidate".to_string(),
        }),
        "unregisterCandidate" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "unregisterCandidate".to_string(),
        }),
        "getGasPerBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getGasPerBlock".to_string(),
        }),
        "getRegisterPrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getRegisterPrice".to_string(),
        }),
        "setRegisterPrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "setRegisterPrice".to_string(),
        }),
        "setGasPerBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "setGasPerBlock".to_string(),
        }),
        "getAccountState" => Some(BuiltinCall::GetNeoAccountState),
        "unclaimedGas" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "unclaimedGas".to_string(),
        }),
        "getCandidateVote" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCandidateVote".to_string(),
        }),
        "getCommittee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCommittee".to_string(),
        }),
        "getCommitteeAddress" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getCommitteeAddress".to_string(),
        }),
        "getNextBlockValidators" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getNextBlockValidators".to_string(),
        }),
        "getAllCandidates" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Neo,
            method: "getAllCandidates".to_string(),
        }),

        // ========== GAS native contract ==========
        "gasTotalSupply" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "totalSupply".to_string(),
        }),
        "gasBalanceOf" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "balanceOf".to_string(),
        }),
        "gasTransfer" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "transfer".to_string(),
        }),
        "gasDecimals" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "decimals".to_string(),
        }),
        "gasSymbol" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Gas,
            method: "symbol".to_string(),
        }),
        // See the `neoName` note above: native GAS has no callable `name` method.

        // ========== ContractManagement native contract ==========
        "deployContract" => Some(BuiltinCall::DeployContract),
        "updateContract" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "update".to_string(),
        }),
        "destroyContract" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "destroy".to_string(),
        }),
        "getContract" => Some(BuiltinCall::GetContract),
        "getContractById" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "getContractById".to_string(),
        }),
        "listContracts" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "getContractHashes".to_string(),
        }),
        "hasMethod" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "hasMethod".to_string(),
        }),
        "isContract" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "isContract".to_string(),
        }),
        "getMinimumDeploymentFee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "getMinimumDeploymentFee".to_string(),
        }),
        "setMinimumDeploymentFee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::ContractManagement,
            method: "setMinimumDeploymentFee".to_string(),
        }),

        // ========== Policy native contract ==========
        "getFeePerByte" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getFeePerByte".to_string(),
        }),
        "setFeePerByte" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setFeePerByte".to_string(),
        }),
        "getExecFeeFactor" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getExecFeeFactor".to_string(),
        }),
        "getExecPicoFeeFactor" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getExecPicoFeeFactor".to_string(),
        }),
        "setExecFeeFactor" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setExecFeeFactor".to_string(),
        }),
        "getStoragePrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getStoragePrice".to_string(),
        }),
        "getMillisecondsPerBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getMillisecondsPerBlock".to_string(),
        }),
        "setMillisecondsPerBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setMillisecondsPerBlock".to_string(),
        }),
        "getMaxValidUntilBlockIncrement" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getMaxValidUntilBlockIncrement".to_string(),
        }),
        "setMaxValidUntilBlockIncrement" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setMaxValidUntilBlockIncrement".to_string(),
        }),
        "getMaxTraceableBlocks" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getMaxTraceableBlocks".to_string(),
        }),
        "setMaxTraceableBlocks" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setMaxTraceableBlocks".to_string(),
        }),
        "getAttributeFee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getAttributeFee".to_string(),
        }),
        "setAttributeFee" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setAttributeFee".to_string(),
        }),
        "setStoragePrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setStoragePrice".to_string(),
        }),
        "blockAccount" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "blockAccount".to_string(),
        }),
        "unblockAccount" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "unblockAccount".to_string(),
        }),
        "isBlocked" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "isBlocked".to_string(),
        }),
        "getBlockedAccounts" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getBlockedAccounts".to_string(),
        }),
        "recoverFund" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "recoverFund".to_string(),
        }),
        "setWhitelistFeeContract" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "setWhitelistFeeContract".to_string(),
        }),
        "removeWhitelistFeeContract" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "removeWhitelistFeeContract".to_string(),
        }),
        "getWhitelistFeeContracts" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Policy,
            method: "getWhitelistFeeContracts".to_string(),
        }),

        // ========== Oracle native contract ==========
        "requestOracleData" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "request".to_string(),
        }),
        "getOraclePrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "getPrice".to_string(),
        }),
        "setOraclePrice" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "setPrice".to_string(),
        }),
        "oracleFinish" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "finish".to_string(),
        }),
        "oracleVerify" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "verify".to_string(),
        }),
        "oracleRequest" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Oracle,
            method: "request".to_string(),
        }),

        // ========== RoleManagement native contract ==========
        "designateAsRole" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::RoleManagement,
            method: "designateAsRole".to_string(),
        }),
        "getDesignatedByRole" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::RoleManagement,
            method: "getDesignatedByRole".to_string(),
        }),

        // ========== Ledger native contract ==========
        "currentIndex" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "currentIndex".to_string(),
        }),
        "currentHash" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "currentHash".to_string(),
        }),
        "getBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getBlock".to_string(),
        }),
        // NOTE: no `getBlockHash` — the live LedgerContract native ABI has no
        // such method (verified against mainnet getnativecontracts); it would
        // fault method-not-found on a real node. Use `blockhash(n)` (the EVM
        // global, lowered to getBlock(n).hash) or `getBlock(n)` directly.
        "getBlockByIndex" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getBlock".to_string(),
        }),
        "getBlockByHash" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getBlock".to_string(),
        }),
        "getTransaction" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransaction".to_string(),
        }),
        "getTransactionHeight" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransactionHeight".to_string(),
        }),
        "getTransactionFromBlock" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransactionFromBlock".to_string(),
        }),
        "getTransactionSigners" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransactionSigners".to_string(),
        }),
        "getTransactionVMState" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Ledger,
            method: "getTransactionVMState".to_string(),
        }),
        // NOTE: no `getBlockSystemFee` — no such method exists on the live
        // LedgerContract native ABI (nor any block-system-fee getter); the old
        // registration compiled to a native call that faults on a real node.

        // ========== Helpers ==========
        "externalNativeCall" => Some(BuiltinCall::ContractCall),

        // ========== Notary native contract ==========
        "notaryVerify" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "verify".to_string(),
        }),
        "notaryBalanceOf" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "balanceOf".to_string(),
        }),
        "notaryExpirationOf" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "expirationOf".to_string(),
        }),
        "notaryLockDepositUntil" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "lockDepositUntil".to_string(),
        }),
        "notaryWithdraw" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "withdraw".to_string(),
        }),
        "notaryGetMaxNotValidBeforeDelta" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "getMaxNotValidBeforeDelta".to_string(),
        }),
        "notarySetMaxNotValidBeforeDelta" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "setMaxNotValidBeforeDelta".to_string(),
        }),
        "notaryOnNEP17Payment" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Notary,
            method: "onNEP17Payment".to_string(),
        }),

        // ========== Treasury native contract ==========
        "treasuryVerify" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Treasury,
            method: "verify".to_string(),
        }),
        "treasuryOnNEP17Payment" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Treasury,
            method: "onNEP17Payment".to_string(),
        }),
        "treasuryOnNEP11Payment" => Some(BuiltinCall::NativeCall {
            contract: NativeContract::Treasury,
            method: "onNEP11Payment".to_string(),
        }),

        _ => None,
    }
}
