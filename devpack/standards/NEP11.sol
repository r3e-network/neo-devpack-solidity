// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-11 Non-Fungible Token Standard
 * @dev NEP-11-style implementation of Neo N3 non-fungible tokens for Solidity
 * @author Jimmy <jimmy@r3e.network>
 *
 * NEP-11 is Neo's enhanced non-fungible token standard, providing:
 * - Standard ERC-721 compatibility
 * - Neo-specific features (onNEP11Payment callback)
 * - Divisible and indivisible NFT support
 * - Advanced metadata capabilities
 * - Integration with Neo native features
 *
 * Spec conformance notes:
 * - Token IDs are dynamic `bytes` (NEP-11 ByteString, max 64 bytes) and
 *   surface in the manifest as `ByteArray`.
 * - `tokensOf` / `tokens` return a NeoVM storage iterator
 *   (`Syscalls.Iterator`); the manifest declares returntype
 *   `InteropInterface`, exactly like the official C# devpack.
 * - Known remaining deviation: `properties` returns serialized `bytes`
 *   (manifest `ByteArray`) instead of the spec's `Map` — Solidity has no
 *   construct that produces a NeoVM Map stack item as a return value.
 */

import "../contracts/FrameworkBase.sol";
import "../contracts/Types.sol";
import "../libraries/Neo.sol";
import "../libraries/Runtime.sol";
import "../libraries/Storage.sol";

/**
 * @title INEP11
 * @dev Interface for NEP-11 non-fungible token standard
 */
interface INEP11 {
    // Standard NEP-11 functions
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address owner) external view returns (uint256);
    function tokensOf(address owner) external view returns (Syscalls.Iterator memory);
    function ownerOf(bytes memory tokenId) external view returns (address);
    function transfer(address to, bytes memory tokenId, Any calldata data) external returns (bool);
    function properties(bytes memory tokenId) external view returns (bytes memory);

    // Events
    event Transfer(address indexed from, address indexed to, uint256 indexed amount, bytes tokenId);
}

/**
 * @title INEP11Divisible
 * @dev Extended interface for divisible NEP-11 tokens.
 *
 * Divisible NFTs allow fractional ownership. In addition to the base INEP11
 * methods, divisible tokens expose:
 *   - balanceOf(owner, tokenId): fractional balance per token
 *   - transfer(from, to, amount, tokenId, data): fractional transfer
 *   - ownerOf(tokenId): returns ALL owners (not a single address)
 */
interface INEP11Divisible is INEP11 {
    function balanceOf(address owner, bytes memory tokenId) external view returns (uint256);
    function transfer(address from, address to, uint256 amount, bytes memory tokenId, Any calldata data) external returns (bool);
    function ownersOf(bytes memory tokenId) external view returns (address[] memory);
}

/**
 * @title INEP11Receiver
 * @dev Interface for contracts that can receive NEP-11 tokens
 */
interface INEP11Receiver {
    function onNEP11Payment(
        address from,
        uint256 amount,
        bytes calldata tokenId,
        Any calldata data
    ) external;
}

/**
 * @title NEP11
 * @dev Complete NEP-11 token implementation with Neo N3 integration
 */
contract NEP11 is INEP11, FrameworkBase {
    using Neo for *;
    using Runtime for *;

    // ========== Enumeration Index (raw storage) ==========
    //
    // NEP-11 requires `tokens()` / `tokensOf(owner)` to return iterators.
    // Solidity mappings live in keccak-derived slots that cannot be
    // prefix-scanned, so the contract maintains a parallel raw-storage index
    // (mirroring the C# devpack's Prefix_Token / Prefix_AccountToken maps):
    //   - token index:   NEP11_TOKEN_INDEX_PREFIX ++ tokenId            => 0x01
    //   - account index: NEP11_ACCOUNT_INDEX_PREFIX ++ owner ++ tokenId => 0x01
    // Both are scanned with FindOptions.KeysOnly | FindOptions.RemovePrefix
    // so iterator values are the bare token ids.
    bytes constant NEP11_TOKEN_INDEX_PREFIX = "nep11.token.";
    bytes constant NEP11_ACCOUNT_INDEX_PREFIX = "nep11.acct.";
    // Neo N3 FindOptions.KeysOnly (0x01) | FindOptions.RemovePrefix (0x02)
    uint8 constant NEP11_FIND_TOKEN_IDS = 0x03;

    // Token metadata
    string private _name;
    string private _symbol;
    uint8 private _decimals;
    uint256 private _totalSupply;
    uint256 private _currentTokenId;

    // Token tracking (tokenId is NEP-11 ByteString: dynamic bytes, <= 64 bytes)
    mapping(bytes => address) private _owners;
    mapping(address => uint256) private _balances;
    mapping(bytes => address) private _tokenApprovals;
    mapping(address => mapping(address => bool)) private _operatorApprovals;

    // Token properties and metadata
    mapping(bytes => bytes) private _tokenProperties;
    mapping(bytes => string) private _tokenURIs;

    // NEP-11 specific features
    bool private _transfersEnabled = true;
    address private _minter;
    string private _baseURI;
    uint256 private _maxSupply;
    bool private _isDivisible;

    // Events
    event Transfer(address indexed from, address indexed to, uint256 indexed amount, bytes tokenId);
    event Approval(address indexed owner, address indexed approved, bytes tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    event Mint(address indexed to, bytes tokenId);
    event Burn(bytes tokenId);

    // NEP-11 specific events
    event BaseURIUpdated(string newBaseURI);
    event TokenPropertiesUpdated(bytes indexed tokenId, bytes properties);
    event TransfersEnabled();
    event TransfersDisabled();

    // Custom errors
    error NEP11InvalidTokenId(bytes tokenId);
    error NEP11NotOwnerNorApproved(address caller, bytes tokenId);
    error NEP11InvalidReceiver(address receiver);
    error NEP11TransfersDisabled();
    error NEP11ExceedsMaxSupply(uint256 newTotal, uint256 maxSupply);
    error NEP11NotMinter(address caller);
    error NEP11TokenExists(bytes tokenId);

    // Modifiers
    modifier whenTransfersEnabled() {
        if (!_transfersEnabled) revert NEP11TransfersDisabled();
        _;
    }

    modifier onlyMinter() {
        if (msg.sender != _minter) revert NEP11NotMinter(msg.sender);
        _;
    }

    modifier tokenExists(bytes memory tokenId) {
        if (_owners[tokenId] == address(0)) revert NEP11InvalidTokenId(tokenId);
        _;
    }

    modifier validReceiver(address to) {
        if (to == address(0)) revert NEP11InvalidReceiver(to);
        _;
    }

    /**
     * @dev Constructor
     */
    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        string memory baseURI_,
        uint256 maxSupply_,
        bool isDivisible_
    ) FrameworkBase() {
        require(bytes(name_).length > 0, "NEP11: name cannot be empty");
        require(bytes(symbol_).length > 0, "NEP11: symbol cannot be empty");

        _name = name_;
        _symbol = symbol_;
        _decimals = decimals_;
        _baseURI = baseURI_;
        _maxSupply = maxSupply_;
        _isDivisible = isDivisible_;
        _minter = msg.sender;
        _currentTokenId = 1;
    }

    // ========== View Functions ==========

    /**
     * @dev Returns the token collection name
     */
    function name() public view returns (string memory) {
        return _name;
    }

    /**
     * @dev Returns the token collection symbol
     */
    function symbol() public view override returns (string memory) {
        return _symbol;
    }

    /**
     * @dev Returns the number of decimals (0 for indivisible, >0 for divisible)
     */
    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    /**
     * @dev Returns the total supply of tokens
     */
    function totalSupply() public view override returns (uint256) {
        return _totalSupply;
    }

    /**
     * @dev Returns the number of tokens owned by owner
     */
    function balanceOf(address owner) public view override returns (uint256) {
        require(owner != address(0), "NEP11: balance query for zero address");
        return _balances[owner];
    }

    /**
     * @dev Returns an iterator over the token IDs owned by `owner`
     *      (NEP-11: returntype InteropInterface). Iterator values are the
     *      bare token ids (FindOptions.KeysOnly | RemovePrefix).
     */
    function tokensOf(address owner) public view override returns (Syscalls.Iterator memory) {
        require(owner != address(0), "NEP11: tokens query for zero address");
        return Storage.find(
            bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, owner),
            NEP11_FIND_TOKEN_IDS
        );
    }

    /**
     * @dev Returns an iterator over all token IDs (NEP-11 optional method,
     *      returntype InteropInterface).
     */
    function tokens() public view returns (Syscalls.Iterator memory) {
        return Storage.find(NEP11_TOKEN_INDEX_PREFIX, NEP11_FIND_TOKEN_IDS);
    }

    /**
     * @dev Returns the owner of the token
     */
    function ownerOf(bytes memory tokenId) public view override tokenExists(tokenId) returns (address) {
        return _owners[tokenId];
    }

    /**
     * @dev Returns the approved address for a token
     */
    function getApproved(bytes memory tokenId) public view tokenExists(tokenId) returns (address) {
        return _tokenApprovals[tokenId];
    }

    /**
     * @dev Returns if the operator is approved for all tokens of owner
     */
    function isApprovedForAll(address owner, address operator) public view returns (bool) {
        return _operatorApprovals[owner][operator];
    }

    /**
     * @dev Returns token properties.
     *
     * NEP-11 deviation: the spec types this as a Map of property name to
     * value; Solidity cannot construct a NeoVM Map stack item as a return
     * value, so the devpack returns the serialized properties blob
     * (manifest type ByteArray). See STANDARDS_MAPPING.md.
     */
    function properties(bytes memory tokenId) public view override tokenExists(tokenId) returns (bytes memory) {
        return _tokenProperties[tokenId];
    }

    /**
     * @dev Returns token URI
     */
    function tokenURI(bytes memory tokenId) public view tokenExists(tokenId) returns (string memory) {
        string memory _tokenURI = _tokenURIs[tokenId];

        // If token has specific URI, return it
        if (bytes(_tokenURI).length > 0) {
            return _tokenURI;
        }

        // Otherwise, construct from base URI
        return string(abi.encodePacked(_baseURI, _toHexString(tokenId)));
    }

    /**
     * @dev Returns base URI
     */
    function baseURI() public view returns (string memory) {
        return _baseURI;
    }

    /**
     * @dev Returns max supply
     */
    function maxSupply() public view returns (uint256) {
        return _maxSupply;
    }

    /**
     * @dev Returns if token is divisible
     */
    function isDivisible() public view returns (bool) {
        return _isDivisible;
    }

    // ========== Transfer Functions ==========

    /**
     * @dev NEP-11 transfer function
     */
    function transfer(
        address to,
        bytes memory tokenId,
        Any calldata data
    ) public override whenTransfersEnabled validReceiver(to) tokenExists(tokenId) returns (bool) {
        address owner = ownerOf(tokenId);

        require(
            msg.sender == owner ||
            getApproved(tokenId) == msg.sender ||
            isApprovedForAll(owner, msg.sender) ||
            Runtime.checkWitness(owner),
            "NEP11: unauthorized transfer"
        );

        _transfer(owner, to, tokenId, data);
        return true;
    }

    /**
     * @dev Safe transfer with callback
     */
    function safeTransfer(
        address from,
        address to,
        bytes memory tokenId,
        bytes memory data
    ) public {
        require(ownerOf(tokenId) == from, "NEP11: transfer from incorrect owner");
        transfer(to, tokenId, data);
    }

    /**
     * @dev Approve another address to transfer specific token
     */
    function approve(address to, bytes memory tokenId) public tokenExists(tokenId) {
        address owner = ownerOf(tokenId);
        require(to != owner, "NEP11: approval to current owner");

        require(
            msg.sender == owner || isApprovedForAll(owner, msg.sender),
            "NEP11: approve caller is not owner nor approved for all"
        );

        _approve(to, tokenId);
    }

    /**
     * @dev Approve or remove operator for all tokens
     */
    function setApprovalForAll(address operator, bool approved) public {
        require(operator != msg.sender, "NEP11: approve to caller");

        _operatorApprovals[msg.sender][operator] = approved;
        emit ApprovalForAll(msg.sender, operator, approved);
    }

    // ========== Minting and Burning ==========

    /**
     * @dev Mint new token
     */
    function mint(
        address to,
        bytes memory tokenId,
        bytes memory properties
    ) public onlyMinter validReceiver(to) returns (bool) {
        if (_owners[tokenId] != address(0)) revert NEP11TokenExists(tokenId);
        if (!isValidTokenId(tokenId)) revert NEP11InvalidTokenId(tokenId);

        if (_maxSupply > 0 && _totalSupply >= _maxSupply) {
            revert NEP11ExceedsMaxSupply(_totalSupply + 1, _maxSupply);
        }

        _mint(to, tokenId, properties);
        return true;
    }

    /**
     * @dev Mint with auto-generated ID (32-byte big-endian counter value)
     */
    function mintAuto(address to, bytes memory properties) public onlyMinter returns (bytes memory) {
        bytes memory tokenId = bytes.concat(bytes32(_currentTokenId));
        _currentTokenId++;

        mint(to, tokenId, properties);
        return tokenId;
    }

    /**
     * @dev Batch mint tokens
     */
    function batchMint(
        address[] memory recipients,
        bytes[] memory tokenIds,
        bytes[] memory properties
    ) public onlyMinter returns (bool) {
        require(recipients.length == tokenIds.length, "NEP11: array length mismatch");
        require(recipients.length == properties.length, "NEP11: array length mismatch");
        require(recipients.length > 0, "NEP11: empty arrays");
        require(recipients.length <= 100, "NEP11: too many tokens");

        for (uint256 i = 0; i < recipients.length; i++) {
            mint(recipients[i], tokenIds[i], properties[i]);
        }

        return true;
    }

    /**
     * @dev Burn token
     */
    function burn(bytes memory tokenId) public tokenExists(tokenId) {
        address owner = ownerOf(tokenId);

        require(
            msg.sender == owner ||
            getApproved(tokenId) == msg.sender ||
            isApprovedForAll(owner, msg.sender),
            "NEP11: burn caller is not owner nor approved"
        );

        _burn(tokenId);
    }

    // ========== Enumeration Functions ==========

    /**
     * @dev Returns token by index. O(totalSupply) iterator walk over the
     *      raw-storage token index (kept for ERC-721-Enumerable parity; not
     *      part of NEP-11, which exposes iterators instead).
     */
    function tokenByIndex(uint256 index) public view returns (bytes memory) {
        require(index < totalSupply(), "NEP11: global index out of bounds");
        Syscalls.Iterator memory it = Storage.find(NEP11_TOKEN_INDEX_PREFIX, NEP11_FIND_TOKEN_IDS);
        uint256 i = 0;
        while (Syscalls.iteratorNext(it)) {
            if (i == index) {
                return Syscalls.iteratorValue(it);
            }
            i++;
        }
        revert("NEP11: global index out of bounds");
    }

    /**
     * @dev Returns token of owner by index. O(balanceOf(owner)) iterator
     *      walk over the raw-storage account index.
     */
    function tokenOfOwnerByIndex(address owner, uint256 index) public view returns (bytes memory) {
        require(index < balanceOf(owner), "NEP11: owner index out of bounds");
        Syscalls.Iterator memory it = Storage.find(
            bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, owner),
            NEP11_FIND_TOKEN_IDS
        );
        uint256 i = 0;
        while (Syscalls.iteratorNext(it)) {
            if (i == index) {
                return Syscalls.iteratorValue(it);
            }
            i++;
        }
        revert("NEP11: owner index out of bounds");
    }

    // ========== Metadata Functions ==========

    /**
     * @dev Set token properties
     */
    function setProperties(bytes memory tokenId, bytes memory properties)
        public
        onlyOwner
    {
        _setProperties(tokenId, properties);
    }

    /**
     * @dev Internal property setter for derived contracts whose callers are
     * authenticated by their own logic (e.g. an authorized oracle callback or
     * a quorum-gated curation execution) rather than by `onlyOwner`. Mirrors
     * the `setTokenURI` / `_setTokenURI` split. Still enforces token existence.
     */
    function _setProperties(bytes memory tokenId, bytes memory properties)
        internal
        tokenExists(tokenId)
    {
        _tokenProperties[tokenId] = properties;
        emit TokenPropertiesUpdated(tokenId, properties);
    }

    /**
     * @dev Set token URI
     */
    function setTokenURI(bytes memory tokenId, string memory uri)
        public
        onlyOwner
        tokenExists(tokenId)
    {
        _setTokenURI(tokenId, uri);
    }

    /**
     * @dev Internal token URI setter for derived contracts.
     */
    function _setTokenURI(bytes memory tokenId, string memory uri) internal {
        _tokenURIs[tokenId] = uri;
    }

    /**
     * @dev Set base URI for all tokens
     */
    function setBaseURI(string memory newBaseURI) public onlyOwner {
        _baseURI = newBaseURI;
        emit BaseURIUpdated(newBaseURI);
    }

    // ========== Admin Functions ==========

    /**
     * @dev Enable transfers
     */
    function enableTransfers() public onlyOwner {
        require(!_transfersEnabled, "NEP11: transfers already enabled");
        _transfersEnabled = true;
        emit TransfersEnabled();
    }

    /**
     * @dev Disable transfers
     */
    function disableTransfers() public onlyOwner {
        require(_transfersEnabled, "NEP11: transfers already disabled");
        _transfersEnabled = false;
        emit TransfersDisabled();
    }

    /**
     * @dev Change minter
     */
    function changeMinter(address newMinter) public onlyOwner {
        require(newMinter != address(0), "NEP11: new minter is zero address");
        _minter = newMinter;
    }

    /**
     * @dev Set maximum supply
     */
    function setMaxSupply(uint256 newMaxSupply) public onlyOwner {
        require(newMaxSupply >= _totalSupply, "NEP11: max supply below current supply");
        _maxSupply = newMaxSupply;
    }

    // ========== Internal Functions ==========

    /**
     * @dev Internal transfer function
     */
    function _transfer(address from, address to, bytes memory tokenId, bytes memory data) internal {
        require(ownerOf(tokenId) == from, "NEP11: transfer from incorrect owner");

        // Clear approvals
        _approve(address(0), tokenId);

        // Move the account-index entry to the new owner
        Storage.remove(bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, from, tokenId));
        Storage.put(bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, to, tokenId), hex"01");

        // Update balances and ownership
        _balances[from] -= 1;
        _balances[to] += 1;
        _owners[tokenId] = to;

        emit Transfer(from, to, 1, tokenId);

        // Call onNEP11Payment if recipient is a contract.
        // M-DEV1 fix: short-circuit self-transfers (to == address(this)). The
        // NFT contract itself almost never implements INEP11Receiver, so
        // calling onNEP11Payment on self would revert and hard-block any
        // escrow / custody flow. This mirrors NEP17.sol's self-escrow guard
        // and lets contracts custody their own tokens.
        if (to != address(this) && to.code.length > 0) {
            try INEP11Receiver(to).onNEP11Payment(from, 1, tokenId, data) {
                // Success
            } catch {
                revert NEP11InvalidReceiver(to);
            }
        }
    }

    /**
     * @dev Internal mint function
     */
    function _mint(address to, bytes memory tokenId, bytes memory properties) internal {
        require(to != address(0), "NEP11: mint to zero address");
        require(_owners[tokenId] == address(0), "NEP11: token already minted");

        // Record the enumeration index entries
        Storage.put(bytes.concat(NEP11_TOKEN_INDEX_PREFIX, tokenId), hex"01");
        Storage.put(bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, to, tokenId), hex"01");

        // Update state
        _balances[to] += 1;
        _owners[tokenId] = to;
        _totalSupply += 1;

        // Set properties
        _tokenProperties[tokenId] = properties;

        emit Transfer(address(0), to, 1, tokenId);
        emit Mint(to, tokenId);

        // Call onNEP11Payment if recipient is a contract.
        // M-DEV1 fix: same self-escrow short-circuit as _transfer — minting to
        // self (custody) must not require the NFT contract to implement its
        // own receiver callback.
        if (to != address(this) && to.code.length > 0) {
            try INEP11Receiver(to).onNEP11Payment(address(0), 1, tokenId, "") {
                // Success
            } catch {
                // Mint can proceed even if recipient doesn't implement interface
            }
        }
    }

    /**
     * @dev Internal burn function
     */
    function _burn(bytes memory tokenId) internal {
        address owner = ownerOf(tokenId);

        // Clear approvals
        _approve(address(0), tokenId);

        // Drop the enumeration index entries
        Storage.remove(bytes.concat(NEP11_TOKEN_INDEX_PREFIX, tokenId));
        Storage.remove(bytes.concat(NEP11_ACCOUNT_INDEX_PREFIX, owner, tokenId));

        // Update state
        _balances[owner] -= 1;
        delete _owners[tokenId];
        delete _tokenProperties[tokenId];
        delete _tokenURIs[tokenId];
        _totalSupply -= 1;

        emit Transfer(owner, address(0), 1, tokenId);
        emit Burn(tokenId);
    }

    /**
     * @dev Internal approve function
     */
    function _approve(address to, bytes memory tokenId) internal {
        _tokenApprovals[tokenId] = to;
        emit Approval(ownerOf(tokenId), to, tokenId);
    }

    // ========== Neo Integration Functions ==========

    /**
     * @dev Get contract metadata for Neo
     */
    function getContractMetadata() public view virtual returns (
        string memory standard,
        string memory name,
        string memory version,
        string memory author
    ) {
        return (
            "NEP-11",
            "Neo N3 Non-Fungible Token",
            "1.0.0",
            "Jimmy <jimmy@r3e.network>"
        );
    }

    /**
     * @dev Get collection statistics
     */
    function getCollectionStats() public view virtual returns (
        uint256 totalTokens,
        uint256 totalHolders,
        uint256 maxTokens,
        bool divisible
    ) {
        return (_totalSupply, _getUniqueHolders(), _maxSupply, _isDivisible);
    }

    /**
     * @dev Get unique holders count.
     *
     * Walks the raw-storage token index iterator, collecting distinct owners.
     * Cost is O(totalSupply) — use with caution on large collections.
     */
    function _getUniqueHolders() private view returns (uint256) {
        uint256 total = _totalSupply;
        if (total == 0) return 0;

        // Cap iteration to avoid excessive gas usage
        uint256 cap = total < 10000 ? total : 10000;

        // Simple counting via a temporary array of seen addresses
        address[] memory seen = new address[](cap);
        uint256 count = 0;
        uint256 scanned = 0;

        Syscalls.Iterator memory it = Storage.find(NEP11_TOKEN_INDEX_PREFIX, NEP11_FIND_TOKEN_IDS);
        while (scanned < cap && Syscalls.iteratorNext(it)) {
            scanned++;
            address owner = _owners[Syscalls.iteratorValue(it)];
            if (owner == address(0)) continue;

            bool found = false;
            for (uint256 j = 0; j < count; j++) {
                if (seen[j] == owner) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                seen[count] = owner;
                count++;
            }
        }

        return count;
    }

    // ========== Utility Functions ==========

    /**
     * @dev Convert a token id to a hex string
     */
    function _toHexString(bytes memory value) private pure returns (string memory) {
        bytes memory buffer = new bytes(value.length * 2);
        for (uint256 i = 0; i < value.length; i++) {
            buffer[i * 2] = _hexChar(uint8(value[i]) / 16);
            buffer[i * 2 + 1] = _hexChar(uint8(value[i]) % 16);
        }
        return string(buffer);
    }

    /**
     * @dev Get hex character
     */
    function _hexChar(uint8 value) private pure returns (bytes1) {
        if (value < 10) {
            return bytes1(uint8(bytes1('0')) + value);
        } else {
            return bytes1(uint8(bytes1('a')) + value - 10);
        }
    }

    /**
     * @dev Generate unique token ID
     * @notice Internal — prevents external callers from predicting token IDs.
     */
    function generateTokenId(address minter, uint256 nonce) internal view returns (bytes memory) {
        return bytes.concat(keccak256(abi.encode(minter, nonce, block.timestamp)));
    }

    /**
     * @dev Validate token ID format (NEP-11: ByteString, 1..64 bytes)
     */
    function isValidTokenId(bytes memory tokenId) public pure returns (bool) {
        return tokenId.length > 0 && tokenId.length <= 64;
    }
}
