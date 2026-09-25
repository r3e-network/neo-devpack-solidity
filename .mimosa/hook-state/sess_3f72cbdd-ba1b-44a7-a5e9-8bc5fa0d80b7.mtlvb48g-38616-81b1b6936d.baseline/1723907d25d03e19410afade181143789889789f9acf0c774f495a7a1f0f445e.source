using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("DomainExposer")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-5267 EIP-712 domain retrieval mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class DomainExposer : SmartContract
{
    [Safe] public static string GetName() => "MyDApp";
    [Safe] public static string GetVersion() => "1";

    /// <summary>
    /// Convenience method for clients that prefer an explicit domain query over
    /// reading the manifest + network ID. The manifest already contains
    /// the contract name and hash; this just bundles them.
    /// </summary>
    [Safe]
    public static (string, BigInteger, UInt160) Domain()
    {
        return ("MyDApp", (BigInteger)Runtime.GetNetwork(), Runtime.ExecutingScriptHash);
    }
}
