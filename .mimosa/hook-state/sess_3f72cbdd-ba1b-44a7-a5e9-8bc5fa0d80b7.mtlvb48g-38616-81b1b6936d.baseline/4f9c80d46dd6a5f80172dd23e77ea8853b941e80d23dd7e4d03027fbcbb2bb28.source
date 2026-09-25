using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("GuardedNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-6147 NFT guard — guard-takes-over delegate model, in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class GuardedNFT : SmartContract
{
    private const byte Prefix_Owner = 0x01;
    private const byte Prefix_Guard = 0x02;
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger TokenCount()
    {
        var raw = Storage.Get(NextIdKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static UInt160 OwnerOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    [Safe]
    public static UInt160 GuardOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Guard }, tokenId));
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    public static ByteString Mint(UInt160 to)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var raw = Storage.Get(NextIdKey);
        BigInteger n = raw is null ? 1 : (BigInteger)raw + 1;
        Storage.Put(NextIdKey, n);
        ByteString tokenId = (ByteString)n.ToByteArray();
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, tokenId), to);
        return tokenId;
    }

    public static void SetGuard(ByteString tokenId, UInt160 guard)
    {
        var owner = OwnerOf(tokenId);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner only");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Guard }, tokenId), guard);
    }

    /// Transfer is gated by whichever account is currently in control:
    /// the guard if set, otherwise the owner.
    public static bool Transfer(UInt160 to, ByteString tokenId)
    {
        var owner = OwnerOf(tokenId);
        ExecutionEngine.Assert(!owner.Equals(UInt160.Zero), "nonexistent");
        var guard = GuardOf(tokenId);
        var required = guard.Equals(UInt160.Zero) ? owner : guard;
        if (!Runtime.CheckWitness(required)) throw new System.Exception("not authorized");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, tokenId), to);
        return true;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
