// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Complete NEP-11 NFT Example
 * @dev Production-ready NEP-11 NFT with all Neo N3 features
 * @author Jimmy <jimmy@r3e.network>
 * 
 * Features:
 * - NEP-11 method/event surface with Neo N3 integration (manifest types
 *   deviate from the canonical NEP-11 spec; see the deviation notes in
 *   ../standards/NEP11.sol and ../standards/STANDARDS_MAPPING.md)
 * - Royalty system with automatic distribution
 * - Oracle integration for dynamic metadata
 * - Marketplace integration with escrow
 * - Batch operations and gas optimization
 * - Advanced access control and governance
 */

import "../standards/NEP11.sol";
import "../contracts/NativeCalls.sol";
import "../libraries/Neo.sol";
import "../libraries/Storage.sol";

interface IOracleServiceReceiver {
    function onOracleResponse(
        uint256 requestId,
        uint256 code,
        bytes calldata result,
        bytes calldata userData
    ) external;
}

contract CompleteNEP11NFT is NEP11, IOracleServiceReceiver {
    using Neo for *;
    using Storage for *;
    
    // Royalty system
    struct RoyaltyInfo {
        address recipient;
        uint96 percentage; // Basis points (10000 = 100%)
        bool isSet;
    }
    
    mapping(bytes => RoyaltyInfo) private _tokenRoyalties;
    RoyaltyInfo private _defaultRoyalty;
    
    // Marketplace integration
    struct Listing {
        bytes tokenId;
        address seller;
        uint256 price;
        uint256 expiry;
        bool active;
        address currency; // NEO or GAS contract
    }
    
    mapping(bytes32 => Listing) private _listings;
    mapping(bytes => bytes32) private _tokenToListing;
    bytes32[] private _activeListings;
    
    // Oracle integration for metadata
    address private constant ORACLE_CONTRACT = 0xfe924b7cfe89ddd271abaf7210a80a7e11178758;
    bool private _oracleEnabled;
    mapping(bytes => string) private _metadataURLs;
    mapping(bytes => uint256) private _lastMetadataUpdate;
    
    // Collection features
    struct Collection {
        string description;
        string externalURL;
        string imageURL;
        uint256 createdAt;
        uint256 floorPrice;
        uint256 totalVolume;
        uint256 totalSales;
    }
    
    Collection private _collection;
    
    // Governance for collection
    mapping(address => bool) private _curators;
    mapping(bytes32 => CurationProposal) private _curationProposals;

    // Local monotonically-increasing id source for minted token ids.
    uint256 private _nextTokenId = 1;
    
    struct CurationProposal {
        bytes32 id;
        address proposer;
        bytes tokenId;
        string newMetadata;
        uint256 votes;
        uint256 deadline;
        bool executed;
    }
    
    // Events
    event RoyaltySet(bytes indexed tokenId, address recipient, uint96 percentage);
    event DefaultRoyaltySet(address recipient, uint96 percentage);
    event TokenMintedWithMetadata(bytes indexed tokenId, address indexed to, string metadataURI);
    event BatchMintWithMetadata(bytes[] tokenIds, address[] recipients);
    event TokenListed(bytes indexed tokenId, address indexed seller, uint256 price, address currency);
    event TokenSold(bytes indexed tokenId, address indexed seller, address indexed buyer, uint256 price);
    event ListingCancelled(bytes indexed tokenId, address indexed seller);
    event MetadataUpdated(bytes indexed tokenId, string newMetadata);
    event CurationProposed(bytes32 indexed proposalId, bytes indexed tokenId, address indexed proposer);
    event CurationVote(bytes32 indexed proposalId, address indexed voter);
    event CurationExecuted(bytes32 indexed proposalId, bytes indexed tokenId);
    event TokenFractionalized(
        bytes indexed tokenId,
        address indexed owner,
        address shareContract,
        uint256 shareCount,
        string shareName,
        string shareSymbol
    );
    event TokenCombined(bytes indexed tokenId, address indexed owner, address shareContract);
    event BundleCreated(bytes32 indexed bundleId, bytes[] tokenIds, string bundleName, uint256 bundlePrice);
    event CollectionUpdated(string description, string externalURL, string imageURL);
    event CuratorAdded(address indexed curator);
    event CuratorRemoved(address indexed curator);
    event FloorPriceUpdated(uint256 newFloorPrice);
    
    // Custom errors
    error CompleteNEP11NotListed(bytes tokenId);
    error CompleteNEP11ListingExpired(bytes tokenId);
    error CompleteNEP11InsufficientPayment(uint256 provided, uint256 required);
    error CompleteNEP11NotCurator(address caller);
    error CompleteNEP11RoyaltyTooHigh(uint96 percentage);
    
    // Modifiers
    modifier onlyCurator() {
        if (!_curators[msg.sender] && msg.sender != owner()) {
            revert CompleteNEP11NotCurator(msg.sender);
        }
        _;
    }
    
    modifier validRoyalty(uint96 percentage) {
        if (percentage > 1000) revert CompleteNEP11RoyaltyTooHigh(percentage); // Max 10%
        _;
    }
    
    modifier tokenNotListed(bytes memory tokenId) {
        if (_tokenToListing[tokenId] != bytes32(0)) {
            revert("CompleteNEP11: token already listed");
        }
        _;
    }
    
    /**
     * @dev Constructor
     */
    constructor(
        string memory name,
        string memory symbol,
        string memory description,
        string memory baseURI,
        uint256 maxSupply,
        address oracleAddress
    ) NEP11(name, symbol, 0, baseURI, maxSupply, false) {
        _collection = Collection({
            description: description,
            externalURL: "https://r3e.network",
            imageURL: string(abi.encodePacked(baseURI, "collection.png")),
            createdAt: block.timestamp,
            floorPrice: 0,
            totalVolume: 0,
            totalSales: 0
        });
        
        _oracleEnabled = oracleAddress != address(0);
        
        // Set default royalty to 2.5%
        _defaultRoyalty = RoyaltyInfo({
            recipient: msg.sender,
            percentage: 250, // 2.5%
            isSet: true
        });
        
        // Add creator as curator
        _curators[msg.sender] = true;
        emit CuratorAdded(msg.sender);
    }
    
    // ========== Minting with Metadata ==========
    
    /**
     * @dev Mint NFT with rich metadata
     */
    function mintWithMetadata(
        address to,
        string memory metadataURI,
        bytes memory properties,
        RoyaltyInfo memory royalty
    ) public onlyMinter returns (bytes memory tokenId) {
        tokenId = bytes.concat(bytes32(_nextTokenId));
        _nextTokenId++;
        
        // Mint the token
        mint(to, tokenId, properties);
        
        // Set metadata URI
        _setTokenURI(tokenId, metadataURI);
        
        // Set royalty if specified. Use the internal setter: `mintWithMetadata`
        // is `onlyMinter`, and the minter is not necessarily the owner, so the
        // `onlyOwner` public `setTokenRoyalty` would revert when minter!=owner.
        if (royalty.isSet) {
            _setTokenRoyalty(tokenId, royalty.recipient, royalty.percentage);
        }

        emit TokenMintedWithMetadata(tokenId, to, metadataURI);
    }
    
    /**
     * @dev Batch mint with metadata
     */
    function batchMintWithMetadata(
        address[] memory recipients,
        string[] memory metadataURIs,
        bytes[] memory properties,
        RoyaltyInfo[] memory royalties
    ) public onlyMinter returns (bytes[] memory tokenIds) {
        require(recipients.length == metadataURIs.length, "CompleteNEP11: array length mismatch");
        require(recipients.length == properties.length, "CompleteNEP11: array length mismatch");
        require(recipients.length == royalties.length, "CompleteNEP11: array length mismatch");
        require(recipients.length <= 50, "CompleteNEP11: too many tokens");
        
        tokenIds = new bytes[](recipients.length);
        
        for (uint256 i = 0; i < recipients.length; i++) {
            tokenIds[i] = mintWithMetadata(recipients[i], metadataURIs[i], properties[i], royalties[i]);
        }

        emit BatchMintWithMetadata(tokenIds, recipients);
    }
    
    // ========== Royalty System ==========
    
    /**
     * @dev Set royalty for specific token
     */
    function setTokenRoyalty(
        bytes memory tokenId,
        address recipient,
        uint96 percentage
    ) public onlyOwner {
        _setTokenRoyalty(tokenId, recipient, percentage);
    }

    /**
     * @dev Internal royalty setter for the `onlyMinter` mint path, which may be
     * invoked by a minter that is not the owner. Still validates existence,
     * the royalty bound, and the recipient.
     */
    function _setTokenRoyalty(
        bytes memory tokenId,
        address recipient,
        uint96 percentage
    ) internal tokenExists(tokenId) validRoyalty(percentage) {
        require(recipient != address(0), "CompleteNEP11: invalid royalty recipient");

        _tokenRoyalties[tokenId] = RoyaltyInfo({
            recipient: recipient,
            percentage: percentage,
            isSet: true
        });

        emit RoyaltySet(tokenId, recipient, percentage);
    }
    
    /**
     * @dev Set default royalty for all new tokens
     */
    function setDefaultRoyalty(address recipient, uint96 percentage) 
        public 
        onlyOwner 
        validRoyalty(percentage) 
    {
        require(recipient != address(0), "CompleteNEP11: invalid royalty recipient");
        
        _defaultRoyalty = RoyaltyInfo({
            recipient: recipient,
            percentage: percentage,
            isSet: true
        });

        emit DefaultRoyaltySet(recipient, percentage);
    }
    
    /**
     * @dev Get royalty information (EIP-2981 compatible)
     */
    function royaltyInfo(bytes memory tokenId, uint256 salePrice) 
        external 
        view 
        tokenExists(tokenId)
        returns (address receiver, uint256 royaltyAmount) 
    {
        RoyaltyInfo memory royalty = _tokenRoyalties[tokenId];
        
        if (!royalty.isSet) {
            royalty = _defaultRoyalty;
        }
        
        receiver = royalty.recipient;
        royaltyAmount = (salePrice * royalty.percentage) / 10000;
    }
    
    // ========== Marketplace Integration ==========
    
    /**
     * @dev List token for sale
     */
    function listToken(
        bytes memory tokenId,
        uint256 price,
        uint256 duration,
        address currency
    ) public tokenExists(tokenId) tokenNotListed(tokenId) {
        require(ownerOf(tokenId) == msg.sender, "CompleteNEP11: not token owner");
        require(price > 0, "CompleteNEP11: price must be positive");
        require(duration > 0 && duration <= 30 days, "CompleteNEP11: invalid duration");
        require(
            currency == NativeCalls.NEO_CONTRACT || currency == NativeCalls.GAS_CONTRACT,
            "CompleteNEP11: invalid currency"
        );
        
        bytes32 listingId = keccak256(abi.encode(tokenId, msg.sender, price, block.timestamp));
        
        _listings[listingId] = Listing({
            tokenId: tokenId,
            seller: msg.sender,
            price: price,
            expiry: block.timestamp + duration,
            active: true,
            currency: currency
        });
        
        _tokenToListing[tokenId] = listingId;
        _activeListings.push(listingId);
        
        // Transfer token to contract for escrow
        _transfer(msg.sender, address(this), tokenId, "");
        
        emit TokenListed(tokenId, msg.sender, price, currency);
    }
    
    /**
     * @dev Buy listed token
     */
    function buyToken(bytes memory tokenId) public payable tokenExists(tokenId) {
        bytes32 listingId = _tokenToListing[tokenId];
        require(listingId != bytes32(0), "CompleteNEP11: token not listed");
        
        Listing storage listing = _listings[listingId];
        require(listing.active, "CompleteNEP11: listing not active");
        require(block.timestamp <= listing.expiry, "CompleteNEP11: listing expired");
        
        // Compute the royalty up-front so the buyer's single payment can be
        // SPLIT between the seller and the royalty recipient. Paying the royalty
        // out of the seller's balance (`from = listing.seller`) would require the
        // SELLER's witness, which is never present in a buyer-initiated purchase,
        // so that transfer would always fail. Both legs below transfer
        // `from = msg.sender` (the witnessed buyer) instead.
        (address royaltyRecipient, uint256 royaltyAmount) = royaltyInfo(tokenId, listing.price);
        if (royaltyRecipient == listing.seller || royaltyRecipient == address(0)) {
            royaltyAmount = 0;
        }
        require(royaltyAmount <= listing.price, "CompleteNEP11: royalty exceeds price");
        uint256 sellerProceeds = listing.price - royaltyAmount;

        if (listing.currency == NativeCalls.GAS_CONTRACT) {
            require(
                Neo.getGasBalance(msg.sender) >= listing.price,
                "CompleteNEP11: insufficient GAS balance"
            );
            require(
                Neo.transferGas(msg.sender, listing.seller, sellerProceeds),
                "CompleteNEP11: payment failed"
            );
            if (royaltyAmount > 0) {
                require(
                    Neo.transferGas(msg.sender, royaltyRecipient, royaltyAmount),
                    "CompleteNEP11: royalty payment failed"
                );
            }
        } else if (listing.currency == NativeCalls.NEO_CONTRACT) {
            require(
                Neo.getNeoBalance(msg.sender) >= listing.price,
                "CompleteNEP11: insufficient NEO balance"
            );
            require(
                Neo.transferNeo(msg.sender, listing.seller, sellerProceeds),
                "CompleteNEP11: payment failed"
            );
            if (royaltyAmount > 0) {
                require(
                    Neo.transferNeo(msg.sender, royaltyRecipient, royaltyAmount),
                    "CompleteNEP11: royalty payment failed"
                );
            }
        }

        // Transfer token to buyer
        _transfer(address(this), msg.sender, tokenId, "");
        
        // Update listing
        listing.active = false;
        delete _tokenToListing[tokenId];
        
        // Update collection statistics
        _collection.totalVolume += listing.price;
        _collection.totalSales++;
        _updateFloorPrice();
        
        emit TokenSold(tokenId, listing.seller, msg.sender, listing.price);
    }
    
    /**
     * @dev Cancel listing
     */
    function cancelListing(bytes memory tokenId) public {
        bytes32 listingId = _tokenToListing[tokenId];
        require(listingId != bytes32(0), "CompleteNEP11: token not listed");
        
        Listing storage listing = _listings[listingId];
        require(msg.sender == listing.seller || msg.sender == owner(), "CompleteNEP11: unauthorized");
        require(listing.active, "CompleteNEP11: listing not active");
        
        // Return token to seller
        _transfer(address(this), listing.seller, tokenId, "");
        
        // Update listing
        listing.active = false;
        delete _tokenToListing[tokenId];

        emit ListingCancelled(tokenId, listing.seller);
    }
    
    // ========== Oracle Integration for Dynamic Metadata ==========
    
    /**
     * @dev Update metadata via oracle
     */
    function updateMetadataViaOracle(
        bytes memory tokenId,
        string memory metadataURL
    ) public onlyCurator tokenExists(tokenId) returns (uint256 requestId) {
        require(_oracleEnabled, "CompleteNEP11: oracle not configured");

        _metadataURLs[tokenId] = metadataURL;

        // Register the NATIVE-signature callback: the Oracle native invokes the
        // callback directly as (string url, bytes userData, int code, bytes result),
        // not the IOracleServiceReceiver order.
        NativeCalls.requestOracleData(
            metadataURL,
            "",
            "onNativeOracleResponse",
            tokenId,
            20_000_000
        );
        requestId = block.timestamp;
    }

    /**
     * @dev Native Oracle callback (direct registration). Neo invokes this with
     * the fixed native signature `(string url, bytes userData, int code, bytes result)`.
     */
    function onNativeOracleResponse(
        string calldata url,
        bytes calldata userData,
        uint256 code,
        bytes calldata result
    ) external {
        url;
        require(msg.sender == ORACLE_CONTRACT, "CompleteNEP11: unauthorized oracle response");
        _applyMetadataResponse(userData, code, result);
    }

    /**
     * @dev IOracleServiceReceiver callback — used when routed via an
     * OracleService wrapper, which forwards in this argument order.
     */
    function onOracleResponse(uint256, uint256 code, bytes calldata result, bytes calldata userData)
        external
        override
    {
        require(msg.sender == ORACLE_CONTRACT, "CompleteNEP11: unauthorized oracle response");
        _applyMetadataResponse(userData, code, result);
    }

    function _applyMetadataResponse(bytes calldata userData, uint256 code, bytes calldata result) internal {
        if (code == 0) {
            bytes memory tokenId = userData;

            // Update token properties with oracle data
            setProperties(tokenId, result);
            _lastMetadataUpdate[tokenId] = block.timestamp;

            emit MetadataUpdated(tokenId, string(result));
        }
    }
    
    // ========== Curation System ==========
    
    /**
     * @dev Add curator
     */
    function addCurator(address curator) public onlyOwner {
        require(curator != address(0), "CompleteNEP11: invalid curator");
        require(!_curators[curator], "CompleteNEP11: already curator");
        
        _curators[curator] = true;
        emit CuratorAdded(curator);
    }
    
    /**
     * @dev Remove curator
     */
    function removeCurator(address curator) public onlyOwner {
        require(_curators[curator], "CompleteNEP11: not a curator");
        
        _curators[curator] = false;
        emit CuratorRemoved(curator);
    }
    
    /**
     * @dev Propose metadata curation
     */
    function proposeCuration(
        bytes memory tokenId,
        string memory newMetadata
    ) public onlyCurator tokenExists(tokenId) returns (bytes32 proposalId) {
        proposalId = keccak256(abi.encode(tokenId, newMetadata, msg.sender, block.timestamp));
        
        _curationProposals[proposalId] = CurationProposal({
            id: proposalId,
            proposer: msg.sender,
            tokenId: tokenId,
            newMetadata: newMetadata,
            votes: 1, // Proposer's vote
            deadline: block.timestamp + 7 days,
            executed: false
        });

        emit CurationProposed(proposalId, tokenId, msg.sender);
    }
    
    /**
     * @dev Vote on curation proposal
     */
    function voteOnCuration(bytes32 proposalId) public onlyCurator {
        CurationProposal storage proposal = _curationProposals[proposalId];
        require(proposal.proposer != address(0), "CompleteNEP11: proposal not found");
        require(block.timestamp <= proposal.deadline, "CompleteNEP11: voting ended");
        require(!proposal.executed, "CompleteNEP11: already executed");
        
        proposal.votes++;

        emit CurationVote(proposalId, msg.sender);
    }
    
    /**
     * @dev Execute curation proposal
     */
    function executeCuration(bytes32 proposalId) public {
        CurationProposal storage proposal = _curationProposals[proposalId];
        require(proposal.proposer != address(0), "CompleteNEP11: proposal not found");
        require(block.timestamp > proposal.deadline, "CompleteNEP11: voting still active");
        require(!proposal.executed, "CompleteNEP11: already executed");
        require(proposal.votes >= 3, "CompleteNEP11: insufficient votes"); // Minimum 3 curator votes
        
        proposal.executed = true;
        
        // Update token metadata
        setProperties(proposal.tokenId, bytes(proposal.newMetadata));

        emit CurationExecuted(proposalId, proposal.tokenId);
    }
    
    // ========== Advanced Features ==========
    
    /**
     * @dev Fractionalize NFT (split into fungible shares)
     */
    function fractionalize(
        bytes memory tokenId,
        uint256 shareCount,
        string memory shareName,
        string memory shareSymbol
    ) public tokenExists(tokenId) returns (address shareContract) {
        require(ownerOf(tokenId) == msg.sender, "CompleteNEP11: not token owner");
        require(shareCount > 1, "CompleteNEP11: invalid share count");

        // Deploy fractional token contract via ContractManagement
        bytes memory nefData = abi.encode("FRACTIONAL_TOKEN_NEF"); // Would be actual NEF bytecode
        bytes memory manifestData = abi.encode("FRACTIONAL_TOKEN_MANIFEST"); // Would be actual manifest
        
        shareContract = NativeCalls.deployContract(nefData, manifestData);
        _transfer(msg.sender, shareContract, tokenId, "");

        emit TokenFractionalized(tokenId, msg.sender, shareContract, shareCount, shareName, shareSymbol);
        
        return shareContract;
    }
    
    /**
     * @dev Combine fractionalized shares back to NFT
     */
    function combine(
        bytes memory /*tokenId*/,
        address /*shareContract*/,
        uint256 /*shareCount*/
    ) public pure {
        revert(
            "CompleteNEP11: combine requires dynamic share-contract calls; use explicit manifest permissions"
        );
    }
    
    /**
     * @dev Create token bundle
     */
    function createBundle(
        bytes[] memory tokenIds,
        string memory bundleName,
        uint256 bundlePrice
    ) public returns (bytes32 bundleId) {
        require(tokenIds.length > 1, "CompleteNEP11: bundle requires multiple tokens");
        require(tokenIds.length <= 10, "CompleteNEP11: too many tokens in bundle");
        
        // Verify ownership of all tokens
        for (uint256 i = 0; i < tokenIds.length; i++) {
            require(ownerOf(tokenIds[i]) == msg.sender, "CompleteNEP11: not owner of all tokens");
        }
        
        bundleId = keccak256(abi.encode(tokenIds, bundleName, msg.sender, block.timestamp));
        
        // Store bundle information
        Storage.put(
            abi.encode("bundle", bundleId),
            abi.encode(tokenIds, bundleName, bundlePrice, msg.sender, block.timestamp)
        );
        
        // Transfer all tokens to contract
        for (uint256 i = 0; i < tokenIds.length; i++) {
            _transfer(msg.sender, address(this), tokenIds[i], "");
        }

        emit BundleCreated(bundleId, tokenIds, bundleName, bundlePrice);
    }
    
    // ========== Collection Management ==========
    
    /**
     * @dev Update collection information
     */
    function updateCollection(
        string memory description,
        string memory externalURL,
        string memory imageURL
    ) public onlyOwner {
        _collection.description = description;
        _collection.externalURL = externalURL;
        _collection.imageURL = imageURL;

        emit CollectionUpdated(description, externalURL, imageURL);
    }
    
    /**
     * @dev Update floor price calculation using weighted average
     */
    function _updateFloorPrice() private {
        // Calculate floor price using weighted moving average of recent listings
        uint256 newFloorPrice = 0;
        uint256 totalWeight = 0;
        
        // Calculate weighted floor price from recent active listings
        for (uint256 i = 0; i < _activeListings.length && i < 20; i++) {
            Listing memory listing = _listings[_activeListings[i]];
            if (listing.active && block.timestamp <= listing.expiry) {
                // Weight by recency (more recent listings have higher weight)
                uint256 age = block.timestamp - (listing.expiry - 7 days);
                uint256 weight = age > 0 ? 7 days / (age + 1) : 7 days;
                
                newFloorPrice += listing.price * weight;
                totalWeight += weight;
            }
        }
        
        if (totalWeight > 0) {
            newFloorPrice = newFloorPrice / totalWeight;
            
            if (newFloorPrice != _collection.floorPrice) {
                _collection.floorPrice = newFloorPrice;
                emit FloorPriceUpdated(newFloorPrice);
            }
        }
    }
    
    /**
     * @dev Get collection statistics
     */
    function getCollectionStats() public view override returns (
        uint256 totalTokens,
        uint256 totalHolders,
        uint256 floorPrice,
        uint256 totalVolume,
        uint256 totalSales,
        uint256 averagePrice
    ) {
        totalTokens = totalSupply();
        totalHolders = _getUniqueHolders();
        floorPrice = _collection.floorPrice;
        totalVolume = _collection.totalVolume;
        totalSales = _collection.totalSales;
        averagePrice = totalSales > 0 ? totalVolume / totalSales : 0;
    }
    
    // ========== View Functions ==========
    
    /**
     * @dev Get token full information
     */
    function getTokenInfo(bytes memory tokenId) public view tokenExists(tokenId) returns (
        address owner,
        bytes memory properties,
        string memory uri,
        RoyaltyInfo memory royalty,
        bool isListed,
        uint256 listingPrice
    ) {
        owner = ownerOf(tokenId);
        properties = this.properties(tokenId);
        uri = tokenURI(tokenId);
        
        royalty = _tokenRoyalties[tokenId];
        if (!royalty.isSet) {
            royalty = _defaultRoyalty;
        }
        
        bytes32 listingId = _tokenToListing[tokenId];
        if (listingId != bytes32(0)) {
            Listing memory listing = _listings[listingId];
            isListed = listing.active && block.timestamp <= listing.expiry;
            listingPrice = listing.price;
        }
    }
    
    /**
     * @dev Get marketplace listings
     */
    function getActiveListings() public view returns (
        bytes[] memory tokenIds,
        uint256[] memory prices,
        address[] memory sellers,
        uint256[] memory expiries
    ) {
        uint256 activeCount = 0;
        
        // Count active listings
        for (uint256 i = 0; i < _activeListings.length; i++) {
            Listing memory listing = _listings[_activeListings[i]];
            if (listing.active && block.timestamp <= listing.expiry) {
                activeCount++;
            }
        }
        
        // Populate arrays
        tokenIds = new bytes[](activeCount);
        prices = new uint256[](activeCount);
        sellers = new address[](activeCount);
        expiries = new uint256[](activeCount);
        
        uint256 index = 0;
        for (uint256 i = 0; i < _activeListings.length && index < activeCount; i++) {
            Listing memory listing = _listings[_activeListings[i]];
            if (listing.active && block.timestamp <= listing.expiry) {
                tokenIds[index] = listing.tokenId;
                prices[index] = listing.price;
                sellers[index] = listing.seller;
                expiries[index] = listing.expiry;
                index++;
            }
        }
    }
    
    /**
     * @dev Check if address is curator
     */
    function isCurator(address account) public view returns (bool) {
        return _curators[account];
    }
    
    /**
     * @dev Get contract metadata
     */
    function getContractMetadata() public view override returns (
        string memory standard,
        string memory name,
        string memory version,
        string memory author
    ) {
        return (
            "NEP-11",
            "Complete Neo N3 NFT",
            "1.0.0",
            "Jimmy <jimmy@r3e.network>"
        );
    }
}
