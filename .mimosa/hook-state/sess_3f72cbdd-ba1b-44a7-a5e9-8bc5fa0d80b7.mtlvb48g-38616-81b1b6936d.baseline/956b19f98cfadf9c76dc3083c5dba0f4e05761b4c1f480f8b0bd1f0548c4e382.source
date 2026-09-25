using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("MultiAssetVault")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-7575 multi-asset share vault mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class MultiAssetVault : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] ShareKey = { 0xfe };
    private static readonly byte[] CountKey = { 0xfd };
    private const byte Prefix_Asset = 0x01;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static UInt160 Share()
    {
        var raw = Storage.Get(ShareKey);
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    [Safe]
    public static BigInteger AssetCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static bool IsRegistered(UInt160 asset)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Asset }, asset));
        return raw is not null;
    }

    public static void SetShare(UInt160 shareToken)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(ShareKey, shareToken);
    }

    public static void RegisterAsset(UInt160 asset)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        ExecutionEngine.Assert(!IsRegistered(asset), "dup");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Asset }, asset), 1);
        Storage.Put(CountKey, AssetCount() + 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
