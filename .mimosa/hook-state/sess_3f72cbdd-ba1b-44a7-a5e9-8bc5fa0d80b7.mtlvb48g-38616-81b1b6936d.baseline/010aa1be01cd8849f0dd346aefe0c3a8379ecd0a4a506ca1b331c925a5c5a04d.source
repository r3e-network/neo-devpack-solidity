using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("UpgradeableContract")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1967 mirror — uses NEP-22 in-place update on Neo (no proxy needed).")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class UpgradeableContract : SmartContract
{
    private static readonly byte[] OwnerKey    = { 0xff };
    private static readonly byte[] VersionKey  = { 0xfe };
    private static readonly byte[] GreetingKey = { 0xfd };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    [Safe]
    public static BigInteger GetVersion()
    {
        var raw = Storage.Get(VersionKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static string GetGreeting() => (string)Storage.Get(GreetingKey);

    public static void SetGreeting(string greeting)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        Storage.Put(GreetingKey, greeting);
    }

    /// NEP-22 — standard contract update entrypoint. Replaces bytecode + manifest atomically.
    public static void Update(ByteString nefFile, string manifest, object? data = null)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        ContractManagement.Update(nefFile, manifest, data);
    }

    public static void _deploy(object data, bool update)
    {
        if (update)
        {
            // Migration step: bump version on every update.
            var v = GetVersion();
            Storage.Put(VersionKey, v + 1);
            return;
        }

        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
        Storage.Put(VersionKey, 1);
        Storage.Put(GreetingKey, "Hello from v1");
    }
}
