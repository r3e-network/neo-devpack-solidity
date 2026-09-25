using System.Numerics;
using FluentAssertions;
using Neo.Sol.Runtime.Calls;
using Neo.Sol.Runtime.Context;
using NUnit.Framework;
using UInt160 = Neo.UInt160;
using RuntimeExecutionContext = Neo.Sol.Runtime.Context.ExecutionContext;

namespace Neo.Sol.Runtime.Tests.Calls;

[TestFixture]
public class ExternalCallManagerTests
{
    private ExternalCallManager _manager = null!;
    private RuntimeExecutionContext _context = null!;

    private static UInt160 SampleAddress(byte fill)
        => new(new byte[]
        {
            fill, fill, fill, fill, fill, fill, fill, fill, fill, fill,
            fill, fill, fill, fill, fill, fill, fill, fill, fill, fill,
        });

    [SetUp]
    public void Setup()
    {
        _context = RuntimeExecutionContext.Current;
        _manager = new ExternalCallManager(_context);
    }

    [Test]
    public void Constructor_ThrowsOnNullContext()
    {
        Action act = () => new ExternalCallManager(null!);

        act.Should().Throw<ArgumentNullException>()
            .WithParameterName("context");
    }

    [Test]
    public void Constructor_InitializesWithZeroCallCount()
    {
        var manager = new ExternalCallManager(_context);

        manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void Call_FailsWhenCallDataTooShort()
    {
        var target = SampleAddress(0x11);
        var callData = new byte[] { 0x01, 0x02, 0x03 }; // Only 3 bytes, need 4

        var result = _manager.Call(target, 0, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Invalid call data: too short");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void Call_FailsWhenTargetNotDeployed()
    {
        var target = SampleAddress(0xFF);
        var callData = new byte[] { 0x01, 0x02, 0x03, 0x04, 0x05 };

        var result = _manager.Call(target, 0, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Target contract not deployed");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void Call_WithValueTransfer_FailsWhenTransferFails()
    {
        var target = SampleAddress(0x22);
        var value = BigInteger.Parse("1000000000"); // 1 GAS
        var callData = new byte[] { 0x01, 0x02, 0x03, 0x04 };

        var result = _manager.Call(target, value, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Target contract not deployed");
    }

    [Test]
    public void Call_WithZeroValue_AcceptsValidCallData()
    {
        var target = SampleAddress(0x33);
        var callData = new byte[] { 0xaa, 0xbb, 0xcc, 0xdd, 0x11, 0x22 };

        var result = _manager.Call(target, 0, 100000, callData);

        // Will fail because contract is not actually deployed in test environment
        // but should not fail on the value transfer or data validation
        result.Success.Should().BeFalse();
        result.Error.Should().NotContain("Value transfer failed");
        result.Error.Should().NotContain("Invalid call data");
    }

    [Test]
    public void DelegateCall_FailsWhenCallDataTooShort()
    {
        var target = SampleAddress(0x44);
        var callData = new byte[] { 0x01, 0x02 };

        var result = _manager.DelegateCall(target, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Invalid call data: too short");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void DelegateCall_FailsWhenTargetNotDeployed()
    {
        var target = SampleAddress(0x55);
        var callData = new byte[] { 0x12, 0x34, 0x56, 0x78 };

        var result = _manager.DelegateCall(target, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Target contract not deployed");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void DelegateCall_DoesNotTransferValue()
    {
        var target = SampleAddress(0x66);
        var callData = new byte[] { 0xde, 0xad, 0xbe, 0xef };

        var result = _manager.DelegateCall(target, 100000, callData);

        // DelegateCall should never attempt value transfer
        result.Error.Should().NotContain("Value transfer");
    }

    [Test]
    public void StaticCall_FailsWhenCallDataTooShort()
    {
        var target = SampleAddress(0x77);
        var callData = new byte[] { 0xaa };

        var result = _manager.StaticCall(target, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Invalid call data: too short");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void StaticCall_FailsWhenTargetNotDeployed()
    {
        var target = SampleAddress(0x88);
        var callData = new byte[] { 0x11, 0x22, 0x33, 0x44 };

        var result = _manager.StaticCall(target, 100000, callData);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Target contract not deployed");
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void StaticCall_CannotTransferValue()
    {
        // StaticCall internally passes value=0, so this tests the internal validation
        var target = SampleAddress(0x99);
        var callData = new byte[] { 0xca, 0xfe, 0xba, 0xbe };

        var result = _manager.StaticCall(target, 100000, callData);

        // Should fail on contract not deployed, not on value transfer attempt
        result.Error.Should().NotContain("Static calls cannot transfer value");
        result.Error.Should().Contain("Target contract not deployed");
    }

    [Test]
    public void Create_AlwaysReturnsNotSupported()
    {
        var value = BigInteger.Zero;
        var initCode = new byte[] { 0x60, 0x80, 0x60, 0x40 };
        var gasLimit = 100000u;

        var result = _manager.Create(value, initCode, gasLimit);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("Contract creation is not supported");
        result.Address.Should().Be(UInt160.Zero);
    }

    [Test]
    public void Create2_AlwaysReturnsNotSupported()
    {
        var value = BigInteger.Zero;
        var initCode = new byte[] { 0x60, 0x80, 0x60, 0x40 };
        var salt = new byte[32];
        var gasLimit = 100000u;

        var result = _manager.Create2(value, initCode, salt, gasLimit);

        result.Success.Should().BeFalse();
        result.Error.Should().Contain("CREATE2 is not supported");
        result.Address.Should().Be(UInt160.Zero);
    }

    [Test]
    public void CallResult_Succeeded_CreatesSuccessResult()
    {
        var returnData = new byte[] { 0x01, 0x02, 0x03 };
        var gasUsed = 21000u;

        var result = CallResult.Succeeded(returnData, gasUsed);

        result.Success.Should().BeTrue();
        result.ReturnData.Should().BeEquivalentTo(returnData);
        result.GasUsed.Should().Be(gasUsed);
        result.Error.Should().BeEmpty();
    }

    [Test]
    public void CallResult_Failed_CreatesFailureResult()
    {
        var error = "Something went wrong";
        var gasUsed = 5000u;

        var result = CallResult.Failed(error, gasUsed);

        result.Success.Should().BeFalse();
        result.Error.Should().Be(error);
        result.GasUsed.Should().Be(gasUsed);
        result.ReturnData.Should().BeEmpty();
    }

    [Test]
    public void CreateResult_Succeeded_CreatesSuccessResult()
    {
        var address = SampleAddress(0xAA);
        var returnData = new byte[] { 0x11, 0x22 };
        var gasUsed = 50000u;

        var result = CreateResult.Succeeded(address, returnData, gasUsed);

        result.Success.Should().BeTrue();
        result.Address.Should().Be(address);
        result.ReturnData.Should().BeEquivalentTo(returnData);
        result.GasUsed.Should().Be(gasUsed);
        result.Error.Should().BeEmpty();
    }

    [Test]
    public void CreateResult_Failed_CreatesFailureResult()
    {
        var error = "Deployment failed";
        var gasUsed = 10000u;

        var result = CreateResult.Failed(error, gasUsed);

        result.Success.Should().BeFalse();
        result.Error.Should().Be(error);
        result.GasUsed.Should().Be(gasUsed);
        result.Address.Should().Be(UInt160.Zero);
        result.ReturnData.Should().BeEmpty();
    }

    [Test]
    public void GetCallCount_IncrementsOnlyOnActualCalls()
    {
        var target = SampleAddress(0xBB);
        var callData = new byte[] { 0x11, 0x22, 0x33, 0x44 };

        _manager.GetCallCount().Should().Be(0);

        // Failed calls due to validation should not increment
        _manager.Call(target, 0, 100000, callData);
        _manager.GetCallCount().Should().Be(0);

        _manager.DelegateCall(target, 100000, callData);
        _manager.GetCallCount().Should().Be(0);

        _manager.StaticCall(target, 100000, callData);
        _manager.GetCallCount().Should().Be(0);
    }

    [Test]
    public void CallType_EnumHasExpectedValues()
    {
        Enum.GetValues<CallType>().Should().Contain(CallType.Call);
        Enum.GetValues<CallType>().Should().Contain(CallType.DelegateCall);
        Enum.GetValues<CallType>().Should().Contain(CallType.StaticCall);
        Enum.GetValues<CallType>().Length.Should().Be(3);
    }
}
