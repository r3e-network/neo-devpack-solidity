using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("PermitNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-4494 NFT-permit mirror — witness-scope authorization in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class PermitNFT : SmartContract
{
    private const byte Prefix_Owner = 0x01;
    private const byte Prefix_Nonce = 0x02;
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

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

    [Safe]
    public static BigInteger NonceOf(BigInteger tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Nonce }, (ByteString)tokenId.ToByteArray()));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static BigInteger Mint(UInt160 to)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var id = NextId() + 1;
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, (ByteString)id.ToByteArray()), to);
        Storage.Put(NextIdKey, id);
        return id;
    }

    public static void Permit(BigInteger tokenId)
    {
        var owner = OwnerOf(tokenId);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner must sign");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Nonce }, (ByteString)tokenId.ToByteArray()), NonceOf(tokenId) + 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
