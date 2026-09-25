using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using System.ComponentModel;

namespace R3E.StandardsMirror;

[DisplayName("ClockGov")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-6372 contract-clock pattern in Neo C# — block-number mode.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class ClockGov : SmartContract
{
    [Safe]
    public static uint Clock() => Ledger.CurrentIndex;

    [Safe]
    public static string ClockMode() => "mode=blocknumber&from=default";
}
