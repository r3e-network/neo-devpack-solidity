using System.Numerics;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Neo.Sol.Runtime;
using Neo.Sol.Runtime.ABI;
using Neo.Sol.Runtime.Crypto;
using Neo.Sol.Runtime.Exceptions;

namespace Neo.Sol.Runtime.Tests;

[TestClass]
public class NeoBlockchainIntegrationTests
{
    private static readonly Neo.UInt160 ContractAddress =
        Neo.UInt160.Parse("0x1234567890123456789012345678901234567890");

    [TestMethod]
    public void Integration_ContextFallbacks_AreHostSafe()
    {
        using var runtime = new EvmRuntime(ContractAddress);

        Assert.AreEqual(Neo.UInt160.Zero, runtime.Msg.Sender);
        Assert.AreEqual(BigInteger.Zero, runtime.Msg.Value);
        Assert.AreEqual(0u, runtime.Block.Number);
        Assert.AreEqual(0UL, runtime.Block.Timestamp);
        Assert.AreEqual(Neo.UInt256.Zero, runtime.Block.Hash);
        Assert.AreEqual(Neo.UInt160.Zero, runtime.Tx.Origin);
        Assert.AreEqual(Neo.UInt256.Zero, runtime.Tx.Hash);
    }

    [TestMethod]
    public void Integration_RuntimeStats_ComposeAcrossComponents()
    {
        using var runtime = new EvmRuntime(ContractAddress);

        runtime.Memory.Store(0, new BigInteger(7));
        runtime.Memory.Store(32, new BigInteger(9));
        runtime.Events.Log1("RuntimeIntegrated(uint256)", new BigInteger(1));

        var stats = runtime.GetStats();

        Assert.IsTrue(stats.MemoryStats.TotalSize >= 64);
        Assert.AreEqual(1u, runtime.Events.GetEventCount());
        Assert.IsTrue(stats.GasUsed > 0);
        Assert.AreEqual(0u, stats.RegistryStats.TotalContracts);
    }

    [TestMethod]
    public void Integration_AbiAndCrypto_Pipeline_IsDeterministic()
    {
        var call = Evm.EncodeCall(
            "transfer(address,uint256)",
            Neo.UInt160.Parse("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            new BigInteger(25)
        );
        var selector = Evm.Selector("transfer(address,uint256)");
        var topic = CryptoLib.Keccak256(
            System.Text.Encoding.UTF8.GetBytes("Transfer(address,address,uint256)")
        );

        using var runtime = new EvmRuntime(ContractAddress);
        var hash1 = runtime.Keccak256(call);
        var hash2 = runtime.Keccak256(call);

        CollectionAssert.AreEqual(hash1, hash2);
        Assert.AreEqual(4, selector.Length);
        Assert.AreEqual(32, topic.Length);
        Assert.AreEqual(32, hash1.Length);
    }

    [TestMethod]
    public void Integration_CallManager_FailsGracefully_OutsideNeoVmHost()
    {
        using var runtime = new EvmRuntime(ContractAddress);
        var missingTarget = Neo.UInt160.Parse("0xffffffffffffffffffffffffffffffffffffffff");

        var call = runtime.Calls.Call(missingTarget, BigInteger.Zero, 1000, new byte[] { 0, 0, 0, 0 });
        var create = runtime.Calls.Create(BigInteger.Zero, new byte[] { 0x01 }, 1000);

        Assert.IsFalse(call.Success);
        StringAssert.Contains(call.Error, "Target contract not deployed");
        Assert.IsFalse(create.Success);
        StringAssert.Contains(create.Error, "not supported");
    }

    [TestMethod]
    public void Integration_ExceptionAndRegistry_AreHostSafe()
    {
        using var handler = new EvmExceptionHandler();
        using var runtime = new EvmRuntime(ContractAddress);

        var result = handler.Execute(() => 5 + 7, runtime.Context);
        var recovered = handler.TryRecover<int>(() => throw new InvalidOperationException("boom"), -1, runtime.Context);
        var registryStats = runtime.Registry.GetStats();

        Assert.IsTrue(result.IsSuccess);
        Assert.AreEqual(12, result.Value);
        Assert.AreEqual(-1, recovered);
        Assert.AreEqual(0u, registryStats.TotalContracts);
        Assert.AreEqual(0u, registryStats.ActiveContracts);
    }

    [TestMethod]
    public void Integration_Reset_ClearsMemoryState()
    {
        using var runtime = new EvmRuntime(ContractAddress);

        runtime.Memory.Store(0, new BigInteger(123));
        runtime.Memory.Store(32, new BigInteger(456));
        runtime.Reset();

        var stats = runtime.GetStats();
        Assert.AreEqual(0u, stats.MemoryStats.TotalSize);
        Assert.AreEqual(0u, runtime.Events.GetEventCount());
    }
}
