using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("ModularAccount")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-7579 modular smart account mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class ModularAccount : SmartContract
{
    private static readonly byte[] OwnerKey = { 0xff };
    private static readonly byte[] CountKey = { 0xfe };
    private const byte Prefix_Module = 0x01;

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    [Safe]
    public static BigInteger ModuleCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger ModuleType(UInt160 module)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Module }, module));
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static bool IsModuleInstalled(BigInteger moduleTypeId, UInt160 module) =>
        ModuleType(module) == moduleTypeId;

    public static void InstallModule(BigInteger moduleTypeId, UInt160 module)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        ExecutionEngine.Assert(moduleTypeId >= 1 && moduleTypeId <= 4, "bad type");
        ExecutionEngine.Assert(ModuleType(module) == 0, "already installed");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Module }, module), moduleTypeId);
        Storage.Put(CountKey, ModuleCount() + 1);
    }

    public static void UninstallModule(UInt160 module)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        ExecutionEngine.Assert(ModuleType(module) != 0, "not installed");
        Storage.Delete(Helper.Concat(new byte[] { Prefix_Module }, module));
        Storage.Put(CountKey, ModuleCount() - 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
