# Neo DevPack Solidity — Contract Structure

## File Organization

```
devpack/contracts/
├── Framework.sol              # Thin contract base (deploy hook)
├── FrameworkBase.sol          # Management + upgrade helpers
├── Syscalls.sol               # Unified syscall entry point (backward compatible)
├── NativeCalls.sol            # Unified native calls entry point (backward compatible)
├── NativeContracts.sol        # Canonical native contract address constants
├── Types.sol, NeoBytes.sol, NeoMath.sol, OracleService.sol
│
├── syscalls/                  # Domain-focused syscall libraries ✨ NEW
│   ├── SyscallsBase.sol       # Shared helpers & native contract addresses
│   ├── SyscallsTypes.sol      # Data structures (Block, Transaction, Signer, etc.)
│   ├── SyscallsLedger.sol     # Ledger: blocks, transactions, signers
│   ├── SyscallsContract.sol   # Contract: deployment, management, script hashes
│   ├── SyscallsStorage.sol    # Storage: get/put/delete/find, storage contexts
│   ├── SyscallsRuntime.sol    # Runtime: time, gas, platform, witness, notifications
│   ├── SyscallsCrypto.sol     # Crypto: checkSig, sha256, keccak256, ECDSA, BLS
│   ├── SyscallsStdLib.sol     # StdLib: serialize, encoding, memory, iterators
│   ├── SyscallsPolicy.sol     # Policy: fee factors, storage price, network params
│   ├── SyscallsOracle.sol     # Oracle: oracle requests, pricing
│   └── SyscallsRole.sol       # Role: designated-by-role queries
│
├── native/                    # Domain-focused native contract libraries ✨ NEW
│   ├── NativeTypes.sol        # Data structures (Candidate, AccountState, etc.)
│   ├── NativeNEO.sol          # NEO token & governance operations
│   ├── NativeGAS.sol          # GAS token operations
│   ├── NativeContractMgmt.sol # Contract deployment, updates, queries
│   ├── NativePolicy.sol       # Network policy parameters
│   ├── NativeOracle.sol       # Oracle service operations
│   ├── NativeRole.sol         # Role/permission management
│   ├── NativeLedger.sol       # Blockchain data access
│   ├── NativeNotary.sol       # Notary deposit services
│   └── NativeTreasury.sol     # Treasury fund management
│
├── standards/                 # NEP standard contracts
│   ├── NEP11.sol, NEP17.sol, NEP22.sol, NEP24.sol
│   ├── NEP26.sol, NEP27.sol, NEP29.sol, NEP30.sol, NEP31.sol
│
├── compat/                    # EVM compatibility layer
└── examples/                  # Complete example contracts
```

## Usage

### Option 1: Unified entry point (backward compatible)
```solidity
import "devpack/contracts/Syscalls.sol";
import "devpack/contracts/NativeCalls.sol";

contract MyContract {
    function myFunc() public {
        uint256 block = Syscalls.getCurrentIndex();
        uint256 supply = NativeCalls.neoTotalSupply();
    }
}
```

### Option 2: Domain-specific imports (recommended for new code)
```solidity
import "devpack/contracts/syscalls/SyscallsLedger.sol";
import "devpack/contracts/native/NativeNEO.sol";

contract MyContract {
    function myFunc() public {
        uint256 block = SyscallsLedger.getCurrentIndex();
        uint256 supply = NativeNEO.neoTotalSupply();
    }
}
```
