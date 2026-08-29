using System.Numerics;
using System.Text;
using System.Collections.Generic;
using Neo.Sol.Runtime;
using Neo.Sol.Runtime.Crypto;

namespace Neo.Sol.Runtime.ABI;

/// <summary>
/// Ethereum ABI encoding implementation
/// Provides encoding/decoding of function calls and return values
/// </summary>
public static class AbiEncoder
{
    private const int WORD_SIZE = 32;
    
    /// <summary>
    /// Encode function call data
    /// </summary>
    /// <param name="functionSignature">Function signature (e.g., "transfer(address,uint256)")</param>
    /// <param name="parameters">Function parameters</param>
    /// <returns>Encoded call data</returns>
    public static byte[] EncodeCall(string functionSignature, params object[] parameters)
    {
        var selector = CalculateFunctionSelector(functionSignature);
        var encodedParams = EncodeParameters(parameters);
        
        var result = new byte[4 + encodedParams.Length];
        Array.Copy(selector, 0, result, 0, 4);
        Array.Copy(encodedParams, 0, result, 4, encodedParams.Length);
        
        return result;
    }
    
    /// <summary>
    /// Calculate function selector (first 4 bytes of keccak256 hash)
    /// </summary>
    /// <param name="functionSignature">Function signature</param>
    /// <returns>4-byte selector</returns>
    public static byte[] CalculateFunctionSelector(string functionSignature)
    {
        var hash = CryptoLib.Keccak256(Encoding.UTF8.GetBytes(functionSignature));
        var selector = new byte[4];
        Array.Copy(hash, 0, selector, 0, 4);
        return selector;
    }
    
    /// <summary>
    /// Encode multiple parameters according to ABI specification
    /// </summary>
    /// <param name="parameters">Parameters to encode</param>
    /// <returns>Encoded parameter data</returns>
    public static byte[] EncodeParameters(params object[] parameters)
    {
        if (parameters.Length == 0) return Array.Empty<byte>();
        
        var headParts = new List<byte[]>();
        var tailParts = new List<byte[]>();
        var currentTailOffset = parameters.Length * WORD_SIZE;
        
        foreach (var param in parameters)
        {
            var (head, tail) = EncodeParameter(param, currentTailOffset);
            headParts.Add(head);
            tailParts.Add(tail);
            currentTailOffset += tail.Length;
        }
        
        // Combine head and tail parts
        var totalLength = headParts.Sum(p => p.Length) + tailParts.Sum(p => p.Length);
        var result = new byte[totalLength];
        var offset = 0;
        
        foreach (var head in headParts)
        {
            Array.Copy(head, 0, result, offset, head.Length);
            offset += head.Length;
        }
        
        foreach (var tail in tailParts)
        {
            Array.Copy(tail, 0, result, offset, tail.Length);
            offset += tail.Length;
        }
        
        return result;
    }
    
    /// <summary>
    /// Encode a single parameter
    /// </summary>
    /// <param name="parameter">Parameter to encode</param>
    /// <param name="tailOffset">Current tail offset for dynamic types</param>
    /// <returns>Head and tail parts</returns>
    private static (byte[] head, byte[] tail) EncodeParameter(object parameter, int tailOffset)
    {
        return parameter switch
        {
            bool b => (EncodeUint256(b ? 1 : 0), Array.Empty<byte>()),
            byte b => (EncodeUint256(b), Array.Empty<byte>()),
            ushort u => (EncodeUint256(u), Array.Empty<byte>()),
            uint u => (EncodeUint256(u), Array.Empty<byte>()),
            ulong u => (EncodeUint256(u), Array.Empty<byte>()),
            BigInteger bi => (EncodeUint256(bi), Array.Empty<byte>()),
            int i => (EncodeInt256(i), Array.Empty<byte>()),
            long l => (EncodeInt256(l), Array.Empty<byte>()),
            string s => EncodeString(s, tailOffset),
            byte[] bytes => EncodeBytes(bytes, tailOffset),
            UInt160 addr => (EncodeAddress(addr), Array.Empty<byte>()),
            object[] array => EncodeArray(array, tailOffset),
            _ => throw new ArgumentException($"Unsupported parameter type: {parameter.GetType()}")
        };
    }
    
    /// <summary>
    /// Encode uint256 value
    /// </summary>
    /// <param name="value">Value to encode</param>
    /// <returns>32-byte encoded value</returns>
    public static byte[] EncodeUint256(BigInteger value)
    {
        var bytes = value.ToByteArray(isUnsigned: true, isBigEndian: true);
        var result = new byte[WORD_SIZE];
        
        if (bytes.Length <= WORD_SIZE)
        {
            Array.Copy(bytes, 0, result, WORD_SIZE - bytes.Length, bytes.Length);
        }
        else
        {
            Array.Copy(bytes, bytes.Length - WORD_SIZE, result, 0, WORD_SIZE);
        }
        
        return result;
    }
    
    /// <summary>
    /// Encode int256 value
    /// </summary>
    /// <param name="value">Value to encode</param>
    /// <returns>32-byte encoded value</returns>
    public static byte[] EncodeInt256(BigInteger value)
    {
        var bytes = value.ToByteArray(isUnsigned: false, isBigEndian: true);
        var result = new byte[WORD_SIZE];
        
        if (value >= 0)
        {
            // Positive number - pad with zeros
            if (bytes.Length <= WORD_SIZE)
            {
                Array.Copy(bytes, 0, result, WORD_SIZE - bytes.Length, bytes.Length);
            }
            else
            {
                Array.Copy(bytes, bytes.Length - WORD_SIZE, result, 0, WORD_SIZE);
            }
        }
        else
        {
            // Negative number - pad with 0xFF (two's complement)
            if (bytes.Length <= WORD_SIZE)
            {
                for (int i = 0; i < WORD_SIZE - bytes.Length; i++)
                {
                    result[i] = 0xFF;
                }
                Array.Copy(bytes, 0, result, WORD_SIZE - bytes.Length, bytes.Length);
            }
            else
            {
                Array.Copy(bytes, bytes.Length - WORD_SIZE, result, 0, WORD_SIZE);
            }
        }
        
        return result;
    }
    
    /// <summary>
    /// Encode address value
    /// </summary>
    /// <param name="address">160-bit address</param>
    /// <returns>32-byte encoded address</returns>
    public static byte[] EncodeAddress(UInt160 address)
    {
        var result = new byte[WORD_SIZE];
        var addressBytes = NeoTypeConversions.ToByteArray(address);
        Array.Copy(addressBytes, 0, result, WORD_SIZE - 20, 20); // Addresses are 20 bytes
        return result;
    }
    
    /// <summary>
    /// Encode string value (dynamic type)
    /// </summary>
    /// <param name="value">String to encode</param>
    /// <param name="tailOffset">Tail offset for pointer</param>
    /// <returns>Head and tail parts</returns>
    private static (byte[] head, byte[] tail) EncodeString(string value, int tailOffset)
    {
        var stringBytes = Encoding.UTF8.GetBytes(value);
        return EncodeBytes(stringBytes, tailOffset);
    }
    
    /// <summary>
    /// Encode byte array (dynamic type)
    /// </summary>
    /// <param name="value">Bytes to encode</param>
    /// <param name="tailOffset">Tail offset for pointer</param>
    /// <returns>Head and tail parts</returns>
    private static (byte[] head, byte[] tail) EncodeBytes(byte[] value, int tailOffset)
    {
        var head = EncodeUint256(tailOffset);
        
        // Tail: length + data (padded to word boundary)
        var lengthBytes = EncodeUint256(value.Length);
        var paddedLength = ((value.Length + WORD_SIZE - 1) / WORD_SIZE) * WORD_SIZE;
        var paddedData = new byte[paddedLength];
        Array.Copy(value, 0, paddedData, 0, value.Length);
        
        var tail = new byte[lengthBytes.Length + paddedData.Length];
        Array.Copy(lengthBytes, 0, tail, 0, lengthBytes.Length);
        Array.Copy(paddedData, 0, tail, lengthBytes.Length, paddedData.Length);
        
        return (head, tail);
    }
    
    /// <summary>
    /// Encode array (dynamic type)
    /// </summary>
    /// <param name="array">Array to encode</param>
    /// <param name="tailOffset">Tail offset for pointer</param>
    /// <returns>Head and tail parts</returns>
    private static (byte[] head, byte[] tail) EncodeArray(object[] array, int tailOffset)
    {
        var head = EncodeUint256(tailOffset);
        
        // Encode array length
        var lengthBytes = EncodeUint256(array.Length);
        
        // Encode array elements
        var encodedElements = EncodeParameters(array);
        
        var tail = new byte[lengthBytes.Length + encodedElements.Length];
        Array.Copy(lengthBytes, 0, tail, 0, lengthBytes.Length);
        Array.Copy(encodedElements, 0, tail, lengthBytes.Length, encodedElements.Length);
        
        return (head, tail);
    }
}

/// <summary>
/// Ethereum ABI decoder implementation
/// </summary>
public static class AbiDecoder
{
    private const int WORD_SIZE = 32;
    
    /// <summary>
    /// Decode uint256 from bytes
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset in data</param>
    /// <returns>Decoded BigInteger</returns>
    public static BigInteger DecodeUint256(byte[] data, int offset = 0)
    {
        if (data.Length < offset + WORD_SIZE)
            throw new ArgumentException("Insufficient data for uint256");
            
        var word = new byte[WORD_SIZE];
        Array.Copy(data, offset, word, 0, WORD_SIZE);
        return new BigInteger(word, isUnsigned: true, isBigEndian: true);
    }
    
    /// <summary>
    /// Decode int256 from bytes
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset in data</param>
    /// <returns>Decoded BigInteger</returns>
    public static BigInteger DecodeInt256(byte[] data, int offset = 0)
    {
        if (data.Length < offset + WORD_SIZE)
            throw new ArgumentException("Insufficient data for int256");
            
        var word = new byte[WORD_SIZE];
        Array.Copy(data, offset, word, 0, WORD_SIZE);
        return new BigInteger(word, isUnsigned: false, isBigEndian: true);
    }
    
    /// <summary>
    /// Decode address from bytes
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset in data</param>
    /// <returns>Decoded UInt160 address</returns>
    public static UInt160 DecodeAddress(byte[] data, int offset = 0)
    {
        if (data.Length < offset + WORD_SIZE)
            throw new ArgumentException("Insufficient data for address");
            
        var addressBytes = new byte[20];
        Array.Copy(data, offset + 12, addressBytes, 0, 20); // Address is in the last 20 bytes
        return new UInt160(addressBytes);
    }
    
    /// <summary>
    /// Decode boolean from bytes
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset in data</param>
    /// <returns>Decoded boolean</returns>
    public static bool DecodeBool(byte[] data, int offset = 0)
    {
        var value = DecodeUint256(data, offset);
        return value != 0;
    }
    
    /// <summary>
    /// Decode dynamic bytes from encoded data
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset to the pointer</param>
    /// <returns>Decoded bytes</returns>
    public static byte[] DecodeBytes(byte[] data, int offset = 0)
    {
        var pointer = (int)DecodeUint256(data, offset);
        var length = (int)DecodeUint256(data, pointer);
        
        var result = new byte[length];
        Array.Copy(data, pointer + WORD_SIZE, result, 0, length);
        return result;
    }
    
    /// <summary>
    /// Decode string from encoded data
    /// </summary>
    /// <param name="data">Encoded data</param>
    /// <param name="offset">Offset to the pointer</param>
    /// <returns>Decoded string</returns>
    public static string DecodeString(byte[] data, int offset = 0)
    {
        var bytes = DecodeBytes(data, offset);
        return Encoding.UTF8.GetString(bytes);
    }
}
