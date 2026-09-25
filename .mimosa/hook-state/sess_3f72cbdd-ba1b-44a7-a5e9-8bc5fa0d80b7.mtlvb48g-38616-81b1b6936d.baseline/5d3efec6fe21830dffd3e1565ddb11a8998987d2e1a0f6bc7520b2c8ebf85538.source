using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("MultiSig")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1271 contract-signature pattern in Neo C# (multi-sig wallet).")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
[SupportedStandards(NepStandard.Nep30)]
public class MultiSig : SmartContract
{
    private const byte Prefix_Owner     = 0x01;   // address -> 1 (owner set)
    private static readonly byte[] ThresholdKey = { 0xA0 };
    private static readonly byte[] AdminKey     = { 0xff };

    [Safe]
    public static bool IsOwner(UInt160 a)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, a));
        return raw != null;
    }

    [Safe]
    public static BigInteger Threshold()
    {
        var raw = Storage.Get(ThresholdKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    /// NEP-30 verify — invoked by the protocol when this account is a signer.
    /// Counts how many configured owners have witnessed the transaction; passes
    /// when threshold met.
    [Safe]
    public static bool Verify()
    {
        var iter = Storage.Find(new byte[] { Prefix_Owner }, FindOptions.KeysOnly | FindOptions.RemovePrefix);
        BigInteger approvals = 0;
        while (iter.Next())
        {
            var owner = (UInt160)iter.Value;
            if (Runtime.CheckWitness(owner)) approvals++;
        }
        return approvals >= Threshold();
    }

    public static void Setup(UInt160[] owners, BigInteger threshold)
    {
        var admin = (UInt160)Storage.Get(AdminKey);
        if (!Runtime.CheckWitness(admin)) throw new System.Exception("admin only");
        ExecutionEngine.Assert(owners.Length > 0, "no owners");
        ExecutionEngine.Assert(threshold > 0 && threshold <= owners.Length, "bad threshold");

        // wipe any prior owners
        var iter = Storage.Find(new byte[] { Prefix_Owner }, FindOptions.KeysOnly | FindOptions.RemovePrefix);
        while (iter.Next())
        {
            var oldOwner = (UInt160)iter.Value;
            Storage.Delete(Helper.Concat(new byte[] { Prefix_Owner }, oldOwner));
        }
        foreach (var o in owners)
        {
            Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, o), 1);
        }
        Storage.Put(ThresholdKey, threshold);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 admin = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(admin.IsValid && !admin.IsZero, "invalid admin");
        Storage.Put(AdminKey, admin);
    }
}
