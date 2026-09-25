using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("SelfDestruct")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-6780 SELFDESTRUCT-nerf mirror — Neo's ContractManagement.Destroy is an authorized op.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class SelfDestruct : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] ArmedKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static bool IsArmed()
    {
        var raw = Storage.Get(ArmedKey);
        return raw is not null && (BigInteger)raw != 0;
    }

    public static void ArmDestruct()
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(ArmedKey, 1);
    }

    /// On Neo, calling Destroy() removes the contract immediately when invoked
    /// by an authorized caller — gated behind the arm flag for safety.
    public static void Destroy()
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        ExecutionEngine.Assert(IsArmed(), "must arm first");
        ContractManagement.Destroy();
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
