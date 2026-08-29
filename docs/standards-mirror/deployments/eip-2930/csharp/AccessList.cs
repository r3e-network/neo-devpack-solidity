using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("AccessList")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-2930 access-list mirror — Neo witness scopes.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class AccessList : SmartContract
{
    [Safe] public static BigInteger ScopeNone()            => 0x00;
    [Safe] public static BigInteger ScopeCalledByEntry()   => 0x01;
    [Safe] public static BigInteger ScopeCustomContracts() => 0x10;
    [Safe] public static BigInteger ScopeCustomGroups()    => 0x20;
    [Safe] public static BigInteger ScopeWitnessRules()    => 0x40;
    [Safe] public static BigInteger ScopeGlobal()          => 0x80;

    [Safe]
    public static bool CallerHasGlobalScope()
    {
        var sender = Runtime.Transaction.Sender;
        return Runtime.CheckWitness(sender);
    }
}
