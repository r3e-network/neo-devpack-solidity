using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;

namespace R3E.StandardsMirror;

[DisplayName("Diamond")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2535 Diamond router in Neo C# — method-name routing to facet contracts.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class Diamond : SmartContract
{
    private const byte Prefix_Facet = 0x01;
    private static readonly byte[] OwnerKey = { 0xff };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    public static void AddFacet(string method, UInt160 facet)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        var key = Helper.Concat(new byte[] { Prefix_Facet },
                                (ByteString)method);
        Storage.Put(key, facet);
    }

    public static void RemoveFacet(string method)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        var key = Helper.Concat(new byte[] { Prefix_Facet },
                                (ByteString)method);
        Storage.Delete(key);
    }

    [Safe]
    public static UInt160 GetFacet(string method)
    {
        var key = Helper.Concat(new byte[] { Prefix_Facet },
                                (ByteString)method);
        var raw = Storage.Get(key);
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    /// Dispatch — clients invoke "Dispatch" with the target method name, and
    /// the diamond routes to the appropriate facet contract via Contract.Call.
    public static object Dispatch(string method, object[] args)
    {
        var facet = GetFacet(method);
        if (facet.Equals(UInt160.Zero)) throw new System.Exception("no facet for method");
        return Contract.Call(facet, method, CallFlags.All, args);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
