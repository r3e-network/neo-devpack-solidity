using System.Numerics;
using FluentAssertions;
using Neo.Sol.Runtime.Memory;
using NUnit.Framework;

namespace Neo.Sol.Runtime.Tests.Memory;

[TestFixture]
public class EvmMemoryManagerTests
{
    private EvmMemoryManager _memoryManager = null!;
    
    [SetUp]
    public void SetUp()
    {
        _memoryManager = new EvmMemoryManager();
    }
    
    [TearDown]
    public void TearDown()
    {
        _memoryManager.Dispose();
    }
    
    [Test]
    public void Constructor_ShouldInitializeEmptyMemory()
    {
        // Arrange & Act
        var memory = new EvmMemoryManager();
        
        // Assert
        memory.Size.Should().Be(0);
    }
    
    [Test]
    public void Store_ShouldStoreWordAtAddress()
    {
        // Arrange
        var address = 0u;
        var value = new byte[32];
        value[31] = 0x42; // Set last byte to 0x42
        
        // Act
        _memoryManager.Store(address, value);
        
        // Assert
        _memoryManager.Size.Should().Be(32);
        var loaded = _memoryManager.Load(address);
        loaded.Should().BeEquivalentTo(value);
    }
    
    [Test]
    public void Store_BigInteger_ShouldStoreBigEndian()
    {
        // Arrange
        var address = 0u;
        var value = new BigInteger(0x123456789ABCDEF0);
        
        // Act
        _memoryManager.Store(address, value);
        
        // Assert
        var loaded = _memoryManager.LoadBigInteger(address);
        loaded.Should().Be(value);
    }
    
    [Test]
    public void Load_UnallocatedMemory_ShouldReturnZero()
    {
        // Arrange
        var address = 64u;
        
        // Act
        var result = _memoryManager.Load(address);
        
        // Assert
        result.Should().HaveCount(32);
        result.Should().OnlyContain(b => b == 0);
    }
    
    [Test]
    public void StoreBytes_ShouldStoreArbitraryData()
    {
        // Arrange
        var address = 0u;
        var data = new byte[] { 0x01, 0x02, 0x03, 0x04, 0x05 };
        
        // Act
        _memoryManager.StoreBytes(address, data);
        
        // Assert
        var loaded = _memoryManager.LoadBytes(address, (uint)data.Length);
        loaded.Should().BeEquivalentTo(data);
    }
    
    [Test]
    public void ExpandMemory_ShouldGrowMemorySize()
    {
        // Arrange
        var newSize = 1024u;
        
        // Act
        _memoryManager.ExpandMemory(newSize);
        
        // Assert
        _memoryManager.Size.Should().Be(newSize);
    }
    
    [Test]
    public void CalculateExpansionCost_ShouldUseQuadraticFormula()
    {
        // Arrange
        _memoryManager.ExpandMemory(32); // 1 word
        var newSize = 64u; // 2 words
        
        // Act
        var cost = _memoryManager.CalculateExpansionCost(newSize);
        
        // Assert
        // Cost for 1 word: 1*3 + 1*1/512 = 3
        // Cost for 2 words: 2*3 + 2*2/512 = 6
        // Expansion cost: 6 - 3 = 3
        cost.Should().Be(3);
    }
    
    [Test]
    public void Store_WordSizeValidation_ShouldThrowForWrongSize()
    {
        // Arrange
        var address = 0u;
        var invalidValue = new byte[16]; // Wrong size
        
        // Act & Assert
        Assert.Throws<ArgumentException>(() => _memoryManager.Store(address, invalidValue));
    }
    
    [Test]
    public void ExpandMemory_ExceedsLimit_ShouldThrow()
    {
        // Arrange
        var excessiveSize = 64 * 1024 * 1024u + 1; // Exceed 64MB limit
        
        // Act & Assert
        Assert.Throws<InvalidOperationException>(() => _memoryManager.ExpandMemory(excessiveSize));
    }
    
    [Test]
    public void GetStats_ShouldReturnAccurateStatistics()
    {
        // Arrange
        _memoryManager.Store(0, new byte[32]);
        _memoryManager.Store(32, new byte[32]);
        
        // Act
        var stats = _memoryManager.GetStats();
        
        // Assert
        stats.TotalSize.Should().Be(64);
        stats.AllocatedPages.Should().BeGreaterThan(0);
        stats.UtilizationRatio.Should().BeGreaterThan(0);
    }
    
    [Test]
    public void Clear_ShouldResetMemory()
    {
        // Arrange
        _memoryManager.Store(0, new byte[32]);
        _memoryManager.Size.Should().Be(32);
        
        // Act
        _memoryManager.Clear();
        
        // Assert
        _memoryManager.Size.Should().Be(0);
        var stats = _memoryManager.GetStats();
        stats.AllocatedPages.Should().Be(0);
    }
    
    [Test]
    public void ConcurrentAccess_ShouldHandleMultipleOperations()
    {
        // Arrange
        var tasks = new List<Task>();
        var addresses = Enumerable.Range(0, 10).Select(i => (uint)(i * 32)).ToArray();
        
        // Act
        foreach (var addr in addresses)
        {
            tasks.Add(Task.Run(() =>
            {
                var value = new byte[32];
                value[31] = (byte)(addr / 32); // Unique value for each address
                _memoryManager.Store(addr, value);
            }));
        }
        
        Task.WaitAll(tasks.ToArray());
        
        // Assert
        foreach (var addr in addresses)
        {
            var loaded = _memoryManager.Load(addr);
            loaded[31].Should().Be((byte)(addr / 32));
        }
    }
}
