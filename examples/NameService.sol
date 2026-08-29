// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title NameService - Decentralized name registration for Neo N3
/// @author Neo DevPack for Solidity Team
/// @notice Register and manage human-readable names mapped to addresses
/// @dev Similar to ENS but simplified for Neo N3
contract NameService {
    // Name record structure
    struct NameRecord {
        address owner;
        address resolver;
        uint256 registrationTime;
        uint256 expirationTime;
        bool exists;
    }

    // Resolver record for additional data
    struct ResolverData {
        address addr;
        string contentHash;
        string email;
        string url;
        string description;
    }

    // State variables
    mapping(bytes32 => NameRecord) public names;
    mapping(bytes32 => ResolverData) public resolvers;
    mapping(address => bytes32[]) public ownerNames;
    mapping(bytes32 => mapping(string => string)) public textRecords;

    address public owner;
    uint256 public registrationFee;
    uint256 public renewalFee;
    uint256 public defaultDuration; // in seconds
    uint256 public minNameLength;
    uint256 public collectedFees;

    // Events
    event NameRegistered(bytes32 indexed nameHash, string name, address indexed owner, uint256 expirationTime);
    event NameRenewed(bytes32 indexed nameHash, uint256 newExpirationTime);
    event NameTransferred(bytes32 indexed nameHash, address indexed from, address indexed to);
    event ResolverSet(bytes32 indexed nameHash, address resolver);
    event AddressChanged(bytes32 indexed nameHash, address newAddress);
    event TextRecordSet(bytes32 indexed nameHash, string key, string value);
    event NameReleased(bytes32 indexed nameHash, address indexed owner);
    event FeesUpdated(uint256 registrationFee, uint256 renewalFee);

    /// @notice Contract constructor
    /// @param _registrationFee Fee for registering a name
    /// @param _renewalFee Fee for renewing a name
    /// @param _defaultDuration Default registration duration in seconds
    /// @param _minNameLength Minimum name length
    constructor(
        uint256 _registrationFee,
        uint256 _renewalFee,
        uint256 _defaultDuration,
        uint256 _minNameLength
    ) {
        owner = msg.sender;
        registrationFee = _registrationFee;
        renewalFee = _renewalFee;
        defaultDuration = _defaultDuration;
        minNameLength = _minNameLength;
        collectedFees = 0;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "NameService: caller is not owner");
        _;
    }

    modifier onlyNameOwner(bytes32 nameHash) {
        require(names[nameHash].owner == msg.sender, "NameService: caller is not name owner");
        _;
    }

    modifier nameExists(bytes32 nameHash) {
        require(names[nameHash].exists, "NameService: name does not exist");
        _;
    }

    /// @notice Hash a name string
    /// @param name The name to hash
    /// @return The keccak256 hash of the name
    function hashName(string memory name) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(name));
    }

    /// @notice Check if a name is available for registration
    /// @param name The name to check
    /// @return True if available
    function isAvailable(string memory name) public view returns (bool) {
        bytes32 nameHash = hashName(name);
        if (!names[nameHash].exists) return true;
        return block.timestamp >= names[nameHash].expirationTime;
    }

    /// @notice Register a new name
    /// @param name The name to register
    /// @param duration Registration duration in seconds (0 for default)
    /// @return nameHash The hash of the registered name
    function register(string memory name, uint256 duration) public returns (bytes32 nameHash) {
        require(bytes(name).length >= minNameLength, "NameService: name too short");
        require(isAvailable(name), "NameService: name not available");

        nameHash = hashName(name);
        uint256 actualDuration = duration > 0 ? duration : defaultDuration;
        uint256 expirationTime = block.timestamp + actualDuration;

        // Remove from previous owner if expired
        if (names[nameHash].exists && names[nameHash].owner != address(0)) {
            _removeFromOwnerList(names[nameHash].owner, nameHash);
        }

        names[nameHash] = NameRecord({
            owner: msg.sender,
            resolver: msg.sender,
            registrationTime: block.timestamp,
            expirationTime: expirationTime,
            exists: true
        });

        resolvers[nameHash] = ResolverData({
            addr: msg.sender,
            contentHash: "",
            email: "",
            url: "",
            description: ""
        });

        ownerNames[msg.sender].push(nameHash);
        collectedFees += registrationFee;

        emit NameRegistered(nameHash, name, msg.sender, expirationTime);
        return nameHash;
    }

    /// @notice Renew a name registration
    /// @param nameHash The hash of the name to renew
    /// @param duration Additional duration in seconds (0 for default)
    function renew(bytes32 nameHash, uint256 duration) public nameExists(nameHash) {
        NameRecord storage record = names[nameHash];
        require(
            record.owner == msg.sender || block.timestamp >= record.expirationTime,
            "NameService: not authorized"
        );

        uint256 actualDuration = duration > 0 ? duration : defaultDuration;
        uint256 baseTime = block.timestamp > record.expirationTime ? block.timestamp : record.expirationTime;
        record.expirationTime = baseTime + actualDuration;

        collectedFees += renewalFee;

        emit NameRenewed(nameHash, record.expirationTime);
    }

    /// @notice Transfer name ownership
    /// @param nameHash The hash of the name
    /// @param newOwner The new owner address
    function transfer(bytes32 nameHash, address newOwner) public nameExists(nameHash) onlyNameOwner(nameHash) {
        require(newOwner != address(0), "NameService: new owner is zero address");
        require(block.timestamp < names[nameHash].expirationTime, "NameService: name expired");

        address oldOwner = names[nameHash].owner;
        names[nameHash].owner = newOwner;

        _removeFromOwnerList(oldOwner, nameHash);
        ownerNames[newOwner].push(nameHash);

        emit NameTransferred(nameHash, oldOwner, newOwner);
    }

    /// @notice Set resolver address for a name
    /// @param nameHash The hash of the name
    /// @param resolver The resolver address
    function setResolver(bytes32 nameHash, address resolver) public nameExists(nameHash) onlyNameOwner(nameHash) {
        names[nameHash].resolver = resolver;
        emit ResolverSet(nameHash, resolver);
    }

    /// @notice Set the resolved address for a name
    /// @param nameHash The hash of the name
    /// @param addr The address to resolve to
    function setAddress(bytes32 nameHash, address addr) public nameExists(nameHash) onlyNameOwner(nameHash) {
        resolvers[nameHash].addr = addr;
        emit AddressChanged(nameHash, addr);
    }

    /// @notice Set content hash for a name
    /// @param nameHash The hash of the name
    /// @param contentHash The content hash
    function setContentHash(bytes32 nameHash, string memory contentHash) public nameExists(nameHash) onlyNameOwner(nameHash) {
        resolvers[nameHash].contentHash = contentHash;
    }

    /// @notice Set email for a name
    /// @param nameHash The hash of the name
    /// @param email The email address
    function setEmail(bytes32 nameHash, string memory email) public nameExists(nameHash) onlyNameOwner(nameHash) {
        resolvers[nameHash].email = email;
    }

    /// @notice Set URL for a name
    /// @param nameHash The hash of the name
    /// @param url The URL
    function setUrl(bytes32 nameHash, string memory url) public nameExists(nameHash) onlyNameOwner(nameHash) {
        resolvers[nameHash].url = url;
    }

    /// @notice Set description for a name
    /// @param nameHash The hash of the name
    /// @param description The description
    function setDescription(bytes32 nameHash, string memory description) public nameExists(nameHash) onlyNameOwner(nameHash) {
        resolvers[nameHash].description = description;
    }

    /// @notice Set a text record for a name
    /// @param nameHash The hash of the name
    /// @param key The record key
    /// @param value The record value
    function setTextRecord(bytes32 nameHash, string memory key, string memory value) public nameExists(nameHash) onlyNameOwner(nameHash) {
        textRecords[nameHash][key] = value;
        emit TextRecordSet(nameHash, key, value);
    }

    /// @notice Resolve a name to an address
    /// @param name The name to resolve
    /// @return The resolved address
    function resolve(string memory name) public view returns (address) {
        bytes32 nameHash = hashName(name);
        require(names[nameHash].exists, "NameService: name not registered");
        require(block.timestamp < names[nameHash].expirationTime, "NameService: name expired");
        return resolvers[nameHash].addr;
    }

    /// @notice Get name record details
    /// @param nameHash The hash of the name
    /// @return owner_ The owner address
    /// @return resolver_ The resolver address
    /// @return expirationTime_ The expiration timestamp
    /// @return isExpired Whether the name is expired
    function getNameRecord(bytes32 nameHash) public view returns (
        address owner_,
        address resolver_,
        uint256 expirationTime_,
        bool isExpired
    ) {
        NameRecord storage record = names[nameHash];
        return (
            record.owner,
            record.resolver,
            record.expirationTime,
            block.timestamp >= record.expirationTime
        );
    }

    /// @notice Get resolver data for a name
    /// @param nameHash The hash of the name
    /// @return addr The resolved address
    /// @return contentHash The content hash
    /// @return email The email
    /// @return url The URL
    function getResolverData(bytes32 nameHash) public view returns (
        address addr,
        string memory contentHash,
        string memory email,
        string memory url
    ) {
        ResolverData storage data = resolvers[nameHash];
        return (data.addr, data.contentHash, data.email, data.url);
    }

    /// @notice Get text record for a name
    /// @param nameHash The hash of the name
    /// @param key The record key
    /// @return The record value
    function getTextRecord(bytes32 nameHash, string memory key) public view returns (string memory) {
        return textRecords[nameHash][key];
    }

    /// @notice Get all names owned by an address
    /// @param addr The owner address
    /// @return Array of name hashes
    function getNamesOf(address addr) public view returns (bytes32[] memory) {
        return ownerNames[addr];
    }

    /// @notice Release a name (give up ownership)
    /// @param nameHash The hash of the name
    function release(bytes32 nameHash) public nameExists(nameHash) onlyNameOwner(nameHash) {
        address oldOwner = names[nameHash].owner;
        names[nameHash].owner = address(0);
        names[nameHash].expirationTime = block.timestamp; // Mark as expired

        _removeFromOwnerList(oldOwner, nameHash);

        emit NameReleased(nameHash, oldOwner);
    }

    /// @notice Internal function to remove name from owner's list
    function _removeFromOwnerList(address addr, bytes32 nameHash) internal {
        bytes32[] storage nameList = ownerNames[addr];
        for (uint256 i = 0; i < nameList.length; i++) {
            if (nameList[i] == nameHash) {
                nameList[i] = nameList[nameList.length - 1];
                nameList.pop();
                break;
            }
        }
    }

    /// @notice Update fees (owner only)
    /// @param _registrationFee New registration fee
    /// @param _renewalFee New renewal fee
    function setFees(uint256 _registrationFee, uint256 _renewalFee) public onlyOwner {
        registrationFee = _registrationFee;
        renewalFee = _renewalFee;
        emit FeesUpdated(_registrationFee, _renewalFee);
    }

    /// @notice Update default duration (owner only)
    /// @param _defaultDuration New default duration
    function setDefaultDuration(uint256 _defaultDuration) public onlyOwner {
        defaultDuration = _defaultDuration;
    }

    /// @notice Update minimum name length (owner only)
    /// @param _minNameLength New minimum length
    function setMinNameLength(uint256 _minNameLength) public onlyOwner {
        minNameLength = _minNameLength;
    }

    /// @notice Withdraw collected fees (owner only)
    /// @return amount The withdrawn amount
    function withdrawFees() public onlyOwner returns (uint256 amount) {
        amount = collectedFees;
        collectedFees = 0;
        return amount;
    }

    /// @notice Check time until expiration
    /// @param nameHash The hash of the name
    /// @return Seconds until expiration, 0 if expired
    function timeUntilExpiration(bytes32 nameHash) public view returns (uint256) {
        if (!names[nameHash].exists) return 0;
        if (block.timestamp >= names[nameHash].expirationTime) return 0;
        return names[nameHash].expirationTime - block.timestamp;
    }
}
