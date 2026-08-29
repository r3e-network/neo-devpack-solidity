---
title: "Native Contracts: NEO Token Contract"
description: "NEO Token Contract from Native Contracts."
---

# NEO Token Contract

[Back to Native Contracts](/internals/native-contracts)

The NEO token is the governance token of the Neo N3 network. It is indivisible (0 decimals), has a fixed supply of 100,000,000, and is used for voting, validator election, and GAS generation.

## Methods

| Method                   | Signature                                 | Return           | Safe | Description                                                   |
| ------------------------ | ----------------------------------------- | ---------------- | :--: | ------------------------------------------------------------- |
| `name`                   | `name()`                                  | `string`         |  ✅  | Returns `"NEO"`.                                              |
| `symbol`                 | `symbol()`                                | `string`         |  ✅  | Returns `"NEO"`.                                              |
| `decimals`               | `decimals()`                              | `uint8`          |  ✅  | Returns `0` (indivisible).                                    |
| `totalSupply`            | `totalSupply()`                           | `uint256`        |  ✅  | Returns `100000000`.                                          |
| `balanceOf`              | `balanceOf(address)`                      | `uint256`        |  ✅  | NEO balance of account.                                       |
| `transfer`               | `transfer(address,address,uint256,bytes)` | `bool`           |  ❌  | Transfer NEO tokens. Requires witness of `from`.              |
| `vote`                   | `vote(address,bytes)`                     | `bool`           |  ❌  | Vote for a validator candidate. Pass `null` pubkey to cancel. |
| `getCandidates`          | `getCandidates()`                         | `NeoCandidate[]` |  ✅  | First 256 registered candidates with vote counts.             |
| `getAllCandidates`       | `getAllCandidates()`                      | `Iterator`       |  ✅  | Iterator over all registered candidates.                      |
| `getCandidateVote`       | `getCandidateVote(bytes)`                 | `int256`         |  ✅  | Vote count for a public key. Returns `-1` if not found.       |
| `registerCandidate`      | `registerCandidate(bytes)`                | `bool`           |  ❌  | Register a public key as validator candidate.                 |
| `unregisterCandidate`    | `unregisterCandidate(bytes)`              | `bool`           |  ❌  | Remove candidate registration.                                |
| `getCommittee`           | `getCommittee()`                          | `bytes[]`        |  ✅  | Current committee member public keys.                         |
| `getNextBlockValidators` | `getNextBlockValidators()`                | `address[]`      |  ✅  | Validators for the next block.                                |
| `getGasPerBlock`         | `getGasPerBlock()`                        | `uint256`        |  ✅  | GAS generated per block.                                      |
| `setGasPerBlock`         | `setGasPerBlock(uint256)`                 | `void`           |  ❌  | Set GAS per block (committee only).                           |
| `getRegisterPrice`       | `getRegisterPrice()`                      | `uint256`        |  ✅  | GAS cost to register as candidate.                            |
| `setRegisterPrice`       | `setRegisterPrice(uint256)`               | `void`           |  ❌  | Set registration price (committee only).                      |
| `getAccountState`        | `getAccountState(address)`                | `AccountState`   |  ✅  | Balance, vote target, and last GAS claim height.              |
| `unclaimedGas`           | `unclaimedGas(address,uint256)`           | `uint256`        |  ✅  | Unclaimed GAS for account at block height.                    |

## Code Example

```solidity
import "devpack/contracts/NativeCalls.sol";

contract Governance {
    function getMyNeoBalance() public view returns (uint256) {
        return NativeCalls.neoBalanceOf(address(this));
    }

    function voteForCandidate(bytes memory publicKey) public {
        require(Runtime.checkWitness(msg.sender), "unauthorized");
        NativeCalls.vote(msg.sender, publicKey);
    }

    function claimableGas(address account) public view returns (uint256) {
        uint256 height = NativeCalls.currentIndex();
        return NativeCalls.unclaimedGas(account, height);
    }
}
```

---
