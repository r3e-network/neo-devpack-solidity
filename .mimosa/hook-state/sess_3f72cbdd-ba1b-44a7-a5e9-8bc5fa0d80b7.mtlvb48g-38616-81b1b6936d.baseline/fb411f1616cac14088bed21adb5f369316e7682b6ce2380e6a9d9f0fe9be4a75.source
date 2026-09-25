using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("TrustedForwarder")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-2771 trusted-forwarder pattern in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class TrustedForwarder : SmartContract
{
    private const byte Prefix_Nonce = 0x01;

    [Safe]
    public static BigInteger GetNonce(UInt160 signer)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Nonce }, signer));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static void BumpNonce(UInt160 signer)
    {
        if (!Runtime.CheckWitness(signer)) throw new System.Exception("signer must sign");
        Storage.Put(Helper.Concat(new byte[] { Prefix_Nonce }, signer), GetNonce(signer) + 1);
    }

    /// Forwards a call from `signer` to `target.method(args...)`. The signer
    /// must have signed the transaction (Runtime.CheckWitness gates this);
    /// Neo's witness model gives us native meta-tx without a separate
    /// signature verification step.
    public static object Execute(UInt160 signer, UInt160 target, string method, object[] args)
    {
        if (!Runtime.CheckWitness(signer)) throw new System.Exception("signer must sign");
        var key = Helper.Concat(new byte[] { Prefix_Nonce }, signer);
        var nonce = GetNonce(signer) + 1;
        Storage.Put(key, nonce);
        return Contract.Call(target, method, CallFlags.All, args);
    }
}
