using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;
using System.ComponentModel;
using System.Numerics;

namespace R3E.StandardsMirror;

[DisplayName("FlashLender")]
[ContractAuthor("R3E Network", "neo-devpack-solidity@r3e.network")]
[ContractDescription("ERC-3156 flash-loan provider in Neo C#.")]
[ContractVersion("1.0.0")]
[ContractPermission(Permission.Any, Method.Any)]
public class FlashLender : SmartContract
{
    public const string CallbackSuccess = "ERC3156FlashBorrower.onFlashLoan";

    private const byte Prefix_Owner = 0xff;
    private static readonly byte[] TokenKey  = { 0xA0 };
    private static readonly byte[] FeeKey    = { 0xA1 };

    [Safe] public static UInt160 GetOwner() => (UInt160)Storage.Get(new byte[] { Prefix_Owner });
    [Safe] public static UInt160 Token() => (UInt160)Storage.Get(TokenKey);
    [Safe]
    public static BigInteger FeeBps()
    {
        var raw = Storage.Get(FeeKey);
        if (raw is null) return 0;
        return (BigInteger)raw;
    }

    public static void Setup(UInt160 token, BigInteger feeBps)
    {
        var owner = GetOwner();
        if (!Runtime.CheckWitness(owner)) throw new System.Exception("owner only");
        var existing = Storage.Get(TokenKey);
        if (existing != null && existing.Length > 0) throw new System.Exception("already set");
        ExecutionEngine.Assert(feeBps >= 0 && feeBps <= 10000, "invalid feeBps");
        Storage.Put(TokenKey, token);
        Storage.Put(FeeKey, feeBps);
    }

    [Safe]
    public static BigInteger MaxFlashLoan(UInt160 t)
    {
        if (!t.Equals(Token())) return 0;
        return (BigInteger)Contract.Call(Token(), "balanceOf", CallFlags.ReadOnly,
            new object[] { Runtime.ExecutingScriptHash });
    }

    [Safe]
    public static BigInteger FlashFee(UInt160 t, BigInteger amount)
    {
        if (!t.Equals(Token())) throw new System.Exception("wrong token");
        return amount * FeeBps() / 10000;
    }

    public static bool FlashLoanRequest(UInt160 receiver, UInt160 t,
                                        BigInteger amount, object data)
    {
        if (!t.Equals(Token())) throw new System.Exception("wrong token");
        ExecutionEngine.Assert(amount > 0, "amount must be > 0");

        var fee = FlashFee(t, amount);
        var balanceBefore = MaxFlashLoan(t);

        // Send principal
        Contract.Call(Token(), "transfer", CallFlags.All, new object[] {
            Runtime.ExecutingScriptHash, receiver, amount, "flashloan-principal"
        });

        // Borrower must return the success magic string
        var ret = (string)Contract.Call(receiver, "onFlashLoan", CallFlags.All, new object[] {
            Runtime.CallingScriptHash, Token(), amount, fee, data
        });
        if (ret != CallbackSuccess) throw new System.Exception("callback failed");

        // Verify repayment
        var balanceAfter = MaxFlashLoan(t);
        if (balanceAfter < balanceBefore + fee) throw new System.Exception("not repaid");
        return true;
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        UInt160 owner = data is null ? Runtime.Transaction.Sender : (UInt160)data;
        ExecutionEngine.Assert(owner.IsValid && !owner.IsZero, "invalid owner");
        Storage.Put(new byte[] { Prefix_Owner }, owner);
    }
}
