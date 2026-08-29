using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("PersonalSign")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("EIP-191 personal_sign mirror — CryptoLib.VerifyWithECDsa secp256r1 in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class PersonalSign : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] PubKeyKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe] public static BigInteger PrefixLength() => 19;

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

    /// Verify a secp256r1 signature over `message`. Returns true iff the signature
    /// is valid for the registered public key. This is the Neo equivalent of
    /// `ecrecover(hash, v, r, s) == registeredSigner`.
    [Safe]
    public static bool VerifyMessage(ByteString message, ByteString signature)
    {
        var pubKey = GetRegisteredPubKey();
        if (pubKey is null) return false;
        return CryptoLib.VerifyWithECDsa(message, pubKey, signature, NamedCurveHash.secp256r1SHA256);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
