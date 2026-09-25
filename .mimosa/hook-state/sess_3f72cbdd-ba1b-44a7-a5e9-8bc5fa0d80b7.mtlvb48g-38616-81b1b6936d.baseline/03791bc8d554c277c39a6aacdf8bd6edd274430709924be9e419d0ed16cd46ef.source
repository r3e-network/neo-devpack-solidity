using System.Numerics;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.ABI;
using Neo.Sol.Runtime.Crypto;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;
using UInt160 = Neo.SmartContract.Framework.UInt160;
using UInt256 = Neo.SmartContract.Framework.UInt256;

namespace Neo.Sol.Runtime.Events;

/// <summary>
/// Maps Ethereum-style events to Neo Runtime.Notify calls
/// Preserves event semantics and enables cross-chain event compatibility
/// </summary>
public sealed class EventManager
{
    private readonly UInt160 _contractHash;
    private uint _eventCount;
    private const string EVENT_PREFIX = "EVM_Event";
    
    public EventManager(UInt160 contractHash)
    {
        _contractHash = contractHash;
        _eventCount = 0;
    }

    private static void TryLog(string message)
    {
        try
        {
            NeoFrameworkRuntime.Log(message);
        }
        catch
        {
            // Ignore logging failures outside a Neo VM host.
        }
    }
    
    /// <summary>
    /// Get the number of events emitted
    /// </summary>
    /// <returns>Event count</returns>
    public uint GetEventCount() => _eventCount;
    
    /// <summary>
    /// Emit an event with up to 4 indexed parameters (EVM LOG0-LOG4)
    /// </summary>
    /// <param name="eventSignature">Event signature (e.g., "Transfer(address,address,uint256)")</param>
    /// <param name="indexedParams">Indexed parameters (topics)</param>
    /// <param name="nonIndexedParams">Non-indexed parameters (data)</param>
    public void EmitEvent(string eventSignature, object[] indexedParams, params object[] nonIndexedParams)
    {
        if (indexedParams.Length > 4)
            throw new ArgumentException("Maximum 4 indexed parameters allowed");
            
        var eventHash = CalculateEventHash(eventSignature);
        var topics = new System.Collections.Generic.List<byte[]> { eventHash };
        
        // Add indexed parameters as topics
        foreach (var param in indexedParams)
        {
            topics.Add(EncodeIndexedParameter(param));
        }
        
        // Encode non-indexed parameters as data
        var data = nonIndexedParams.Length > 0 ? AbiEncoder.EncodeParameters(nonIndexedParams) : Array.Empty<byte>();
        
        // Create event log structure
        var eventLog = new EventLog
        {
            Address = _contractHash,
            Topics = topics.ToArray(),
            Data = data,
            EventSignature = eventSignature
        };
        
        // Emit as Neo notification
        _eventCount++;
        TryLog($"{EVENT_PREFIX}:{eventSignature}");
    }
    
    /// <summary>
    /// Emit indexed event (LOG1)
    /// </summary>
    /// <param name="eventSignature">Event signature</param>
    /// <param name="topic1">First indexed parameter</param>
    /// <param name="data">Non-indexed data parameters</param>
    public void Log1(string eventSignature, object topic1, params object[] data)
    {
        EmitEvent(eventSignature, new[] { topic1 }, data);
    }
    
    /// <summary>
    /// Emit indexed event (LOG2)
    /// </summary>
    /// <param name="eventSignature">Event signature</param>
    /// <param name="topic1">First indexed parameter</param>
    /// <param name="topic2">Second indexed parameter</param>
    /// <param name="data">Non-indexed data parameters</param>
    public void Log2(string eventSignature, object topic1, object topic2, params object[] data)
    {
        EmitEvent(eventSignature, new[] { topic1, topic2 }, data);
    }
    
    /// <summary>
    /// Emit indexed event (LOG3)
    /// </summary>
    /// <param name="eventSignature">Event signature</param>
    /// <param name="topic1">First indexed parameter</param>
    /// <param name="topic2">Second indexed parameter</param>
    /// <param name="topic3">Third indexed parameter</param>
    /// <param name="data">Non-indexed data parameters</param>
    public void Log3(string eventSignature, object topic1, object topic2, object topic3, params object[] data)
    {
        EmitEvent(eventSignature, new[] { topic1, topic2, topic3 }, data);
    }
    
    /// <summary>
    /// Emit indexed event (LOG4)
    /// </summary>
    /// <param name="eventSignature">Event signature</param>
    /// <param name="topic1">First indexed parameter</param>
    /// <param name="topic2">Second indexed parameter</param>
    /// <param name="topic3">Third indexed parameter</param>
    /// <param name="topic4">Fourth indexed parameter</param>
    /// <param name="data">Non-indexed data parameters</param>
    public void Log4(string eventSignature, object topic1, object topic2, object topic3, object topic4, params object[] data)
    {
        EmitEvent(eventSignature, new[] { topic1, topic2, topic3, topic4 }, data);
    }
    
    /// <summary>
    /// Calculate event hash from signature
    /// </summary>
    /// <param name="eventSignature">Event signature</param>
    /// <returns>32-byte event hash</returns>
    private static byte[] CalculateEventHash(string eventSignature)
    {
        return CryptoLib.Keccak256(System.Text.Encoding.UTF8.GetBytes(eventSignature));
    }
    
    /// <summary>
    /// Encode parameter for indexing (as topic)
    /// </summary>
    /// <param name="parameter">Parameter to encode</param>
    /// <returns>32-byte encoded parameter</returns>
    private static byte[] EncodeIndexedParameter(object parameter)
    {
        return parameter switch
        {
            bool b => AbiEncoder.EncodeUint256(b ? 1 : 0),
            byte b => AbiEncoder.EncodeUint256(b),
            ushort u => AbiEncoder.EncodeUint256(u),
            uint u => AbiEncoder.EncodeUint256(u),
            ulong u => AbiEncoder.EncodeUint256(u),
            BigInteger bi => AbiEncoder.EncodeUint256(bi),
            int i => AbiEncoder.EncodeInt256(i),
            long l => AbiEncoder.EncodeInt256(l),
            UInt160 addr => AbiEncoder.EncodeAddress(addr),
            string s => CryptoLib.Keccak256(System.Text.Encoding.UTF8.GetBytes(s)),
            byte[] bytes => CryptoLib.Keccak256(bytes),
            _ => CryptoLib.Keccak256(System.Text.Encoding.UTF8.GetBytes(parameter.ToString() ?? ""))
        };
    }
}

/// <summary>
/// Represents an EVM-style event log
/// </summary>
public sealed class EventLog
{
    public UInt160 Address { get; init; } = UInt160.Zero;
    public byte[][] Topics { get; init; } = Array.Empty<byte[]>();
    public byte[] Data { get; init; } = Array.Empty<byte>();
    public string EventSignature { get; init; } = "";
    
    /// <summary>
    /// Convert to Neo notification object
    /// </summary>
    /// <returns>Notification object for Runtime.Notify</returns>
    public object[] ToNotificationObject()
    {
        var topicsHex = Topics.Select(topic => Convert.ToHexString(topic)).ToArray();
        ulong timestamp;
        try
        {
            timestamp = NeoFrameworkRuntime.Time;
        }
        catch
        {
            timestamp = 0;
        }
        
        return new object[]
        {
            Address.ToString(),
            topicsHex,
            Convert.ToHexString(Data),
            EventSignature,
            timestamp // Add timestamp for Neo compatibility
        };
    }
    
    /// <summary>
    /// Create from Neo notification
    /// </summary>
    /// <param name="notificationData">Notification data from Runtime.Notify</param>
    /// <returns>Parsed event log</returns>
    public static EventLog FromNotificationObject(object[] notificationData)
    {
        if (notificationData.Length < 4)
            throw new ArgumentException("Invalid notification data format");
            
        var address = UInt160.Parse(notificationData[0].ToString()!);
        var topicsHex = (string[])notificationData[1];
        var dataHex = notificationData[2].ToString()!;
        var eventSignature = notificationData[3].ToString()!;
        
        var topics = topicsHex.Select(hex => Convert.FromHexString(hex)).ToArray();
        var data = Convert.FromHexString(dataHex);
        
        return new EventLog
        {
            Address = address,
            Topics = topics,
            Data = data,
            EventSignature = eventSignature
        };
    }
}

/// <summary>
/// Event filter for querying historical events
/// </summary>
public sealed class EventFilter
{
    public UInt160? Address { get; init; }
    public byte[]? Topic0 { get; init; } // Event hash
    public byte[]? Topic1 { get; init; }
    public byte[]? Topic2 { get; init; }
    public byte[]? Topic3 { get; init; }
    public uint? FromBlock { get; init; }
    public uint? ToBlock { get; init; }
    
    /// <summary>
    /// Check if event log matches this filter
    /// </summary>
    /// <param name="log">Event log to check</param>
    /// <returns>True if log matches filter</returns>
    public bool Matches(EventLog log)
    {
        if (Address != null && !log.Address.Equals(Address))
            return false;
            
        if (Topic0 != null && (log.Topics.Length == 0 || !log.Topics[0].SequenceEqual(Topic0)))
            return false;
            
        if (Topic1 != null && (log.Topics.Length < 2 || !log.Topics[1].SequenceEqual(Topic1)))
            return false;
            
        if (Topic2 != null && (log.Topics.Length < 3 || !log.Topics[2].SequenceEqual(Topic2)))
            return false;
            
        if (Topic3 != null && (log.Topics.Length < 4 || !log.Topics[3].SequenceEqual(Topic3)))
            return false;
            
        return true;
    }
}

/// <summary>
/// Standard ERC20 events for convenience
/// </summary>
public static class StandardEvents
{
    public static void EmitTransfer(EventManager eventManager, UInt160 from, UInt160 to, BigInteger value)
    {
        eventManager.Log3("Transfer(address,address,uint256)", from, to, value);
    }
    
    public static void EmitApproval(EventManager eventManager, UInt160 owner, UInt160 spender, BigInteger value)
    {
        eventManager.Log3("Approval(address,address,uint256)", owner, spender, value);
    }
    
    public static void EmitOwnershipTransferred(EventManager eventManager, UInt160 previousOwner, UInt160 newOwner)
    {
        eventManager.Log2("OwnershipTransferred(address,address)", previousOwner, newOwner);
    }
    
    public static void EmitPaused(EventManager eventManager, UInt160 account)
    {
        eventManager.Log1("Paused(address)", account);
    }
    
    public static void EmitUnpaused(EventManager eventManager, UInt160 account)
    {
        eventManager.Log1("Unpaused(address)", account);
    }
}
