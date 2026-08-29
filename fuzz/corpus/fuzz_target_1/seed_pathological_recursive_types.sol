// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract RecursiveTypes {
    // Self-referencing via storage pointer (via mapping / array).
    struct Node {
        uint256 value;
        Node[] children;
    }

    struct LinkedList {
        uint256 data;
        LinkedList[] next;
    }

    struct Tree {
        bytes32 id;
        mapping(uint256 => Tree) branches;
    }

    Node public root;
    mapping(bytes32 => Tree) private trees;
    LinkedList public head;

    // Mutually recursive structs
    struct A { B[] bs; }
    struct B { A[] as_; }

    A public a;
    B public b;

    function addChild(uint256 v) external {
        root.children.push();
        root.children[root.children.length - 1].value = v;
    }
}
