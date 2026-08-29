using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("SmartAccount")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-4337 smart-account mirror — NEP-30 verify trigger in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class SmartAccount : SmartContract
{
    private static readonly byte[] OwnerKey = { 0xff };
    private static readonly byte[] NonceKey = { 0xfe };
    private static readonly byte[] PubKeyKey = { 0xfd };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(OwnerKey);

    [Safe]
    public static BigInteger GetNonce()
    {
        var raw = Storage.Get(NonceKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static ECPoint GetPubKey()
    {
        var raw = Storage.Get(PubKeyKey);
        return raw is null ? null! : (ECPoint)raw;
    }

    public static void SetPubKey(ECPoint pubKey)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        Storage.Put(PubKeyKey, pubKey);
    }

    /// Equivalent to validateUserOp + execute. The owner witness authorizes the
    /// nonce mutation; the registered pubKey verifies the supplied userOp hash.
    public static bool ValidateAndBumpNonce(ByteString userOpHash, ByteString signature)
    {
        if (!Runtime.CheckWitness(GetOwner())) throw new System.Exception("owner only");
        var pubKey = GetPubKey();
        if (pubKey is null) return false;
        if (!CryptoLib.VerifyWithECDsa(userOpHash, pubKey, signature, NamedCurveHash.secp256r1SHA256))
            return false;
        Storage.Put(NonceKey, GetNonce() + 1);
        return true;
    }

    /// On Neo this is the actual verify() trigger. Returns true iff the registered
    /// owner has signed the transaction.
    public static bool Verify() => Runtime.CheckWitness(GetOwner());

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(OwnerKey, owner);
    }
}
