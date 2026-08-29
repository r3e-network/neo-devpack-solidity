using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("TypedTx")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-2718 typed-transaction envelope mirror — Neo has one tx format.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class TypedTx : SmartContract
{
    [Safe] public static BigInteger NeoTxVersion() => Runtime.Transaction.Version;

    [Safe] public static BigInteger EvmLegacyType()     => 0x00;
    [Safe] public static BigInteger EvmAccessListType() => 0x01;
    [Safe] public static BigInteger EvmDynamicFeeType() => 0x02;
    [Safe] public static BigInteger EvmBlobType()       => 0x03;
    [Safe] public static BigInteger EvmSetCodeType()    => 0x04;
}
