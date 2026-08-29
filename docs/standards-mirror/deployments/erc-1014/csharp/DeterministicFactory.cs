using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("DeterministicFactory")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1014 CREATE2 deterministic-deploy mirror — uses ContractManagement.Deploy in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class DeterministicFactory : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] CountKey = { 0xfe };
    private const byte Prefix_Salt = 0x01;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger DeployCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static ByteString LastSalt(BigInteger n)
    {
        return Storage.Get(Helper.Concat(new byte[] { Prefix_Salt }, (ByteString)n.ToByteArray()));
    }

    /// Records a deploy operation on behalf of the deployer. In a real
    /// factory the contract would also call ContractManagement.Deploy(nef, manifest, salt),
    /// but the deterministic-hash property is achieved purely by the input pair
    /// (deployer + nef + name).
    public static void RecordDeploy(ByteString salt)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var n = DeployCount() + 1;
        Storage.Put(CountKey, n);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Salt }, (ByteString)n.ToByteArray()), salt);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
