using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("SoulboundCert")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-5192 minimal soulbound NFT in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class SoulboundCert : SmartContract
{
    private const byte Prefix_Owner    = 0x01;
    private const byte Prefix_Locked   = 0x02;
    private static readonly byte[] IssuerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [Safe] public static string Symbol() => "SBC";

    [Safe]
    public static bool Locked(ByteString tokenId)
        => Storage.Get(Helper.Concat(new byte[] { Prefix_Locked }, tokenId)) != null;

    [Safe]
    public static UInt160 OwnerOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        if (raw is null) throw new System.Exception("nonexistent");
        return (UInt160)raw;
    }

    public static ByteString Issue(UInt160 to, bool soulbound)
    {
        var issuer = (UInt160)Storage.Get(IssuerKey);
        if (!Runtime.CheckWitness(issuer)) throw new System.Exception("issuer only");

        var raw = Storage.Get(NextIdKey);
        BigInteger n = raw is null ? 1 : (BigInteger)raw + 1;
        Storage.Put(NextIdKey, n);
        ByteString tokenId = (ByteString)n.ToByteArray();

        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, tokenId), to);
        if (soulbound)
            Storage.Put(Helper.Concat(new byte[] { Prefix_Locked }, tokenId), 1);
        return tokenId;
    }

    public static void Burn(ByteString tokenId)
    {
        var owner = OwnerOf(tokenId);
        var issuer = (UInt160)Storage.Get(IssuerKey);
        bool authorized = Runtime.CheckWitness(owner) || Runtime.CheckWitness(issuer);
        if (!authorized) throw new System.Exception("not authorized");
        Storage.Delete(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        Storage.Delete(Helper.Concat(new byte[] { Prefix_Locked }, tokenId));
    }

    public static void Unlock(ByteString tokenId)
    {
        var issuer = (UInt160)Storage.Get(IssuerKey);
        if (!Runtime.CheckWitness(issuer)) throw new System.Exception("issuer only");
        Storage.Delete(Helper.Concat(new byte[] { Prefix_Locked }, tokenId));
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 issuer = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(issuer.IsValid && !issuer.IsZero, "invalid issuer");
        Storage.Put(IssuerKey, issuer);
    }
}
