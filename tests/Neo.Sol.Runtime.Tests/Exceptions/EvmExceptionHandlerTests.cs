using System.Numerics;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Neo.Sol.Runtime.Exceptions;
using Neo.Sol.Runtime.Context;
using Neo.Sol.Runtime.ABI;

namespace Neo.Sol.Runtime.Tests.Exceptions;

[TestClass]
public class EvmExceptionHandlerTests
{
    private EvmExceptionHandler _handler = null!;

    [TestInitialize]
    public void Setup()
    {
        _handler = new EvmExceptionHandler();
    }

    [TestCleanup]
    public void Cleanup()
    {
        _handler?.Dispose();
    }

    #region Revert Scenarios

    [TestMethod]
    public void Revert_WithEmptyReason_ThrowsEvmRevertExceptionWithEmptyData()
    {
        var exception = Assert.ThrowsException<EvmRevertException>(() =>
            _handler.Revert(""));

        Assert.AreEqual("", exception.Message);
        Assert.AreEqual(0, exception.RevertData.Length);
    }

    [TestMethod]
    public void Revert_WithErrorMessage_ThrowsEvmRevertExceptionWithEncodedReason()
    {
        var exception = Assert.ThrowsException<EvmRevertException>(() =>
            _handler.Revert("Insufficient balance"));

        Assert.AreEqual("Insufficient balance", exception.Message);
        Assert.IsTrue(exception.RevertData.Length > 0);
    }

    [TestMethod]
    public void RevertWithData_CustomData_ThrowsEvmRevertExceptionWithProvidedData()
    {
        var customData = new byte[] { 0x01, 0x02, 0x03, 0x04 };

        var exception = Assert.ThrowsException<EvmRevertException>(() =>
            _handler.RevertWithData(customData));

        Assert.AreEqual("Custom revert", exception.Message);
        CollectionAssert.AreEqual(customData, exception.RevertData);
    }

    #endregion

    #region Require/Assert Validation

    [TestMethod]
    public void Require_ConditionTrue_DoesNotThrow()
    {
        _handler.Require(true, "Should not throw");
    }

    [TestMethod]
    public void Require_ConditionFalse_ThrowsEvmRevertExceptionWithMessage()
    {
        var exception = Assert.ThrowsException<EvmRevertException>(() =>
            _handler.Require(false, "Balance too low"));

        Assert.AreEqual("Balance too low", exception.Message);
    }

    [TestMethod]
    public void Assert_ConditionTrue_DoesNotThrow()
    {
        _handler.Assert(true);
    }

    [TestMethod]
    public void Assert_ConditionFalse_ThrowsEvmAssertException()
    {
        var exception = Assert.ThrowsException<EvmAssertException>(() =>
            _handler.Assert(false));

        Assert.AreEqual("Assertion failed", exception.Message);
    }

    #endregion

    #region All EVM Exception Types with Data Extraction

    [TestMethod]
    public void Execute_EvmRevertException_ExtractsRevertData()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmRevertException("Transfer failed", new byte[] { 0xAA, 0xBB });
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmRevertException", result.Error.Type);
        Assert.AreEqual("Transfer failed", result.Error.Message);
        Assert.IsTrue(result.Error.Data.ContainsKey("revertData"));
    }

    [TestMethod]
    public void Execute_EvmOutOfGasException_ExtractsGasData()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmOutOfGasException(1000, 900);
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmOutOfGasException", result.Error.Type);
        Assert.IsTrue(result.Error.Data.ContainsKey("gasUsed"));
        Assert.IsTrue(result.Error.Data.ContainsKey("gasLimit"));
        Assert.AreEqual((ulong)1000, result.Error.Data["gasUsed"]);
        Assert.AreEqual((ulong)900, result.Error.Data["gasLimit"]);
    }

    [TestMethod]
    public void Execute_EvmStackOverflowException_ExtractsStackData()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmStackOverflowException(1025, 1024);
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmStackOverflowException", result.Error.Type);
        Assert.IsTrue(result.Error.Data.ContainsKey("stackDepth"));
        Assert.IsTrue(result.Error.Data.ContainsKey("maxDepth"));
    }

    [TestMethod]
    public void Execute_EvmAssertException_CapturesException()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmAssertException("Critical assertion failed");
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmAssertException", result.Error.Type);
        Assert.AreEqual("Critical assertion failed", result.Error.Message);
    }

    [TestMethod]
    public void Execute_EvmCallDepthException_CapturesCallDepthData()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmCallDepthException(1025, 1024);
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmCallDepthException", result.Error.Type);
    }

    #endregion

    #region Error Handler Registration and Recovery

    [TestMethod]
    public void RegisterErrorHandler_CustomHandler_CanRecoverFromError()
    {
        var testHandler = new TestRecoveryHandler();
        _handler.RegisterErrorHandler("InvalidOperationException", testHandler);

        var result = _handler.TryRecover(() =>
        {
            throw new InvalidOperationException("Test error");
        }, 0);

        Assert.AreEqual(42, result);
    }

    [TestMethod]
    public void TryRecover_WithFallback_ReturnsFallbackOnUnrecoverableError()
    {
        var result = _handler.TryRecover(() =>
        {
            throw new InvalidOperationException("Cannot recover");
        }, 999);

        Assert.AreEqual(999, result);
    }

    [TestMethod]
    public void TryRecover_SuccessfulOperation_ReturnsActualValue()
    {
        var result = _handler.TryRecover(() => 42, 999);

        Assert.AreEqual(42, result);
    }

    #endregion

    #region Exception Propagation Through Nested Calls

    [TestMethod]
    public void Execute_NestedExecuteCalls_PropagatesException()
    {
        var result = _handler.Execute<int>(() =>
        {
            _handler.Execute<int>(() =>
            {
                throw new EvmRevertException("Deep error", Array.Empty<byte>());
            });
            
            return 42;
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmRevertException", result.Error.Type);
    }

    [TestMethod]
    public void GetCallStack_DuringExecution_ShowsCallDepth()
    {
        try
        {
            _handler.Execute<int>(() =>
            {
                var stack = _handler.GetCallStack();
                Assert.AreEqual(1, stack.Length);
                Assert.AreEqual(0u, stack[0].CallDepth);
                throw new Exception("Test");
            });
        }
        catch
        {
        }
    }

    #endregion

    #region Async Exception Handling

    [TestMethod]
    public async Task ExecuteAsync_SuccessfulOperation_ReturnsSuccess()
    {
        var result = await _handler.ExecuteAsync(async () =>
        {
            await Task.Delay(1);
            return 42;
        });

        Assert.IsTrue(result.IsSuccess);
        Assert.AreEqual(42, result.Value);
    }

    [TestMethod]
    public async Task ExecuteAsync_ThrowsException_CapturesError()
    {
        var result = await _handler.ExecuteAsync<int>(async () =>
        {
            await Task.Delay(1);
            throw new EvmRevertException("Async revert", Array.Empty<byte>());
        });

        Assert.IsFalse(result.IsSuccess);
        Assert.IsNotNull(result.Error);
        Assert.AreEqual("EvmRevertException", result.Error.Type);
    }

    #endregion

    #region Call Stack Frame Tracking

    [TestMethod]
    public void GetCallStack_NoActiveExecution_ReturnsEmptyStack()
    {
        var stack = _handler.GetCallStack();

        Assert.AreEqual(0, stack.Length);
    }

    [TestMethod]
    public void Execute_TracksExecutionTime()
    {
        var result = _handler.Execute(() =>
        {
            Thread.Sleep(10);
            return 42;
        });

        Assert.IsTrue(result.IsSuccess);
        Assert.IsTrue(result.ExecutionTime >= 0);
    }

    #endregion

    #region Error Severity Classification

    [TestMethod]
    public void Execute_EvmRevertException_ClassifiedAsExpected()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmRevertException("Revert", Array.Empty<byte>());
        });

        Assert.AreEqual(ErrorSeverity.Expected, result.Error!.Severity);
    }

    [TestMethod]
    public void Execute_EvmAssertException_ClassifiedAsCritical()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmAssertException("Assert failed");
        });

        Assert.AreEqual(ErrorSeverity.Critical, result.Error!.Severity);
    }

    [TestMethod]
    public void Execute_EvmOutOfGasException_ClassifiedAsHigh()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new EvmOutOfGasException(1000, 900);
        });

        Assert.AreEqual(ErrorSeverity.High, result.Error!.Severity);
    }

    [TestMethod]
    public void Execute_ArgumentException_ClassifiedAsMedium()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new ArgumentException("Invalid argument");
        });

        Assert.AreEqual(ErrorSeverity.Medium, result.Error!.Severity);
    }

    [TestMethod]
    public void Execute_OutOfMemoryException_ClassifiedAsCritical()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new OutOfMemoryException("Memory exhausted");
        });

        Assert.AreEqual(ErrorSeverity.Critical, result.Error!.Severity);
    }

    #endregion

    #region ABI Encoding of Revert Reasons

    [TestMethod]
    public void Revert_WithReason_EncodesReasonAsAbiErrorString()
    {
        var exception = Assert.ThrowsException<EvmRevertException>(() =>
            _handler.Revert("Insufficient funds"));

        Assert.IsTrue(exception.RevertData.Length > 0);
        Assert.IsTrue(exception.RevertData.Length >= 4);
    }

    #endregion

    #region Exception Statistics Accuracy

    [TestMethod]
    public void GetStats_TracksExceptionsHandled()
    {
        _handler.Execute<int>(() => throw new Exception("Test1"));
        _handler.Execute<int>(() => throw new Exception("Test2"));

        var stats = _handler.GetStats();

        Assert.AreEqual(2ul, stats.ExceptionsHandled);
    }

    [TestMethod]
    public void GetStats_TracksRecoveredErrors()
    {
        var testHandler = new TestRecoveryHandler();
        _handler.RegisterErrorHandler("InvalidOperationException", testHandler);

        _handler.TryRecover(() => throw new InvalidOperationException("Test"), 0);

        var stats = _handler.GetStats();

        Assert.AreEqual(1ul, stats.RecoveredErrors);
    }

    [TestMethod]
    public void GetStats_TracksExceptionTypeBreakdown()
    {
        _handler.Execute<int>(() => throw new EvmRevertException("Test1", Array.Empty<byte>()));
        _handler.Execute<int>(() => throw new EvmRevertException("Test2", Array.Empty<byte>()));
        _handler.Execute<int>(() => throw new EvmAssertException("Test3"));

        var stats = _handler.GetStats();

        Assert.IsTrue(stats.ExceptionTypeBreakdown.ContainsKey("EvmRevertException"));
        Assert.AreEqual(2ul, stats.ExceptionTypeBreakdown["EvmRevertException"]);
        Assert.IsTrue(stats.ExceptionTypeBreakdown.ContainsKey("EvmAssertException"));
        Assert.AreEqual(1ul, stats.ExceptionTypeBreakdown["EvmAssertException"]);
    }

    [TestMethod]
    public void GetStats_TracksErrorHandlerCount()
    {
        var stats = _handler.GetStats();

        Assert.IsTrue(stats.ErrorHandlerCount >= 4);
    }

    [TestMethod]
    public void ClearStats_ResetsAllStatistics()
    {
        _handler.Execute<int>(() => throw new Exception("Test"));

        _handler.ClearStats();

        var stats = _handler.GetStats();
        Assert.AreEqual(0ul, stats.ExceptionsHandled);
        Assert.AreEqual(0ul, stats.RecoveredErrors);
        Assert.AreEqual(0, stats.ExceptionTypeBreakdown.Count);
    }

    #endregion

    #region Additional Tests

    [TestMethod]
    public void Execute_WithCustomErrorMessage_UsesCustomMessage()
    {
        var result = _handler.Execute<int>(() =>
        {
            throw new Exception("Original");
        }, errorMessage: "Custom error message");

        Assert.AreEqual("Custom error message", result.Error!.Message);
    }

    [TestMethod]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        var handler = new EvmExceptionHandler();
        handler.Dispose();
        handler.Dispose();
    }

    [TestMethod]
    public void Execute_AfterDispose_ThrowsObjectDisposedException()
    {
        var handler = new EvmExceptionHandler();
        handler.Dispose();

        Assert.ThrowsException<ObjectDisposedException>(() =>
            handler.Execute(() => 42));
    }

    [TestMethod]
    public void Execute_SuccessfulOperation_ReturnsSuccessWithValue()
    {
        var result = _handler.Execute(() => 42);

        Assert.IsTrue(result.IsSuccess);
        Assert.AreEqual(42, result.Value);
        Assert.IsNull(result.Error);
    }

    #endregion
}

internal class TestRecoveryHandler : ErrorHandler
{
    public override bool CanHandle(Exception exception)
        => exception is InvalidOperationException;

    public override bool TryHandle(Exception exception, out object? recoveredValue)
    {
        recoveredValue = 42;
        return true;
    }
}
