using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("CompactSig")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-2098 compact signature mirror — Neo's native 64-byte secp256r1 in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class CompactSig : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe] public static BigInteger CompactSize() => 64;
    [Safe] public static BigInteger LegacySize() => 65;

    /// Verify a 64-byte signature (r || s) — Neo's native secp256r1 form, which
    /// is what EIP-2098 ports to EVM.
    [Safe]
    public static bool VerifyCompact(ByteString message, ECPoint pubKey, ByteString sig64) =>
        CryptoLib.VerifyWithECDsa(message, pubKey, sig64, NamedCurveHash.secp256r1SHA256);

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
