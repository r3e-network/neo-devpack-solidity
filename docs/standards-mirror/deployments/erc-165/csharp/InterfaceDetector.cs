using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("InterfaceDetector")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-165 supportsInterface compatibility shim, in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class InterfaceDetector : SmartContract
{
    [Safe] public static BigInteger IdErc165() => 0x01ffc9a7;
    [Safe] public static BigInteger IdErc721() => 0x80ac58cd;
    [Safe] public static BigInteger IdErc1155() => 0xd9b67a26;
    [Safe] public static BigInteger IdErc20Like() => 0x36372b07;

    [Safe]
    public static bool SupportsInterface(BigInteger id)
    {
        if (id == 0x01ffc9a7) return true;
        if (id == 0x80ac58cd) return true;
        if (id == 0xd9b67a26) return true;
        if (id == 0x36372b07) return true;
        return false;
    }
}
