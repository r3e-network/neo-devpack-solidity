# Language Influences

Solidity was originally designed by Gavin Wood, Christian Reitwiessner, Alex Beregszaszi, Liana Husikyan, Yoichi Hirai and several former Ethereum core contributors. 

The language design was heavily influenced by C++, Python, and JavaScript, aiming to provide a familiar curly-braces syntax for developers entering the blockchain space.

## Neo DevPack for Solidity Influences

The `neo-devpack-solidity` compiler leverages this massive linguistic legacy to bring the largest smart contract developer base into the Neo ecosystem. 

Because Neo N3 was historically programmed using C#, Python, Go, and Java, the underlying Neo Virtual Machine (NeoVM) was designed to support complex, object-oriented concepts natively (like `Arrays`, `Maps`, and `ByteArrays`), rather than the low-level, word-based memory arrays of the EVM.

Neo DevPack for Solidity acts as a bridge between these two worlds:
1.  **Frontend Influence:** It retains 100% of Solidity's syntactical influences by utilizing the upstream `solang-parser`.
2.  **Backend Influence:** It maps those frontend concepts to Neo's native C#-inspired `StackItem` execution environment. For instance, Solidity `structs` are dynamically mapped to NeoVM `Array` items, bridging Solidity's memory layouts with Neo's high-level execution stack.