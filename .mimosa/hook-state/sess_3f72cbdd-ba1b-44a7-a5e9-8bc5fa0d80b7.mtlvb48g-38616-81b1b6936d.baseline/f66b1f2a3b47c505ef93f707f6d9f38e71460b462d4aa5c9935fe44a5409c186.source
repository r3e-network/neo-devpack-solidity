using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("TypedData")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-712 typed-data verifier mirror — domain separator + ECDsa verify in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class TypedData : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] PubKeyKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static ByteString GetDomainSeparator() =>
        (ByteString)new byte[] {
            0x47, 0xe7, 0x95, 0x34, 0xa2, 0x45, 0x95, 0x2e,
            0x8b, 0x16, 0x89, 0x3a, 0x33, 0x6b, 0x85, 0xa3,
            0xd9, 0xea, 0x9f, 0xa8, 0xc5, 0x73, 0xf3, 0xd8,
            0x03, 0xaf, 0xb9, 0x2a, 0x79, 0x46, 0x92, 0x18
        };

    [Safe]
    public static ECPoint GetRegisteredPubKey()
    {
        var raw = Storage.Get(PubKeyKey);
        return raw is null ? null! : (ECPoint)raw;
    }

    public static void SetRegisteredPubKey(ECPoint pubKey)
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(PubKeyKey, pubKey);
    }

    /// Verify a signature over a digest precomputed off-chain
    /// (domainSeparator || structHash → digest).
    [Safe]
    public static bool VerifyDigest(ByteString digest, ByteString signature)
    {
        var pubKey = GetRegisteredPubKey();
        if (pubKey is null) return false;
        return CryptoLib.VerifyWithECDsa(digest, pubKey, signature, NamedCurveHash.secp256r1SHA256);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
