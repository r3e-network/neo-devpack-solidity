using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

public class RoyaltyNFTState : Nep11TokenState
{
    public string Description = string.Empty;
}

[DisplayName("RoyaltyNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2981 / NEP-24 mirror demo — multi-recipient royalty NFT in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractSourceCode("https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments/erc-2981/csharp")]
[ContractPermission(Permission.Any, Method.OnNEP11Payment)]
[SupportedStandards(NepStandard.Nep11, NepStandard.Nep24)]
public class RoyaltyNFT : Nep11Token<RoyaltyNFTState>
{
    private const byte Prefix_Owner          = 0xff;
    private const byte Prefix_DefaultRoyalty = 0xfa;   // -> serialized [(receiver, bps)]
    private const byte Prefix_TokenRoyalty   = 0xfb;   // tokenId -> serialized [(receiver, bps)]

    public override string Symbol { [Safe] get => "DCSHROY"; }

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    private static bool IsOwner() => Runtime.CheckWitness(GetOwner());

    public static ByteString Mint(UInt160 to, string nameValue, string description)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        ExecutionEngine.Assert(to.IsValid && !to.IsZero, "invalid recipient");

        ByteString tokenId = NewTokenId();
        Nep11Token<RoyaltyNFTState>.Mint(tokenId, new RoyaltyNFTState
        {
            Owner = to,
            Name = nameValue,
            Description = description
        });
        return tokenId;
    }

    public static void SetDefaultRoyalty(object[] recipients)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        BigInteger total = 0;
        foreach (var entry in recipients)
        {
            var pair = (object[])entry;
            total += (BigInteger)pair[1];
        }
        ExecutionEngine.Assert(total <= 10_000, "total bps > 100%");
        Storage.Put(new byte[] { Prefix_DefaultRoyalty }, StdLib.Serialize(recipients));
    }

    public static void SetTokenRoyalty(ByteString tokenId, object[] recipients)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        BigInteger total = 0;
        foreach (var entry in recipients)
        {
            var pair = (object[])entry;
            total += (BigInteger)pair[1];
        }
        ExecutionEngine.Assert(total <= 10_000, "total bps > 100%");
        var key = Helper.Concat(new byte[] { Prefix_TokenRoyalty }, tokenId);
        Storage.Put(key, StdLib.Serialize(recipients));
    }

    /// <summary>
    /// NEP-24 royaltyInfo. Returns array of [receiver, amount] pairs supporting splits.
    /// </summary>
    [Safe]
    public static object[] RoyaltyInfo(ByteString tokenId, UInt160 royaltyToken, BigInteger salePrice)
    {
        var key = Helper.Concat(new byte[] { Prefix_TokenRoyalty }, tokenId);
        ByteString raw = Storage.Get(key) ?? Storage.Get(new byte[] { Prefix_DefaultRoyalty });
        if (raw is null) return new object[0];

        var recipients = (object[])StdLib.Deserialize(raw);
        var output = new object[recipients.Length];
        for (int i = 0; i < recipients.Length; i++)
        {
            var pair = (object[])recipients[i];
            var receiver = (UInt160)pair[0];
            var bps      = (BigInteger)pair[1];
            output[i] = new object[] { receiver, salePrice * bps / 10_000 };
        }
        return output;
    }

    [Safe]
    public static BigInteger RoyaltyAmount(ByteString tokenId, UInt160 royaltyToken, BigInteger salePrice)
    {
        var info = RoyaltyInfo(tokenId, royaltyToken, salePrice);
        if (info.Length == 0) return 0;
        var first = (object[])info[0];
        return (BigInteger)first[1];
    }

    public static void Update(ByteString nefFile, string manifest, object? data = null)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        ContractManagement.Update(nefFile, manifest, data);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(new byte[] { Prefix_Owner }, owner);

        // Default royalty: deployer gets 5% (500 bps)
        var defaultPair = new object[] { new object[] { owner, (BigInteger)500 } };
        Storage.Put(new byte[] { Prefix_DefaultRoyalty }, StdLib.Serialize(defaultPair));
    }
}
