// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title EventIndexedShowcase
 * @notice Demonstrates events with indexed params,
 *         anonymous events, and multi-topic events.
 */
contract EventIndexedShowcase {
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 amount
    );

    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );

    event MetadataUpdate(
        uint256 indexed tokenId,
        string key,
        string value
    );

    /// @notice Anonymous event — no topic[0] selector
    event AnonymousLog(uint256 data) anonymous;

    mapping(address => uint256) public balances;
    mapping(address => mapping(address => uint256)) public allowances;

    function mint(address to, uint256 amount) public {
        balances[to] += amount;
        emit Transfer(address(0), to, amount);
    }

    function transfer(address from, address to, uint256 amount, bytes memory data) public returns (bool) {
        data;
        require(from == msg.sender, "from must be caller");
        require(balances[from] >= amount, "insufficient");
        balances[from] -= amount;
        balances[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }

    function approve(address spender, uint256 value) public {
        allowances[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
    }

    function updateMetadata(
        uint256 tokenId,
        string memory key,
        string memory val
    ) public {
        emit MetadataUpdate(tokenId, key, val);
    }

    function emitAnonymous(uint256 data) public {
        emit AnonymousLog(data);
    }
}
