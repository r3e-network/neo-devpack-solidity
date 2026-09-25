using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("SetCode")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-7702 set-code-for-EOAs mirror — Neo accounts are first-class smart contracts.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class SetCode : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] DelegateKey = { 0xfe };
    private static readonly byte[] CountKey = { 0xfd };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static UInt160 GetDelegate()
    {
        var raw = Storage.Get(DelegateKey);
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    [Safe]
    public static BigInteger DelegationCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void SetDelegate(UInt160 target)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(DelegateKey, target);
        Storage.Put(CountKey, DelegationCount() + 1);
    }

    /// The Neo equivalent of "EOA delegating to contract code" is just deploying
    /// a contract whose verify() trigger checks the original signer's witness.
    public static bool Verify() => Runtime.CheckWitness(GetDeployer());

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
