// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title StorageConcatShowcase
 * @notice Demonstrates storage features: nested mappings,
 *         dynamic array push/pop, and struct-in-mapping access.
 */
contract StorageConcatShowcase {
    mapping(address => mapping(uint256 => uint256)) public nestedMap;
    mapping(address => uint256[]) public userBalances;

    struct UserInfo {
        string name;
        uint256 score;
        bool active;
    }
    mapping(address => UserInfo) private users;

    uint256[] public scores;

    function setNested(address user, uint256 key, uint256 val) public {
        nestedMap[user][key] = val;
    }

    function getNested(
        address user,
        uint256 key
    ) public view returns (uint256) {
        return nestedMap[user][key];
    }

    function pushScore(uint256 s) public {
        scores.push(s);
    }

    function popScore() public {
        scores.pop();
    }

    function scoresLength() public view returns (uint256) {
        return scores.length;
    }

    function setUser(string memory name, uint256 score) public {
        users[msg.sender] = UserInfo(name, score, true);
    }

    function getUserScore() public view returns (uint256) {
        return users[msg.sender].score;
    }

    function deactivateUser() public {
        users[msg.sender].active = false;
    }
}
