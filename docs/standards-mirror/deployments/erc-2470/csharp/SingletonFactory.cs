using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("SingletonFactory")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2470 mirror — Neo's deploys are already deterministic, so the factory is each deployer themselves.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class SingletonFactory : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] CountKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger DeployCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    /// Deploy a new contract with deterministic script hash from
    /// (caller, nef.script, manifest.name).
    public static UInt160 Deploy(ByteString nef, string manifest, object data)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var contract = ContractManagement.Deploy(nef, manifest, data);
        Storage.Put(CountKey, DeployCount() + 1);
        return contract.Hash;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
