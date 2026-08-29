using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("HookedToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-777 mirror in Neo C# — NEP-17 with safe-by-construction recipient callback.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.OnNEP17Payment)]
[SupportedStandards(NepStandard.Nep17)]
public class HookedToken : Nep17Token
{
    private const byte Prefix_Owner = 0xff;

    public override string Symbol { [Safe] get => "DHOOK"; }
    public override byte Decimals { [Safe] get => 8; }

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    public static new void Mint(UInt160 to, BigInteger amount)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        Nep17Token.Mint(to, amount);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(new byte[] { Prefix_Owner }, owner);
    }
}
