using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("VotingToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-5805 voting token with delegation in Neo C# (block-clock mode).")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class VotingToken : SmartContract
{
    private const byte Prefix_Balance = 0x01;
    private const byte Prefix_Delegate = 0x02;
    private const byte Prefix_Votes = 0x03;
    private static readonly byte[] OwnerKey = { 0xff };
    private static readonly byte[] TotalSupplyKey = { 0xfe };

    [Safe] public static string Symbol() => "DVOTE";
    [Safe] public static byte Decimals() => 0;
    [Safe] public static uint Clock() => Ledger.CurrentIndex;
    [Safe] public static string ClockMode() => "mode=blocknumber&from=default";

    [Safe]
    public static BigInteger TotalSupply()
    {
        var raw = Storage.Get(TotalSupplyKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger BalanceOf(UInt160 account)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Balance }, account));
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static UInt160 DelegateOf(UInt160 account)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Delegate }, account));
        return raw is null ? UInt160.Zero : (UInt160)raw;
    }

    [Safe]
    public static BigInteger GetVotes(UInt160 delegatee)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Votes }, delegatee));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void Mint(UInt160 to, BigInteger amount)
    {
        var owner = (UInt160)Storage.Get(OwnerKey);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner only");
        ExecutionEngine.Assert(amount > 0, "amount > 0");

        var newBal = BalanceOf(to) + amount;
        Storage.Put(Helper.Concat(new byte[] { Prefix_Balance }, to), newBal);
        Storage.Put(TotalSupplyKey, TotalSupply() + amount);

        var d = DelegateOf(to);
        if (!d.Equals(UInt160.Zero))
            Storage.Put(Helper.Concat(new byte[] { Prefix_Votes }, d), GetVotes(d) + amount);
    }

    public static void Delegate(UInt160 delegatee)
    {
        var caller = Runtime.Transaction.Sender;
        if (!Runtime.CheckWitness(caller)) throw new System.Exception("must sign");

        var current = DelegateOf(caller);
        var bal = BalanceOf(caller);
        if (!current.Equals(UInt160.Zero))
        {
            var prev = GetVotes(current);
            Storage.Put(Helper.Concat(new byte[] { Prefix_Votes }, current),
                        prev > bal ? prev - bal : 0);
        }
        Storage.Put(Helper.Concat(new byte[] { Prefix_Delegate }, caller), delegatee);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Votes }, delegatee),
                    GetVotes(delegatee) + bal);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
