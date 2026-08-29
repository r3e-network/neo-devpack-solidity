using System.Numerics;
using System.Collections.Concurrent;
using System.Diagnostics;
using Neo.SmartContract.Framework;

namespace Neo.Sol.Runtime.Memory;

/// <summary>
/// EVM-compatible memory manager with 32-byte word addressing
/// Provides linear memory growth, word-aligned access patterns, and garbage collection
/// Production-grade implementation with performance monitoring and optimization
/// </summary>
public sealed class EvmMemoryManager : IDisposable
{
    private readonly ConcurrentDictionary<uint, MemoryPage> _pages = new();
    private uint _highestAddress = 0;
    private readonly object _lock = new object();
    private readonly Timer _gcTimer;
    private readonly Stopwatch _accessTimer = new();
    private bool _disposed = false;
    
    // Performance tracking
    private ulong _totalAllocations = 0;
    private ulong _totalDeallocations = 0;
    private ulong _gcCollections = 0;
    private ulong _cacheHits = 0;
    private ulong _cacheMisses = 0;
    
    // EVM memory constants
    private const uint WORD_SIZE = 32;
    private const uint PAGE_SIZE = 4096; // 4KB pages for better cache alignment
    private const uint MAX_MEMORY_SIZE = 64 * 1024 * 1024; // 64MB limit for production
    private const uint GC_THRESHOLD_PAGES = 1024; // Trigger GC after 1024 pages
    private const int GC_INTERVAL_MS = 30000; // GC every 30 seconds
    private const uint PAGE_ACCESS_THRESHOLD = 100; // Consider page cold after 100 operations
    
    /// <summary>
    /// Initialize memory manager with garbage collection
    /// </summary>
    public EvmMemoryManager()
    {
        _accessTimer.Start();
        _gcTimer = new Timer(RunGarbageCollection, null, GC_INTERVAL_MS, GC_INTERVAL_MS);
    }
    
    /// <summary>
    /// Current memory size in bytes
    /// </summary>
    public uint Size => _highestAddress;
    
    /// <summary>
    /// Number of allocated pages
    /// </summary>
    public uint PageCount => (uint)_pages.Count;
    
    /// <summary>
    /// Total memory allocated in bytes
    /// </summary>
    public uint AllocatedMemory => (uint)_pages.Count * PAGE_SIZE;
    
    /// <summary>
    /// Memory utilization ratio (0.0 to 1.0)
    /// </summary>
    public double UtilizationRatio => AllocatedMemory > 0 ? (double)_highestAddress / AllocatedMemory : 0.0;
    
    /// <summary>
    /// Memory expansion cost (quadratic growth as per EVM spec)
    /// </summary>
    /// <param name="newSize">Target memory size</param>
    /// <returns>Gas cost for expansion</returns>
    public ulong CalculateExpansionCost(uint newSize)
    {
        if (newSize <= _highestAddress) return 0;
        
        var currentWords = (_highestAddress + WORD_SIZE - 1) / WORD_SIZE;
        var newWords = (newSize + WORD_SIZE - 1) / WORD_SIZE;
        
        // EVM memory cost formula: words * 3 + words^2 / 512
        var currentCost = currentWords * 3 + (currentWords * currentWords) / 512;
        var newCost = newWords * 3 + (newWords * newWords) / 512;
        
        return newCost - currentCost;
    }
    
    /// <summary>
    /// Expand memory to accommodate the specified size with optimized allocation
    /// </summary>
    /// <param name="size">Target memory size</param>
    /// <exception cref="InvalidOperationException">Memory limit exceeded</exception>
    /// <exception cref="ObjectDisposedException">Memory manager disposed</exception>
    public void ExpandMemory(uint size)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmMemoryManager));
            
        if (size > MAX_MEMORY_SIZE)
            throw new InvalidOperationException($"Memory size {size} exceeds maximum {MAX_MEMORY_SIZE}");
            
        if (size <= _highestAddress) return;
        
        lock (_lock)
        {
            if (size <= _highestAddress) return; // Double-check after acquiring lock
            
            // Allocate pages as needed
            var startPage = _highestAddress / PAGE_SIZE;
            var endPage = (size - 1) / PAGE_SIZE;
            var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
            
            for (var page = startPage; page <= endPage; page++)
            {
                if (!_pages.ContainsKey(page))
                {
                    var memoryPage = new MemoryPage
                    {
                        Data = new byte[PAGE_SIZE],
                        LastAccessed = currentTime,
                        AccessCount = 1,
                        IsActive = true
                    };
                    
                    _pages.TryAdd(page, memoryPage);
                    Interlocked.Increment(ref _totalAllocations);
                }
                else
                {
                    // Update access time for existing page
                    _pages[page].LastAccessed = currentTime;
                    _pages[page].AccessCount++;
                }
            }
            
            _highestAddress = size;
            
            // Trigger GC if we have too many pages
            if (_pages.Count > GC_THRESHOLD_PAGES)
            {
                _ = Task.Run(() => RunGarbageCollection(null));
            }
        }
    }
    
    /// <summary>
    /// Store 32-byte word at specified address with performance tracking
    /// </summary>
    /// <param name="address">Memory address</param>
    /// <param name="value">32-byte value to store</param>
    /// <exception cref="ArgumentException">Invalid value size</exception>
    /// <exception cref="ObjectDisposedException">Memory manager disposed</exception>
    public void Store(uint address, byte[] value)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmMemoryManager));
            
        if (value.Length != WORD_SIZE)
            throw new ArgumentException($"Value must be exactly {WORD_SIZE} bytes");
            
        ExpandMemory(address + WORD_SIZE);
        var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
        
        for (int i = 0; i < WORD_SIZE; i++)
        {
            var byteAddress = address + (uint)i;
            var pageNumber = byteAddress / PAGE_SIZE;
            var offset = byteAddress % PAGE_SIZE;
            
            if (_pages.TryGetValue(pageNumber, out var page))
            {
                page.Data[offset] = value[i];
                page.LastAccessed = currentTime;
                page.AccessCount++;
                page.IsModified = true;
                Interlocked.Increment(ref _cacheHits);
            }
            else
            {
                Interlocked.Increment(ref _cacheMisses);
                // This should not happen after ExpandMemory, but handle gracefully
                ExpandMemory(byteAddress + 1);
                if (_pages.TryGetValue(pageNumber, out page))
                {
                    page.Data[offset] = value[i];
                }
            }
        }
    }
    
    /// <summary>
    /// Store BigInteger as 32-byte word (big-endian)
    /// </summary>
    /// <param name="address">Memory address</param>
    /// <param name="value">BigInteger value</param>
    public void Store(uint address, BigInteger value)
    {
        var bytes = value.ToByteArray(isUnsigned: true, isBigEndian: true);
        var word = new byte[WORD_SIZE];
        
        // Pad with zeros on the left for big-endian format
        if (bytes.Length <= WORD_SIZE)
        {
            Array.Copy(bytes, 0, word, WORD_SIZE - bytes.Length, bytes.Length);
        }
        else
        {
            // Truncate to 32 bytes (overflow behavior)
            Array.Copy(bytes, bytes.Length - WORD_SIZE, word, 0, WORD_SIZE);
        }
        
        Store(address, word);
    }
    
    /// <summary>
    /// Load 32-byte word from specified address with performance tracking
    /// </summary>
    /// <param name="address">Memory address</param>
    /// <returns>32-byte word</returns>
    /// <exception cref="ObjectDisposedException">Memory manager disposed</exception>
    public byte[] Load(uint address)
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmMemoryManager));
            
        ExpandMemory(address + WORD_SIZE);
        var result = new byte[WORD_SIZE];
        var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
        
        for (int i = 0; i < WORD_SIZE; i++)
        {
            var byteAddress = address + (uint)i;
            var pageNumber = byteAddress / PAGE_SIZE;
            var offset = byteAddress % PAGE_SIZE;
            
            if (_pages.TryGetValue(pageNumber, out var page))
            {
                result[i] = page.Data[offset];
                page.LastAccessed = currentTime;
                page.AccessCount++;
                Interlocked.Increment(ref _cacheHits);
            }
            else
            {
                Interlocked.Increment(ref _cacheMisses);
                // Unallocated memory returns zero
                result[i] = 0;
            }
        }
        
        return result;
    }
    
    /// <summary>
    /// Load BigInteger from 32-byte word (big-endian)
    /// </summary>
    /// <param name="address">Memory address</param>
    /// <returns>BigInteger value</returns>
    public BigInteger LoadBigInteger(uint address)
    {
        var bytes = Load(address);
        return new BigInteger(bytes, isUnsigned: true, isBigEndian: true);
    }
    
    /// <summary>
    /// Load arbitrary number of bytes from memory
    /// </summary>
    /// <param name="address">Start address</param>
    /// <param name="size">Number of bytes to load</param>
    /// <returns>Byte array</returns>
    public byte[] LoadBytes(uint address, uint size)
    {
        if (size == 0) return Array.Empty<byte>();
        
        ExpandMemory(address + size);
        
        var result = new byte[size];
        
        for (uint i = 0; i < size; i++)
        {
            var byteAddress = address + i;
            var page = byteAddress / PAGE_SIZE;
            var offset = byteAddress % PAGE_SIZE;
            
            if (_pages.ContainsKey(page))
            {
                result[i] = _pages[page].Data[offset];
            }
        }
        
        return result;
    }
    
    /// <summary>
    /// Store arbitrary bytes to memory
    /// </summary>
    /// <param name="address">Start address</param>
    /// <param name="data">Data to store</param>
    public void StoreBytes(uint address, byte[] data)
    {
        if (data.Length == 0) return;
        
        ExpandMemory(address + (uint)data.Length);
        
        for (int i = 0; i < data.Length; i++)
        {
            var byteAddress = address + (uint)i;
            var page = byteAddress / PAGE_SIZE;
            var offset = byteAddress % PAGE_SIZE;
            _pages[page].Data[offset] = data[i];
        }
    }
    
    /// <summary>
    /// Run garbage collection to free unused memory pages
    /// </summary>
    /// <param name="state">Timer state (unused)</param>
    private void RunGarbageCollection(object? state)
    {
        if (_disposed) return;
        
        try
        {
            lock (_lock)
            {
                var currentTime = (ulong)_accessTimer.ElapsedMilliseconds;
                var pagesToRemove = new System.Collections.Generic.List<uint>();
                
                foreach (var kvp in _pages)
                {
                    var pageNumber = kvp.Key;
                    var page = kvp.Value;
                    
                    // Remove pages that haven't been accessed recently and are beyond highest address
                    var pageStartAddress = pageNumber * PAGE_SIZE;
                    if (pageStartAddress >= _highestAddress || 
                        (!page.IsActive && 
                         currentTime - page.LastAccessed > PAGE_ACCESS_THRESHOLD && 
                         page.AccessCount < 10))
                    {
                        pagesToRemove.Add(pageNumber);
                    }
                }
                
                foreach (var pageNumber in pagesToRemove)
                {
                    if (_pages.TryRemove(pageNumber, out _))
                    {
                        Interlocked.Increment(ref _totalDeallocations);
                    }
                }
                
                if (pagesToRemove.Count > 0)
                {
                    Interlocked.Increment(ref _gcCollections);
                }
            }
        }
        catch
        {
            // Ignore GC errors to prevent disrupting main execution
        }
    }
    
    /// <summary>
    /// Clear all memory and reset state
    /// </summary>
    public void Clear()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmMemoryManager));
            
        lock (_lock)
        {
            _pages.Clear();
            _highestAddress = 0;
            _totalAllocations = 0;
            _totalDeallocations = 0;
            _gcCollections = 0;
            _cacheHits = 0;
            _cacheMisses = 0;
        }
    }
    
    /// <summary>
    /// Force immediate garbage collection
    /// </summary>
    public void ForceGarbageCollection()
    {
        RunGarbageCollection(null);
    }
    
    /// <summary>
    /// Get comprehensive memory usage statistics
    /// </summary>
    /// <returns>Detailed memory statistics</returns>
    public MemoryStats GetStats()
    {
        lock (_lock)
        {
            var activePages = _pages.Values.Count(p => p.IsActive);
            var modifiedPages = _pages.Values.Count(p => p.IsModified);
            
            return new MemoryStats
            {
                TotalSize = _highestAddress,
                AllocatedPages = (uint)_pages.Count,
                ActivePages = (uint)activePages,
                ModifiedPages = (uint)modifiedPages,
                AllocatedBytes = (uint)_pages.Count * PAGE_SIZE,
                UtilizationRatio = AllocatedMemory > 0 ? (double)_highestAddress / AllocatedMemory : 0.0,
                CacheHitRatio = (_cacheHits + _cacheMisses) > 0 ? (double)_cacheHits / (_cacheHits + _cacheMisses) : 0.0,
                TotalAllocations = _totalAllocations,
                TotalDeallocations = _totalDeallocations,
                GCCollections = _gcCollections,
                FragmentationRatio = CalculateFragmentation()
            };
        }
    }
    
    /// <summary>
    /// Calculate memory fragmentation ratio
    /// </summary>
    /// <returns>Fragmentation ratio (0.0 to 1.0)</returns>
    private double CalculateFragmentation()
    {
        if (_pages.Count == 0) return 0.0;
        
        var pageNumbers = _pages.Keys.OrderBy(x => x).ToArray();
        if (pageNumbers.Length <= 1) return 0.0;
        
        var gaps = 0;
        for (int i = 1; i < pageNumbers.Length; i++)
        {
            if (pageNumbers[i] - pageNumbers[i-1] > 1)
            {
                gaps += (int)(pageNumbers[i] - pageNumbers[i-1] - 1);
            }
        }
        
        var totalPossiblePages = pageNumbers[^1] - pageNumbers[0] + 1;
        return totalPossiblePages > 0 ? (double)gaps / totalPossiblePages : 0.0;
    }
    
    /// <summary>
    /// Dispose memory manager and cleanup resources
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            _gcTimer?.Dispose();
            _accessTimer?.Stop();
            Clear();
            _disposed = true;
        }
    }
}

/// <summary>
/// Comprehensive memory usage statistics
/// </summary>
public record MemoryStats
{
    /// <summary>Total memory size in bytes</summary>
    public uint TotalSize { get; init; }
    
    /// <summary>Number of allocated pages</summary>
    public uint AllocatedPages { get; init; }
    
    /// <summary>Number of active pages</summary>
    public uint ActivePages { get; init; }
    
    /// <summary>Number of modified pages</summary>
    public uint ModifiedPages { get; init; }
    
    /// <summary>Total allocated bytes</summary>
    public uint AllocatedBytes { get; init; }
    
    /// <summary>Memory utilization ratio (0.0 to 1.0)</summary>
    public double UtilizationRatio { get; init; }
    
    /// <summary>Cache hit ratio (0.0 to 1.0)</summary>
    public double CacheHitRatio { get; init; }
    
    /// <summary>Total number of allocations</summary>
    public ulong TotalAllocations { get; init; }
    
    /// <summary>Total number of deallocations</summary>
    public ulong TotalDeallocations { get; init; }
    
    /// <summary>Number of garbage collections performed</summary>
    public ulong GCCollections { get; init; }
    
    /// <summary>Memory fragmentation ratio (0.0 to 1.0)</summary>
    public double FragmentationRatio { get; init; }
}

/// <summary>
/// Represents a memory page with metadata for garbage collection
/// </summary>
internal sealed class MemoryPage
{
    /// <summary>Page data</summary>
    public byte[] Data { get; set; } = Array.Empty<byte>();
    
    /// <summary>Last access time in milliseconds</summary>
    public ulong LastAccessed { get; set; }
    
    /// <summary>Number of times this page was accessed</summary>
    public ulong AccessCount { get; set; }
    
    /// <summary>Whether this page is actively used</summary>
    public bool IsActive { get; set; } = true;
    
    /// <summary>Whether this page has been modified</summary>
    public bool IsModified { get; set; } = false;
}
