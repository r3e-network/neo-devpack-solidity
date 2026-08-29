using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("ConsensualSBT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-5484 consensual soulbound — explicit burn-authorization in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class ConsensualSBT : SmartContract
{
    private const byte Prefix_Owner = 0x01;
    private const byte Prefix_BurnAuth = 0x02;
    private static readonly byte[] IssuerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [Safe]
    public static UInt160 OwnerOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    [Safe]
    public static byte BurnAuthOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_BurnAuth }, tokenId));
        return raw is null ? (byte)0 : (byte)(BigInteger)raw;
    }

    [Safe]
    public static UInt160 GetIssuer() => (UInt160)Storage.Get(IssuerKey);

    public static ByteString Issue(UInt160 to, byte auth)
    {
        if (!Runtime.CheckWitness(GetIssuer())) throw new System.Exception("issuer only");
        ExecutionEngine.Assert(auth <= 3, "bad auth");
        var raw = Storage.Get(NextIdKey);
        BigInteger n = raw is null ? 1 : (BigInteger)raw + 1;
        Storage.Put(NextIdKey, n);
        ByteString tokenId = (ByteString)n.ToByteArray();
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, tokenId), to);
        Storage.Put(Helper.Concat(new byte[] { Prefix_BurnAuth }, tokenId), (BigInteger)auth);
        return tokenId;
    }

    public static void Burn(ByteString tokenId)
    {
        var auth = BurnAuthOf(tokenId);
        var owner = OwnerOf(tokenId);
        var issuer = GetIssuer();
        bool can = false;
        if (auth == 0) can = Runtime.CheckWitness(issuer);
        else if (auth == 1) can = Runtime.CheckWitness(owner);
        else if (auth == 2) can = Runtime.CheckWitness(issuer) || Runtime.CheckWitness(owner);
        if (!can) throw new System.Exception("not authorized");
        Storage.Delete(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        Storage.Delete(Helper.Concat(new byte[] { Prefix_BurnAuth }, tokenId));
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 issuer = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(issuer.IsValid && !issuer.IsZero, "invalid issuer");
        Storage.Put(IssuerKey, issuer);
    }
}
