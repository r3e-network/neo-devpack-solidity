// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Syscalls Data Structures
 */

library SyscallsTypes {
    // ========== Data Structures ==========
    
    // Ledger.getBlock returns a TrimmedBlock stack item, not the full block payload.
    struct Block {
        bytes32 hash;
        uint256 version;
        bytes32 previousHash;
        bytes32 merkleRoot;
        uint256 timestamp;
        uint256 nonce;
        uint256 index;
        uint256 primaryIndex;
        address nextConsensus;
        uint256 txCount;
    }
    
    // Ledger.getTransaction returns the base transaction fields (no signers/witnesses).
    struct Transaction {
        bytes32 hash;
        uint256 version;
        uint256 nonce;
        address sender;
        uint256 systemFee;
        uint256 networkFee;
        uint256 validUntilBlock;
        bytes script;
    }
    
    struct Witness {
        bytes invocationScript;
        bytes verificationScript;
    }

    uint8 constant WITNESS_RULE_DENY = 0x00;
    uint8 constant WITNESS_RULE_ALLOW = 0x01;

    uint8 constant WITNESS_CONDITION_BOOLEAN = 0x00;
    uint8 constant WITNESS_CONDITION_NOT = 0x01;
    uint8 constant WITNESS_CONDITION_AND = 0x02;
    uint8 constant WITNESS_CONDITION_OR = 0x03;
    uint8 constant WITNESS_CONDITION_SCRIPT_HASH = 0x18;
    uint8 constant WITNESS_CONDITION_GROUP = 0x19;
    uint8 constant WITNESS_CONDITION_CALLED_BY_ENTRY = 0x20;
    uint8 constant WITNESS_CONDITION_CALLED_BY_CONTRACT = 0x28;
    uint8 constant WITNESS_CONDITION_CALLED_BY_GROUP = 0x29;

    // WitnessCondition is represented as a NeoVM Array:
    // [type, ...condition-specific data]. Use StdLib.serialize/deserialize
    // if you need to inspect it in Solidity.
    struct WitnessRule {
        uint8 action;
        bytes condition;
    }
    
    // WitnessScope bit flags (see Neo WitnessScope enum).
    uint8 constant WITNESS_SCOPE_NONE = 0x00;
    uint8 constant WITNESS_SCOPE_CALLED_BY_ENTRY = 0x01;
    uint8 constant WITNESS_SCOPE_CUSTOM_CONTRACTS = 0x10;
    uint8 constant WITNESS_SCOPE_CUSTOM_GROUPS = 0x20;
    uint8 constant WITNESS_SCOPE_WITNESS_RULES = 0x40;
    uint8 constant WITNESS_SCOPE_GLOBAL = 0x80;

    struct Signer {
        address account;
        uint8 scopes;
        address[] allowedContracts;
        bytes[] allowedGroups;
        // Witness rules (only present when scopes includes WITNESS_SCOPE_WITNESS_RULES).
        WitnessRule[] rules;
    }
    
    struct StorageContext {
        int256 id;
        bool isReadOnly;
    }
    
    struct Iterator {
        uint256 id;
        bool hasNext;
        bytes currentKey;
        bytes currentValue;
    }
    
    struct Notification {
        address scriptHash;
        string eventName;
        // State array passed to Runtime.notify(...)
        bytes[] state;
    }

    // ContractManagement.getContract returns:
    // [id, updateCounter, hash, nef, manifestStruct]
    struct ContractStateNative {
        int256 id;
        uint256 updateCounter;
        address hash;
        bytes nef;
        bytes manifest;
    }
    
    // TriggerType values (Neo.SmartContract.TriggerType)
    uint8 constant TRIGGER_ON_PERSIST = 0x01;
    uint8 constant TRIGGER_POST_PERSIST = 0x02;
    uint8 constant TRIGGER_VERIFICATION = 0x20;
    uint8 constant TRIGGER_APPLICATION = 0x40;
    uint8 constant TRIGGER_SYSTEM = TRIGGER_ON_PERSIST | TRIGGER_POST_PERSIST;
    uint8 constant TRIGGER_ALL = TRIGGER_SYSTEM | TRIGGER_VERIFICATION | TRIGGER_APPLICATION;
    
}
