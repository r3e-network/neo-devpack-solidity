using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("PermitToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2612 permit-token mirror — witness-scope based authorization in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class PermitToken : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private const byte Prefix_Nonce = 0x01;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger NonceOf(UInt160 holder)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Nonce }, holder));
        return raw is null ? 0 : (BigInteger)raw;
    }

    /// On Neo, the holder authorizes by signing the transaction (witness scope),
    /// not by signing an off-chain message. We just bump the nonce so a stale
    /// permit cannot be replayed.
    public static void Permit(UInt160 holder, BigInteger amount)
    {
        if (!Runtime.CheckWitness(holder)) throw new System.Exception("holder must sign");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Nonce }, holder), NonceOf(holder) + 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
