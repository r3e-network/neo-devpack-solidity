using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("BatchMintNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2309 consecutive-transfer batch mint demo in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class BatchMintNFT : SmartContract
{
    private const byte Prefix_Owner = 0x01;
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [DisplayName("ConsecutiveTransfer")]
    public static event System.Action<BigInteger, BigInteger, UInt160?, UInt160> OnConsecutiveTransfer = null!;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger NextId()
    {
        var raw = Storage.Get(NextIdKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static UInt160 OwnerOf(BigInteger tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, (ByteString)tokenId.ToByteArray()));
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    public static (BigInteger, BigInteger) BatchMint(UInt160 to, BigInteger count)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        ExecutionEngine.Assert(count > 0, "count > 0");
        var current = NextId();
        var fromId = current + 1;
        var toId = current + count;
        for (var i = fromId; i <= toId; i++)
        {
            Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, (ByteString)i.ToByteArray()), to);
        }
        Storage.Put(NextIdKey, toId);
        OnConsecutiveTransfer(fromId, toId, null, to);
        return (fromId, toId);
    }

    [Safe]
    public static BigInteger TotalMinted() => NextId();

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
