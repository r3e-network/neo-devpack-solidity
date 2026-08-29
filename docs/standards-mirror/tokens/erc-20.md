---
title: "ERC-20 — Fungible Token"
description: "ERC-20 — Fungible Token mapped to Neo N3."
---

# ERC-20 — Fungible Token

[Back to Token Standards](/standards-mirror/tokens)

<StandardsMirror>

<StandardEntry
  id="erc-20"
  title="ERC-20 — Fungible Token"
  eip="20"
  status="Final"
  neoMapping="NEP-17"
  category="Fungible"
  parityLabel="NEP-17"
  parityClass="sm-pill-direct"
>

<template #spec>

## ERC-20: Fungible Token Standard

The canonical Ethereum standard for fungible tokens — currencies, stablecoins,
governance tokens, LP tokens. Six methods plus two events that wallets, exchanges, and
DeFi protocols depend on.

### Required Interface

| Method | Returns | Purpose |
| --- | --- | --- |
| `name` / `symbol` / `decimals` | metadata | Token identification |
| `totalSupply()` | `uint256` | Total tokens in circulation |
| `balanceOf(address)` | `uint256` | Account balance |
| `transfer(address,uint256)` | `bool` | Move tokens to recipient |
| `approve(address,uint256)` | `bool` | Authorize a spender |
| `allowance(address,address)` | `uint256` | Read approved allowance |
| `transferFrom(address,address,uint256)` | `bool` | Spend on behalf of owner |

Events: `Transfer(from,to,value)`, `Approval(owner,spender,value)`.

### Authorization Model

ERC-20 uses **`msg.sender` + approve/allowance**: to let another contract spend your
tokens, you grant an allowance, then the spender calls `transferFrom`. This pattern is
the source of well-known vulnerabilities (infinite-approval drains, approve
front-running).

### Neo Equivalent: NEP-17

NEP-17 simplifies and hardens the model: **4-parameter `transfer(from, to, amount,
data)`** with witness-based authorization, no approve/allowance in the core spec, and
an `onNEP17Payment` callback that recipients implement to accept tokens.

::: tip Live on Neo TestNet
Both implementations are deployed on Neo N3 TestNet (network magic `894710606`).

| Implementation | TestNet Address | Contract Hash | Deploy Tx |
| --- | --- | --- | --- |
| **Solidity** (`neo-solc`) | `NZbQsZAbH3eBdZZYShj6CgG1ZkVEbjZhwF` | `0xd76434af829dc4c936c12648aa77932fa94c0f96` | [`0x37897c9d…85be43e`](https://dora.coz.io/transaction/neo3/testnet/0x37897c9d8b04c7d87baf2e256621d9980145fa2ee8891c9b477c9783985be43e) |
| **Neo C#** (`nccs`) | `NRGNZQRrb5TuDo4fA5KPiqZQB29Uybp1zJ` | `0x1f3a9b414de1c60434543dd8a05ac5e08b75b43a` | (re-used from earlier deploy) |

Checked-in snapshot: Solidity 3 / 5 assertions pass; Neo C# 4 / 4 assertions pass. This is a validation snapshot, not an all-green parity certification; see [TestNet Results](/standards-mirror/deployments/RESULTS) for failure details.

Source pairs: [`docs/standards-mirror/deployments/erc-20/`](https://github.com/r3e-network/neo-devpack-solidity/tree/main/docs/standards-mirror/deployments/erc-20).
:::

</template>

<template #solidity>

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract ERC20 {
    string  public name     = "Demo Token";
    string  public symbol   = "DEMO";
    uint8   public decimals = 18;
    uint256 public totalSupply;

    mapping(address => uint256)                      public balanceOf;
    mapping(address => mapping(address => uint256))  public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    constructor(uint256 initialSupply) {
        totalSupply           = initialSupply;
        balanceOf[msg.sender] = initialSupply;
        emit Transfer(address(0), msg.sender, initialSupply);
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        require(allowed >= amount, "ERC20: insufficient allowance");
        if (allowed != type(uint256).max) {
            allowance[from][msg.sender] = allowed - amount;
        }
        _transfer(from, to, amount);
        return true;
    }

    function _transfer(address from, address to, uint256 amount) internal {
        require(to != address(0),               "ERC20: zero recipient");
        require(balanceOf[from] >= amount,      "ERC20: insufficient balance");
        unchecked {
            balanceOf[from] -= amount;
            balanceOf[to]   += amount;
        }
        emit Transfer(from, to, amount);
    }
}
```

</template>

<template #csharp>

```csharp
using System;
using System.ComponentModel;
using System.Numerics;
using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;

namespace R3E.Examples;

[DisplayName("DemoToken")]
[ContractPermission("*", "*")]
[SupportedStandards(NepStandard.Nep17)]
public class DemoToken : SmartContract
{
    private const byte Prefix_TotalSupply = 0x00;
    private const byte Prefix_Balance     = 0x01;
    private static readonly UInt160 Owner = (UInt160)"0x0000000000000000000000000000000000000000";

    [DisplayName("Transfer")]
    public static event Action<UInt160, UInt160, BigInteger> OnTransfer;

    [Safe]
    public static string Symbol()   => "DEMO";
    public static byte   Decimals() => 8;

    public static BigInteger TotalSupply()
        => (BigInteger)(Storage.Get(Storage.CurrentContext, new byte[] { Prefix_TotalSupply }) ?? ByteString.Empty);

    public static BigInteger BalanceOf(UInt160 account)
    {
        if (!account.IsValid) throw new Exception("invalid account");
        var key = new byte[] { Prefix_Balance }.Concat(account);
        return (BigInteger)(Storage.Get(Storage.CurrentContext, key) ?? ByteString.Empty);
    }

    public static bool Transfer(UInt160 from, UInt160 to, BigInteger amount, object data)
    {
        if (!from.IsValid || !to.IsValid)        throw new Exception("invalid address");
        if (amount < 0)                          throw new Exception("amount < 0");
        if (!Runtime.CheckWitness(from) && !from.Equals(Runtime.CallingScriptHash))
            throw new Exception("no authorization");

        if (amount != 0)
        {
            var fromBal = BalanceOf(from);
            if (fromBal < amount) return false;
            UpdateBalance(from, fromBal - amount);
            UpdateBalance(to,   BalanceOf(to) + amount);
        }

        OnTransfer(from, to, amount);
        if (ContractManagement.GetContract(to) != null)
            Contract.Call(to, "onNEP17Payment", CallFlags.All,
                new object[] { from, amount, data });
        return true;
    }

    private static void UpdateBalance(UInt160 account, BigInteger value)
    {
        var key = new byte[] { Prefix_Balance }.Concat(account);
        if (value == 0) Storage.Delete(Storage.CurrentContext, key);
        else            Storage.Put(Storage.CurrentContext, key, value);
    }

    public static void _deploy(object data, bool update)
    {
        if (update) return;
        var initial = 1_000_000_00000000;
        Storage.Put(Storage.CurrentContext, new byte[] { Prefix_TotalSupply }, initial);
        UpdateBalance(Owner, initial);
        OnTransfer(UInt160.Zero, Owner, initial);
    }
}
```

</template>

</StandardEntry>

</StandardsMirror>
