using System.Numerics;
using System.Collections.Concurrent;
using System.Diagnostics;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.Crypto;
using NeoFrameworkStorage = Neo.SmartContract.Framework.Services.Storage;

namespace Neo.Sol.Runtime.Storage;

/// <summary>
/// EVM-compatible storage manager preserving Solidity storage layout semantics
/// Maps EVM storage slots to Neo storage keys with collision resistance
/// Production-grade implementation with advanced caching, metrics, and optimization
/// </summary>
public sealed class StorageManager : IDisposable
{
    private readonly StorageContext _context;
    private readonly ConcurrentDictionary<BigInteger, CachedSlot> _cache = new();
    private readonly ConcurrentHashSet<BigInteger> _modifiedSlots = new();
    private readonly ReaderWriterLockSlim _lock = new();
    private readonly Timer _cacheCleanupTimer;
    private readonly Stopwatch _accessTimer = new();
    private bool _disposed = false;
    
    // Performance tracking
    private ulong _cacheHits = 0;
    private ulong _cacheMisses = 0;
    private ulong _storageReads = 0;
    private ulong _storageWrites = 0;
    private ulong _cacheEvictions = 0;
    
    // Storage layout constants
    private const int SLOT_SIZE = 32; // 32-byte storage slots
    private const int MAX_PACKED_SLOTS = 8; // Maximum number of variables that can be packed
    private const int MAX_CACHE_SIZE = 10000; // Maximum cached slots
    private const int CACHE_CLEANUP_INTERVAL_MS = 60000; // Cleanup every minute
    private const ulong CACHE_TTL_MS = 300000; // 5 minute TTL for cached slots
    
    public StorageManager(StorageContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
        _accessTimer.Start();
        _cacheCleanupTimer = new Timer(CleanupCache, null, CACHE_CLEANUP_INTERVAL_MS, CACHE_CLEANUP_INTERVAL_MS);
    }
    
    /// <summary>
    /// Generate Neo storage key from EVM storage slot
    /// Uses keccak256 to prevent collisions and maintain determinism
    /// </summary>
    /// <param name="slot">EVM storage slot</param>
    /// <returns>Neo storage key</returns>
    private byte[] GenerateStorageKey(BigInteger slot)
    {
        // Prefix with "evm_storage_" to separate from other Neo storage
        var prefix = "evm_storage_"u8.ToArray();
        var slotBytes = slot.ToByteArray(isUnsigned: true, isBigEndian: true);
        
        // Pad slot to 32 bytes for consistent key generation
        var paddedSlot = new byte[SLOT_SIZE];
        if (slotBytes.Length <= SLOT_SIZE)
        {
            Array.Copy(slotBytes, 0, paddedSlot, SLOT_SIZE - slotBytes.Length, slotBytes.Length);
        }
        else
        {
            Array.Copy(slotBytes, slotBytes.Length - SLOT_SIZE, paddedSlot, 0, SLOT_SIZE);
        }
        
        var combined = new byte[prefix.Length + paddedSlot.Length];
        Array.Copy(prefix, 0, combined, 0, prefix.Length);
        Array.Copy(paddedSlot, 0, combined, prefix.Length, paddedSlot.Length);
        
        // Use keccak256 for consistent, collision-resistant key generation
        return CryptoLib.Keccak256(combined);
    }
    
    /// <summary>
    /// Load value from storage slot with advanced caching
    /// </summary>
    /// <param name="slot">Storage slot number</param>
    /// <returns>32-byte value (zero if not set)</returns>
    /// <exception cref="ObjectDisposedException">Storage manager disposed</exception>
    public byte[] Load(BigInteger slot)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(StorageManager));
            
        var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
        
        // Check cache first
        if (_cache.TryGetValue(slot, out var cachedSlot))
        {
            cachedSlot.LastAccessed = currentTime;
            cachedSlot.AccessCount++;
            Interlocked.Increment(ref _cacheHits);
            return cachedSlot.Value;
        }
        
        Interlocked.Increment(ref _cacheMisses);
        Interlocked.Increment(ref _storageReads);
        
        var key = GenerateStorageKey(slot);
        var value = NeoFrameworkStorage.Get(_context, key);
        
        // EVM storage always returns 32 bytes, pad with zeros if needed
        var result = new byte[SLOT_SIZE];
        if (value != null && value.Length > 0)
        {
            var valueBytes = (byte[])value;
            if (value.Length <= SLOT_SIZE)
            {
                Array.Copy(valueBytes, 0, result, SLOT_SIZE - value.Length, value.Length);
            }
            else
            {
                Array.Copy(valueBytes, value.Length - SLOT_SIZE, result, 0, SLOT_SIZE);
            }
        }
        
        // Cache the result with metadata
        var cached = new CachedSlot
        {
            Value = result,
            LastAccessed = currentTime,
            AccessCount = 1,
            IsModified = false
        };
        
        // Implement cache eviction if necessary
        if (_cache.Count >= MAX_CACHE_SIZE)
        {
            EvictOldestCacheEntry();
        }
        
        _cache.TryAdd(slot, cached);
        return result;
    }
    
    /// <summary>
    /// Load BigInteger from storage slot
    /// </summary>
    /// <param name="slot">Storage slot number</param>
    /// <returns>BigInteger value</returns>
    public BigInteger LoadBigInteger(BigInteger slot)
    {
        var bytes = Load(slot);
        return new BigInteger(bytes, isUnsigned: true, isBigEndian: true);
    }
    
    /// <summary>
    /// Store value to storage slot with write-through caching
    /// </summary>
    /// <param name="slot">Storage slot number</param>
    /// <param name="value">32-byte value to store</param>
    /// <exception cref="ArgumentException">Invalid value size</exception>
    /// <exception cref="ObjectDisposedException">Storage manager disposed</exception>
    public void Store(BigInteger slot, byte[] value)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(StorageManager));
            
        if (value.Length != SLOT_SIZE)
            throw new ArgumentException($"Value must be exactly {SLOT_SIZE} bytes");
        
        var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
        Interlocked.Increment(ref _storageWrites);
        
        // Update cache with metadata
        var cachedSlot = new CachedSlot
        {
            Value = (byte[])value.Clone(),
            LastAccessed = currentTime,
            AccessCount = _cache.TryGetValue(slot, out var existing) ? existing.AccessCount + 1 : 1,
            IsModified = true
        };
        
        _cache.AddOrUpdate(slot, cachedSlot, (_, _) => cachedSlot);
        _modifiedSlots.Add(slot);
        
        // Store to Neo storage immediately (write-through)
        var key = GenerateStorageKey(slot);
        
        // Optimize storage: don't store zero values
        if (IsZero(value))
        {
            NeoFrameworkStorage.Delete(_context, key);
        }
        else
        {
            // Compress storage if beneficial
            var compressedValue = CompressValueIfBeneficial(value);
            NeoFrameworkStorage.Put(_context, key, compressedValue);
        }
    }
    
    /// <summary>
    /// Store BigInteger to storage slot
    /// </summary>
    /// <param name="slot">Storage slot number</param>
    /// <param name="value">BigInteger value</param>
    public void Store(BigInteger slot, BigInteger value)
    {
        var bytes = value.ToByteArray(isUnsigned: true, isBigEndian: true);
        var paddedBytes = new byte[SLOT_SIZE];
        
        if (bytes.Length <= SLOT_SIZE)
        {
            Array.Copy(bytes, 0, paddedBytes, SLOT_SIZE - bytes.Length, bytes.Length);
        }
        else
        {
            Array.Copy(bytes, bytes.Length - SLOT_SIZE, paddedBytes, 0, SLOT_SIZE);
        }
        
        Store(slot, paddedBytes);
    }
    
    /// <summary>
    /// Calculate storage slot for array element
    /// Based on Solidity's array storage layout
    /// </summary>
    /// <param name="arraySlot">Base slot of the array</param>
    /// <param name="index">Array index</param>
    /// <returns>Storage slot for the element</returns>
    public static BigInteger CalculateArrayElementSlot(BigInteger arraySlot, BigInteger index)
    {
        // For dynamic arrays: keccak256(arraySlot) + index
        var arraySlotBytes = arraySlot.ToByteArray(isUnsigned: true, isBigEndian: true);
        var paddedSlot = new byte[SLOT_SIZE];
        
        if (arraySlotBytes.Length <= SLOT_SIZE)
        {
            Array.Copy(arraySlotBytes, 0, paddedSlot, SLOT_SIZE - arraySlotBytes.Length, arraySlotBytes.Length);
        }
        
        var baseSlotHash = CryptoLib.Keccak256(paddedSlot);
        var baseSlot = new BigInteger(baseSlotHash, isUnsigned: true, isBigEndian: true);
        
        return baseSlot + index;
    }
    
    /// <summary>
    /// Calculate storage slot for mapping element
    /// Based on Solidity's mapping storage layout
    /// </summary>
    /// <param name="mappingSlot">Base slot of the mapping</param>
    /// <param name="key">Mapping key</param>
    /// <returns>Storage slot for the element</returns>
    public static BigInteger CalculateMappingElementSlot(BigInteger mappingSlot, byte[] key)
    {
        // For mappings: keccak256(key || mappingSlot)
        var mappingSlotBytes = mappingSlot.ToByteArray(isUnsigned: true, isBigEndian: true);
        var paddedSlot = new byte[SLOT_SIZE];
        
        if (mappingSlotBytes.Length <= SLOT_SIZE)
        {
            Array.Copy(mappingSlotBytes, 0, paddedSlot, SLOT_SIZE - mappingSlotBytes.Length, mappingSlotBytes.Length);
        }
        
        // Pad key to 32 bytes
        var paddedKey = new byte[SLOT_SIZE];
        if (key.Length <= SLOT_SIZE)
        {
            Array.Copy(key, 0, paddedKey, SLOT_SIZE - key.Length, key.Length);
        }
        else
        {
            Array.Copy(key, key.Length - SLOT_SIZE, paddedKey, 0, SLOT_SIZE);
        }
        
        var combined = new byte[SLOT_SIZE * 2];
        Array.Copy(paddedKey, 0, combined, 0, SLOT_SIZE);
        Array.Copy(paddedSlot, 0, combined, SLOT_SIZE, SLOT_SIZE);
        
        var hash = CryptoLib.Keccak256(combined);
        return new BigInteger(hash, isUnsigned: true, isBigEndian: true);
    }
    
    /// <summary>
    /// Calculate storage slot for mapping with BigInteger key
    /// </summary>
    /// <param name="mappingSlot">Base slot of the mapping</param>
    /// <param name="key">Mapping key as BigInteger</param>
    /// <returns>Storage slot for the element</returns>
    public static BigInteger CalculateMappingElementSlot(BigInteger mappingSlot, BigInteger key)
    {
        var keyBytes = key.ToByteArray(isUnsigned: true, isBigEndian: true);
        var paddedKey = new byte[SLOT_SIZE];
        
        if (keyBytes.Length <= SLOT_SIZE)
        {
            Array.Copy(keyBytes, 0, paddedKey, SLOT_SIZE - keyBytes.Length, keyBytes.Length);
        }
        else
        {
            Array.Copy(keyBytes, keyBytes.Length - SLOT_SIZE, paddedKey, 0, SLOT_SIZE);
        }
        
        return CalculateMappingElementSlot(mappingSlot, paddedKey);
    }
    
    /// <summary>
    /// Check if byte array represents zero
    /// </summary>
    /// <param name="value">Byte array to check</param>
    /// <returns>True if all bytes are zero</returns>
    private static bool IsZero(byte[] value)
    {
        return value.All(b => b == 0);
    }
    
    /// <summary>
    /// Get all modified storage slots since last commit
    /// </summary>
    /// <returns>Set of modified slot numbers</returns>
    public IReadOnlySet<BigInteger> GetModifiedSlots()
    {
        return new HashSet<BigInteger>(_modifiedSlots.Snapshot());
    }
    
    /// <summary>
    /// Cleanup expired cache entries
    /// </summary>
    /// <param name="state">Timer state (unused)</param>
    private void CleanupCache(object? state)
    {
        if (_disposed) return;
        
        try
        {
            var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
            var expiredSlots = new System.Collections.Generic.List<BigInteger>();
            
            foreach (var kvp in _cache)
            {
                if (currentTime - kvp.Value.LastAccessed > CACHE_TTL_MS && 
                    !kvp.Value.IsModified)
                {
                    expiredSlots.Add(kvp.Key);
                }
            }
            
            foreach (var slot in expiredSlots)
            {
                if (_cache.TryRemove(slot, out _))
                {
                    Interlocked.Increment(ref _cacheEvictions);
                }
            }
        }
        catch
        {
            // Ignore cleanup errors
        }
    }
    
    /// <summary>
    /// Evict the oldest cache entry to make room for new entries
    /// </summary>
    private void EvictOldestCacheEntry()
    {
        BigInteger? oldestSlot = null;
        ulong oldestTime = ulong.MaxValue;
        
        foreach (var kvp in _cache)
        {
            if (kvp.Value.LastAccessed < oldestTime && !kvp.Value.IsModified)
            {
                oldestTime = kvp.Value.LastAccessed;
                oldestSlot = kvp.Key;
            }
        }
        
        if (oldestSlot.HasValue && _cache.TryRemove(oldestSlot.Value, out _))
        {
            Interlocked.Increment(ref _cacheEvictions);
        }
    }
    
    /// <summary>
    /// Compress value if it would reduce storage size
    /// </summary>
    /// <param name="value">Value to potentially compress</param>
    /// <returns>Original or compressed value</returns>
    private byte[] CompressValueIfBeneficial(byte[] value)
    {
        // Simple run-length encoding for values with repeated bytes
        var consecutiveZeros = 0;
        for (int i = value.Length - 1; i >= 0; i--)
        {
            if (value[i] == 0)
                consecutiveZeros++;
            else
                break;
        }
        
        // If more than half the value is trailing zeros, store compressed
        if (consecutiveZeros > SLOT_SIZE / 2)
        {
            var nonZeroLength = SLOT_SIZE - consecutiveZeros;
            var compressed = new byte[nonZeroLength + 1];
            compressed[0] = (byte)consecutiveZeros; // Store zero count
            Array.Copy(value, 0, compressed, 1, nonZeroLength);
            return compressed;
        }
        
        return value;
    }
    
    /// <summary>
    /// Clear the cache and modified slots tracking
    /// </summary>
    public void ClearCache()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(StorageManager));
            
        _cache.Clear();
        _modifiedSlots.Clear();
        _cacheHits = 0;
        _cacheMisses = 0;
        _storageReads = 0;
        _storageWrites = 0;
        _cacheEvictions = 0;
    }
    
    /// <summary>
    /// Batch load multiple storage slots efficiently
    /// </summary>
    /// <param name="slots">Storage slot numbers</param>
    /// <returns>Dictionary mapping slots to their values</returns>
    public Dictionary<BigInteger, byte[]> BatchLoad(IEnumerable<BigInteger> slots)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(StorageManager));
            
        var result = new Dictionary<BigInteger, byte[]>();
        var slotsToFetch = new System.Collections.Generic.List<BigInteger>();
        
        // Check cache first
        foreach (var slot in slots)
        {
            if (_cache.TryGetValue(slot, out var cached))
            {
                result[slot] = cached.Value;
                Interlocked.Increment(ref _cacheHits);
            }
            else
            {
                slotsToFetch.Add(slot);
                Interlocked.Increment(ref _cacheMisses);
            }
        }
        
        // Batch fetch missing slots
        foreach (var slot in slotsToFetch)
        {
            result[slot] = Load(slot);
        }
        
        return result;
    }
    
    /// <summary>
    /// Batch store multiple storage slots efficiently
    /// </summary>
    /// <param name="updates">Dictionary mapping slots to their new values</param>
    public void BatchStore(Dictionary<BigInteger, byte[]> updates)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(StorageManager));
            
        foreach (var kvp in updates)
        {
            Store(kvp.Key, kvp.Value);
        }
    }
    
    /// <summary>
    /// Get comprehensive storage statistics
    /// </summary>
    /// <returns>Detailed storage usage statistics</returns>
    public StorageStats GetStats()
    {
        var totalOperations = _cacheHits + _cacheMisses;
        var modifiedCount = _cache.Values.Count(c => c.IsModified);
        
        return new StorageStats
        {
            CachedSlots = (uint)_cache.Count,
            ModifiedSlots = (uint)_modifiedSlots.Count,
            ModifiedCachedSlots = (uint)modifiedCount,
            CacheHitRatio = totalOperations > 0 ? (double)_cacheHits / totalOperations : 0.0,
            StorageReads = _storageReads,
            StorageWrites = _storageWrites,
            CacheEvictions = _cacheEvictions,
            CacheUtilization = MAX_CACHE_SIZE > 0 ? (double)_cache.Count / MAX_CACHE_SIZE : 0.0
        };
    }
    
    /// <summary>
    /// Dispose storage manager and cleanup resources
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            _cacheCleanupTimer?.Dispose();
            _lock?.Dispose();
            _accessTimer?.Stop();
            ClearCache();
            _disposed = true;
        }
    }
}

/// <summary>
/// Comprehensive storage usage statistics
/// </summary>
public record StorageStats
{
    /// <summary>Number of cached storage slots</summary>
    public uint CachedSlots { get; init; }
    
    /// <summary>Number of modified storage slots</summary>
    public uint ModifiedSlots { get; init; }
    
    /// <summary>Number of modified slots in cache</summary>
    public uint ModifiedCachedSlots { get; init; }
    
    /// <summary>Cache hit ratio (0.0 to 1.0)</summary>
    public double CacheHitRatio { get; init; }
    
    /// <summary>Total storage read operations</summary>
    public ulong StorageReads { get; init; }
    
    /// <summary>Total storage write operations</summary>
    public ulong StorageWrites { get; init; }
    
    /// <summary>Number of cache evictions performed</summary>
    public ulong CacheEvictions { get; init; }
    
    /// <summary>Cache utilization ratio (0.0 to 1.0)</summary>
    public double CacheUtilization { get; init; }
}

/// <summary>
/// Cached storage slot with metadata
/// </summary>
internal sealed class CachedSlot
{
    /// <summary>Slot value</summary>
    public byte[] Value { get; set; } = Array.Empty<byte>();
    
    /// <summary>Last access time in milliseconds</summary>
    public ulong LastAccessed { get; set; }
    
    /// <summary>Number of times accessed</summary>
    public ulong AccessCount { get; set; }
    
    /// <summary>Whether the slot has been modified</summary>
    public bool IsModified { get; set; }
}

/// <summary>
/// Thread-safe HashSet implementation
/// </summary>
internal sealed class ConcurrentHashSet<T> : IDisposable where T : notnull
{
    private readonly HashSet<T> _set = new();
    private readonly ReaderWriterLockSlim _lock = new();
    private bool _disposed = false;
    
    public void Add(T item)
    {
        if (_disposed) return;
        _lock.EnterWriteLock();
        try
        {
            _set.Add(item);
        }
        finally
        {
            _lock.ExitWriteLock();
        }
    }
    
    public bool Contains(T item)
    {
        if (_disposed) return false;
        _lock.EnterReadLock();
        try
        {
            return _set.Contains(item);
        }
        finally
        {
            _lock.ExitReadLock();
        }
    }
    
    public void Clear()
    {
        if (_disposed) return;
        _lock.EnterWriteLock();
        try
        {
            _set.Clear();
        }
        finally
        {
            _lock.ExitWriteLock();
        }
    }
    
    public int Count
    {
        get
        {
            if (_disposed) return 0;
            _lock.EnterReadLock();
            try
            {
                return _set.Count;
            }
            finally
            {
                _lock.ExitReadLock();
            }
        }
    }

    public T[] Snapshot()
    {
        if (_disposed) return Array.Empty<T>();
        _lock.EnterReadLock();
        try
        {
            return _set.ToArray();
        }
        finally
        {
            _lock.ExitReadLock();
        }
    }
    
    public void Dispose()
    {
        if (!_disposed)
        {
            _lock.Dispose();
            _disposed = true;
        }
    }
}
