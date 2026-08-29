using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("DIDRegistry")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1056 lightweight DID registry in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class DIDRegistry : SmartContract
{
    private const byte Prefix_Owner    = 0x01;   // identity -> owner
    private const byte Prefix_Delegate = 0x02;   // identity+type+delegate -> validTo
    private const byte Prefix_Changed  = 0x03;   // identity -> last-change block

    [Safe]
    public static UInt160 IdentityOwner(UInt160 identity)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, identity));
        return raw is null ? identity : (UInt160)raw;
    }

    public static void ChangeOwner(UInt160 identity, UInt160 newOwner)
    {
        var owner = IdentityOwner(identity);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("not owner");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, identity), newOwner);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Changed }, identity),
                    (BigInteger)Ledger.CurrentIndex);
    }

    public static void AddDelegate(UInt160 identity, ByteString delegateType,
                                   UInt160 delegateAddr, BigInteger validitySec)
    {
        var owner = IdentityOwner(identity);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("not owner");
        var validTo = (BigInteger)(Runtime.Time / 1000) + validitySec;
        var key = Helper.Concat(new byte[] { Prefix_Delegate },
                  Helper.Concat(identity,
                  Helper.Concat(delegateType, delegateAddr)));
        Storage.Put(key, validTo);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Changed }, identity),
                    (BigInteger)Ledger.CurrentIndex);
    }

    [Safe]
    public static bool ValidDelegate(UInt160 identity, ByteString delegateType, UInt160 delegateAddr)
    {
        var key = Helper.Concat(new byte[] { Prefix_Delegate },
                  Helper.Concat(identity,
                  Helper.Concat(delegateType, delegateAddr)));
        var raw = Storage.Get(key);
        if (raw is null) return false;
        var validTo = (BigInteger)raw;
        return validTo > (BigInteger)(Runtime.Time / 1000);
    }
}
