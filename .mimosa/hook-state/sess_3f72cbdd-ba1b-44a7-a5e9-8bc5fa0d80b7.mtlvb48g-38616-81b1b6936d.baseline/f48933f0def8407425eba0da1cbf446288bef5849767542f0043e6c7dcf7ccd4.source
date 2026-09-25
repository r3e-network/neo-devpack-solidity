using System.Numerics;
using System.Text;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.ABI;
using Neo.Sol.Runtime.Context;
using Neo.Sol.Runtime.Crypto;
using ExecutionContext = Neo.Sol.Runtime.Context.ExecutionContext;
using NeoFrameworkCallFlags = Neo.SmartContract.Framework.Services.CallFlags;
using NeoFrameworkContract = Neo.SmartContract.Framework.Services.Contract;
using NeoFrameworkContractManagement = Neo.SmartContract.Framework.Native.ContractManagement;
using NeoFrameworkGas = Neo.SmartContract.Framework.Native.GAS;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;

namespace Neo.Sol.Runtime.Calls;

/// <summary>
/// Manages external contract calls with EVM-compatible semantics
/// Provides call, delegatecall, and staticcall functionality
/// </summary>
public sealed class ExternalCallManager
{
    private readonly ExecutionContext _context;
    private uint _callCount;
    
    public ExternalCallManager(ExecutionContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
        _callCount = 0;
    }
    
    /// <summary>
    /// Get the number of external calls made
    /// </summary>
    /// <returns>Call count</returns>
    public uint GetCallCount() => _callCount;
    
    /// <summary>
    /// Perform external contract call (EVM CALL opcode equivalent)
    /// </summary>
    /// <param name="target">Target contract address</param>
    /// <param name="value">Value to transfer (in GAS)</param>
    /// <param name="gasLimit">Gas limit for the call</param>
    /// <param name="callData">Call data payload</param>
    /// <returns>Call result</returns>
    public CallResult Call(UInt160 target, BigInteger value, uint gasLimit, byte[] callData)
    {
        return ExecuteCall(target, value, gasLimit, callData, CallType.Call);
    }
    
    /// <summary>
    /// Perform delegate call (EVM DELEGATECALL opcode equivalent)
    /// </summary>
    /// <param name="target">Target contract address</param>
    /// <param name="gasLimit">Gas limit for the call</param>
    /// <param name="callData">Call data payload</param>
    /// <returns>Call result</returns>
    public CallResult DelegateCall(UInt160 target, uint gasLimit, byte[] callData)
    {
        return ExecuteCall(target, 0, gasLimit, callData, CallType.DelegateCall);
    }
    
    /// <summary>
    /// Perform static call (EVM STATICCALL opcode equivalent)
    /// </summary>
    /// <param name="target">Target contract address</param>
    /// <param name="gasLimit">Gas limit for the call</param>
    /// <param name="callData">Call data payload</param>
    /// <returns>Call result</returns>
    public CallResult StaticCall(UInt160 target, uint gasLimit, byte[] callData)
    {
        return ExecuteCall(target, 0, gasLimit, callData, CallType.StaticCall);
    }
    
    /// <summary>
    /// Create new contract (EVM CREATE opcode equivalent)
    /// </summary>
    /// <param name="value">Value to transfer to new contract</param>
    /// <param name="initCode">Contract initialization code</param>
    /// <param name="gasLimit">Gas limit for deployment</param>
    /// <returns>Deployment result with new contract address</returns>
    public CreateResult Create(BigInteger value, byte[] initCode, uint gasLimit)
    {
        _ = value;
        _ = initCode;
        _ = gasLimit;
        return CreateResult.Failed(
            "Contract creation is not supported by the optional Neo.Sol.Runtime shim"
        );
    }
    
    /// <summary>
    /// Create new contract with deterministic address (EVM CREATE2 opcode equivalent)
    /// </summary>
    /// <param name="value">Value to transfer to new contract</param>
    /// <param name="initCode">Contract initialization code</param>
    /// <param name="salt">Salt for address calculation</param>
    /// <param name="gasLimit">Gas limit for deployment</param>
    /// <returns>Deployment result with new contract address</returns>
    public CreateResult Create2(BigInteger value, byte[] initCode, byte[] salt, uint gasLimit)
    {
        _ = value;
        _ = initCode;
        _ = salt;
        _ = gasLimit;
        return CreateResult.Failed(
            "CREATE2 is not supported by the optional Neo.Sol.Runtime shim"
        );
    }
    
    /// <summary>
    /// Execute a contract call
    /// </summary>
    /// <param name="target">Target contract</param>
    /// <param name="callData">Call data</param>
    /// <param name="gasLimit">Gas limit</param>
    /// <param name="callType">Type of call</param>
    /// <returns>Call result</returns>
    private CallResult ExecuteCall(
        UInt160 target,
        BigInteger value,
        uint gasLimit,
        byte[] callData,
        CallType callType
    )
    {
        var originalSender = _context.Msg.Sender;
        var originalValue = _context.Msg.Value;
        var originalData = _context.Msg.Data;

        try
        {
            if (!IsContractDeployed(target))
            {
                return CallResult.Failed("Target contract not deployed");
            }

            if (callData.Length < 4)
            {
                return CallResult.Failed("Invalid call data: too short");
            }

            if (callType == CallType.StaticCall && value > 0)
            {
                return CallResult.Failed("Static calls cannot transfer value");
            }

            if (callType == CallType.Call && value > 0 && !TransferGas(_context.Msg.Sender, target, value))
            {
                return CallResult.Failed("Value transfer failed");
            }

            var selector = callData[..4];
            var parameters = callData.Length > 4 ? callData[4..] : Array.Empty<byte>();
            var methodName = GetMethodNameFromSelector(selector);
            object[] args = parameters.Length > 0
                ? new object[] { parameters }
                : Array.Empty<object>();

            var frameworkTarget = ToFrameworkAddress(target);
            var flags = callType switch
            {
                CallType.Call => NeoFrameworkCallFlags.All,
                CallType.DelegateCall => NeoFrameworkCallFlags.ReadStates | NeoFrameworkCallFlags.WriteStates,
                CallType.StaticCall => NeoFrameworkCallFlags.ReadOnly,
                _ => NeoFrameworkCallFlags.All,
            };

            _context.Msg.Sender = NeoTypeConversions.ToCoreUInt160(NeoFrameworkRuntime.ExecutingScriptHash);
            _context.Msg.Value = value;
            _context.Msg.Data = callData;

            var result = callType switch
            {
                CallType.Call => NeoFrameworkContract.Call(frameworkTarget, methodName, flags, args),
                CallType.DelegateCall => NeoFrameworkContract.Call(frameworkTarget, methodName, flags, args),
                CallType.StaticCall => NeoFrameworkContract.Call(frameworkTarget, methodName, flags, args),
                _ => throw new ArgumentException($"Unsupported call type: {callType}")
            };

            _callCount++;
            var returnData = result != null ? SerializeResult(result) : Array.Empty<byte>();
            return CallResult.Succeeded(returnData, EstimateGasUsed(callData.Length));
        }
        catch (Exception ex)
        {
            return CallResult.Failed($"Execution failed: {ex.Message}");
        }
        finally
        {
            _context.Msg.Sender = originalSender;
            _context.Msg.Value = originalValue;
            _context.Msg.Data = originalData;
        }
    }

    private static Neo.SmartContract.Framework.UInt160 ToFrameworkAddress(UInt160 address)
        => (Neo.SmartContract.Framework.UInt160)NeoTypeConversions.ToByteArray(address);

    private static bool IsContractDeployed(UInt160 address)
    {
        try
        {
            return NeoFrameworkContractManagement.GetContract(ToFrameworkAddress(address)) != null;
        }
        catch
        {
            return false;
        }
    }
    
    /// <summary>
    /// Transfer GAS between accounts
    /// </summary>
    /// <param name="from">From address</param>
    /// <param name="to">To address</param>
    /// <param name="amount">Amount to transfer</param>
    /// <returns>True if successful</returns>
    private bool TransferGas(UInt160 from, UInt160 to, BigInteger amount)
    {
        if (amount <= 0 || to == UInt160.Zero)
        {
            return false;
        }

        try
        {
            return NeoFrameworkGas.Transfer(
                ToFrameworkAddress(from),
                ToFrameworkAddress(to),
                amount,
                null
            );
        }
        catch
        {
            return false;
        }
    }
    
    /// <summary>
    /// Get method name from function selector
    /// </summary>
    /// <param name="selector">4-byte function selector</param>
    /// <returns>Method name</returns>
    private string GetMethodNameFromSelector(byte[] selector)
    {
        _ = selector;
        return "invoke";
    }
    
    /// <summary>
    /// Serialize call result to bytes
    /// </summary>
    /// <param name="result">Call result</param>
    /// <returns>Serialized bytes</returns>
    private byte[] SerializeResult(object result)
    {
        // Simplified serialization - actual implementation would need
        // to handle various Neo types and convert to ABI encoding
        if (result is byte[] bytes)
            return bytes;
        if (result is ByteString byteString)
            return (byte[])byteString;
        if (result is string str)
            return Encoding.UTF8.GetBytes(str);
        if (result is BigInteger bi)
            return AbiEncoder.EncodeUint256(bi);
        if (result is bool boolean)
            return AbiEncoder.EncodeUint256(boolean ? 1 : 0);
            
        return Encoding.UTF8.GetBytes(result.ToString() ?? string.Empty);
    }
    
    /// <summary>
    /// Estimate gas used for operation
    /// </summary>
    /// <param name="dataSize">Size of call data</param>
    /// <returns>Estimated gas used</returns>
    private uint EstimateGasUsed(int dataSize)
    {
        // Simplified gas estimation
        return (uint)(21000 + dataSize * 16); // Base cost + data cost
    }
}

/// <summary>
/// Type of external call
/// </summary>
public enum CallType
{
    Call,         // Regular external call
    DelegateCall, // Delegate call (runs in caller's context)
    StaticCall    // Static call (read-only)
}

/// <summary>
/// Result of an external call
/// </summary>
public sealed record CallResult
{
    public bool Success { get; init; }
    public byte[] ReturnData { get; init; } = Array.Empty<byte>();
    public string Error { get; init; } = "";
    public uint GasUsed { get; init; }
    
    public static CallResult Succeeded(byte[] returnData, uint gasUsed = 0)
        => new() { Success = true, ReturnData = returnData, GasUsed = gasUsed };
        
    public static CallResult Failed(string error, uint gasUsed = 0)
        => new() { Success = false, Error = error, GasUsed = gasUsed };
}

/// <summary>
/// Result of contract creation
/// </summary>
public sealed record CreateResult
{
    public bool Success { get; init; }
    public UInt160 Address { get; init; } = UInt160.Zero;
    public byte[] ReturnData { get; init; } = Array.Empty<byte>();
    public string Error { get; init; } = "";
    public uint GasUsed { get; init; }
    
    public static CreateResult Succeeded(UInt160 address, byte[] returnData, uint gasUsed = 0)
        => new() { Success = true, Address = address, ReturnData = returnData, GasUsed = gasUsed };
        
    public static CreateResult Failed(string error, uint gasUsed = 0)
        => new() { Success = false, Error = error, GasUsed = gasUsed };
}
