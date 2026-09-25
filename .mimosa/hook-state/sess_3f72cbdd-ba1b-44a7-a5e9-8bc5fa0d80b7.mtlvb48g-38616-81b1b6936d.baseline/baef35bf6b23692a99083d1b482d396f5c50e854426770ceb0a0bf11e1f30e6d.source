using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("Push0")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-3855 PUSH0 opcode mirror — NeoVM had PUSH0 since day one.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class Push0 : SmartContract
{
    [Safe] public static BigInteger Zero() => 0;
    [Safe] public static bool FalseBool() => false;
}
