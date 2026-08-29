using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;

namespace R3E.StandardsMirror;

[DisplayName("InterfaceRegistry")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-1820 interface registry in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class InterfaceRegistry : SmartContract
{
    private const byte Prefix_Implementer = 0x01;
    private const byte Prefix_Manager     = 0x02;

    [Safe]
    public static UInt160 GetImplementer(UInt160 account, ByteString interfaceHash)
    {
        var key = Helper.Concat(new byte[] { Prefix_Implementer },
                  Helper.Concat(account, interfaceHash));
        return (UInt160)Storage.Get(key);
    }

    [Safe]
    public static UInt160 GetManager(UInt160 account)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Manager }, account));
        return raw is null ? account : (UInt160)raw;
    }

    public static void SetImplementer(UInt160 account, ByteString interfaceHash, UInt160 impl)
    {
        var manager = GetManager(account);
        if (!Runtime.CheckWitness(manager)) throw new System.Exception("not manager");
        var key = Helper.Concat(new byte[] { Prefix_Implementer },
                  Helper.Concat(account, interfaceHash));
        Storage.Put(key, impl);
    }

    public static void SetManager(UInt160 account, UInt160 newManager)
    {
        if (!Runtime.CheckWitness(GetManager(account))) throw new System.Exception("not manager");
        var key = Helper.Concat(new byte[] { Prefix_Manager }, account);
        if (newManager.Equals(account))
            Storage.Delete(key);
        else
            Storage.Put(key, newManager);
    }
}
