using System.Numerics;
using FluentAssertions;
using Neo.Sol.Runtime.Context;
using NUnit.Framework;
using RuntimeExecutionContext = Neo.Sol.Runtime.Context.ExecutionContext;

namespace Neo.Sol.Runtime.Tests.Context;

[TestFixture]
public class ExecutionContextTests
{
    [Test]
    public void Current_ShouldReturnSingleton()
    {
        var first = RuntimeExecutionContext.Current;
        var second = RuntimeExecutionContext.Current;

        first.Should().BeSameAs(second);
        first.Msg.Should().NotBeNull();
        first.Tx.Should().NotBeNull();
        first.Block.Should().NotBeNull();
    }

    [Test]
    public void MsgContext_ShouldProvideSafeDefaultsOutsideNeoVm()
    {
        var msg = RuntimeExecutionContext.Current.Msg;

        msg.Sender.Should().NotBeNull();
        msg.Value.Should().Be(BigInteger.Zero);
        msg.Data.Should().NotBeNull();
        msg.Data.Should().BeEmpty();
        msg.Gas.Should().BeGreaterThan(0);
        msg.Sig.Should().BeEmpty();
    }

    [Test]
    public void TxContext_ShouldProvideSafeDefaultsOutsideNeoVm()
    {
        var tx = RuntimeExecutionContext.Current.Tx;

        tx.Origin.Should().NotBeNull();
        tx.GasPrice.Should().BeGreaterOrEqualTo(BigInteger.Zero);
        tx.Hash.Should().NotBeNull();
        tx.Nonce.Should().BeGreaterOrEqualTo(BigInteger.Zero);
    }

    [Test]
    public void BlockContext_ShouldProvideDeterministicFallbacksOutsideNeoVm()
    {
        var block = RuntimeExecutionContext.Current.Block;

        block.Number.Should().BeGreaterOrEqualTo(0);
        block.Timestamp.Should().BeGreaterOrEqualTo(0);
        block.GasLimit.Should().BeGreaterThan(BigInteger.Zero);
        block.Difficulty.Should().BeGreaterOrEqualTo(BigInteger.Zero);
        block.BaseFee.Should().BeGreaterOrEqualTo(BigInteger.Zero);
        block.Coinbase.Should().NotBeNull();
        block.Hash.Should().NotBeNull();
    }

    [Test]
    public void BlockContext_GetBlockHash_ShouldReturnZeroForUnavailableBlocks()
    {
        var block = RuntimeExecutionContext.Current.Block;

        var hash = block.GetBlockHash(block.Number + 1);

        hash.Should().Be(Neo.UInt256.Zero);
        block.IsBlockHashAvailable(block.Number + 1).Should().BeFalse();
    }

    [Test]
    public void GasContext_ShouldCalculateMemoryExpansionCost()
    {
        GasContext.CalculateMemoryCost(0, 0).Should().Be(0);

        var small = GasContext.CalculateMemoryCost(0, 32);
        var larger = GasContext.CalculateMemoryCost(0, 96);

        small.Should().BeGreaterThan(0);
        larger.Should().BeGreaterThan(small);
    }

    [Test]
    public void GasContext_ConsumeGas_ShouldRejectExcessiveAmounts()
    {
        GasContext.ConsumeGas(long.MaxValue).Should().BeFalse();
    }
}
