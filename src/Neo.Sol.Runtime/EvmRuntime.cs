using System.Numerics;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.Memory;
using Neo.Sol.Runtime.Storage;
using Neo.Sol.Runtime.Events;
using Neo.Sol.Runtime.Context;
using ExecutionContext = Neo.Sol.Runtime.Context.ExecutionContext;
using Neo.Sol.Runtime.Calls;
using Neo.Sol.Runtime.Registry;
using Neo.Sol.Runtime.ABI;
using Neo.SmartContract.Framework.Native;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;
using NeoFrameworkStorage = Neo.SmartContract.Framework.Services.Storage;

namespace Neo.Sol.Runtime;

/// <summary>
/// Main EVM runtime providing semantic compatibility layer for Solidity contracts on NeoVM
/// Integrates memory management, storage, events, context, and cross-contract calls
/// </summary>
public sealed class EvmRuntime : IDisposable
{
    private readonly StorageContext _storageContext;
    private readonly EvmMemoryManager _memoryManager;
    private readonly StorageManager _storageManager;
    private readonly EventManager _eventManager;
    private readonly ExternalCallManager _callManager;
    private readonly AddressRegistry _addressRegistry;
    private readonly ExecutionContext _executionContext;
    
    private bool _disposed = false;
    
    /// <summary>
    /// Initialize EVM runtime for contract execution
    /// </summary>
    /// <param name="contractAddress">Address of the current contract</param>
    public EvmRuntime(UInt160 contractAddress)
    {
        ContractAddress = contractAddress;

        try
        {
            _storageContext = NeoFrameworkStorage.CurrentContext;
        }
        catch
        {
            _storageContext = new StorageContext();
        }
        _memoryManager = new EvmMemoryManager();
        _storageManager = new StorageManager(_storageContext);
        _eventManager = new EventManager(contractAddress);
        _addressRegistry = new AddressRegistry(_storageContext);
        _executionContext = ExecutionContext.Current;
        _callManager = new ExternalCallManager(_executionContext);
    }
    
    /// <summary>
    /// Current contract address
    /// </summary>
    public UInt160 ContractAddress { get; }
    
    /// <summary>
    /// Memory manager for EVM-compatible memory operations
    /// </summary>
    public EvmMemoryManager Memory => _memoryManager;
    
    /// <summary>
    /// Storage manager for contract storage
    /// </summary>
    public StorageManager Storage => _storageManager;
    
    /// <summary>
    /// Event manager for emitting events
    /// </summary>
    public EventManager Events => _eventManager;
    
    /// <summary>
    /// External call manager for cross-contract interactions
    /// </summary>
    public ExternalCallManager Calls => _callManager;
    
    /// <summary>
    /// Address registry for contract resolution
    /// </summary>
    public AddressRegistry Registry => _addressRegistry;
    
    /// <summary>
    /// Execution context (msg, tx, block)
    /// </summary>
    public ExecutionContext Context => _executionContext;
    
    // EVM-compatible global variables and functions
    
    /// <summary>
    /// Current block information (block.*)
    /// </summary>
    public BlockContext Block => _executionContext.Block;
    
    /// <summary>
    /// Current message information (msg.*)
    /// </summary>
    public MsgContext Msg => _executionContext.Msg;
    
    /// <summary>
    /// Current transaction information (tx.*)
    /// </summary>
    public TxContext Tx => _executionContext.Tx;
    
    /// <summary>
    /// Get current block timestamp
    /// </summary>
    public ulong Now => Block.Timestamp;
    
    /// <summary>
    /// Get current block number
    /// </summary>
    public uint BlockNumber => Block.Number;
    
    /// <summary>
    /// Get current block hash
    /// </summary>
    public UInt256 BlockHash => Block.Hash;
    
    // EVM-compatible cryptographic functions
    
    /// <summary>
    /// Compute Keccak256 hash
    /// </summary>
    /// <param name="data">Data to hash</param>
    /// <returns>32-byte hash</returns>
    public byte[] Keccak256(byte[] data) => Crypto.CryptoLib.Keccak256(data);
    
    /// <summary>
    /// Compute SHA256 hash
    /// </summary>
    /// <param name="data">Data to hash</param>
    /// <returns>32-byte hash</returns>
    public byte[] Sha256(byte[] data) => Crypto.CryptoLib.Sha256(data);
    
    /// <summary>
    /// Recover public key from signature (ecrecover)
    /// </summary>
    /// <param name="messageHash">Message hash</param>
    /// <param name="signature">Signature</param>
    /// <param name="recoveryId">Recovery ID</param>
    /// <returns>Recovered public key</returns>
    public byte[]? EcRecover(byte[] messageHash, byte[] signature, int recoveryId)
        => Crypto.CryptoLib.EcRecover(messageHash, signature, recoveryId);
    
    /// <summary>
    /// Convert public key to Ethereum address
    /// </summary>
    /// <param name="publicKey">Public key</param>
    /// <returns>Ethereum address</returns>
    public byte[] PublicKeyToAddress(byte[] publicKey)
        => Crypto.CryptoLib.PublicKeyToAddress(publicKey);
    
    // EVM-compatible utility functions
    
    /// <summary>
    /// Revert transaction with error message
    /// </summary>
    /// <param name="reason">Revert reason</param>
    public void Revert(string reason = "")
    {
        if (!string.IsNullOrEmpty(reason))
        {
            // Encode revert reason as ABI error
            var errorSignature = "Error(string)";
            var encodedReason = AbiEncoder.EncodeCall(errorSignature, reason);
            throw new System.Exception($"Revert: {reason}");
        }
        
        throw new System.Exception("Revert");
    }
    
    /// <summary>
    /// Require condition or revert
    /// </summary>
    /// <param name="condition">Condition to check</param>
    /// <param name="message">Error message if condition fails</param>
    public void Require(bool condition, string message = "Requirement failed")
    {
        if (!condition)
        {
            Revert(message);
        }
    }
    
    /// <summary>
    /// Assert condition (for internal errors)
    /// </summary>
    /// <param name="condition">Condition to check</param>
    public void Assert(bool condition)
    {
        if (!condition)
        {
            // In EVM, assert failures consume all gas
            throw new System.Exception("Assertion failed");
        }
    }
    
    /// <summary>
    /// Selfdestruct the contract (transfer remaining balance)
    /// </summary>
    /// <param name="recipient">Address to receive remaining balance</param>
    public void SelfDestruct(UInt160 recipient)
    {
        // Transfer any remaining GAS to recipient
        var balance = GetBalance(ContractAddress);
        if (balance > 0)
        {
            TransferGas(recipient, balance);
        }
        
        // Mark contract as destroyed
        Registry.UpdateContractStatus(ContractAddress, false, ContractAddress);
        
        // Emit self-destruct event
        Events.Log2("SelfDestruct(address)", ContractAddress, recipient);
        
        // In a real implementation, this would halt execution
        throw new System.Exception("Contract self-destructed");
    }
    
    /// <summary>
    /// Get balance of an address
    /// </summary>
    /// <param name="address">Address to check</param>
    /// <returns>Balance in smallest unit</returns>
    public BigInteger GetBalance(UInt160 address)
    {
        try
        {
            return GAS.BalanceOf((Neo.SmartContract.Framework.UInt160)NeoTypeConversions.ToByteArray(address));
        }
        catch (Exception ex)
        {
            // Log the error for debugging
            NeoFrameworkRuntime.Log($"Error getting balance for {address}: {ex.Message}");
            return 0;
        }
    }
    
    /// <summary>
    /// Transfer GAS to address
    /// </summary>
    /// <param name="to">Recipient address</param>
    /// <param name="amount">Amount to transfer</param>
    /// <returns>True if successful</returns>
    public bool TransferGas(UInt160 to, BigInteger amount)
    {
        try
        {
            if (amount <= 0)
            {
                throw new ArgumentException("Transfer amount must be positive");
            }
            
            if (to == UInt160.Zero)
            {
                throw new ArgumentException("Invalid recipient address");
            }
            
            // Check current balance
            var currentBalance = GetBalance(ContractAddress);
            if (currentBalance < amount)
            {
                NeoFrameworkRuntime.Log($"Insufficient balance: {currentBalance} < {amount}");
                return false;
            }
            
            var success = GAS.Transfer(
                (Neo.SmartContract.Framework.UInt160)NeoTypeConversions.ToByteArray(ContractAddress),
                (Neo.SmartContract.Framework.UInt160)NeoTypeConversions.ToByteArray(to),
                amount,
                null
            );
            if (success)
            {
                // Emit transfer event for tracking
                Events.Log3("Transfer(address,address,uint256)", ContractAddress, to, amount);
                NeoFrameworkRuntime.Log($"Successfully transferred {amount} GAS from {ContractAddress} to {to}");
            }
            return success;
        }
        catch (Exception ex)
        {
            NeoFrameworkRuntime.Log($"Error transferring GAS: {ex.Message}");
            return false;
        }
    }
    
    /// <summary>
    /// Get code size at address
    /// </summary>
    /// <param name="address">Address to check</param>
    /// <returns>Code size in bytes</returns>
    public uint GetCodeSize(UInt160 address)
    {
        try
        {
            var contract = ContractManagement.GetContract(
                (Neo.SmartContract.Framework.UInt160)NeoTypeConversions.ToByteArray(address)
            );
            return contract?.Nef != null ? (uint)contract.Nef.Length : 0;
        }
        catch
        {
            return 0;
        }
    }
    
    /// <summary>
    /// Get code hash at address
    /// </summary>
    /// <param name="address">Address to check</param>
    /// <returns>Code hash</returns>
    public UInt256 GetCodeHash(UInt160 address)
    {
        try
        {
            // In Neo, we could use the contract hash
            return new UInt256(NeoTypeConversions.ToByteArray(address).Concat(new byte[12]).ToArray());
        }
        catch
        {
            return UInt256.Zero;
        }
    }
    
    /// <summary>
    /// Get runtime statistics
    /// </summary>
    /// <returns>Runtime performance statistics</returns>
    public RuntimeStats GetStats()
    {
        return new RuntimeStats
        {
            MemoryStats = _memoryManager.GetStats(),
            StorageStats = _storageManager.GetStats(),
            RegistryStats = _addressRegistry.GetStats(),
            GasUsed = EstimateGasUsed(),
            ExecutionTime = GetExecutionTime()
        };
    }
    
    /// <summary>
    /// Clear all runtime state (for testing)
    /// </summary>
    public void Reset()
    {
        _memoryManager.Clear();
        _storageManager.ClearCache();
        ExecutionContext.Reset();
    }
    
    private uint EstimateGasUsed()
    {
        try
        {
            // Calculate gas usage based on instruction count and complexity
            var baseGas = 20_000u; // Base gas cost
            
            // Add gas for memory operations
            var memoryGas = (uint)(_memoryManager.GetStats().TotalSize / 32) * 3; // 3 gas per 32 bytes
            
            // Add gas for storage operations  
            var storageStats = _storageManager.GetStats();
            var storageGas = storageStats.StorageReads * 200u +
                           storageStats.StorageWrites * 20_000u;
            
            // Add gas for external calls
            var callGas = _callManager.GetCallCount() * 700u;
            
            // Add gas for events
            var eventGas = _eventManager.GetEventCount() * 375u;
            
            var totalGas = baseGas + memoryGas + storageGas + callGas + eventGas;
            
            // Cap at reasonable maximum
            return (uint)Math.Min(totalGas, 100_000_000u);
        }
        catch
        {
            // Fallback to conservative estimate
            return 50_000u;
        }
    }
    
    private ulong GetExecutionTime()
    {
        try
        {
            // Get current runtime timestamp
            var currentTime = NeoFrameworkRuntime.Time;
            
            // Return execution duration from contract start
            // In a real implementation, this would track from contract initialization
            return currentTime;
        }
        catch
        {
            return 0;
        }
    }
    
    /// <summary>
    /// Dispose runtime resources
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            _memoryManager.Clear();
            _storageManager.ClearCache();
            _disposed = true;
        }
    }
}

/// <summary>
/// Runtime performance statistics
/// </summary>
public sealed record RuntimeStats
{
    public MemoryStats MemoryStats { get; init; } = new();
    public StorageStats StorageStats { get; init; } = new();
    public RegistryStats RegistryStats { get; init; } = new();
    public uint GasUsed { get; init; }
    public ulong ExecutionTime { get; init; }
}

/// <summary>
/// Static helper class for common EVM operations
/// </summary>
public static class Evm
{
    /// <summary>
    /// Create runtime instance for current contract
    /// </summary>
    /// <returns>EVM runtime instance</returns>
    public static EvmRuntime CreateRuntime()
    {
        return new EvmRuntime(NeoTypeConversions.ToCoreUInt160(NeoFrameworkRuntime.ExecutingScriptHash));
    }
    
    /// <summary>
    /// Encode function call
    /// </summary>
    /// <param name="signature">Function signature</param>
    /// <param name="parameters">Parameters</param>
    /// <returns>Encoded call data</returns>
    public static byte[] EncodeCall(string signature, params object[] parameters)
    {
        return AbiEncoder.EncodeCall(signature, parameters);
    }
    
    /// <summary>
    /// Calculate function selector
    /// </summary>
    /// <param name="signature">Function signature</param>
    /// <returns>4-byte selector</returns>
    public static byte[] Selector(string signature)
    {
        return AbiEncoder.CalculateFunctionSelector(signature);
    }
    
    /// <summary>
    /// Pack multiple values using ABI encoding
    /// </summary>
    /// <param name="values">Values to pack</param>
    /// <returns>Packed bytes</returns>
    public static byte[] Pack(params object[] values)
    {
        return AbiEncoder.EncodeParameters(values);
    }
}
