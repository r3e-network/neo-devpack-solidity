using System.Numerics;
using System.Text;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Neo.Sol.Runtime;
using Neo.Sol.Runtime.ABI;

namespace Neo.Sol.Runtime.Tests.ABI;

[TestClass]
public class AbiEncoderTests
{
    #region Function Selector Tests

    [TestMethod]
    public void CalculateFunctionSelector_Transfer_ReturnsExpectedSelector()
    {
        // Known Ethereum selector for "transfer(address,uint256)"
        var selector = AbiEncoder.CalculateFunctionSelector("transfer(address,uint256)");

        Assert.AreEqual(4, selector.Length);
        // Expected: 0xa9059cbb
        Assert.AreEqual(0xa9, selector[0]);
        Assert.AreEqual(0x05, selector[1]);
        Assert.AreEqual(0x9c, selector[2]);
        Assert.AreEqual(0xbb, selector[3]);
    }

    [TestMethod]
    public void CalculateFunctionSelector_BalanceOf_ReturnsExpectedSelector()
    {
        // Known Ethereum selector for "balanceOf(address)"
        var selector = AbiEncoder.CalculateFunctionSelector("balanceOf(address)");

        Assert.AreEqual(4, selector.Length);
        // Expected: 0x70a08231
        Assert.AreEqual(0x70, selector[0]);
        Assert.AreEqual(0xa0, selector[1]);
        Assert.AreEqual(0x82, selector[2]);
        Assert.AreEqual(0x31, selector[3]);
    }

    [TestMethod]
    public void CalculateFunctionSelector_Approve_ReturnsExpectedSelector()
    {
        // Known Ethereum selector for "approve(address,uint256)"
        var selector = AbiEncoder.CalculateFunctionSelector("approve(address,uint256)");

        Assert.AreEqual(4, selector.Length);
        // Expected: 0x095ea7b3
        Assert.AreEqual(0x09, selector[0]);
        Assert.AreEqual(0x5e, selector[1]);
        Assert.AreEqual(0xa7, selector[2]);
        Assert.AreEqual(0xb3, selector[3]);
    }

    #endregion

    #region Static Type Encoding Tests - Unsigned Integers

    [TestMethod]
    public void EncodeUint256_Zero_ReturnsZeroPaddedWord()
    {
        var encoded = AbiEncoder.EncodeUint256(0);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.All(b => b == 0));
    }

    [TestMethod]
    public void EncodeUint256_MaxUint8_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeUint256(255);

        Assert.AreEqual(32, encoded.Length);
        Assert.AreEqual(255, encoded[31]);
        Assert.IsTrue(encoded.Take(31).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeUint256_MaxUint16_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeUint256(65535);

        Assert.AreEqual(32, encoded.Length);
        Assert.AreEqual(0xFF, encoded[30]);
        Assert.AreEqual(0xFF, encoded[31]);
        Assert.IsTrue(encoded.Take(30).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeUint256_MaxUint32_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeUint256(4294967295);

        Assert.AreEqual(32, encoded.Length);
        Assert.AreEqual(0xFF, encoded[28]);
        Assert.AreEqual(0xFF, encoded[29]);
        Assert.AreEqual(0xFF, encoded[30]);
        Assert.AreEqual(0xFF, encoded[31]);
        Assert.IsTrue(encoded.Take(28).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeUint256_MaxUint64_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeUint256(ulong.MaxValue);

        Assert.AreEqual(32, encoded.Length);
        for (int i = 24; i < 32; i++)
        {
            Assert.AreEqual(0xFF, encoded[i]);
        }
        Assert.IsTrue(encoded.Take(24).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeUint256_Max256Bit_EncodesCorrectly()
    {
        var max256 = BigInteger.Pow(2, 256) - 1;
        var encoded = AbiEncoder.EncodeUint256(max256);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.All(b => b == 0xFF));
    }

    #endregion

    #region Static Type Encoding Tests - Signed Integers

    [TestMethod]
    public void EncodeInt256_Zero_ReturnsZeroPaddedWord()
    {
        var encoded = AbiEncoder.EncodeInt256(0);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.All(b => b == 0));
    }

    [TestMethod]
    public void EncodeInt256_PositiveValue_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeInt256(42);

        Assert.AreEqual(32, encoded.Length);
        Assert.AreEqual(42, encoded[31]);
        Assert.IsTrue(encoded.Take(31).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeInt256_NegativeOne_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeInt256(-1);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.All(b => b == 0xFF));
    }

    [TestMethod]
    public void EncodeInt256_NegativeValue_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeInt256(-42);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.Take(31).All(b => b == 0xFF));
        Assert.AreEqual(0xD6, encoded[31]);
    }

    [TestMethod]
    public void EncodeInt256_MinInt64_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeInt256(long.MinValue);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.Take(24).All(b => b == 0xFF));
        Assert.AreEqual(0x80, encoded[24]);
        Assert.IsTrue(encoded.Skip(25).All(b => b == 0));
    }

    #endregion

    #region Boolean and Address Tests

    [TestMethod]
    public void EncodeParameters_BoolTrue_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters(true);

        Assert.AreEqual(32, encoded.Length);
        Assert.AreEqual(1, encoded[31]);
        Assert.IsTrue(encoded.Take(31).All(b => b == 0));
    }

    [TestMethod]
    public void EncodeParameters_BoolFalse_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters(false);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.All(b => b == 0));
    }

    [TestMethod]
    public void EncodeAddress_ValidAddress_EncodesCorrectly()
    {
        var address = UInt160.Parse("0x1234567890123456789012345678901234567890");
        var encoded = AbiEncoder.EncodeAddress(address);

        Assert.AreEqual(32, encoded.Length);
        Assert.IsTrue(encoded.Take(12).All(b => b == 0));
        Assert.AreEqual(20, encoded.Skip(12).Count());
    }

    #endregion

    #region Dynamic Type Tests - Strings

    [TestMethod]
    public void EncodeParameters_EmptyString_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters("");

        Assert.AreEqual(64, encoded.Length);
        Assert.AreEqual(0x20, encoded[31]);
        Assert.AreEqual(0, encoded[63]);
    }

    [TestMethod]
    public void EncodeParameters_AsciiString_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters("Hello");

        Assert.AreEqual(96, encoded.Length);
        Assert.AreEqual(0x20, encoded[31]);
        Assert.AreEqual(5, encoded[63]);
        Assert.AreEqual((byte)'H', encoded[64]);
    }

    [TestMethod]
    public void EncodeParameters_UnicodeString_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters("Hello 世界");
        var utf8Bytes = Encoding.UTF8.GetBytes("Hello 世界");

        var expectedLength = 32 + 32 + ((utf8Bytes.Length + 31) / 32) * 32;
        Assert.AreEqual(expectedLength, encoded.Length);
        Assert.AreEqual(utf8Bytes.Length, encoded[63]);
    }

    [TestMethod]
    public void EncodeParameters_LongString_EncodesCorrectly()
    {
        var longString = new string('A', 100);
        var encoded = AbiEncoder.EncodeParameters(longString);

        Assert.AreEqual(32 + 32 + 128, encoded.Length);
        Assert.AreEqual(100, encoded[63]);
    }

    #endregion

    #region Dynamic Type Tests - Bytes

    [TestMethod]
    public void EncodeParameters_EmptyBytes_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters(Array.Empty<byte>());

        Assert.AreEqual(64, encoded.Length);
        Assert.AreEqual(0x20, encoded[31]);
        Assert.AreEqual(0, encoded[63]);
    }

    [TestMethod]
    public void EncodeParameters_SingleByte_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters(new byte[] { 0x42 });

        Assert.AreEqual(96, encoded.Length);
        Assert.AreEqual(1, encoded[63]);
        Assert.AreEqual(0x42, encoded[64]);
    }

    [TestMethod]
    public void EncodeParameters_32ByteArray_EncodesCorrectly()
    {
        var data = Enumerable.Range(0, 32).Select(i => (byte)i).ToArray();
        var encoded = AbiEncoder.EncodeParameters(data);

        Assert.AreEqual(96, encoded.Length);
        Assert.AreEqual(32, encoded[63]);
        for (int i = 0; i < 32; i++)
        {
            Assert.AreEqual((byte)i, encoded[64 + i]);
        }
    }

    #endregion

    #region Array Tests

    [TestMethod]
    public void EncodeParameters_HomogeneousUintArray_EncodesCorrectly()
    {
        var array = new object[] { (uint)1, (uint)2, (uint)3 };
        var encoded = AbiEncoder.EncodeParameters(array);

        Assert.AreEqual(32 + 32 + 96, encoded.Length);
        Assert.AreEqual(0x20, encoded[31]);
        Assert.AreEqual(3, encoded[63]);
    }

    #endregion

    #region Multiple Parameters Tests

    [TestMethod]
    public void EncodeParameters_MixedStaticTypes_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters((uint)42, true, (ulong)1000);

        Assert.AreEqual(96, encoded.Length);
        Assert.AreEqual(42, encoded[31]);
        Assert.AreEqual(1, encoded[63]);
    }

    [TestMethod]
    public void EncodeParameters_MixedStaticAndDynamicTypes_EncodesCorrectly()
    {
        var encoded = AbiEncoder.EncodeParameters((uint)123, "test");

        Assert.AreEqual(128, encoded.Length);
        Assert.AreEqual(123, encoded[31]);
        Assert.AreEqual(0x40, encoded[63]);
    }

    #endregion

    #region Round-Trip Tests

    [TestMethod]
    public void RoundTrip_Uint256_PreservesValue()
    {
        var original = new BigInteger(12345);
        var encoded = AbiEncoder.EncodeUint256(original);
        var decoded = AbiDecoder.DecodeUint256(encoded);

        Assert.AreEqual(original, decoded);
    }

    [TestMethod]
    public void RoundTrip_Int256Positive_PreservesValue()
    {
        var original = new BigInteger(54321);
        var encoded = AbiEncoder.EncodeInt256(original);
        var decoded = AbiDecoder.DecodeInt256(encoded);

        Assert.AreEqual(original, decoded);
    }

    [TestMethod]
    public void RoundTrip_Int256Negative_PreservesValue()
    {
        var original = new BigInteger(-12345);
        var encoded = AbiEncoder.EncodeInt256(original);
        var decoded = AbiDecoder.DecodeInt256(encoded);

        Assert.AreEqual(original, decoded);
    }

    [TestMethod]
    public void RoundTrip_Bool_PreservesValue()
    {
        var encodedTrue = AbiEncoder.EncodeParameters(true);
        var decodedTrue = AbiDecoder.DecodeBool(encodedTrue);
        Assert.IsTrue(decodedTrue);

        var encodedFalse = AbiEncoder.EncodeParameters(false);
        var decodedFalse = AbiDecoder.DecodeBool(encodedFalse);
        Assert.IsFalse(decodedFalse);
    }

    [TestMethod]
    public void RoundTrip_Address_PreservesValue()
    {
        var original = UInt160.Parse("0xABCDEF1234567890ABCDEF1234567890ABCDEF12");
        var encoded = AbiEncoder.EncodeAddress(original);
        var decoded = AbiDecoder.DecodeAddress(encoded);

        Assert.AreEqual(original, decoded);
    }

    [TestMethod]
    public void RoundTrip_String_PreservesValue()
    {
        var original = "Hello, Ethereum ABI!";
        var encoded = AbiEncoder.EncodeParameters(original);
        var decoded = AbiDecoder.DecodeString(encoded);

        Assert.AreEqual(original, decoded);
    }

    [TestMethod]
    public void RoundTrip_Bytes_PreservesValue()
    {
        var original = new byte[] { 1, 2, 3, 4, 5, 0xFF, 0xAB, 0xCD };
        var encoded = AbiEncoder.EncodeParameters(original);
        var decoded = AbiDecoder.DecodeBytes(encoded);

        CollectionAssert.AreEqual(original, decoded);
    }

    #endregion

    #region Edge Cases and Security Tests

    [TestMethod]
    public void DecodeBytes_MalformedPointer_ThrowsException()
    {
        var data = new byte[32];
        for (int i = 0; i < 32; i++)
        {
            data[i] = 0xFF;
        }

        Assert.ThrowsException<ArgumentException>(() => AbiDecoder.DecodeBytes(data));
    }

    [TestMethod]
    public void DecodeBytes_OversizedLength_ThrowsException()
    {
        var data = new byte[96];
        data[31] = 0x20;
        for (int i = 32; i < 64; i++)
        {
            data[i] = 0xFF;
        }

        Assert.ThrowsException<ArgumentException>(() => AbiDecoder.DecodeBytes(data));
    }

    [TestMethod]
    public void DecodeBytes_InsufficientData_ThrowsException()
    {
        var data = new byte[64];
        data[31] = 0x64;

        Assert.ThrowsException<ArgumentException>(() => AbiDecoder.DecodeBytes(data));
    }

    [TestMethod]
    public void DecodeUint256_InsufficientData_ThrowsException()
    {
        var data = new byte[16];

        Assert.ThrowsException<ArgumentException>(() => AbiDecoder.DecodeUint256(data));
    }

    [TestMethod]
    public void EncodeCall_CompleteFunction_ProducesValidCallData()
    {
        var address = UInt160.Parse("0x1234567890123456789012345678901234567890");
        var amount = new BigInteger(1000);

        var callData = AbiEncoder.EncodeCall("transfer(address,uint256)", address, amount);

        Assert.IsTrue(callData.Length > 4);
        Assert.AreEqual(0xa9, callData[0]);
        Assert.AreEqual(0x05, callData[1]);
        Assert.AreEqual(0x9c, callData[2]);
        Assert.AreEqual(0xbb, callData[3]);
    }

    #endregion
}
