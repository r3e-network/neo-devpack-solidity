using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("AsyncVault")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-7540 async deposit/redeem vault mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class AsyncVault : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] DepositCountKey = { 0xfe };
    private static readonly byte[] RedeemCountKey = { 0xfd };
    private static readonly byte[] ClaimedDepositCountKey = { 0xfc };
    private const byte Prefix_DepositOwner = 0x01;
    private const byte Prefix_DepositAssets = 0x02;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger PendingDepositCount()
    {
        var raw = Storage.Get(DepositCountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger PendingRedeemCount()
    {
        var raw = Storage.Get(RedeemCountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger ClaimedDepositCount()
    {
        var raw = Storage.Get(ClaimedDepositCountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static BigInteger RequestDeposit(BigInteger assets)
    {
        if (!Runtime.CheckWitness(Runtime.Transaction.Sender)) throw new System.Exception("witness");
        var id = PendingDepositCount() + 1;
        Storage.Put(DepositCountKey, id);
        ByteString idKey = (ByteString)id.ToByteArray();
        Storage.Put(Helper.Concat(new byte[] { Prefix_DepositOwner }, idKey), Runtime.Transaction.Sender);
        Storage.Put(Helper.Concat(new byte[] { Prefix_DepositAssets }, idKey), assets);
        return id;
    }

    public static BigInteger RequestRedeem(BigInteger shares)
    {
        if (!Runtime.CheckWitness(Runtime.Transaction.Sender)) throw new System.Exception("witness");
        var id = PendingRedeemCount() + 1;
        Storage.Put(RedeemCountKey, id);
        return id;
    }

    [Safe]
    public static BigInteger PendingDepositRequest(BigInteger requestId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_DepositAssets }, (ByteString)requestId.ToByteArray()));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void ClaimDeposit(BigInteger requestId)
    {
        ByteString idKey = (ByteString)requestId.ToByteArray();
        var ownerRaw = Storage.Get(Helper.Concat(new byte[] { Prefix_DepositOwner }, idKey));
        if (ownerRaw is null) throw new System.Exception("no pending deposit");
        var owner = (UInt160)ownerRaw;
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner must sign");

        var assetsKey = Helper.Concat(new byte[] { Prefix_DepositAssets }, idKey);
        var assetsRaw = Storage.Get(assetsKey);
        if (assetsRaw is null || (BigInteger)assetsRaw == 0) throw new System.Exception("no pending deposit");
        Storage.Delete(assetsKey);
        Storage.Put(ClaimedDepositCountKey, ClaimedDepositCount() + 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
