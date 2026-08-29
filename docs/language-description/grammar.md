# Language Grammar

## Overview

The grammar of Neo DevPack for Solidity is strictly identical to the upstream Solidity 0.8.x language parser. Because Neo DevPack for Solidity uses the `solang-parser` internally, it accepts the exact same AST and grammar rules as Ethereum Solidity.

If you are building an editor integration or parser for Neo DevPack for Solidity, you can use any existing Solidity syntax highlighter or grammar tool.

For the full formal language specification, please refer to the [official Ethereum Solidity Parser Grammar](https://docs.soliditylang.org/en/latest/grammar.html).