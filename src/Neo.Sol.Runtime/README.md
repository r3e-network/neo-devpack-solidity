# Neo.Sol.Runtime

> [!IMPORTANT]
> **Standalone experimental library — not part of the `neo-solc` compiler pipeline.**
> Neo.Sol.Runtime is a standalone, experimental EVM-emulation library. It is **not used by
> `neo-solc`**, is **not shipped in releases** (no NuGet package is published), and its storage
> layout (EVM keccak-style slot mapping) is **intentionally different** from the layout used by
> compiler-emitted contracts, which use `SHA256(variable_name)` base slots plus
> `keccak256(serialize(key) || slot)` for mapping elements. Contracts produced by `neo-solc`
> cannot read or write storage written through this library, and vice versa.

A comprehensive runtime library providing EVM semantic emulation on NeoVM. This library enables Solidity-style smart contract development on the Neo blockchain by providing EVM-compatible memory management, storage layouts, ABI encoding/decoding, cryptographic functions, event systems, and cross-contract interaction patterns.

## Features

### 🧠 Memory Management
- **32-byte word addressing** with EVM-compatible memory layout
- **Quadratic gas cost calculation** following EVM memory expansion rules
- **Page-based allocation** for efficient memory usage
- **Memory statistics** and performance monitoring

### 💾 Storage System
- **EVM storage slot mapping** to Neo storage keys
- **Collision-resistant key generation** using Keccak256
- **Solidity storage layout preservation** for arrays and mappings
- **Storage caching** for optimized access patterns

### 🔧 ABI Encoding/Decoding
- **Full ABI specification compliance** for function calls and returns
- **Dynamic type support** for strings, bytes, and arrays  
- **Static type encoding** for integers, addresses, and booleans
- **Function selector calculation** and parameter encoding

### 🔐 Cryptographic Library
- **Keccak256 hashing** for EVM compatibility
- **SHA256 support** for additional security functions
- **ecrecover implementation** for signature verification
- **Ethereum address generation** from public keys

### 📡 Event System
- **EVM LOG0-LOG4 opcode compatibility** with indexed parameters
- **Runtime.Notify integration** for Neo blockchain events
- **Standard ERC event patterns** (Transfer, Approval, etc.)
- **Event filtering and querying** capabilities

### 🌐 Context Objects
- **msg.* context** (sender, value, data, gas) with Neo mapping
- **tx.* context** (origin, gasprice, nonce) for transaction info
- **block.* context** (number, timestamp, hash) for blockchain state
- **Gas tracking and management** utilities

### 📞 External Calls
- **CALL, DELEGATECALL, STATICCALL** opcode equivalents
- **Contract creation** with CREATE and CREATE2 semantics
- **Address calculation** for deterministic deployment
- **Call result handling** and error management

### 📋 Address Registry
- **Contract registration and metadata** management
- **Interface support tracking** (EIP-165 compatibility)
- **ENS-style name resolution** for human-readable addresses
- **Cross-contract discovery** and interaction patterns

## Quick Start

### Basic Usage

```csharp
using Neo.Sol.Runtime;

public class MyContract : SmartContract
{
    private static readonly EvmRuntime runtime = Evm.CreateRuntime();
    
    public static void Main()
    {
        // Access EVM-compatible context
        var sender = runtime.Msg.Sender;
        var value = runtime.Msg.Value;
        var blockNumber = runtime.Block.Number;
        
        // Use memory operations
        runtime.Memory.Store(0, BigInteger.Parse("123456789"));
        var stored = runtime.Memory.LoadBigInteger(0);
        
        // Store data with Solidity layout
        var slot = BigInteger.Zero;
        runtime.Storage.Store(slot, stored);
        
        // Emit events
        runtime.Events.Log2("ValueStored(uint256,address)", stored, sender);
    }
}
```

### Memory Operations

```csharp
// Store 32-byte words
var memAddr = 0u;
var value = new BigInteger(0x123456789ABCDEF);
runtime.Memory.Store(memAddr, value);

// Load data back
var loaded = runtime.Memory.LoadBigInteger(memAddr);

// Store arbitrary bytes
var data = System.Text.Encoding.UTF8.GetBytes("Hello, Neo!");
runtime.Memory.StoreBytes(64, data);
var retrieved = runtime.Memory.LoadBytes(64, (uint)data.Length);
```

### Storage with Solidity Layout

```csharp
// Simple storage
var slot0 = new BigInteger(0);
runtime.Storage.Store(slot0, new BigInteger(42));

// Array element storage
var arraySlot = new BigInteger(1);
var index = new BigInteger(5);
var elementSlot = StorageManager.CalculateArrayElementSlot(arraySlot, index);
runtime.Storage.Store(elementSlot, new BigInteger(100));

// Mapping storage
var mappingSlot = new BigInteger(2);
var key = runtime.Msg.Sender; // address key
var mappingElementSlot = StorageManager.CalculateMappingElementSlot(mappingSlot, key.ToArray());
runtime.Storage.Store(mappingElementSlot, new BigInteger(200));
```

### Event Emission

```csharp
// Standard ERC20 Transfer event
var from = UInt160.Parse("0x1234567890123456789012345678901234567890");
var to = UInt160.Parse("0x0987654321098765432109876543210987654321");
var amount = new BigInteger(1000);

StandardEvents.EmitTransfer(runtime.Events, from, to, amount);

// Custom events with multiple topics
runtime.Events.Log3(
    "CustomEvent(uint256,address,bytes32)",
    new BigInteger(42),      // indexed topic 1
    runtime.Msg.Sender,     // indexed topic 2  
    runtime.Keccak256(System.Text.Encoding.UTF8.GetBytes("data")) // indexed topic 3
);
```

### Cross-Contract Calls

```csharp
// External contract call
var targetContract = UInt160.Parse("0x1111111111111111111111111111111111111111");
var callData = Evm.EncodeCall("transfer(address,uint256)", to, amount);

var result = runtime.Calls.Call(targetContract, 0, 100000, callData);
if (result.Success)
{
    // Handle successful call
    var returnData = result.ReturnData;
}

// Static call for read-only operations
var balanceCall = Evm.EncodeCall("balanceOf(address)", runtime.Msg.Sender);
var balanceResult = runtime.Calls.StaticCall(targetContract, 50000, balanceCall);
```

### Cryptographic Operations

```csharp
// Hash functions
var data = System.Text.Encoding.UTF8.GetBytes("Hello, World!");
var keccak = runtime.Keccak256(data);
var sha = runtime.Sha256(data);

// Signature recovery
var messageHash = runtime.Keccak256(System.Text.Encoding.UTF8.GetBytes("message"));
var signature = new byte[64]; // r + s
var recoveryId = 0;
var publicKey = runtime.EcRecover(messageHash, signature, recoveryId);

if (publicKey != null)
{
    var address = runtime.PublicKeyToAddress(publicKey);
}
```

### Address Registry

```csharp
// Register contract
var contractInfo = new ContractInfo
{
    Name = "MyToken",
    Version = "1.0.0",
    Description = "ERC20 token implementation",
    Owner = runtime.Msg.Sender,
    IsActive = true,
    CreatedAt = runtime.Now
};

runtime.Registry.RegisterContract(runtime.ContractAddress, contractInfo);

// Register name
runtime.Registry.RegisterName("mytoken.neo", runtime.ContractAddress, runtime.Msg.Sender);

// Resolve name
var resolved = runtime.Registry.ResolveName("mytoken.neo");
```

## Architecture

The library is organized into several key components:

- **`EvmRuntime`**: Main runtime class providing unified access
- **`EvmMemoryManager`**: EVM-compatible memory with word addressing
- **`StorageManager`**: Neo storage with Solidity layout preservation  
- **`EventManager`**: Event emission with Runtime.Notify integration
- **`ExecutionContext`**: EVM context objects (msg, tx, block)
- **`ExternalCallManager`**: Cross-contract interaction facades
- **`AddressRegistry`**: Contract discovery and name resolution
- **`AbiEncoder/Decoder`**: ABI encoding/decoding utilities
- **`CryptoLib`**: Cryptographic functions (keccak256, ecrecover, sha256)

## Performance Considerations

### Memory Management
- Page-based allocation reduces fragmentation
- Quadratic cost calculation prevents memory abuse
- Statistics tracking for optimization

### Storage Optimization  
- Key collision resistance with Keccak256
- Caching layer reduces redundant Neo storage calls
- Efficient Solidity layout mapping

### Event System
- Minimal overhead with direct Runtime.Notify integration
- Standard event patterns for common use cases
- Indexed parameter optimization

### Cross-Contract Calls
- Call result caching and optimization
- Gas estimation for cost prediction
- Error handling with graceful degradation

## Neo Integration

The library seamlessly integrates with Neo's blockchain infrastructure:

- **Storage Context**: Uses Neo's persistent storage system
- **Runtime Services**: Leverages Neo's Runtime.* functions
- **Gas Tracking**: Integrates with Neo's gas metering
- **Contract Calls**: Uses Neo's Contract.Call infrastructure
- **Event System**: Maps to Neo's Runtime.Notify mechanism

## Testing

Comprehensive test suite covering:

- Memory management edge cases
- Storage layout compatibility
- ABI encoding/decoding accuracy
- Cryptographic function correctness
- Event emission verification
- Cross-contract interaction patterns

Run tests with:
```bash
dotnet test
```

## Contributing

Contributions are welcome! Please ensure:

1. **Test coverage** for new features
2. **Documentation** updates for API changes  
3. **Performance** considerations for runtime operations
4. **Compatibility** with existing EVM semantics

## License

MIT License - see LICENSE file for details.

## Roadmap

- [ ] **Precompiled contracts** (ecmul, ecadd, modexp)
- [ ] **Gas metering** integration with Neo's system
- [ ] **Debug interfaces** for development tooling
- [ ] **Optimizer** for common patterns
- [ ] **Proxy patterns** support (EIP-1967)
- [ ] **Diamond pattern** support (EIP-2535)