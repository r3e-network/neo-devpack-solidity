using System.Numerics;
using FluentAssertions;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.Storage;
using NUnit.Framework;

namespace Neo.Sol.Runtime.Tests.Storage;

[TestFixture]
public class StorageManagerTests
{
    private StorageManager _storageManager = null!;

    [SetUp]
    public void SetUp()
    {
        _storageManager = new StorageManager(new StorageContext());
    }

    [TearDown]
    public void TearDown()
    {
        _storageManager.Dispose();
    }

    [Test]
    public void Constructor_ShouldInitializeEmptyStats()
    {
        var stats = _storageManager.GetStats();

        stats.CachedSlots.Should().Be(0);
        stats.ModifiedSlots.Should().Be(0);
        stats.StorageReads.Should().Be(0);
        stats.StorageWrites.Should().Be(0);
        stats.CacheHitRatio.Should().Be(0);
    }

    [Test]
    public void GetModifiedSlots_ShouldStartEmpty()
    {
        _storageManager.GetModifiedSlots().Should().BeEmpty();
    }

    [Test]
    public void CalculateArrayElementSlot_ShouldBeDeterministic()
    {
        var baseSlot = new BigInteger(5);
        var index = new BigInteger(7);

        var slot1 = StorageManager.CalculateArrayElementSlot(baseSlot, index);
        var slot2 = StorageManager.CalculateArrayElementSlot(baseSlot, index);

        slot1.Should().Be(slot2);
        slot1.Should().NotBe(baseSlot);
    }

    [Test]
    public void CalculateArrayElementSlot_ShouldChangePerIndex()
    {
        var baseSlot = new BigInteger(5);

        var slot1 = StorageManager.CalculateArrayElementSlot(baseSlot, BigInteger.Zero);
        var slot2 = StorageManager.CalculateArrayElementSlot(baseSlot, BigInteger.One);

        slot1.Should().NotBe(slot2);
    }

    [Test]
    public void CalculateMappingElementSlot_WithBytesKey_ShouldBeDeterministic()
    {
        var mappingSlot = new BigInteger(11);
        var key = new byte[] { 0x01, 0x02, 0x03, 0x04 };

        var slot1 = StorageManager.CalculateMappingElementSlot(mappingSlot, key);
        var slot2 = StorageManager.CalculateMappingElementSlot(mappingSlot, key);

        slot1.Should().Be(slot2);
        slot1.Should().NotBe(mappingSlot);
    }

    [Test]
    public void CalculateMappingElementSlot_WithDifferentKeys_ShouldDiffer()
    {
        var mappingSlot = new BigInteger(11);

        var slot1 = StorageManager.CalculateMappingElementSlot(mappingSlot, new byte[] { 0x01 });
        var slot2 = StorageManager.CalculateMappingElementSlot(mappingSlot, new byte[] { 0x02 });

        slot1.Should().NotBe(slot2);
    }

    [Test]
    public void CalculateMappingElementSlot_WithIntegerKey_ShouldBeDeterministic()
    {
        var mappingSlot = new BigInteger(13);
        var key = new BigInteger(42);

        var slot1 = StorageManager.CalculateMappingElementSlot(mappingSlot, key);
        var slot2 = StorageManager.CalculateMappingElementSlot(mappingSlot, key);

        slot1.Should().Be(slot2);
    }

}
