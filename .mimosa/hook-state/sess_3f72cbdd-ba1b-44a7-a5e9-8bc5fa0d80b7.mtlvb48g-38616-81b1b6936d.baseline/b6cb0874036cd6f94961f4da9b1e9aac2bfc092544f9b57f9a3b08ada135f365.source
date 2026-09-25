using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("InitcodeSize")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-3860 initcode size mirror — Neo NEF script size is enforced at deploy.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class InitcodeSize : SmartContract
{
    [Safe] public static BigInteger EvmMaxInitcodeSize() => 49152;

    /// Returns the size of the running contract's compiled NEF script —
    /// effectively a self-introspection of "initcode size" after deploy.
    [Safe]
    public static BigInteger SelfNefScriptSize()
    {
        var c = ContractManagement.GetContract(Runtime.ExecutingScriptHash);
        return c is null ? 0 : c.Nef.Length;
    }
}
