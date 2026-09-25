using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("NamespacedStorage")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-7201 namespaced storage layout demo in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class NamespacedStorage : SmartContract
{
    private static readonly byte[] NamespacePrefix = { 0xA1, 0x52 };
    private static readonly byte[] DeployerKey = { 0xff };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe] public static string NamespaceTag() => "r3e.standards-mirror.demo.v1";

    public static void SetSlot(ByteString key, BigInteger value)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(Helper.Concat((ByteString)NamespacePrefix, key), value);
    }

    [Safe]
    public static BigInteger GetSlot(ByteString key)
    {
        var raw = Storage.Get(Helper.Concat((ByteString)NamespacePrefix, key));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
