using System.Numerics;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Neo.Sol.Runtime;
using Neo.Sol.Runtime.ABI;
using Neo.Sol.Runtime.Crypto;
using Neo.Sol.Runtime.Exceptions;

namespace Neo.Sol.Runtime.Tests;

[TestClass]
public class ComprehensiveRuntimeTests
{
    [TestMethod]
    public void Runtime_CanBeConstructedOutsideNeoVmHost()
    {
        using var runtime = new EvmRuntime(Neo.UInt160.Parse("0x1234567890123456789012345678901234567890"));

        Assert.IsNotNull(runtime);
        Assert.IsNotNull(runtime.Memory);
        Assert.IsNotNull(runtime.Storage);
        Assert.IsNotNull(runtime.Events);
        Assert.IsNotNull(runtime.Context);
        Assert.AreEqual(0u, runtime.Events.GetEventCount());
    }

    [TestMethod]
    public void Runtime_GetStats_IsAvailableOnFreshRuntime()
    {
        using var runtime = new EvmRuntime(Neo.UInt160.Parse("0x1111111111111111111111111111111111111111"));

        runtime.Memory.Store(0, new BigInteger(42));
        var stats = runtime.GetStats();

        Assert.IsNotNull(stats);
        Assert.IsTrue(stats.MemoryStats.TotalSize > 0);
        Assert.AreEqual(0ul, stats.StorageStats.StorageReads);
        Assert.AreEqual(0ul, stats.StorageStats.StorageWrites);
        Assert.IsTrue(stats.GasUsed > 0);
    }

    [TestMethod]
    public void Runtime_CryptoHelpers_ReturnExpectedShapes()
    {
        using var runtime = new EvmRuntime(Neo.UInt160.Parse("0x2222222222222222222222222222222222222222"));
        var publicKey = Enumerable.Range(0, 64).Select(i => (byte)i).ToArray();

        var keccak = runtime.Keccak256("hello"u8.ToArray());
        var sha = runtime.Sha256("hello"u8.ToArray());
        var address = runtime.PublicKeyToAddress(publicKey);

        CollectionAssert.AreEqual(keccak, runtime.Keccak256("hello"u8.ToArray()));
        CollectionAssert.AreEqual(sha, runtime.Sha256("hello"u8.ToArray()));
        Assert.AreEqual(32, keccak.Length);
        Assert.AreEqual(32, sha.Length);
        Assert.AreEqual(20, address.Length);
    }

    [TestMethod]
    public void AbiEncoder_RoundTripsCoreShapes()
    {
        var encodedCall = AbiEncoder.EncodeCall(
            "transfer(address,uint256)",
            Neo.UInt160.Parse("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            new BigInteger(123)
        );
        var encodedUint = AbiEncoder.EncodeUint256(new BigInteger(123));
        var encodedInt = AbiEncoder.EncodeInt256(new BigInteger(-9));
        var encodedAddress = AbiEncoder.EncodeAddress(
            Neo.UInt160.Parse("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );

        Assert.AreEqual(4 + 64, encodedCall.Length);
        Assert.AreEqual(new BigInteger(123), AbiDecoder.DecodeUint256(encodedUint));
        Assert.AreEqual(new BigInteger(-9), AbiDecoder.DecodeInt256(encodedInt));
        Assert.AreEqual(
            Neo.UInt160.Parse("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            AbiDecoder.DecodeAddress(encodedAddress)
        );
    }

    [TestMethod]
    public void ExceptionHandler_ExecuteAndRecover_WorkConsistently()
    {
        using var handler = new EvmExceptionHandler();

        var success = handler.Execute(() => 7 * 6);
        var recovered = handler.TryRecover<int>(() => throw new InvalidOperationException("boom"), -1);
        var failure = handler.Execute<int>(() => throw new ArgumentException("bad"));

        Assert.IsTrue(success.IsSuccess);
        Assert.AreEqual(42, success.Value);
        Assert.AreEqual(-1, recovered);
        Assert.IsFalse(failure.IsSuccess);
        Assert.IsNotNull(failure.Error);
        Assert.AreEqual("ArgumentException", failure.Error.Type);
    }

    [TestMethod]
    public void StorageSlotHelpers_AreDeterministic()
    {
        var mappingSlot1 = Neo.Sol.Runtime.Storage.StorageManager.CalculateMappingElementSlot(
            BigInteger.Zero,
            new BigInteger(123)
        );
        var mappingSlot2 = Neo.Sol.Runtime.Storage.StorageManager.CalculateMappingElementSlot(
            BigInteger.Zero,
            new BigInteger(123)
        );
        var arraySlot1 = Neo.Sol.Runtime.Storage.StorageManager.CalculateArrayElementSlot(
            new BigInteger(5),
            BigInteger.One
        );
        var arraySlot2 = Neo.Sol.Runtime.Storage.StorageManager.CalculateArrayElementSlot(
            new BigInteger(5),
            BigInteger.One
        );

        Assert.AreEqual(mappingSlot1, mappingSlot2);
        Assert.AreEqual(arraySlot1, arraySlot2);
        Assert.AreNotEqual(mappingSlot1, arraySlot1);
    }
}
