// SPDX-License-Identifier: MIT
// ERC721A v4.3.0 (chiru-labs/ERC721A) — gas-optimized batch-mint ERC-721.
// Self-contained: IERC721A + ERC721A + a concrete ERC721ACollection are in this one
// file (the './IERC721A.sol' import was inlined). Contract logic is unmodified; NatSpec
// block/line comments were removed to keep the sample compact.
// Source: https://github.com/chiru-labs/ERC721A (MIT), Creator: Chiru Labs.

pragma solidity ^0.8.4;

interface IERC721A {
    
    error ApprovalCallerNotOwnerNorApproved();

    error ApprovalQueryForNonexistentToken();

    error BalanceQueryForZeroAddress();

    error MintToZeroAddress();

    error MintZeroQuantity();

    error OwnerQueryForNonexistentToken();

    error TransferCallerNotOwnerNorApproved();

    error TransferFromIncorrectOwner();

    error TransferToNonERC721ReceiverImplementer();

    error TransferToZeroAddress();

    error URIQueryForNonexistentToken();

    error MintERC2309QuantityExceedsLimit();

    error OwnershipNotInitializedForExtraData();

    error TokenIdsNotStrictlyAscending();

    error SequentialUpToTooSmall();

    error SequentialMintExceedsLimit();

    error SpotMintTokenIdTooSmall();

    error TokenAlreadyExists();

    error NotCompatibleWithSpotMints();

    struct TokenOwnership {
        address addr;
        uint64 startTimestamp;
        bool burned;
        uint24 extraData;
    }

    
    function totalSupply() external view returns (uint256);

    
    function supportsInterface(bytes4 interfaceId) external view returns (bool);

    
    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);

    event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);

    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    function balanceOf(address owner) external view returns (uint256 balance);

    function ownerOf(uint256 tokenId) external view returns (address owner);

    function safeTransferFrom(
        address from,
        address to,
        uint256 tokenId,
        bytes calldata data
    ) external payable;

    function safeTransferFrom(
        address from,
        address to,
        uint256 tokenId
    ) external payable;

    function transferFrom(
        address from,
        address to,
        uint256 tokenId
    ) external payable;

    function approve(address to, uint256 tokenId) external payable;

    function setApprovalForAll(address operator, bool _approved) external;

    function getApproved(uint256 tokenId) external view returns (address operator);

    function isApprovedForAll(address owner, address operator) external view returns (bool);

    
    function name() external view returns (string memory);

    function symbol() external view returns (string memory);

    function tokenURI(uint256 tokenId) external view returns (string memory);

    
    event ConsecutiveTransfer(uint256 indexed fromTokenId, uint256 toTokenId, address indexed from, address indexed to);
}

interface ERC721A__IERC721Receiver {
    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external returns (bytes4);
}

contract ERC721A is IERC721A {
    struct TokenApprovalRef {
        address value;
    }

    uint256 private constant _BITMASK_ADDRESS_DATA_ENTRY = (1 << 64) - 1;

    uint256 private constant _BITPOS_NUMBER_MINTED = 64;

    uint256 private constant _BITPOS_NUMBER_BURNED = 128;

    uint256 private constant _BITPOS_AUX = 192;

    uint256 private constant _BITMASK_AUX_COMPLEMENT = (1 << 192) - 1;

    uint256 private constant _BITPOS_START_TIMESTAMP = 160;

    uint256 private constant _BITMASK_BURNED = 1 << 224;

    uint256 private constant _BITPOS_NEXT_INITIALIZED = 225;

    uint256 private constant _BITMASK_NEXT_INITIALIZED = 1 << 225;

    uint256 private constant _BITPOS_EXTRA_DATA = 232;

    uint256 private constant _BITMASK_EXTRA_DATA_COMPLEMENT = (1 << 232) - 1;

    uint256 private constant _BITMASK_ADDRESS = (1 << 160) - 1;

    uint256 private constant _MAX_MINT_ERC2309_QUANTITY_LIMIT = 5000;

    bytes32 private constant _TRANSFER_EVENT_SIGNATURE =
        0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef;

    uint256 private _currentIndex;

    uint256 private _burnCounter;

    string private _name;

    string private _symbol;

    mapping(uint256 => uint256) private _packedOwnerships;

    mapping(address => uint256) private _packedAddressData;

    mapping(uint256 => TokenApprovalRef) private _tokenApprovals;

    mapping(address => mapping(address => bool)) private _operatorApprovals;

    uint256 private _spotMinted;

    constructor(string memory name_, string memory symbol_) {
        _name = name_;
        _symbol = symbol_;
        _currentIndex = _startTokenId();

        if (_sequentialUpTo() < _startTokenId()) _revert(SequentialUpToTooSmall.selector);
    }

    
    function _startTokenId() internal view virtual returns (uint256) {
        return 0;
    }

    function _sequentialUpTo() internal view virtual returns (uint256) {
        return type(uint256).max;
    }

    function _nextTokenId() internal view virtual returns (uint256) {
        return _currentIndex;
    }

    function totalSupply() public view virtual override returns (uint256 result) {
        unchecked {
            result = _currentIndex - _burnCounter - _startTokenId();
            if (_sequentialUpTo() != type(uint256).max) result += _spotMinted;
        }
    }

    function _totalMinted() internal view virtual returns (uint256 result) {
        unchecked {
            result = _currentIndex - _startTokenId();
            if (_sequentialUpTo() != type(uint256).max) result += _spotMinted;
        }
    }

    function _totalBurned() internal view virtual returns (uint256) {
        return _burnCounter;
    }

    function _totalSpotMinted() internal view virtual returns (uint256) {
        return _spotMinted;
    }

    
    function balanceOf(address owner) public view virtual override returns (uint256) {
        if (owner == address(0)) _revert(BalanceQueryForZeroAddress.selector);
        return _packedAddressData[owner] & _BITMASK_ADDRESS_DATA_ENTRY;
    }

    function _numberMinted(address owner) internal view returns (uint256) {
        return (_packedAddressData[owner] >> _BITPOS_NUMBER_MINTED) & _BITMASK_ADDRESS_DATA_ENTRY;
    }

    function _numberBurned(address owner) internal view returns (uint256) {
        return (_packedAddressData[owner] >> _BITPOS_NUMBER_BURNED) & _BITMASK_ADDRESS_DATA_ENTRY;
    }

    function _getAux(address owner) internal view returns (uint64) {
        return uint64(_packedAddressData[owner] >> _BITPOS_AUX);
    }

    function _setAux(address owner, uint64 aux) internal virtual {
        uint256 packed = _packedAddressData[owner];
        uint256 auxCasted;
        assembly {
            auxCasted := aux
        }
        packed = (packed & _BITMASK_AUX_COMPLEMENT) | (auxCasted << _BITPOS_AUX);
        _packedAddressData[owner] = packed;
    }

    
    function supportsInterface(bytes4 interfaceId) public view virtual override returns (bool) {
        return
            interfaceId == 0x01ffc9a7 || // ERC165 interface ID for ERC165.
            interfaceId == 0x80ac58cd || // ERC165 interface ID for ERC721.
            interfaceId == 0x5b5e139f; // ERC165 interface ID for ERC721Metadata.
    }

    
    function name() public view virtual override returns (string memory) {
        return _name;
    }

    function symbol() public view virtual override returns (string memory) {
        return _symbol;
    }

    function tokenURI(uint256 tokenId) public view virtual override returns (string memory) {
        if (!_exists(tokenId)) _revert(URIQueryForNonexistentToken.selector);

        string memory baseURI = _baseURI();
        return bytes(baseURI).length != 0 ? string(abi.encodePacked(baseURI, _toString(tokenId))) : '';
    }

    function _baseURI() internal view virtual returns (string memory) {
        return '';
    }

    
    function ownerOf(uint256 tokenId) public view virtual override returns (address) {
        return address(uint160(_packedOwnershipOf(tokenId)));
    }

    function _ownershipOf(uint256 tokenId) internal view virtual returns (TokenOwnership memory) {
        return _unpackedOwnership(_packedOwnershipOf(tokenId));
    }

    function _ownershipAt(uint256 index) internal view virtual returns (TokenOwnership memory) {
        return _unpackedOwnership(_packedOwnerships[index]);
    }

    function _ownershipIsInitialized(uint256 index) internal view virtual returns (bool) {
        return _packedOwnerships[index] != 0;
    }

    function _initializeOwnershipAt(uint256 index) internal virtual {
        if (_packedOwnerships[index] == uint256(0)) {
            _packedOwnerships[index] = _packedOwnershipOf(index);
        }
    }

    function _packedOwnershipOf(uint256 tokenId) private view returns (uint256 packed) {
        if (_startTokenId() <= tokenId) {
            packed = _packedOwnerships[tokenId];

            if (tokenId > _sequentialUpTo()) {
                if (_packedOwnershipExists(packed)) return packed;
                _revert(OwnerQueryForNonexistentToken.selector);
            }

            if (packed == uint256(0)) {
                if (tokenId >= _currentIndex) _revert(OwnerQueryForNonexistentToken.selector);
                for (;;) {
                    unchecked {
                        packed = _packedOwnerships[--tokenId];
                    }
                    if (packed == uint256(0)) continue;
                    if (packed & _BITMASK_BURNED == uint256(0)) return packed;
                    _revert(OwnerQueryForNonexistentToken.selector);
                }
            }
            if (packed & _BITMASK_BURNED == uint256(0)) return packed;
        }
        _revert(OwnerQueryForNonexistentToken.selector);
    }

    function _unpackedOwnership(uint256 packed) private pure returns (TokenOwnership memory ownership) {
        ownership.addr = address(uint160(packed));
        ownership.startTimestamp = uint64(packed >> _BITPOS_START_TIMESTAMP);
        ownership.burned = packed & _BITMASK_BURNED != 0;
        ownership.extraData = uint24(packed >> _BITPOS_EXTRA_DATA);
    }

    function _packOwnershipData(address owner, uint256 flags) private view returns (uint256 result) {
        assembly {
            owner := and(owner, _BITMASK_ADDRESS)
            result := or(owner, or(shl(_BITPOS_START_TIMESTAMP, timestamp()), flags))
        }
    }

    function _nextInitializedFlag(uint256 quantity) private pure returns (uint256 result) {
        assembly {
            result := shl(_BITPOS_NEXT_INITIALIZED, eq(quantity, 1))
        }
    }

    
    function approve(address to, uint256 tokenId) public payable virtual override {
        _approve(to, tokenId, true);
    }

    function getApproved(uint256 tokenId) public view virtual override returns (address) {
        if (!_exists(tokenId)) _revert(ApprovalQueryForNonexistentToken.selector);

        return _tokenApprovals[tokenId].value;
    }

    function setApprovalForAll(address operator, bool approved) public virtual override {
        _operatorApprovals[_msgSenderERC721A()][operator] = approved;
        emit ApprovalForAll(_msgSenderERC721A(), operator, approved);
    }

    function isApprovedForAll(address owner, address operator) public view virtual override returns (bool) {
        return _operatorApprovals[owner][operator];
    }

    function _exists(uint256 tokenId) internal view virtual returns (bool result) {
        if (_startTokenId() <= tokenId) {
            if (tokenId > _sequentialUpTo()) return _packedOwnershipExists(_packedOwnerships[tokenId]);

            if (tokenId < _currentIndex) {
                uint256 packed;
                while ((packed = _packedOwnerships[tokenId]) == uint256(0)) --tokenId;
                result = packed & _BITMASK_BURNED == uint256(0);
            }
        }
    }

    function _packedOwnershipExists(uint256 packed) private pure returns (bool result) {
        assembly {
            result := gt(and(packed, _BITMASK_ADDRESS), and(packed, _BITMASK_BURNED))
        }
    }

    function _isSenderApprovedOrOwner(
        uint256 approvedAddressValue,
        uint256 ownerMasked,
        uint256 msgSenderMasked
    ) private pure returns (bool result) {
        assembly {
            result := or(eq(msgSenderMasked, ownerMasked), eq(msgSenderMasked, approvedAddressValue))
        }
    }

    function _getApprovedSlotAndValue(uint256 tokenId)
        private
        view
        returns (uint256 approvedAddressSlot, uint256 approvedAddressValue)
    {
        TokenApprovalRef storage tokenApproval = _tokenApprovals[tokenId];
        assembly {
            approvedAddressSlot := tokenApproval.slot
            approvedAddressValue := sload(approvedAddressSlot)
        }
    }

    
    function transferFrom(
        address from,
        address to,
        uint256 tokenId
    ) public payable virtual override {
        uint256 prevOwnershipPacked = _packedOwnershipOf(tokenId);
        uint256 fromMasked = uint160(from);

        if (uint160(prevOwnershipPacked) != fromMasked) _revert(TransferFromIncorrectOwner.selector);

        (uint256 approvedAddressSlot, uint256 approvedAddressValue) = _getApprovedSlotAndValue(tokenId);

        if (!_isSenderApprovedOrOwner(approvedAddressValue, fromMasked, uint160(_msgSenderERC721A())))
            if (!isApprovedForAll(from, _msgSenderERC721A())) _revert(TransferCallerNotOwnerNorApproved.selector);

        _beforeTokenTransfers(from, to, tokenId, 1);

        assembly {
            if approvedAddressValue {
                sstore(approvedAddressSlot, 0) // Equivalent to `delete _tokenApprovals[tokenId]`.
            }
        }

        unchecked {
            --_packedAddressData[from]; // Updates: `balance -= 1`.
            ++_packedAddressData[to]; // Updates: `balance += 1`.

            _packedOwnerships[tokenId] = _packOwnershipData(
                to,
                _BITMASK_NEXT_INITIALIZED | _nextExtraData(from, to, prevOwnershipPacked)
            );

            if (prevOwnershipPacked & _BITMASK_NEXT_INITIALIZED == uint256(0)) {
                uint256 nextTokenId = tokenId + 1;
                if (_packedOwnerships[nextTokenId] == uint256(0)) {
                    if (nextTokenId != _currentIndex) {
                        _packedOwnerships[nextTokenId] = prevOwnershipPacked;
                    }
                }
            }
        }

        uint256 toMasked = uint160(to);
        assembly {
            log4(
                0, // Start of data (0, since no data).
                0, // End of data (0, since no data).
                _TRANSFER_EVENT_SIGNATURE, // Signature.
                fromMasked, // `from`.
                toMasked, // `to`.
                tokenId // `tokenId`.
            )
        }
        if (toMasked == uint256(0)) _revert(TransferToZeroAddress.selector);

        _afterTokenTransfers(from, to, tokenId, 1);
    }

    function safeTransferFrom(
        address from,
        address to,
        uint256 tokenId
    ) public payable virtual override {
        safeTransferFrom(from, to, tokenId, '');
    }

    function safeTransferFrom(
        address from,
        address to,
        uint256 tokenId,
        bytes memory _data
    ) public payable virtual override {
        transferFrom(from, to, tokenId);
        if (to.code.length != 0)
            if (!_checkContractOnERC721Received(from, to, tokenId, _data)) {
                _revert(TransferToNonERC721ReceiverImplementer.selector);
            }
    }

    function _batchTransferFrom(
        address from,
        address to,
        uint256[] memory tokenIds
    ) internal virtual {
        _batchTransferFrom(address(0), from, to, tokenIds);
    }

    function _batchTransferFrom(
        address by,
        address from,
        address to,
        uint256[] memory tokenIds
    ) internal virtual {
        uint256 byMasked = uint160(by);
        uint256 fromMasked = uint160(from);
        uint256 toMasked = uint160(to);
        if (toMasked == uint256(0)) _revert(TransferToZeroAddress.selector);
        bool mayTransfer = _orERC721A(byMasked == uint256(0), byMasked == fromMasked) || isApprovedForAll(from, by);

        if (tokenIds.length == uint256(0)) return;
        uint256 end = _currentIndex;
        (uint256 ptr, uint256 ptrEnd) = _mdataERC721A(tokenIds);

        uint256 prevTokenId;
        uint256 prevOwnershipPacked;
        unchecked {
            do {
                uint256 tokenId = _mloadERC721A(ptr);
                uint256 miniBatchStart = tokenId;
                if (_orERC721A(tokenId < _startTokenId(), end <= tokenId))
                    _revert(OwnerQueryForNonexistentToken.selector);
                if (prevOwnershipPacked != 0)
                    if (tokenId <= prevTokenId) _revert(TokenIdsNotStrictlyAscending.selector);
                for (uint256 j = tokenId; (prevOwnershipPacked = _packedOwnerships[j]) == uint256(0); ) --j;
                if (prevOwnershipPacked & _BITMASK_BURNED != 0) _revert(OwnerQueryForNonexistentToken.selector);
                if (uint160(prevOwnershipPacked) != fromMasked) _revert(TransferFromIncorrectOwner.selector);

                do {
                    (uint256 approvedAddressSlot, uint256 approvedAddressValue) = _getApprovedSlotAndValue(tokenId);
                    _beforeTokenTransfers(address(uint160(fromMasked)), address(uint160(toMasked)), tokenId, 1);
                    if (!mayTransfer)
                        if (byMasked != approvedAddressValue) _revert(TransferCallerNotOwnerNorApproved.selector);
                    assembly {
                        if approvedAddressValue {
                            sstore(approvedAddressSlot, 0) // Equivalent to `delete _tokenApprovals[tokenId]`.
                        }
                        log4(0, 0, _TRANSFER_EVENT_SIGNATURE, fromMasked, toMasked, tokenId)
                    }

                    if (_mloadERC721A(ptr += 0x20) != ++tokenId) break;
                    if (ptr == ptrEnd) break;
                } while (_packedOwnerships[tokenId] == uint256(0));

                _packedOwnerships[miniBatchStart] = _packOwnershipData(
                    address(uint160(toMasked)),
                    _nextExtraData(address(uint160(fromMasked)), address(uint160(toMasked)), prevOwnershipPacked)
                );
                uint256 miniBatchLength = tokenId - miniBatchStart;
                _packedAddressData[address(uint160(fromMasked))] -= miniBatchLength;
                _packedAddressData[address(uint160(toMasked))] += miniBatchLength;
                if (tokenId != end)
                    if (_packedOwnerships[tokenId] == uint256(0)) _packedOwnerships[tokenId] = prevOwnershipPacked;
                _afterTokenTransfers(
                    address(uint160(fromMasked)),
                    address(uint160(toMasked)),
                    miniBatchStart,
                    miniBatchLength
                );
                prevTokenId = tokenId - 1;
            } while (ptr != ptrEnd);
        }
    }

    function _safeBatchTransferFrom(
        address by,
        address from,
        address to,
        uint256[] memory tokenIds,
        bytes memory _data
    ) internal virtual {
        _batchTransferFrom(by, from, to, tokenIds);

        unchecked {
            if (to.code.length != 0) {
                for ((uint256 ptr, uint256 ptrEnd) = _mdataERC721A(tokenIds); ptr != ptrEnd; ptr += 0x20) {
                    if (!_checkContractOnERC721Received(from, to, _mloadERC721A(ptr), _data)) {
                        _revert(TransferToNonERC721ReceiverImplementer.selector);
                    }
                }
            }
        }
    }

    function _beforeTokenTransfers(
        address from,
        address to,
        uint256 startTokenId,
        uint256 quantity
    ) internal virtual {}

    function _afterTokenTransfers(
        address from,
        address to,
        uint256 startTokenId,
        uint256 quantity
    ) internal virtual {}

    function _checkContractOnERC721Received(
        address from,
        address to,
        uint256 tokenId,
        bytes memory _data
    ) private returns (bool) {
        try ERC721A__IERC721Receiver(to).onERC721Received(_msgSenderERC721A(), from, tokenId, _data) returns (
            bytes4 retval
        ) {
            return retval == ERC721A__IERC721Receiver(to).onERC721Received.selector;
        } catch (bytes memory reason) {
            if (reason.length == uint256(0)) {
                _revert(TransferToNonERC721ReceiverImplementer.selector);
            }
            assembly {
                revert(add(32, reason), mload(reason))
            }
        }
    }

    
    function _mint(address to, uint256 quantity) internal virtual {
        uint256 startTokenId = _currentIndex;
        if (quantity == uint256(0)) _revert(MintZeroQuantity.selector);

        _beforeTokenTransfers(address(0), to, startTokenId, quantity);

        unchecked {
            _packedOwnerships[startTokenId] = _packOwnershipData(
                to,
                _nextInitializedFlag(quantity) | _nextExtraData(address(0), to, 0)
            );

            _packedAddressData[to] += quantity * ((1 << _BITPOS_NUMBER_MINTED) | 1);

            uint256 toMasked = uint160(to);

            if (toMasked == uint256(0)) _revert(MintToZeroAddress.selector);

            uint256 end = startTokenId + quantity;
            uint256 tokenId = startTokenId;

            if (end - 1 > _sequentialUpTo()) _revert(SequentialMintExceedsLimit.selector);

            do {
                assembly {
                    log4(
                        0, // Start of data (0, since no data).
                        0, // End of data (0, since no data).
                        _TRANSFER_EVENT_SIGNATURE, // Signature.
                        0, // `address(0)`.
                        toMasked, // `to`.
                        tokenId // `tokenId`.
                    )
                }
            } while (++tokenId != end);

            _currentIndex = end;
        }
        _afterTokenTransfers(address(0), to, startTokenId, quantity);
    }

    function _mintERC2309(address to, uint256 quantity) internal virtual {
        uint256 startTokenId = _currentIndex;
        if (to == address(0)) _revert(MintToZeroAddress.selector);
        if (quantity == uint256(0)) _revert(MintZeroQuantity.selector);
        if (quantity > _MAX_MINT_ERC2309_QUANTITY_LIMIT) _revert(MintERC2309QuantityExceedsLimit.selector);

        _beforeTokenTransfers(address(0), to, startTokenId, quantity);

        unchecked {
            _packedAddressData[to] += quantity * ((1 << _BITPOS_NUMBER_MINTED) | 1);

            _packedOwnerships[startTokenId] = _packOwnershipData(
                to,
                _nextInitializedFlag(quantity) | _nextExtraData(address(0), to, 0)
            );

            if (startTokenId + quantity - 1 > _sequentialUpTo()) _revert(SequentialMintExceedsLimit.selector);

            emit ConsecutiveTransfer(startTokenId, startTokenId + quantity - 1, address(0), to);

            _currentIndex = startTokenId + quantity;
        }
        _afterTokenTransfers(address(0), to, startTokenId, quantity);
    }

    function _safeMint(
        address to,
        uint256 quantity,
        bytes memory _data
    ) internal virtual {
        _mint(to, quantity);

        unchecked {
            if (to.code.length != 0) {
                uint256 end = _currentIndex;
                uint256 index = end - quantity;
                do {
                    if (!_checkContractOnERC721Received(address(0), to, index++, _data)) {
                        _revert(TransferToNonERC721ReceiverImplementer.selector);
                    }
                } while (index < end);
                if (_currentIndex != end) revert();
            }
        }
    }

    function _safeMint(address to, uint256 quantity) internal virtual {
        _safeMint(to, quantity, '');
    }

    function _mintSpot(address to, uint256 tokenId) internal virtual {
        if (tokenId <= _sequentialUpTo()) _revert(SpotMintTokenIdTooSmall.selector);
        uint256 prevOwnershipPacked = _packedOwnerships[tokenId];
        if (_packedOwnershipExists(prevOwnershipPacked)) _revert(TokenAlreadyExists.selector);

        _beforeTokenTransfers(address(0), to, tokenId, 1);

        unchecked {
            _packedOwnerships[tokenId] = _packOwnershipData(
                to,
                _nextInitializedFlag(1) | _nextExtraData(address(0), to, prevOwnershipPacked)
            );

            _packedAddressData[to] += (1 << _BITPOS_NUMBER_MINTED) | 1;

            uint256 toMasked = uint160(to);

            if (toMasked == uint256(0)) _revert(MintToZeroAddress.selector);

            assembly {
                log4(
                    0, // Start of data (0, since no data).
                    0, // End of data (0, since no data).
                    _TRANSFER_EVENT_SIGNATURE, // Signature.
                    0, // `address(0)`.
                    toMasked, // `to`.
                    tokenId // `tokenId`.
                )
            }

            ++_spotMinted;
        }

        _afterTokenTransfers(address(0), to, tokenId, 1);
    }

    function _safeMintSpot(
        address to,
        uint256 tokenId,
        bytes memory _data
    ) internal virtual {
        _mintSpot(to, tokenId);

        unchecked {
            if (to.code.length != 0) {
                uint256 currentSpotMinted = _spotMinted;
                if (!_checkContractOnERC721Received(address(0), to, tokenId, _data)) {
                    _revert(TransferToNonERC721ReceiverImplementer.selector);
                }
                if (_spotMinted != currentSpotMinted) revert();
            }
        }
    }

    function _safeMintSpot(address to, uint256 tokenId) internal virtual {
        _safeMintSpot(to, tokenId, '');
    }

    
    function _approve(address to, uint256 tokenId) internal virtual {
        _approve(to, tokenId, false);
    }

    function _approve(
        address to,
        uint256 tokenId,
        bool approvalCheck
    ) internal virtual {
        address owner = ownerOf(tokenId);

        if (approvalCheck && _msgSenderERC721A() != owner)
            if (!isApprovedForAll(owner, _msgSenderERC721A())) {
                _revert(ApprovalCallerNotOwnerNorApproved.selector);
            }

        _tokenApprovals[tokenId].value = to;
        emit Approval(owner, to, tokenId);
    }

    
    function _burn(uint256 tokenId) internal virtual {
        _burn(tokenId, false);
    }

    function _burn(uint256 tokenId, bool approvalCheck) internal virtual {
        uint256 prevOwnershipPacked = _packedOwnershipOf(tokenId);

        uint256 fromMasked = uint160(prevOwnershipPacked);
        address from = address(uint160(fromMasked));

        (uint256 approvedAddressSlot, uint256 approvedAddressValue) = _getApprovedSlotAndValue(tokenId);

        if (approvalCheck) {
            if (!_isSenderApprovedOrOwner(approvedAddressValue, fromMasked, uint160(_msgSenderERC721A())))
                if (!isApprovedForAll(from, _msgSenderERC721A())) _revert(TransferCallerNotOwnerNorApproved.selector);
        }

        _beforeTokenTransfers(from, address(0), tokenId, 1);

        assembly {
            if approvedAddressValue {
                sstore(approvedAddressSlot, 0) // Equivalent to `delete _tokenApprovals[tokenId]`.
            }
        }

        unchecked {
            _packedAddressData[from] += (1 << _BITPOS_NUMBER_BURNED) - 1;

            _packedOwnerships[tokenId] = _packOwnershipData(
                from,
                (_BITMASK_BURNED | _BITMASK_NEXT_INITIALIZED) | _nextExtraData(from, address(0), prevOwnershipPacked)
            );

            if (prevOwnershipPacked & _BITMASK_NEXT_INITIALIZED == uint256(0)) {
                uint256 nextTokenId = tokenId + 1;
                if (_packedOwnerships[nextTokenId] == uint256(0)) {
                    if (nextTokenId != _currentIndex) {
                        _packedOwnerships[nextTokenId] = prevOwnershipPacked;
                    }
                }
            }
        }

        emit Transfer(from, address(0), tokenId);
        _afterTokenTransfers(from, address(0), tokenId, 1);

        unchecked {
            _burnCounter++;
        }
    }

    function _batchBurn(address by, uint256[] memory tokenIds) internal virtual {
        if (tokenIds.length == uint256(0)) return;
        uint256 end = _currentIndex;
        (uint256 ptr, uint256 ptrEnd) = _mdataERC721A(tokenIds);

        uint256 prevOwnershipPacked;
        address prevTokenOwner;
        uint256 prevTokenId;
        bool mayBurn;
        unchecked {
            do {
                uint256 tokenId = _mloadERC721A(ptr);
                uint256 miniBatchStart = tokenId;
                if (_orERC721A(tokenId < _startTokenId(), end <= tokenId))
                    _revert(OwnerQueryForNonexistentToken.selector);
                if (prevOwnershipPacked != 0)
                    if (tokenId <= prevTokenId) _revert(TokenIdsNotStrictlyAscending.selector);
                for (uint256 j = tokenId; (prevOwnershipPacked = _packedOwnerships[j]) == uint256(0); ) --j;
                if (prevOwnershipPacked & _BITMASK_BURNED != 0) _revert(OwnerQueryForNonexistentToken.selector);

                address tokenOwner = address(uint160(prevOwnershipPacked));
                if (tokenOwner != prevTokenOwner) {
                    prevTokenOwner = tokenOwner;
                    mayBurn = _orERC721A(by == address(0), tokenOwner == by) || isApprovedForAll(tokenOwner, by);
                }

                do {
                    (uint256 approvedAddressSlot, uint256 approvedAddressValue) = _getApprovedSlotAndValue(tokenId);
                    _beforeTokenTransfers(tokenOwner, address(0), tokenId, 1);
                    if (!mayBurn)
                        if (uint160(by) != approvedAddressValue) _revert(TransferCallerNotOwnerNorApproved.selector);
                    assembly {
                        if approvedAddressValue {
                            sstore(approvedAddressSlot, 0) // Equivalent to `delete _tokenApprovals[tokenId]`.
                        }
                        log4(0, 0, _TRANSFER_EVENT_SIGNATURE, and(_BITMASK_ADDRESS, tokenOwner), 0, tokenId)
                    }
                    if (_mloadERC721A(ptr += 0x20) != ++tokenId) break;
                    if (ptr == ptrEnd) break;
                } while (_packedOwnerships[tokenId] == uint256(0));

                _packedOwnerships[miniBatchStart] = _packOwnershipData(
                    tokenOwner,
                    _BITMASK_BURNED | _nextExtraData(tokenOwner, address(0), prevOwnershipPacked)
                );
                uint256 miniBatchLength = tokenId - miniBatchStart;
                _packedAddressData[tokenOwner] += (miniBatchLength << _BITPOS_NUMBER_BURNED) - miniBatchLength;
                if (tokenId != end)
                    if (_packedOwnerships[tokenId] == uint256(0)) _packedOwnerships[tokenId] = prevOwnershipPacked;
                _afterTokenTransfers(tokenOwner, address(0), miniBatchStart, miniBatchLength);
                prevTokenId = tokenId - 1;
            } while (ptr != ptrEnd);
            _burnCounter += tokenIds.length;
        }
    }

    
    function _setExtraDataAt(uint256 index, uint24 extraData) internal virtual {
        uint256 packed = _packedOwnerships[index];
        if (packed == uint256(0)) _revert(OwnershipNotInitializedForExtraData.selector);
        uint256 extraDataCasted;
        assembly {
            extraDataCasted := extraData
        }
        packed = (packed & _BITMASK_EXTRA_DATA_COMPLEMENT) | (extraDataCasted << _BITPOS_EXTRA_DATA);
        _packedOwnerships[index] = packed;
    }

    function _extraData(
        address from,
        address to,
        uint24 previousExtraData
    ) internal view virtual returns (uint24) {}

    function _nextExtraData(
        address from,
        address to,
        uint256 prevOwnershipPacked
    ) private view returns (uint256) {
        uint24 extraData = uint24(prevOwnershipPacked >> _BITPOS_EXTRA_DATA);
        return uint256(_extraData(from, to, extraData)) << _BITPOS_EXTRA_DATA;
    }

    
    function _mdataERC721A(uint256[] memory a) private pure returns (uint256 start, uint256 end) {
        assembly {
            start := add(a, 0x20)
            end := add(start, shl(5, mload(a)))
        }
    }

    function _mloadERC721A(uint256 p) private pure returns (uint256 result) {
        assembly {
            result := mload(p)
        }
    }

    function _orERC721A(bool a, bool b) private pure returns (bool result) {
        assembly {
            result := or(iszero(iszero(a)), iszero(iszero(b)))
        }
    }

    
    function _msgSenderERC721A() internal view virtual returns (address) {
        return msg.sender;
    }

    function _toString(uint256 value) internal pure virtual returns (string memory str) {
        assembly {
            let m := add(mload(0x40), 0xa0)
            mstore(0x40, m)
            str := sub(m, 0x20)
            mstore(str, 0)

            let end := str

            for { let temp := value } 1 {} {
                str := sub(str, 1)
                mstore8(str, add(48, mod(temp, 10)))
                temp := div(temp, 10)
                if iszero(temp) { break }
            }

            let length := sub(end, str)
            str := sub(str, 0x20)
            mstore(str, length)
        }
    }

    function _revert(bytes4 errorSelector) internal pure {
        assembly {
            mstore(0x00, errorSelector)
            revert(0x00, 0x04)
        }
    }
}

contract ERC721ACollection is ERC721A {
    uint256 public constant MAX_SUPPLY = 10000;
    uint256 public constant MAX_PER_WALLET = 5;
    address public owner;

    constructor() ERC721A("Azuki Sample", "AZUKI") {
        owner = msg.sender;
    }

    function mint(uint256 quantity) external {
        require(_totalMinted() + quantity <= MAX_SUPPLY, "Sold out");
        require(_numberMinted(msg.sender) + quantity <= MAX_PER_WALLET, "Over limit");
        _mint(msg.sender, quantity);
    }

    function _startTokenId() internal view virtual override returns (uint256) {
        return 1;
    }
}
