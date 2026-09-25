using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("Achievement")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-5114 soulbound badge — bound to parent NFT, in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class Achievement : SmartContract
{
    private const byte Prefix_ParentNft = 0x01;
    private const byte Prefix_ParentTok = 0x02;
    private static readonly byte[] OwnerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    public static ByteString Attach(UInt160 parentNft, ByteString parentTokenId)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        var raw = Storage.Get(NextIdKey);
        BigInteger n = raw is null ? 1 : (BigInteger)raw + 1;
        Storage.Put(NextIdKey, n);
        ByteString badgeId = (ByteString)n.ToByteArray();
        Storage.Put(Helper.Concat(new byte[] { Prefix_ParentNft }, badgeId), parentNft);
        Storage.Put(Helper.Concat(new byte[] { Prefix_ParentTok }, badgeId), parentTokenId);
        return badgeId;
    }

    [Safe]
    public static (UInt160, ByteString) ParentOf(ByteString badgeId)
    {
        var nft = (UInt160)Storage.Get(Helper.Concat(new byte[] { Prefix_ParentNft }, badgeId));
        var tok = Storage.Get(Helper.Concat(new byte[] { Prefix_ParentTok }, badgeId));
        if (nft is null) throw new System.Exception("badge not attached");
        return (nft, tok);
    }

    [Safe]
    public static BigInteger BadgeCount()
    {
        var raw = Storage.Get(NextIdKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
