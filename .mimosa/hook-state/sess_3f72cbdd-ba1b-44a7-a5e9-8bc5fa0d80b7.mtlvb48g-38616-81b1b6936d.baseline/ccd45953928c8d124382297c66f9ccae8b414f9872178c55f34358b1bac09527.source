using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("Bond")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-3525 semi-fungible token mirror — Neo C# port (no NEP equivalent).")]
[ContractVersion("1.0.0")]
[ContractSourceCode("https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments/erc-3525/csharp")]
[ContractPermission(Permission.Any, Method.Any)]
public class Bond : SmartContract
{
    private const byte Prefix_Owner    = 0x01;   // tokenId -> UInt160
    private const byte Prefix_Slot     = 0x02;   // tokenId -> BigInteger
    private const byte Prefix_Value    = 0x03;   // tokenId -> BigInteger
    private const byte Prefix_NextId   = 0xfe;
    private const byte Prefix_Admin    = 0xff;

    [DisplayName("Transfer")]
    public static event System.Action<UInt160?, UInt160, BigInteger, ByteString> OnTransfer = null!;

    [DisplayName("TransferValue")]
    public static event System.Action<ByteString?, ByteString, BigInteger> OnTransferValue = null!;

    [Safe] public static string Symbol() => "DCSHBND";
    [Safe] public static byte ValueDecimals() => 8;

    [Safe]
    public static UInt160 GetAdmin() => (UInt160)Storage.Get(new byte[] { Prefix_Admin });

    private static bool IsAdmin() => Runtime.CheckWitness(GetAdmin());

    [Safe]
    public static UInt160 OwnerOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Owner }, tokenId));
        if (raw is null) throw new System.Exception("nonexistent token");
        return (UInt160)raw;
    }

    [Safe]
    public static BigInteger SlotOf(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Slot }, tokenId));
        if (raw is null) throw new System.Exception("nonexistent token");
        return (BigInteger)raw;
    }

    [Safe]
    public static BigInteger BalanceOfToken(ByteString tokenId)
    {
        var raw = Storage.Get(Helper.Concat(new byte[] { Prefix_Value }, tokenId));
        return raw is null ? 0 : (BigInteger)raw;
    }

    public static ByteString Mint(UInt160 to, BigInteger slot, BigInteger value)
    {
        if (!IsAdmin()) throw new System.Exception("admin only");
        ExecutionEngine.Assert(to.IsValid && !to.IsZero, "invalid recipient");

        ByteString tokenId = NextId();
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, tokenId), to);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Slot  }, tokenId), slot);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Value }, tokenId), value);

        OnTransfer(null, to, value, tokenId);
        OnTransferValue(null, tokenId, value);
        return tokenId;
    }

    /// <summary>Move `value` from one tokenId to another within the same slot.</summary>
    public static void TransferValueToToken(ByteString fromTokenId, ByteString toTokenId, BigInteger value)
    {
        var owner = OwnerOf(fromTokenId);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("not owner");
        ExecutionEngine.Assert(SlotOf(fromTokenId) == SlotOf(toTokenId), "slot mismatch");

        var fromVal = BalanceOfToken(fromTokenId);
        ExecutionEngine.Assert(fromVal >= value, "insufficient value");

        Storage.Put(Helper.Concat(new byte[] { Prefix_Value }, fromTokenId), fromVal - value);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Value }, toTokenId),
                    BalanceOfToken(toTokenId) + value);
        OnTransferValue(fromTokenId, toTokenId, value);
    }

    /// <summary>Split: move `value` to a new token owned by `to`.</summary>
    public static ByteString TransferValueToAddress(ByteString fromTokenId, UInt160 to, BigInteger value)
    {
        var owner = OwnerOf(fromTokenId);
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("not owner");
        var fromVal = BalanceOfToken(fromTokenId);
        ExecutionEngine.Assert(fromVal >= value, "insufficient value");

        Storage.Put(Helper.Concat(new byte[] { Prefix_Value }, fromTokenId), fromVal - value);

        ByteString newId = NextId();
        var slot = SlotOf(fromTokenId);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Owner }, newId), to);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Slot  }, newId), slot);
        Storage.Put(Helper.Concat(new byte[] { Prefix_Value }, newId), value);

        OnTransfer(null, to, value, newId);
        OnTransferValue(fromTokenId, newId, value);
        return newId;
    }

    private static ByteString NextId()
    {
        var raw = Storage.Get(new byte[] { Prefix_NextId });
        BigInteger n = raw is null ? 1 : (BigInteger)raw + 1;
        Storage.Put(new byte[] { Prefix_NextId }, n);
        return (ByteString)n.ToByteArray();
    }

    public static void Update(ByteString nefFile, string manifest, object? data = null)
    {
        if (!IsAdmin()) throw new System.Exception("admin only");
        ContractManagement.Update(nefFile, manifest, data);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 admin = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(admin.IsValid && !admin.IsZero, "invalid admin");
        Storage.Put(new byte[] { Prefix_Admin }, admin);
    }
}
