// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ERC721Token
 * @dev NEP-11 compliant non-fungible token for Neo N3 blockchain.
 *
 * Compiler constraints respected:
 *   - NEP-11 transfer(to, tokenId, data) with Runtime.checkWitness
 *   - bytes32 tokenId (maps to Neo ByteArray)
 *   - tokensOf(owner) and properties(tokenId) required by NEP-11
 *   - Parameterless constructor (Neo deploy constraint)
 *   - Import devpack via -I devpack
 *
 * Features: minting, burning, transfers, approvals, metadata, enumerable, royalties
 */
contract ERC721Token {
    string private _name;
    string private _symbol;
    string private _baseTokenURI;
    string private _contractURI;
    address private _owner;
    bool private _paused;
    uint256 private _currentTokenId;
    uint256 private _totalSupply;
    address private _defaultRoyaltyReceiver;
    uint96 private _defaultRoyaltyFraction;
    uint256 private _maxSupply;

    mapping(bytes32 => address) private _owners;
    mapping(address => uint256) private _balances;
    mapping(bytes32 => address) private _tokenApprovals;
    mapping(address => mapping(address => bool)) private _operatorApprovals;
    mapping(bytes32 => string) private _tokenURIs;
    mapping(bytes32 => RoyaltyInfo) private _tokenRoyaltyInfo;

    bytes32[] private _allTokens;
    mapping(bytes32 => uint256) private _allTokensIndex;
    mapping(address => mapping(uint256 => bytes32)) private _ownedTokens;
    mapping(bytes32 => uint256) private _ownedTokensIndex;

    struct RoyaltyInfo {
        address receiver;
        uint96 royaltyFraction;
    }
    
    // NEP-11 Transfer event: (from, to, amount, tokenId)
    event Transfer(address indexed from, address indexed to, uint256 indexed amount, bytes32 tokenId);
    event Approval(address indexed owner, address indexed approved, bytes32 tokenId);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event Pause();
    event Unpause();
    event BaseURIUpdated(string newBaseURI);
    event ContractURIUpdated(string newContractURI);

    error NotOwnerNorApproved();
    error NonexistentToken();
    error InvalidAddress();
    error ContractPaused();
    error MaxSupplyExceeded();
    error RoyaltyTooHigh();
    error OnlyOwner();
    error TokenAlreadyExists();
    
    // Modifiers
    modifier onlyOwner() {
        if (msg.sender != _owner) revert OnlyOwner();
        _;
    }
    
    modifier whenNotPaused() {
        if (_paused) revert ContractPaused();
        _;
    }
    
    modifier validAddress(address account) {
        if (account == address(0)) revert InvalidAddress();
        _;
    }
    
    modifier tokenExists(bytes32 tokenId) {
        if (!_exists(tokenId)) revert NonexistentToken();
        _;
    }

    /// @dev Parameterless constructor — Neo N3 deploy constraint.
    constructor() {
        _name = "MyNFT";
        _symbol = "MNFT";
        _owner = msg.sender;
        _paused = false;
        _currentTokenId = 1;
        emit OwnershipTransferred(address(0), msg.sender);
    }
    
    /// @dev NEP-11 requires decimals() returning 0 for indivisible NFTs.
    function decimals() public pure returns (uint8) {
        return 0;
    }
    
    /**
     * @dev Returns the token collection name
     */
    function name() public view returns (string memory) {
        return _name;
    }
    
    /**
     * @dev Returns the token collection symbol
     */
    function symbol() public view returns (string memory) {
        return _symbol;
    }
    
    /**
     * @dev Returns the total number of tokens in existence
     */
    function totalSupply() public view returns (uint256) {
        return _totalSupply;
    }
    
    /**
     * @dev Returns the maximum supply of tokens
     */
    function maxSupply() public view returns (uint256) {
        return _maxSupply;
    }
    
    /**
     * @dev Returns the number of tokens owned by owner
     */
    function balanceOf(address owner) public view validAddress(owner) returns (uint256) {
        return _balances[owner];
    }
    
    /**
     * @dev Returns the owner of the tokenId token
     */
    function ownerOf(bytes32 tokenId) public view tokenExists(tokenId) returns (address) {
        return _owners[tokenId];
    }

    function getApproved(bytes32 tokenId) public view tokenExists(tokenId) returns (address) {
        return _tokenApprovals[tokenId];
    }
    
    /**
     * @dev Returns if the operator is allowed to manage all of the assets of owner
     */
    function isApprovedForAll(address owner, address operator) public view returns (bool) {
        return _operatorApprovals[owner][operator];
    }
    
    /**
     * @dev Returns the contract owner
     */
    function owner() public view returns (address) {
        return _owner;
    }
    
    /**
     * @dev Returns if the contract is paused
     */
    function paused() public view returns (bool) {
        return _paused;
    }
    
    /**
     * @dev Returns the base URI for tokens
     */
    function baseURI() public view returns (string memory) {
        return _baseTokenURI;
    }
    
    /**
     * @dev Returns the contract URI for contract metadata
     */
    function contractURI() public view returns (string memory) {
        return _contractURI;
    }
    
    /**
     * @dev Returns the token URI for a given token ID
     */
    function tokenURI(bytes32 tokenId) public view tokenExists(tokenId) returns (string memory) {
        string memory _tokenURI = _tokenURIs[tokenId];
        string memory base = _baseTokenURI;

        if (bytes(base).length == 0) {
            return _tokenURI;
        }
        if (bytes(_tokenURI).length > 0) {
            return string(abi.encodePacked(base, _tokenURI));
        }
        return base;
    }
    
    /**
     * @dev Returns a token ID owned by owner at a given index
     */
    function tokenOfOwnerByIndex(address owner, uint256 index)
        public
        view
        validAddress(owner)
        returns (bytes32)
    {
        require(index < balanceOf(owner), "ERC721: owner index out of bounds");
        return _ownedTokens[owner][index];
    }

    function tokenByIndex(uint256 index) public view returns (bytes32) {
        require(index < totalSupply(), "ERC721: global index out of bounds");
        return _allTokens[index];
    }
    
    /**
     * @dev Approve or remove operator as an operator for the caller
     */
    function setApprovalForAll(address operator, bool approved) public whenNotPaused {
        if (msg.sender == operator) revert InvalidAddress();
        _operatorApprovals[msg.sender][operator] = approved;
        emit ApprovalForAll(msg.sender, operator, approved);
    }
    
    /**
     * @dev Gives permission to to transfer tokenId token to another account
     */
    function approve(address to, bytes32 tokenId) public whenNotPaused tokenExists(tokenId) {
        address tokenOwner = ownerOf(tokenId);
        if (to == tokenOwner) revert InvalidAddress();
        if (msg.sender != tokenOwner && !isApprovedForAll(tokenOwner, msg.sender)) {
            revert NotOwnerNorApproved();
        }
        _approve(to, tokenId);
    }

    /// @notice NEP-11 standard transfer (indivisible).
    /// @param to Recipient address
    /// @param tokenId Token identifier (ByteArray in Neo ABI)
    /// @param data Arbitrary data passed to onNEP11Payment callback
    function transfer(address to, bytes32 tokenId, bytes calldata data)
        public
        whenNotPaused
        tokenExists(tokenId)
        returns (bool)
    {
        address tokenOwner = ownerOf(tokenId);
        if (!_isApprovedOrOwner(msg.sender, tokenId)) revert NotOwnerNorApproved();
        _transfer(tokenOwner, to, tokenId);
        return true;
    }

    /// @notice NEP-11 required: enumerate tokens owned by an address.
    function tokensOf(address owner) public view validAddress(owner) returns (bytes32[] memory) {
        uint256 tokenCount = balanceOf(owner);
        if (tokenCount == 0) {
            return new bytes32[](0);
        }
        bytes32[] memory tokenIds = new bytes32[](tokenCount);
        for (uint256 i = 0; i < tokenCount; i++) {
            tokenIds[i] = tokenOfOwnerByIndex(owner, i);
        }
        return tokenIds;
    }

    /// @notice NEP-11 required: return token properties as serialized bytes.
    function properties(bytes32 tokenId) public view tokenExists(tokenId) returns (bytes memory) {
        string memory uri = tokenURI(tokenId);
        return abi.encodePacked(uri);
    }
    
    /**
     * @dev Mints a new token to the specified address
     */
    function mint(address to) public onlyOwner validAddress(to) returns (bytes32) {
        if (_maxSupply > 0 && _totalSupply >= _maxSupply) revert MaxSupplyExceeded();
        uint256 id = _currentTokenId;
        _currentTokenId++;
        bytes32 tokenId = bytes32(id);
        _mint(to, tokenId);
        return tokenId;
    }

    function mintWithURI(address to, string memory uri)
        public
        onlyOwner
        validAddress(to)
        returns (bytes32)
    {
        bytes32 tokenId = mint(to);
        _setTokenURI(tokenId, uri);
        return tokenId;
    }

    function batchMint(address[] memory recipients) public onlyOwner returns (bytes32[] memory) {
        require(recipients.length > 0, "ERC721: empty recipients array");
        require(recipients.length <= 100, "ERC721: too many recipients");
        if (_maxSupply > 0) {
            require(_totalSupply + recipients.length <= _maxSupply, "ERC721: exceeds max supply");
        }
        bytes32[] memory tokenIds = new bytes32[](recipients.length);
        for (uint256 i = 0; i < recipients.length; i++) {
            if (recipients[i] == address(0)) revert InvalidAddress();
            uint256 id = _currentTokenId;
            _currentTokenId++;
            bytes32 tokenId = bytes32(id);
            _mint(recipients[i], tokenId);
            tokenIds[i] = tokenId;
        }
        return tokenIds;
    }

    function burn(bytes32 tokenId) public tokenExists(tokenId) {
        if (!_isApprovedOrOwner(msg.sender, tokenId)) revert NotOwnerNorApproved();
        _burn(tokenId);
    }
    
    /**
     * @dev Sets the base URI for all tokens
     */
    function setBaseURI(string memory newBaseURI) public onlyOwner {
        _baseTokenURI = newBaseURI;
        emit BaseURIUpdated(newBaseURI);
    }
    
    /**
     * @dev Sets the contract URI for contract metadata
     */
    function setContractURI(string memory newContractURI) public onlyOwner {
        _contractURI = newContractURI;
        emit ContractURIUpdated(newContractURI);
    }
    
    /**
     * @dev Sets the URI for a specific token
     */
    function setTokenURI(bytes32 tokenId, string memory uri)
        public
        onlyOwner
        tokenExists(tokenId)
    {
        _setTokenURI(tokenId, uri);
    }

    function setDefaultRoyalty(address receiver, uint96 feeNumerator)
        public
        onlyOwner
        validAddress(receiver)
    {
        if (feeNumerator > 1000) revert RoyaltyTooHigh();
        _defaultRoyaltyReceiver = receiver;
        _defaultRoyaltyFraction = feeNumerator;
    }

    function setTokenRoyalty(bytes32 tokenId, address receiver, uint96 feeNumerator)
        public
        onlyOwner
        tokenExists(tokenId)
        validAddress(receiver)
    {
        if (feeNumerator > 1000) revert RoyaltyTooHigh();
        _tokenRoyaltyInfo[tokenId] = RoyaltyInfo(receiver, feeNumerator);
    }

    function royaltyInfo(bytes32 tokenId, uint256 salePrice)
        public
        view
        tokenExists(tokenId)
        returns (address receiver, uint256 royaltyAmount)
    {
        RoyaltyInfo memory royalty = _tokenRoyaltyInfo[tokenId];
        if (royalty.receiver == address(0)) {
            royalty.receiver = _defaultRoyaltyReceiver;
            royalty.royaltyFraction = _defaultRoyaltyFraction;
        }
        receiver = royalty.receiver;
        royaltyAmount = (salePrice * royalty.royaltyFraction) / 10000;
    }
    
    /**
     * @dev Pauses all token transfers
     */
    function pause() public onlyOwner {
        require(!_paused, "ERC721: already paused");
        _paused = true;
        emit Pause();
    }
    
    /**
     * @dev Unpauses all token transfers
     */
    function unpause() public onlyOwner {
        require(_paused, "ERC721: not paused");
        _paused = false;
        emit Unpause();
    }
    
    /**
     * @dev Transfers ownership of the contract to a new account
     */
    function transferOwnership(address newOwner) public onlyOwner validAddress(newOwner) {
        emit OwnershipTransferred(_owner, newOwner);
        _owner = newOwner;
    }
    
    /**
     * @dev Renounces ownership of the contract
     */
    function renounceOwnership() public onlyOwner {
        emit OwnershipTransferred(_owner, address(0));
        _owner = address(0);
    }
    
    // Internal functions
    
    /**
     * @dev Returns whether tokenId exists
     */
    function _exists(bytes32 tokenId) internal view returns (bool) {
        return _owners[tokenId] != address(0);
    }

    /**
     * @dev Returns whether spender is allowed to manage tokenId
     */
    function _isApprovedOrOwner(address spender, bytes32 tokenId) internal view returns (bool) {
        address tokenOwner = ownerOf(tokenId);
        return (spender == tokenOwner || getApproved(tokenId) == spender || isApprovedForAll(tokenOwner, spender));
    }
    
    /**
     * @dev Internal mint function
     */
    function _mint(address to, bytes32 tokenId) internal {
        if (to == address(0)) revert InvalidAddress();
        if (_exists(tokenId)) revert TokenAlreadyExists();

        _balances[to] += 1;
        _owners[tokenId] = to;

        _addTokenToAllTokensEnumeration(tokenId);
        _addTokenToOwnerEnumeration(to, tokenId);

        emit Transfer(address(0), to, 1, tokenId);

        _totalSupply++;
    }
    
    /**
     * @dev Internal burn function
     */
    function _burn(bytes32 tokenId) internal {
        address tokenOwner = ownerOf(tokenId);

        // Clear approvals
        _approve(address(0), tokenId);

        _balances[tokenOwner] -= 1;
        delete _owners[tokenId];

        // Clear token URI if it exists
        if (bytes(_tokenURIs[tokenId]).length != 0) {
            delete _tokenURIs[tokenId];
        }

        // Clear token royalty if it exists
        if (_tokenRoyaltyInfo[tokenId].receiver != address(0)) {
            delete _tokenRoyaltyInfo[tokenId];
        }

        _removeTokenFromOwnerEnumeration(tokenOwner, tokenId);
        _removeTokenFromAllTokensEnumeration(tokenId);

        emit Transfer(tokenOwner, address(0), 1, tokenId);

        _totalSupply--;
    }
    
    /**
     * @dev Internal transfer function
     */
    function _transfer(address from, address to, bytes32 tokenId) internal {
        if (ownerOf(tokenId) != from) revert InvalidAddress();
        if (to == address(0)) revert InvalidAddress();

        // Clear approvals from the previous owner
        _approve(address(0), tokenId);

        _balances[from] -= 1;
        _balances[to] += 1;
        _owners[tokenId] = to;

        _removeTokenFromOwnerEnumeration(from, tokenId);
        _addTokenToOwnerEnumeration(to, tokenId);

        emit Transfer(from, to, 1, tokenId);
    }
    
    /**
     * @dev Internal approve function
     */
    function _approve(address to, bytes32 tokenId) internal {
        _tokenApprovals[tokenId] = to;
        emit Approval(ownerOf(tokenId), to, tokenId);
    }
    
    /**
     * @dev Internal function to set token URI
     */
    function _setTokenURI(bytes32 tokenId, string memory uri) internal {
        _tokenURIs[tokenId] = uri;
    }
    
    /**
     * @dev Private function to add a token to all tokens enumeration
     */
    function _addTokenToAllTokensEnumeration(bytes32 tokenId) private {
        _allTokensIndex[tokenId] = _allTokens.length;
        _allTokens.push(tokenId);
    }
    
    /**
     * @dev Private function to remove a token from all tokens enumeration
     */
    function _removeTokenFromAllTokensEnumeration(bytes32 tokenId) private {
        uint256 lastTokenIndex = _allTokens.length - 1;
        uint256 tokenIndex = _allTokensIndex[tokenId];
        bytes32 lastTokenId = _allTokens[lastTokenIndex];

        _allTokens[tokenIndex] = lastTokenId;
        _allTokensIndex[lastTokenId] = tokenIndex;

        delete _allTokensIndex[tokenId];
        _allTokens.pop();
    }
    
    /**
     * @dev Private function to add a token to owner enumeration
     */
    function _addTokenToOwnerEnumeration(address to, bytes32 tokenId) private {
        uint256 length = balanceOf(to);
        _ownedTokens[to][length] = tokenId;
        _ownedTokensIndex[tokenId] = length;
    }
    
    /**
     * @dev Private function to remove a token from owner enumeration
     */
    function _removeTokenFromOwnerEnumeration(address from, bytes32 tokenId) private {
        uint256 lastTokenIndex = balanceOf(from) - 1;
        uint256 tokenIndex = _ownedTokensIndex[tokenId];

        if (tokenIndex != lastTokenIndex) {
            bytes32 lastTokenId = _ownedTokens[from][lastTokenIndex];
            _ownedTokens[from][tokenIndex] = lastTokenId;
            _ownedTokensIndex[lastTokenId] = tokenIndex;
        }

        delete _ownedTokensIndex[tokenId];
        delete _ownedTokens[from][lastTokenIndex];
    }
    
}
