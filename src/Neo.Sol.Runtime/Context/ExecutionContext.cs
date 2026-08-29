using System.Numerics;
using Neo.Sol.Runtime;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using NeoLedger = Neo.SmartContract.Framework.Native.Ledger;
using NeoToken = Neo.SmartContract.Framework.Native.NEO;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;

namespace Neo.Sol.Runtime.Context;

/// <summary>
/// EVM-compatible execution context providing msg, tx, and block information
/// Maps Neo blockchain context to Ethereum semantics
/// </summary>
public sealed class ExecutionContext
{
    private static ExecutionContext? _current;
    private readonly Lazy<MsgContext> _msg;
    private readonly Lazy<TxContext> _tx;
    private readonly Lazy<BlockContext> _block;
    
    private ExecutionContext()
    {
        _msg = new Lazy<MsgContext>(() => new MsgContext());
        _tx = new Lazy<TxContext>(() => new TxContext());
        _block = new Lazy<BlockContext>(() => new BlockContext());
    }
    
    /// <summary>
    /// Get current execution context (singleton)
    /// </summary>
    public static ExecutionContext Current => _current ??= new ExecutionContext();
    
    /// <summary>
    /// Message context (msg.*)
    /// </summary>
    public MsgContext Msg => _msg.Value;
    
    /// <summary>
    /// Transaction context (tx.*)
    /// </summary>
    public TxContext Tx => _tx.Value;
    
    /// <summary>
    /// Block context (block.*)
    /// </summary>
    public BlockContext Block => _block.Value;
    
    /// <summary>
    /// Reset context (for testing)
    /// </summary>
    internal static void Reset()
    {
        _current = null;
    }
}

/// <summary>
/// Message context (msg.*) providing information about the current call
/// </summary>
public sealed class MsgContext
{
    private UInt160? _sender;
    private BigInteger? _value;
    private byte[]? _data;
    private uint? _gasLimit;
    
    /// <summary>
    /// Address of the account that initiated the transaction (msg.sender)
    /// Maps to Neo's Tx.Sender or calling contract in internal calls
    /// </summary>
    public UInt160 Sender
    {
        get
        {
            if (_sender == null)
            {
                try
                {
                    _sender = NeoTypeConversions.ToCoreUInt160(NeoFrameworkRuntime.CallingScriptHash);
                }
                catch
                {
                    try
                    {
                        _sender = NeoTypeConversions.ToCoreUInt160(NeoFrameworkRuntime.Transaction.Sender);
                    }
                    catch
                    {
                        _sender = UInt160.Zero;
                    }
                }
            }
            return _sender;
        }
        internal set => _sender = value;
    }
    
    /// <summary>
    /// Value sent with the message (msg.value)
    /// In Neo, this would be the GAS transferred with the transaction
    /// </summary>
    public BigInteger Value
    {
        get
        {
            if (_value == null)
            {
                // Neo doesn't have direct equivalent, but we can simulate it
                // This would need to be set by the calling contract or runtime
                _value = 0; // Default to 0 for now
            }
            return _value.Value;
        }
        internal set => _value = value;
    }
    
    /// <summary>
    /// Complete call data payload (msg.data)
    /// </summary>
    public byte[] Data
    {
        get
        {
            if (_data == null)
            {
                // In Neo, this would be the method arguments or raw invocation data
                // For now, return empty array as it depends on the calling contract
                _data = Array.Empty<byte>();
            }
            return _data;
        }
        internal set => _data = value;
    }
    
    /// <summary>
    /// Gas limit for the current call (msg.gas)
    /// Maps to Neo's remaining GAS
    /// </summary>
    public uint Gas
    {
        get
        {
            if (_gasLimit == null)
            {
                // Neo tracks GAS differently, but we can approximate
                try
                {
                    // This would need to be implemented based on Neo's gas tracking
                    _gasLimit = (uint)NeoFrameworkRuntime.GasLeft;
                }
                catch
                {
                    _gasLimit = 1000000; // Default fallback
                }
            }
            return _gasLimit.Value;
        }
        internal set => _gasLimit = value;
    }
    
    /// <summary>
    /// Function selector from msg.data (first 4 bytes)
    /// </summary>
    public byte[] Sig => Data.Length >= 4 ? Data[..4] : Array.Empty<byte>();
}

/// <summary>
/// Transaction context (tx.*) providing information about the current transaction
/// </summary>
public sealed class TxContext
{
    private UInt160? _origin;
    private BigInteger? _gasPrice;
    
    /// <summary>
    /// Transaction origin (tx.origin)
    /// Address that started the transaction chain
    /// </summary>
    public UInt160 Origin
    {
        get
        {
            if (_origin == null)
            {
                try
                {
                    _origin = NeoTypeConversions.ToCoreUInt160(NeoFrameworkRuntime.Transaction.Sender);
                }
                catch
                {
                    _origin = UInt160.Zero;
                }
            }
            return _origin;
        }
        internal set => _origin = value;
    }
    
    /// <summary>
    /// Gas price of the transaction (tx.gasprice)
    /// In Neo, this maps to the network fee per byte
    /// </summary>
    public BigInteger GasPrice
    {
        get
        {
            if (_gasPrice == null)
            {
                // Calculate approximate gas price from Neo transaction
                try
                {
                    var tx = NeoFrameworkRuntime.Transaction;
                    var networkFee = tx.NetworkFee;
                    var txSize = tx.Script?.Length ?? 0;
                    _gasPrice = txSize > 0 ? networkFee / txSize : 0;
                }
                catch
                {
                    _gasPrice = 1000000; // Default GAS price in Neo units
                }
            }
            return _gasPrice.Value;
        }
        internal set => _gasPrice = value;
    }
    
    /// <summary>
    /// Transaction hash
    /// </summary>
    public UInt256 Hash
    {
        get
        {
            try
            {
                return NeoTypeConversions.ToCoreUInt256(NeoFrameworkRuntime.Transaction.Hash);
            }
            catch
            {
                return UInt256.Zero;
            }
        }
    }
    
    /// <summary>
    /// Transaction nonce (sequence number)
    /// In Neo, this maps to the transaction nonce
    /// </summary>
    public BigInteger Nonce
    {
        get
        {
            try
            {
                return NeoFrameworkRuntime.Transaction.Nonce;
            }
            catch
            {
                return BigInteger.Zero;
            }
        }
    }
}

/// <summary>
/// Block context (block.*) providing information about the current block
/// </summary>
public sealed class BlockContext
{
    private UInt160? _coinbase;
    private BigInteger? _difficulty;
    private BigInteger? _gasLimit;
    private BigInteger? _baseFee;
    
    /// <summary>
    /// Block miner address (block.coinbase)
    /// In Neo, this would be the primary consensus node
    /// </summary>
    public UInt160 Coinbase
    {
        get
        {
            if (_coinbase == null)
            {
                // Get actual consensus node from Neo blockchain
                try
                {
                    _coinbase = NeoTypeConversions.ToCoreUInt160(NeoToken.GetCommitteeAddress());
                }
                catch
                {
                    _coinbase = UInt160.Zero; // Safe fallback
                }
            }
            return _coinbase;
        }
        internal set => _coinbase = value;
    }
    
    /// <summary>
    /// Block difficulty (block.difficulty)
    /// Not directly applicable to Neo's dBFT consensus
    /// </summary>
    public BigInteger Difficulty
    {
        get
        {
            if (_difficulty == null)
            {
                // Neo uses dBFT, not PoW, so difficulty is not applicable
                // Return a constant value for compatibility
                _difficulty = 1;
            }
            return _difficulty.Value;
        }
        internal set => _difficulty = value;
    }
    
    /// <summary>
    /// Block gas limit (block.gaslimit)
    /// Maps to Neo's maximum gas per block
    /// </summary>
    public BigInteger GasLimit
    {
        get
        {
            if (_gasLimit == null)
            {
                // Neo has a gas limit per transaction and per block
                _gasLimit = 100000000; // Default Neo gas limit
            }
            return _gasLimit.Value;
        }
        internal set => _gasLimit = value;
    }
    
    /// <summary>
    /// Block number (block.number)
    /// Maps to Neo's block index
    /// </summary>
    public uint Number
    {
        get
        {
            try
            {
                return NeoLedger.CurrentIndex;
            }
            catch
            {
                return 0;
            }
        }
    }
    
    /// <summary>
    /// Block timestamp (block.timestamp)
    /// Unix timestamp of the current block
    /// </summary>
    public ulong Timestamp
    {
        get
        {
            try
            {
                return NeoFrameworkRuntime.Time;
            }
            catch
            {
                return 0;
            }
        }
    }
    
    /// <summary>
    /// Block hash of the current block
    /// </summary>
    public UInt256 Hash
    {
        get
        {
            try
            {
                return NeoTypeConversions.ToCoreUInt256(NeoLedger.CurrentHash);
            }
            catch
            {
                return UInt256.Zero;
            }
        }
    }
    
    /// <summary>
    /// Block base fee per gas (EIP-1559)
    /// Not directly applicable to Neo
    /// </summary>
    public BigInteger BaseFee
    {
        get
        {
            if (_baseFee == null)
            {
                // Neo doesn't have base fee mechanism
                // Return approximate network fee rate
                _baseFee = 1000; // Default base fee
            }
            return _baseFee.Value;
        }
        internal set => _baseFee = value;
    }
    
    /// <summary>
    /// Get block hash by number
    /// </summary>
    /// <param name="blockNumber">Block number</param>
    /// <returns>Block hash</returns>
    public UInt256 GetBlockHash(uint blockNumber)
    {
        if (blockNumber >= Number)
            return UInt256.Zero;
            
        try
        {
            var block = NeoLedger.GetBlock(blockNumber);
            return block != null ? NeoTypeConversions.ToCoreUInt256(block.Hash) : UInt256.Zero;
        }
        catch
        {
            return UInt256.Zero;
        }
    }
    
    /// <summary>
    /// Check if block number is within the last 256 blocks (EVM blockhash limit)
    /// </summary>
    /// <param name="blockNumber">Block number to check</param>
    /// <returns>True if block is within range</returns>
    public bool IsBlockHashAvailable(uint blockNumber)
    {
        return blockNumber < Number && (Number - blockNumber) <= 256;
    }
}

/// <summary>
/// Gas context providing gas-related utilities
/// </summary>
public sealed class GasContext
{
    /// <summary>
    /// Get remaining gas in current execution
    /// </summary>
    /// <returns>Remaining gas</returns>
    public static long GetGasLeft()
    {
        try
        {
            return NeoFrameworkRuntime.GasLeft;
        }
        catch
        {
            return 0;
        }
    }
    
    /// <summary>
    /// Consume gas for operation
    /// </summary>
    /// <param name="amount">Gas amount to consume</param>
    /// <returns>True if gas was successfully consumed</returns>
    public static bool ConsumeGas(long amount)
    {
        var remaining = GetGasLeft();
        return remaining >= amount;
    }
    
    /// <summary>
    /// Calculate gas cost for memory expansion
    /// </summary>
    /// <param name="currentSize">Current memory size</param>
    /// <param name="newSize">New memory size</param>
    /// <returns>Gas cost for expansion</returns>
    public static ulong CalculateMemoryCost(uint currentSize, uint newSize)
    {
        if (newSize <= currentSize) return 0;
        
        var currentWords = (currentSize + 31) / 32;
        var newWords = (newSize + 31) / 32;
        
        // EVM memory cost formula
        var currentCost = currentWords * 3 + (currentWords * currentWords) / 512;
        var newCost = newWords * 3 + (newWords * newWords) / 512;
        
        return newCost - currentCost;
    }
}
