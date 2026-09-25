using FrameworkUInt160 = Neo.SmartContract.Framework.UInt160;
using FrameworkUInt256 = Neo.SmartContract.Framework.UInt256;

namespace Neo.Sol.Runtime;

internal static class NeoTypeConversions
{
    public static UInt160 ToCoreUInt160(FrameworkUInt160 value) => new((byte[])value);

    public static UInt256 ToCoreUInt256(FrameworkUInt256 value) => new((byte[])value);

    public static FrameworkUInt160 ToFrameworkUInt160(UInt160 value) => (FrameworkUInt160)ToByteArray(value);

    public static FrameworkUInt256 ToFrameworkUInt256(UInt256 value) => (FrameworkUInt256)ToByteArray(value);

    public static byte[] ToByteArray(UInt160 value)
    {
        var hex = value.ToString();
        return Convert.FromHexString(hex.StartsWith("0x", StringComparison.OrdinalIgnoreCase) ? hex[2..] : hex);
    }

    public static byte[] ToByteArray(UInt256 value)
    {
        var hex = value.ToString();
        return Convert.FromHexString(hex.StartsWith("0x", StringComparison.OrdinalIgnoreCase) ? hex[2..] : hex);
    }
}
