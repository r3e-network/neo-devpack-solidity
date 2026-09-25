using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("MultiToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1155 multi-token in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class MultiToken : SmartContract
{
    private const byte Prefix_Balance     = 0x01;   // id+owner -> amount
    private const byte Prefix_TotalSupply = 0x02;   // id -> total
    private static readonly byte[] OwnerKey = { 0xff };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    [Safe]
    public static BigInteger BalanceOf(UInt160 account, ByteString id)
    {
        var key = Helper.Concat(new byte[] { Prefix_Balance },
                                Helper.Concat(id, account));
        var raw = Storage.Get(key);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger TotalSupplyOf(ByteString id)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_TotalSupply }, id));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void Mint(UInt160 to, ByteString id, BigInteger amount)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        ExecutionEngine.Assert(amount > 0, "amount must be > 0");
        var key = Helper.Concat(new byte[] { Prefix_Balance },
                                Helper.Concat(id, to));
        var current = BalanceOf(to, id);
        Storage.Put(key, current + amount);
        var supplyKey = Helper.Concat(new byte[] { Prefix_TotalSupply }, id);
        Storage.Put(supplyKey, TotalSupplyOf(id) + amount);
    }

    public static bool SafeTransferFrom(UInt160 from, UInt160 to, ByteString id,
                                        BigInteger amount, object data)
    {
        if (!Runtime.CheckWitness(from)) throw new System.Exception("not authorized");
        ExecutionEngine.Assert(amount > 0, "amount > 0");
        var fromBal = BalanceOf(from, id);
        ExecutionEngine.Assert(fromBal >= amount, "insufficient");

        var fromKey = Helper.Concat(new byte[] { Prefix_Balance },
                                    Helper.Concat(id, from));
        var toKey = Helper.Concat(new byte[] { Prefix_Balance },
                                  Helper.Concat(id, to));
        Storage.Put(fromKey, fromBal - amount);
        Storage.Put(toKey, BalanceOf(to, id) + amount);
        return true;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
