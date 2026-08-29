using System.Numerics;
using System.Diagnostics;
using System.Runtime.ExceptionServices;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.ABI;
using Neo.Sol.Runtime.Context;
using ExecutionContext = Neo.Sol.Runtime.Context.ExecutionContext;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;

namespace Neo.Sol.Runtime.Exceptions;

/// <summary>
/// Production-grade exception handling system for EVM-compatible error propagation
/// Provides comprehensive error handling, debugging support, and error recovery
/// </summary>
public sealed class EvmExceptionHandler : IDisposable
{
    private readonly Stack<ExceptionFrame> _exceptionStack = new();
    private readonly Dictionary<string, ErrorHandler> _errorHandlers = new();
    private readonly Stopwatch _executionTimer = new();
    private bool _disposed = false;
    
    // Performance tracking
    private ulong _exceptionsHandled = 0;
    private ulong _recoveredErrors = 0;
    private readonly Dictionary<Type, ulong> _exceptionTypeCount = new();
    
    // Configuration
    private readonly ExceptionConfig _config;
    
    public EvmExceptionHandler(ExceptionConfig? config = null)
    {
        _config = config ?? ExceptionConfig.Default;
        _executionTimer.Start();
        RegisterStandardErrorHandlers();
    }

    private static void TryLog(string message)
    {
        try
        {
            NeoFrameworkRuntime.Log(message);
        }
        catch
        {
            // Ignore logging failures outside a Neo VM host.
        }
    }
    
    /// <summary>
    /// Execute operation with comprehensive exception handling
    /// </summary>
    /// <typeparam name="T">Return type</typeparam>
    /// <param name="operation">Operation to execute</param>
    /// <param name="context">Execution context</param>
    /// <param name="errorMessage">Custom error message</param>
    /// <returns>Operation result with error handling</returns>
    public ExecutionResult<T> Execute<T>(Func<T> operation, ExecutionContext? context = null, string errorMessage = "")
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmExceptionHandler));
            
        var frame = PushFrame(operation.Method.Name, context);
        
        try
        {
            var result = operation();
            frame.Success = true;
            return ExecutionResult<T>.Success(result, frame.ExecutionTime);
        }
        catch (Exception ex)
        {
            return HandleException<T>(ex, frame, errorMessage);
        }
        finally
        {
            PopFrame();
        }
    }
    
    /// <summary>
    /// Execute async operation with exception handling
    /// </summary>
    /// <typeparam name="T">Return type</typeparam>
    /// <param name="operation">Async operation to execute</param>
    /// <param name="context">Execution context</param>
    /// <param name="errorMessage">Custom error message</param>
    /// <returns>Operation result with error handling</returns>
    public async Task<ExecutionResult<T>> ExecuteAsync<T>(Func<Task<T>> operation, ExecutionContext? context = null, string errorMessage = "")
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(EvmExceptionHandler));
            
        var frame = PushFrame(operation.Method.Name, context);
        
        try
        {
            var result = await operation();
            frame.Success = true;
            return ExecutionResult<T>.Success(result, frame.ExecutionTime);
        }
        catch (Exception ex)
        {
            return HandleException<T>(ex, frame, errorMessage);
        }
        finally
        {
            PopFrame();
        }
    }
    
    /// <summary>
    /// Handle EVM revert with reason
    /// </summary>
    /// <param name="reason">Revert reason</param>
    /// <param name="context">Execution context</param>
    /// <exception cref="EvmRevertException">Always throws revert exception</exception>
    public void Revert(string reason = "", ExecutionContext? context = null)
    {
        var revertData = string.IsNullOrEmpty(reason) ? Array.Empty<byte>() : EncodeRevertReason(reason);
        var exception = new EvmRevertException(reason, revertData);
        
        LogException(exception, context);
        throw exception;
    }
    
    /// <summary>
    /// Handle EVM revert with custom data
    /// </summary>
    /// <param name="errorData">Custom error data</param>
    /// <param name="context">Execution context</param>
    /// <exception cref="EvmRevertException">Always throws revert exception</exception>
    public void RevertWithData(byte[] errorData, ExecutionContext? context = null)
    {
        var exception = new EvmRevertException("Custom revert", errorData);
        
        LogException(exception, context);
        throw exception;
    }
    
    /// <summary>
    /// Handle require assertion
    /// </summary>
    /// <param name="condition">Condition to check</param>
    /// <param name="message">Error message if condition fails</param>
    /// <param name="context">Execution context</param>
    /// <exception cref="EvmRevertException">Throws if condition is false</exception>
    public void Require(bool condition, string message = "Requirement failed", ExecutionContext? context = null)
    {
        if (!condition)
        {
            Revert(message, context);
        }
    }
    
    /// <summary>
    /// Handle assert statement (should never fail in correct code)
    /// </summary>
    /// <param name="condition">Condition to assert</param>
    /// <param name="context">Execution context</param>
    /// <exception cref="EvmAssertException">Throws if assertion fails</exception>
    public void Assert(bool condition, ExecutionContext? context = null)
    {
        if (!condition)
        {
            var exception = new EvmAssertException("Assertion failed");
            LogException(exception, context);
            throw exception;
        }
    }
    
    /// <summary>
    /// Try to recover from error using registered handlers
    /// </summary>
    /// <typeparam name="T">Return type</typeparam>
    /// <param name="operation">Operation to try</param>
    /// <param name="fallbackValue">Fallback value on failure</param>
    /// <param name="context">Execution context</param>
    /// <returns>Result or fallback value</returns>
    public T TryRecover<T>(Func<T> operation, T fallbackValue, ExecutionContext? context = null)
    {
        try
        {
            return operation();
        }
        catch (Exception ex)
        {
            LogException(ex, context);
            
            if (TryHandleError(ex, out var recoveredValue) && recoveredValue is T result)
            {
                Interlocked.Increment(ref _recoveredErrors);
                return result;
            }
            
            return fallbackValue;
        }
    }
    
    /// <summary>
    /// Register custom error handler
    /// </summary>
    /// <param name="errorType">Error type or pattern</param>
    /// <param name="handler">Error handler</param>
    public void RegisterErrorHandler(string errorType, ErrorHandler handler)
    {
        _errorHandlers[errorType] = handler;
    }
    
    /// <summary>
    /// Get current call stack
    /// </summary>
    /// <returns>Array of exception frames</returns>
    public ExceptionFrame[] GetCallStack()
    {
        return _exceptionStack.ToArray();
    }
    
    /// <summary>
    /// Get exception handling statistics
    /// </summary>
    /// <returns>Exception statistics</returns>
    public ExceptionStats GetStats()
    {
        return new ExceptionStats
        {
            ExceptionsHandled = _exceptionsHandled,
            RecoveredErrors = _recoveredErrors,
            CallStackDepth = (uint)_exceptionStack.Count,
            ExecutionTime = (ulong)_executionTimer.ElapsedMilliseconds,
            ExceptionTypeBreakdown = new Dictionary<string, ulong>(_exceptionTypeCount.ToDictionary(kvp => kvp.Key.Name, kvp => kvp.Value)),
            ErrorHandlerCount = (uint)_errorHandlers.Count
        };
    }
    
    /// <summary>
    /// Clear exception statistics
    /// </summary>
    public void ClearStats()
    {
        _exceptionsHandled = 0;
        _recoveredErrors = 0;
        _exceptionTypeCount.Clear();
    }
    
    private ExceptionFrame PushFrame(string operation, ExecutionContext? context)
    {
        var frame = new ExceptionFrame
        {
            Operation = operation,
            Context = context,
            StartTime = (ulong)_executionTimer.ElapsedMilliseconds,
            CallDepth = (uint)_exceptionStack.Count
        };
        
        _exceptionStack.Push(frame);
        return frame;
    }
    
    private void PopFrame()
    {
        if (_exceptionStack.Count > 0)
        {
            var frame = _exceptionStack.Pop();
            frame.EndTime = (ulong)_executionTimer.ElapsedMilliseconds;
        }
    }
    
    private ExecutionResult<T> HandleException<T>(Exception exception, ExceptionFrame frame, string customMessage)
    {
        frame.Exception = exception;
        frame.Success = false;
        frame.EndTime = (ulong)_executionTimer.ElapsedMilliseconds;
        
        Interlocked.Increment(ref _exceptionsHandled);
        TrackExceptionType(exception.GetType());
        
        LogException(exception, frame.Context);
        
        // Try to handle the error
        if (TryHandleError(exception, out var recoveredValue) && recoveredValue is T result)
        {
            Interlocked.Increment(ref _recoveredErrors);
            return ExecutionResult<T>.Success(result, frame.ExecutionTime);
        }
        
        // Create detailed error information
        var error = CreateErrorInfo(exception, frame, customMessage);
        return ExecutionResult<T>.Failure(error, frame.ExecutionTime);
    }
    
    private bool TryHandleError(Exception exception, out object? recoveredValue)
    {
        recoveredValue = null;
        
        // Try specific error handler first
        var exceptionTypeName = exception.GetType().Name;
        if (_errorHandlers.TryGetValue(exceptionTypeName, out var specificHandler))
        {
            return specificHandler.TryHandle(exception, out recoveredValue);
        }
        
        // Try generic handlers
        foreach (var handler in _errorHandlers.Values)
        {
            if (handler.CanHandle(exception) && handler.TryHandle(exception, out recoveredValue))
            {
                return true;
            }
        }
        
        return false;
    }
    
    private ErrorInfo CreateErrorInfo(Exception exception, ExceptionFrame frame, string customMessage)
    {
        var message = !string.IsNullOrEmpty(customMessage) ? customMessage : exception.Message;
        
        return new ErrorInfo
        {
            Type = exception.GetType().Name,
            Message = message,
            StackTrace = exception.StackTrace ?? "",
            Operation = frame.Operation,
            CallDepth = frame.CallDepth,
            ExecutionTime = frame.ExecutionTime,
            Context = GetContextInfo(frame.Context),
            Data = GetExceptionData(exception),
            Severity = GetSeverityLevel(exception)
        };
    }
    
    private void LogException(Exception exception, ExecutionContext? context)
    {
        if (_config.EnableLogging)
        {
            var contextInfo = GetContextInfo(context);
            var logMessage = $"Exception: {exception.GetType().Name} - {exception.Message}";
            
            if (!string.IsNullOrEmpty(contextInfo.ContractHash))
            {
                logMessage += $" | Contract: {contextInfo.ContractHash}";
            }
            
            if (_config.LogStackTrace)
            {
                logMessage += $" | Stack: {exception.StackTrace}";
            }
            
            TryLog(logMessage);
        }
    }
    
    private ContextInfo GetContextInfo(ExecutionContext? context)
    {
        if (context == null)
        {
            return new ContextInfo();
        }
        
        return new ContextInfo
        {
            ContractHash = context.Msg.Sender.ToString(),
            CallDepth = 0,
            GasConsumed = 0,
            ExecutionTime = 0
        };
    }
    
    private Dictionary<string, object> GetExceptionData(Exception exception)
    {
        var data = new Dictionary<string, object>();
        
        if (exception.Data.Count > 0)
        {
            foreach (var key in exception.Data.Keys)
            {
                if (key != null)
                {
                    data[key.ToString()!] = exception.Data[key] ?? "";
                }
            }
        }
        
        // Add EVM-specific data
        switch (exception)
        {
            case EvmRevertException revertEx:
                data["revertData"] = Convert.ToHexString(revertEx.RevertData);
                break;
            case EvmOutOfGasException gasEx:
                data["gasUsed"] = gasEx.GasUsed;
                data["gasLimit"] = gasEx.GasLimit;
                break;
            case EvmStackOverflowException stackEx:
                data["stackDepth"] = stackEx.StackDepth;
                data["maxDepth"] = stackEx.MaxDepth;
                break;
        }
        
        return data;
    }
    
    private ErrorSeverity GetSeverityLevel(Exception exception)
    {
        return exception switch
        {
            EvmRevertException => ErrorSeverity.Expected,
            EvmAssertException => ErrorSeverity.Critical,
            EvmOutOfGasException => ErrorSeverity.High,
            ArgumentException => ErrorSeverity.Medium,
            InvalidOperationException => ErrorSeverity.Medium,
            UnauthorizedAccessException => ErrorSeverity.High,
            OutOfMemoryException => ErrorSeverity.Critical,
            StackOverflowException => ErrorSeverity.Critical,
            _ => ErrorSeverity.Low
        };
    }
    
    private void TrackExceptionType(Type exceptionType)
    {
        if (_exceptionTypeCount.ContainsKey(exceptionType))
        {
            _exceptionTypeCount[exceptionType]++;
        }
        else
        {
            _exceptionTypeCount[exceptionType] = 1;
        }
    }
    
    private byte[] EncodeRevertReason(string reason)
    {
        // Encode as Error(string) ABI format
        var errorSignature = "Error(string)";
        return AbiEncoder.EncodeCall(errorSignature, reason);
    }
    
    private void RegisterStandardErrorHandlers()
    {
        // Register handler for common gas exhaustion
        RegisterErrorHandler("EvmOutOfGasException", new GasExhaustionHandler());
        
        // Register handler for memory issues
        RegisterErrorHandler("OutOfMemoryException", new MemoryHandler());
        
        // Register handler for arithmetic errors
        RegisterErrorHandler("OverflowException", new ArithmeticHandler());
        RegisterErrorHandler("DivideByZeroException", new ArithmeticHandler());
        
        // Register handler for access violations
        RegisterErrorHandler("UnauthorizedAccessException", new AccessHandler());
    }
    
    public void Dispose()
    {
        if (!_disposed)
        {
            _executionTimer.Stop();
            _exceptionStack.Clear();
            _errorHandlers.Clear();
            _exceptionTypeCount.Clear();
            _disposed = true;
        }
    }
}

/// <summary>
/// EVM-specific exception types
/// </summary>

/// <summary>
/// Exception thrown when EVM revert is executed
/// </summary>
public sealed class EvmRevertException : Exception
{
    public byte[] RevertData { get; }
    
    public EvmRevertException(string message, byte[] revertData) : base(message)
    {
        RevertData = revertData;
    }
}

/// <summary>
/// Exception thrown when EVM assert fails
/// </summary>
public sealed class EvmAssertException : Exception
{
    public EvmAssertException(string message) : base(message) { }
}

/// <summary>
/// Exception thrown when running out of gas
/// </summary>
public sealed class EvmOutOfGasException : Exception
{
    public ulong GasUsed { get; }
    public ulong GasLimit { get; }
    
    public EvmOutOfGasException(ulong gasUsed, ulong gasLimit) 
        : base($"Out of gas: used {gasUsed}, limit {gasLimit}")
    {
        GasUsed = gasUsed;
        GasLimit = gasLimit;
    }
}

/// <summary>
/// Exception thrown on stack overflow
/// </summary>
public sealed class EvmStackOverflowException : Exception
{
    public uint StackDepth { get; }
    public uint MaxDepth { get; }
    
    public EvmStackOverflowException(uint stackDepth, uint maxDepth)
        : base($"Stack overflow: depth {stackDepth}, max {maxDepth}")
    {
        StackDepth = stackDepth;
        MaxDepth = maxDepth;
    }
}

/// <summary>
/// Exception thrown when call depth exceeded
/// </summary>
public sealed class EvmCallDepthException : Exception
{
    public uint CallDepth { get; }
    public uint MaxCallDepth { get; }
    
    public EvmCallDepthException(uint callDepth, uint maxCallDepth)
        : base($"Call depth exceeded: {callDepth} > {maxCallDepth}")
    {
        CallDepth = callDepth;
        MaxCallDepth = maxCallDepth;
    }
}

/// <summary>
/// Data structures for exception handling
/// </summary>

/// <summary>
/// Exception frame for call stack tracking
/// </summary>
public sealed class ExceptionFrame
{
    public string Operation { get; set; } = "";
    public ExecutionContext? Context { get; set; }
    public ulong StartTime { get; set; }
    public ulong EndTime { get; set; }
    public uint CallDepth { get; set; }
    public Exception? Exception { get; set; }
    public bool Success { get; set; }
    
    public ulong ExecutionTime => EndTime > StartTime ? EndTime - StartTime : 0;
}

/// <summary>
/// Comprehensive error information
/// </summary>
public sealed record ErrorInfo
{
    public string Type { get; init; } = "";
    public string Message { get; init; } = "";
    public string StackTrace { get; init; } = "";
    public string Operation { get; init; } = "";
    public uint CallDepth { get; init; }
    public ulong ExecutionTime { get; init; }
    public ContextInfo Context { get; init; } = new();
    public Dictionary<string, object> Data { get; init; } = new();
    public ErrorSeverity Severity { get; init; }
}

/// <summary>
/// Execution context information
/// </summary>
public sealed record ContextInfo
{
    public string ContractHash { get; init; } = "";
    public uint CallDepth { get; init; }
    public ulong GasConsumed { get; init; }
    public ulong ExecutionTime { get; init; }
}

/// <summary>
/// Exception handling statistics
/// </summary>
public sealed record ExceptionStats
{
    public ulong ExceptionsHandled { get; init; }
    public ulong RecoveredErrors { get; init; }
    public uint CallStackDepth { get; init; }
    public ulong ExecutionTime { get; init; }
    public Dictionary<string, ulong> ExceptionTypeBreakdown { get; init; } = new();
    public uint ErrorHandlerCount { get; init; }
}

/// <summary>
/// Execution result with error handling
/// </summary>
public sealed record ExecutionResult<T>
{
    public bool IsSuccess { get; init; }
    public T? Value { get; init; }
    public ErrorInfo? Error { get; init; }
    public ulong ExecutionTime { get; init; }
    
    public static ExecutionResult<T> Success(T value, ulong executionTime = 0)
        => new() { IsSuccess = true, Value = value, ExecutionTime = executionTime };
        
    public static ExecutionResult<T> Failure(ErrorInfo error, ulong executionTime = 0)
        => new() { IsSuccess = false, Error = error, ExecutionTime = executionTime };
}

/// <summary>
/// Error severity levels
/// </summary>
public enum ErrorSeverity
{
    Low,        // Minor issues, easily recoverable
    Medium,     // Moderate issues, may need attention
    High,       // Serious issues, likely to impact execution
    Critical,   // Severe issues, execution cannot continue
    Expected    // Expected errors like revert
}

/// <summary>
/// Configuration for exception handling
/// </summary>
public sealed record ExceptionConfig
{
    public bool EnableLogging { get; init; } = true;
    public bool LogStackTrace { get; init; } = true;
    public bool EnableRecovery { get; init; } = true;
    public uint MaxCallDepth { get; init; } = 1024;
    public uint MaxStackSize { get; init; } = 1024;
    
    public static ExceptionConfig Default => new();
}

/// <summary>
/// Base error handler interface
/// </summary>
public abstract class ErrorHandler
{
    public abstract bool CanHandle(Exception exception);
    public abstract bool TryHandle(Exception exception, out object? recoveredValue);
}

/// <summary>
/// Standard error handlers
/// </summary>

public sealed class GasExhaustionHandler : ErrorHandler
{
    public override bool CanHandle(Exception exception)
        => exception is EvmOutOfGasException;
        
    public override bool TryHandle(Exception exception, out object? recoveredValue)
    {
        recoveredValue = null;
        
        if (exception is EvmOutOfGasException gasEx)
        {
            // Log gas exhaustion but cannot recover
            try
            {
                NeoFrameworkRuntime.Log($"Gas exhausted: {gasEx.GasUsed}/{gasEx.GasLimit}");
            }
            catch
            {
            }
            return false;
        }
        
        return false;
    }
}

public sealed class MemoryHandler : ErrorHandler
{
    public override bool CanHandle(Exception exception)
        => exception is OutOfMemoryException;
        
    public override bool TryHandle(Exception exception, out object? recoveredValue)
    {
        recoveredValue = null;
        
        // Trigger garbage collection
        GC.Collect();
        GC.WaitForPendingFinalizers();
        
        try
        {
            NeoFrameworkRuntime.Log("Memory exhausted - triggered GC");
        }
        catch
        {
        }
        return false;
    }
}

public sealed class ArithmeticHandler : ErrorHandler
{
    public override bool CanHandle(Exception exception)
        => exception is OverflowException || exception is DivideByZeroException;
        
    public override bool TryHandle(Exception exception, out object? recoveredValue)
    {
        recoveredValue = exception switch
        {
            OverflowException => BigInteger.Zero,
            DivideByZeroException => BigInteger.Zero,
            _ => null
        };
        
        return recoveredValue != null;
    }
}

public sealed class AccessHandler : ErrorHandler
{
    public override bool CanHandle(Exception exception)
        => exception is UnauthorizedAccessException;
        
        public override bool TryHandle(Exception exception, out object? recoveredValue)
        {
            recoveredValue = null;
            try
            {
                NeoFrameworkRuntime.Log($"Access denied: {exception.Message}");
            }
            catch
            {
            }
            return false;
        }
}
