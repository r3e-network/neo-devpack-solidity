# Neo DevPack for Solidity by Example

This section provides practical examples of smart contracts written in Solidity and compiled for the Neo N3 blockchain. Each example demonstrates how standard EVM patterns map seamlessly to NeoVM, along with Neo-specific best practices.

## 1. Simple Storage

The simplest way to interact with Neo Storage using standard Solidity.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Runtime} from "@neo/Runtime.sol";

contract SimpleStorage {
    uint256 private storedData;
    address public owner;

    event DataChanged(address indexed setter, uint256 newValue);

    constructor() {
        owner = Runtime.getCallingScriptHash(); // Native NeoVM intrinsic
    }

    function set(uint256 x) public {
        require(Runtime.checkWitness(owner), "Unauthorized");
        storedData = x;
        emit DataChanged(owner, x);
    }

    function get() public view returns (uint256) {
        return storedData;
    }
}
```

**Key Takeaways:**
*   **State Variables:** `storedData` and `owner` automatically map to Neo's Storage API.
*   **Intrinsics:** `Runtime.getCallingScriptHash()` accurately replaces `msg.sender` logic for initialization.
*   **Witness Checks:** `Runtime.checkWitness()` is the Neo-native, secure way to verify transaction signers.

---

## 2. NEP-17 Token (ERC-20 Equivalent)

Creating a fungible token on Neo N3 using the official NEP-17 standard. Neo N3 uses explicit `onNEP17Payment` hooks for contract interactions.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Runtime} from "@neo/Runtime.sol";

contract SimpleToken {
    string public symbol = "STK";
    uint8 public decimals = 8;
    uint256 public totalSupply;
    
    mapping(address => uint256) private balances;

    event Transfer(address indexed from, address indexed to, uint256 amount);

    constructor(uint256 initialSupply) {
        address creator = Runtime.getCallingScriptHash();
        totalSupply = initialSupply;
        balances[creator] = initialSupply;
        emit Transfer(address(0), creator, initialSupply);
    }

    function transfer(address from, address to, uint256 amount, any data) public returns (bool) {
        require(Runtime.checkWitness(from), "Unauthorized transfer");
        require(balances[from] >= amount, "Insufficient balance");

        balances[from] -= amount;
        balances[to] += amount;

        emit Transfer(from, to, amount);

        // Required by NEP-17: trigger the receiver's callback if it's a contract
        if (ContractManagement.isContract(to)) {
            // Note: In real applications, handle dynamic calls via Neo's native libraries.
        }
        
        return true;
    }

    function balanceOf(address account) public view returns (uint256) {
        return balances[account];
    }
}
```

**Key Takeaways:**
*   **Decimals:** NEP-17 conventionally expects 8 decimals instead of EVM's 18.
*   **Authentication:** Always use `Runtime.checkWitness(from)` to validate that the `from` account authorized the transfer.
*   **Data Parameter:** NEP-17 requires an `any data` parameter on `transfer`, enabling powerful callback patterns.

---

## 3. Governance DAO

Building a decentralized governance structure using the same logic found in EVM DAOs, compiled natively for NeoVM.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Runtime} from "@neo/Runtime.sol";

contract SimpleGovernor {
    struct Proposal {
        uint256 id;
        address proposer;
        string description;
        uint256 votesFor;
        uint256 votesAgainst;
        bool executed;
    }

    uint256 public proposalCount;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public hasVoted;

    event ProposalCreated(uint256 id, string description);
    event Voted(uint256 id, address voter, bool support);

    function propose(string memory description) public returns (uint256) {
        address proposer = Runtime.getCallingScriptHash();
        require(Runtime.checkWitness(proposer), "Unauthorized");

        proposalCount++;
        proposals[proposalCount] = Proposal({
            id: proposalCount,
            proposer: proposer,
            description: description,
            votesFor: 0,
            votesAgainst: 0,
            executed: false
        });

        emit ProposalCreated(proposalCount, description);
        return proposalCount;
    }

    function vote(uint256 proposalId, bool support) public {
        address voter = Runtime.getCallingScriptHash();
        require(Runtime.checkWitness(voter), "Unauthorized");
        require(!hasVoted[proposalId][voter], "Already voted");

        Proposal storage p = proposals[proposalId];
        
        if (support) {
            p.votesFor++;
        } else {
            p.votesAgainst++;
        }

        hasVoted[proposalId][voter] = true;
        emit Voted(proposalId, voter, support);
    }
}
```

**Key Takeaways:**
*   **Struct Storage:** Nested mappings and complex struct state persist beautifully in NeoVM without EVM storage layout limitations.
*   **Semantic Consistency:** The logic flows exactly as an Ethereum developer would expect, but with enhanced execution security through `Runtime.checkWitness`.