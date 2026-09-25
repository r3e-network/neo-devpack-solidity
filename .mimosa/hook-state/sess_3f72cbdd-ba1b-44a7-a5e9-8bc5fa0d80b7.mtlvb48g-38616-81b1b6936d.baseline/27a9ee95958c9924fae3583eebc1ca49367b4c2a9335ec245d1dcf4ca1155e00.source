using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("FeeAware")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-3198 BASEFEE introspection mirror — Policy.GetFeePerByte in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class FeeAware : SmartContract
{
    /// Reads Neo's per-byte fee at call time — the equivalent of EVM BASEFEE.
    [Safe]
    public static BigInteger GetCurrentFeePerByte() => Policy.GetFeePerByte();

    /// Reads the execution-fee multiplier — combined with per-syscall fees, this is
    /// the second half of Neo's fee model.
    [Safe]
    public static BigInteger GetExecFeeFactor() => Policy.GetExecFeeFactor();
}
