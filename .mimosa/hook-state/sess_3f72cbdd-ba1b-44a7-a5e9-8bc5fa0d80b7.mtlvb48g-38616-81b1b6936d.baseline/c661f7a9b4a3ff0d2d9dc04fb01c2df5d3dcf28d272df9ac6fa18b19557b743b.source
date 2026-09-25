using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("TransientGuard")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-1153 transient-storage reentrancy guard pattern in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class TransientGuard : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] CallsKey = { 0xfe };
    private static readonly byte[] LockKey = { 0xfd };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger CallsCompleted()
    {
        var raw = Storage.Get(CallsKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static bool IsLocked()
    {
        var raw = Storage.Get(LockKey);
        return raw is not null && (BigInteger)raw != 0;
    }

    public static void GuardedCall()
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        ExecutionEngine.Assert(!IsLocked(), "re-entry blocked");
        Storage.Put(LockKey, 1);
        Storage.Put(CallsKey, CallsCompleted() + 1);
        Storage.Delete(LockKey);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
