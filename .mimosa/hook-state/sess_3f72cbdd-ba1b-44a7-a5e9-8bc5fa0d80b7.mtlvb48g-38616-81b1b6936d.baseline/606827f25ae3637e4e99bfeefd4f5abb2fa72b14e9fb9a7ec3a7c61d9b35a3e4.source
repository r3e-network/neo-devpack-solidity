using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("DemoToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-20 / NEP-17 mirror demo — fungible token in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractSourceCode("https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments/erc-20/csharp")]
[ContractPermission(Permission.Any, Method.Any)]
[SupportedStandards(NepStandard.Nep17)]
public class DemoToken : Nep17Token
{
    // bytecode-bump-2026-04-28-v2
    private const byte Prefix_Owner = 0xff;

    public override string Symbol { [Safe] get => "DEMOCSH"; }
    public override byte Decimals { [Safe] get => 8; }

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    private static bool IsOwner() => Runtime.CheckWitness(GetOwner());

    public static new void Mint(UInt160 to, BigInteger amount)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        Nep17Token.Mint(to, amount);
    }

    public static new void Burn(UInt160 account, BigInteger amount)
    {
        if (!IsOwner()) throw new System.Exception("owner only");
        Nep17Token.Burn(account, amount);
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

        // Initial mint: 1,000,000 tokens at 8 decimals = 100_000_000_000_000
        Nep17Token.Mint(owner, 100_000_000_000_000);
    }
}
