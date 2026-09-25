using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("PreDeploySig")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-6492 pre-deploy signature verification mirror in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class PreDeploySig : SmartContract
{
    private static readonly byte[] DeployerKey = { 0xff };
    private static readonly byte[] CountKey = { 0xfe };

    [Safe] public static UInt160 GetDeployer() => (UInt160)Storage.Get(DeployerKey);

    [Safe]
    public static BigInteger VerificationCount()
    {
        var raw = Storage.Get(CountKey);
        return raw is null ? 0 : (BigInteger)raw;
    }

    [Safe]
    public static ByteString GetMagic() =>
        (ByteString)new byte[] {
            0x64, 0x92, 0x64, 0x92, 0x64, 0x92, 0x64, 0x92,
            0x64, 0x92, 0x64, 0x92, 0x64, 0x92, 0x64, 0x92,
            0x64, 0x92, 0x64, 0x92, 0x64, 0x92, 0x64, 0x92,
            0x64, 0x92, 0x64, 0x92, 0x64, 0x92, 0x64, 0x92
        };

    public static void RecordVerification()
    {
        if (!Runtime.CheckWitness(GetDeployer())) throw new System.Exception("deployer only");
        Storage.Put(CountKey, VerificationCount() + 1);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 d = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(d.IsValid && !d.IsZero, "invalid deployer");
        Storage.Put(DeployerKey, d);
    }
}
