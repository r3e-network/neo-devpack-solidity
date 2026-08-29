using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("ExpirableToken")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-7818 expirable token in Neo C# — balances expire after N epochs.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class ExpirableToken : SmartContract
{
    private const byte Prefix_BalanceAtEpoch = 0x01;
    private static readonly byte[] DurationKey  = { 0xA0 };
    private static readonly byte[] RetentionKey = { 0xA1 };
    private static readonly byte[] OwnerKey     = { 0xff };

    [Safe] public static string Symbol() => "DEXP";

    [Safe]
    public static BigInteger CurrentEpoch()
    {
        var dur = (BigInteger)Storage.Get(DurationKey);
        if (dur == 0) return 0;
        return (BigInteger)(Runtime.Time / 1000) / dur;
    }

    [Safe]
    public static BigInteger BalanceAtEpoch(UInt160 owner, BigInteger epoch)
    {
        var key = Helper.Concat(new byte[] { Prefix_BalanceAtEpoch },
                  Helper.Concat(owner, (ByteString)epoch.ToByteArray()));
        var raw = Storage.Get(key);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static BigInteger BalanceOf(UInt160 owner)
    {
        var ret = (BigInteger)Storage.Get(RetentionKey);
        if (ret == 0) return 0;
        var cur = CurrentEpoch();
        var from = cur >= ret ? cur - ret + 1 : 0;
        BigInteger total = 0;
        for (var i = from; i <= cur; i++)
        {
            total += BalanceAtEpoch(owner, i);
        }
        return total;
    }

    public static void Mint(UInt160 to, BigInteger amount)
    {
        if (!Runtime.CheckWitness((UInt160)Storage.Get(OwnerKey)))
            throw new System.Exception("owner only");
        var epoch = CurrentEpoch();
        var prev = BalanceAtEpoch(to, epoch);
        var key = Helper.Concat(new byte[] { Prefix_BalanceAtEpoch },
                  Helper.Concat(to, (ByteString)epoch.ToByteArray()));
        Storage.Put(key, prev + amount);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = Runtime.Transaction.Sender;
        Storage.Put(OwnerKey, owner);
        // Default: 1-day epoch, retain for 30 days.
        Storage.Put(DurationKey,  86400);
        Storage.Put(RetentionKey, 30);
    }
}
