using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;

namespace R3E.StandardsMirror;

[DisplayName("Ownable")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-173 ownership pattern in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class Ownable : SmartContract
{
    private const byte Prefix_Owner = 0xff;
    private const byte Prefix_Pending = 0xfe;

    [DisplayName("OwnershipTransferred")]
    public static event System.Action<UInt160?, UInt160> OnOwnershipTransferred = null!;

    [Safe]
    public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });

    [Safe]
    public static UInt160 PendingOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Pending });

    public static void TransferOwnership(UInt160 newOwner)
    {
        var owner = GetOwner();
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("not owner");
        ExecutionEngine.Assert(newOwner.IsValid && !newOwner.IsZero, "invalid newOwner");
        Storage.Put(new byte[] { Prefix_Pending }, newOwner);
    }

    public static void AcceptOwnership()
    {
        var pending = PendingOwner();
        if (pending is null || !Runtime.CheckWitness(pending))
            throw new System.Exception("not pending owner");
        var prev = GetOwner();
        Storage.Put(new byte[] { Prefix_Owner }, pending);
        Storage.Delete(new byte[] { Prefix_Pending });
        OnOwnershipTransferred(prev, pending);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 initial = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(initial.IsValid && !initial.IsZero, "invalid owner");
        Storage.Put(new byte[] { Prefix_Owner }, initial);
        OnOwnershipTransferred(null, initial);
    }
}
