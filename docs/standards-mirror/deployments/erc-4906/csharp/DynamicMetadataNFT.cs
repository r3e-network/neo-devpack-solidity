using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("DynamicMetadataNFT")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-4906 metadata-update notifications mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class DynamicMetadataNFT : SmartContract
{
    private const byte Prefix_Uri = 0x01;
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] NextIdKey = { 0xfe };

    [DisplayName("MetadataUpdate")]
    public static event System.Action<BigInteger> OnMetadataUpdate = null!;

    [DisplayName("BatchMetadataUpdate")]
    public static event System.Action<BigInteger, BigInteger> OnBatchMetadataUpdate = null!;

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger NextId()
    {
        var raw = Storage.Get(NextIdKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    private static byte[] UriKey(BigInteger id) =>
        Helper.Concat(new byte[] { Prefix_Uri }, (ByteString)id.ToByteArray());

    public static BigInteger Mint(string uri)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var id = NextId() + 1;
        Storage.Put(UriKey(id), uri);
        Storage.Put(NextIdKey, id);
        return id;
    }

    [Safe]
    public static string TokenURI(BigInteger id)
    {
        var raw = Storage.Get(UriKey(id));
        ExecutionEngine.Assert(raw is not null, "nonexistent");
        return (string)raw!;
    }

    public static void SetTokenURI(BigInteger id, string uri)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        var raw = Storage.Get(UriKey(id));
        ExecutionEngine.Assert(raw is not null, "nonexistent");
        Storage.Put(UriKey(id), uri);
        OnMetadataUpdate(id);
    }

    public static void SetBatchTokenURI(BigInteger fromId, BigInteger toId, string uri)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        ExecutionEngine.Assert(fromId <= toId, "bad range");
        for (var i = fromId; i <= toId; i++)
        {
            var raw = Storage.Get(UriKey(i));
            ExecutionEngine.Assert(raw is not null, "nonexistent in range");
            Storage.Put(UriKey(i), uri);
        }
        OnBatchMetadataUpdate(fromId, toId);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
