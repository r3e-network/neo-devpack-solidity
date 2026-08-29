using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("YieldVault")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-4626 yield vault in Neo C# — wraps a NEP-17 underlying.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
[SupportedStandards(NepStandard.Nep17, NepStandard.Nep27)]
public class YieldVault : Nep17Token
{
    private const byte Prefix_Owner = 0xff;
    private static readonly byte[] AssetKey = { 0xA0 };

    public override string Symbol { [Safe] get => "vDEMOC"; }
    public override byte Decimals { [Safe] get => 8; }

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    [Safe]
    public static UInt160 Asset() => (UInt160)Storage.Get(AssetKey);

    public static void SetAsset(UInt160 newAsset)
    {
        var owner = GetOwner();
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner only");
        var existing = Storage.Get(AssetKey);
        if (existing != null && existing.Length > 0) throw new System.Exception("asset already set");
        Storage.Put(AssetKey, newAsset);
    }

    [Safe]
    public static BigInteger TotalAssets()
    {
        var asset = Asset();
        if (asset is null || !asset.IsValid) return 0;
        return (BigInteger)Contract.Call(asset, "balanceOf", CallFlags.ReadOnly,
            new object[] { Runtime.ExecutingScriptHash });
    }

    [Safe]
    public static BigInteger ConvertToShares(BigInteger assets)
    {
        var supply = TotalSupply;
        if (supply == 0) return assets;
        var totAst = TotalAssets();
        if (totAst == 0) return assets;
        return assets * supply / totAst;
    }

    [Safe]
    public static BigInteger ConvertToAssets(BigInteger shares)
    {
        var supply = TotalSupply;
        if (supply == 0) return shares;
        return shares * TotalAssets() / supply;
    }

    /// <summary>
    /// NEP-27 callback — auto-deposit when the configured asset NEP-17 sends to us.
    /// </summary>
    public static void OnNEP17Payment(UInt160 from, BigInteger amount, object data)
    {
        var asset = Asset();
        if (!Runtime.CallingScriptHash.Equals(asset)) throw new System.Exception("only configured asset");
        if (from is null) return;
        var shares = ConvertToShares(amount);
        if (shares == 0) throw new System.Exception("zero shares (deposit too small)");
        Mint(from, shares);
    }

    public static bool Redeem(UInt160 from, UInt160 to, BigInteger shares)
    {
        if (!Runtime.CheckWitness(from)) throw new System.Exception("not owner");
        var assets = ConvertToAssets(shares);
        Burn(from, shares);
        Contract.Call(Asset(), "transfer", CallFlags.All,
            new object[] { Runtime.ExecutingScriptHash, to, assets, "redeem" });
        return true;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(new byte[] { Prefix_Owner }, owner);
    }
}
