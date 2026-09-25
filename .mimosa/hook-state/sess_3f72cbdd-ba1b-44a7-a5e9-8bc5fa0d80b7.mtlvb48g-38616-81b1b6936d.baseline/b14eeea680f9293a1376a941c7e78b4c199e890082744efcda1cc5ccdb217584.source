using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;

namespace R3E.StandardsMirror;

public class DemoNFTState : Nep11TokenState
{
    public string Description = string.Empty;
    public string Image = string.Empty;
}

[DisplayName("DemoNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-721 / NEP-11 mirror demo — non-fungible token in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractSourceCode("https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments/erc-721/csharp")]
[ContractPermission(Permission.Any, Method.OnNEP11Payment)]
[SupportedStandards(NepStandard.Nep11)]
public class DemoNFT : Nep11Token<DemoNFTState>
{
    private const byte Prefix_Owner = 0xff;

    public override string Symbol { [Safe] get => "DCSHNFT"; }

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    private static bool IsOwner() => Runtime.CheckWitness(GetOwner());

    public static ByteString Mint(UInt160 to, string name, string description, string image)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        ExecutionEngine.Assert(to.IsValid && !to.IsZero, "invalid recipient");

        ByteString tokenId = NewTokenId();
        Nep11Token<DemoNFTState>.Mint(tokenId, new DemoNFTState
        {
            Owner = to,
            Name = name,
            Description = description,
            Image = image
        });
        return tokenId;
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
    }
}
