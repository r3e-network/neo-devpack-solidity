using System.Diagnostics;
using System.Numerics;
using FluentAssertions;
using Neo.Sol.Runtime.Crypto;
using Neo.Sol.Runtime.Exceptions;
using Neo.Sol.Runtime.Memory;
using Neo.Sol.Runtime.Storage;
using Neo.Sol.Runtime.ABI;
using NUnit.Framework;

namespace Neo.Sol.Runtime.Tests;

[TestFixture]
public class PerformanceBenchmarks
{
    [Test]
    public void MemoryManager_RepeatedStoreLoad_CompletesWithinReasonableBudget()
    {
        using var memory = new EvmMemoryManager();
        var stopwatch = Stopwatch.StartNew();

        for (uint i = 0; i < 2_000; i++)
        {
            var value = new BigInteger(i + 1);
            memory.Store(i * 32, value);
            memory.LoadBigInteger(i * 32).Should().Be(value);
        }

        stopwatch.Stop();

        var stats = memory.GetStats();
        stats.TotalAllocations.Should().BeGreaterThan(0);
        stats.TotalSize.Should().BeGreaterThan(0);
        stopwatch.Elapsed.Should().BeLessThan(TimeSpan.FromSeconds(5));
    }

    [Test]
    public void StorageSlotMath_RemainsResponsive()
    {
        var stopwatch = Stopwatch.StartNew();

        BigInteger lastMappingSlot = BigInteger.Zero;
        BigInteger lastArraySlot = BigInteger.Zero;
        for (var i = 0; i < 2_000; i++)
        {
            lastMappingSlot = StorageManager.CalculateMappingElementSlot(new BigInteger(i % 17), new BigInteger(i));
            lastArraySlot = StorageManager.CalculateArrayElementSlot(new BigInteger(i % 13), new BigInteger(i));
        }

        stopwatch.Stop();

        lastMappingSlot.Should().NotBe(BigInteger.Zero);
        lastArraySlot.Should().NotBe(BigInteger.Zero);
        stopwatch.Elapsed.Should().BeLessThan(TimeSpan.FromSeconds(5));
    }

    [Test]
    public void CryptoLib_HotLoop_ProducesStableHashesQuickly()
    {
        var payload = Enumerable.Repeat((byte)0x5A, 128).ToArray();
        byte[]? last = null;
        var stopwatch = Stopwatch.StartNew();

        for (var i = 0; i < 2_000; i++)
        {
            last = CryptoLib.Keccak256(payload);
        }

        stopwatch.Stop();

        last.Should().NotBeNull();
        last.Should().HaveCount(32);
        stopwatch.Elapsed.Should().BeLessThan(TimeSpan.FromSeconds(5));
    }

    [Test]
    public void AbiEncoding_HotLoop_RemainsResponsive()
    {
        var stopwatch = Stopwatch.StartNew();
        byte[]? last = null;

        for (var i = 0; i < 2_000; i++)
        {
            last = AbiEncoder.EncodeCall(
                "bench(uint256,bytes32)",
                i,
                Enumerable.Repeat((byte)(i % 251), 32).ToArray()
            );
        }

        stopwatch.Stop();

        last.Should().NotBeNull();
        last.Should().NotBeEmpty();
        stopwatch.Elapsed.Should().BeLessThan(TimeSpan.FromSeconds(5));
    }

    [Test]
    public void ExceptionHandler_SuccessfulCalls_StayResponsive()
    {
        using var handler = new EvmExceptionHandler();
        var stopwatch = Stopwatch.StartNew();

        for (var i = 0; i < 2_000; i++)
        {
            var result = handler.Execute(() => i + 1);
            result.IsSuccess.Should().BeTrue();
        }

        stopwatch.Stop();

        var stats = handler.GetStats();
        stats.ExceptionsHandled.Should().Be(0);
        stopwatch.Elapsed.Should().BeLessThan(TimeSpan.FromSeconds(5));
    }
}
