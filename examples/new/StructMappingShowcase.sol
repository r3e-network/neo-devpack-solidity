// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title StructMappingShowcase
 * @notice Demonstrates nested structs, structs in mappings,
 *         struct arrays, and struct return values.
 */
contract StructMappingShowcase {
    struct Profile {
        string name;
        uint256 age;
        address wallet;
    }

    struct Organization {
        string orgName;
        Profile leader;
        uint256 memberCount;
    }

    mapping(address => Profile) private profiles;
    mapping(uint256 => Organization) private orgs;
    Profile[] private allProfiles;
    uint256 public orgCount;

    function createProfile(string memory _name, uint256 _age) public {
        Profile memory p = Profile(_name, _age, msg.sender);
        profiles[msg.sender] = p;
        allProfiles.push(p);
    }

    function getProfile(address _addr) public view returns (Profile memory) {
        return profiles[_addr];
    }

    function createOrg(string memory _orgName) public {
        Profile memory leader = profiles[msg.sender];
        require(leader.wallet != address(0), "create profile first");

        orgs[orgCount] = Organization(_orgName, leader, 1);
        orgCount += 1;
    }

    function getOrg(uint256 _id) public view returns (Organization memory) {
        return orgs[_id];
    }

    function getProfileCount() public view returns (uint256) {
        return allProfiles.length;
    }
}
