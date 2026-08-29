// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-24 NFT Royalty Standard
 * @dev Neo N3 NEP-24 standard interface + a minimal mixin implementation.
 *
 * NEP-24 specifies a `royaltyInfo` method for NEP-11 NFTs so marketplaces can
 * fetch royalty recipients and amounts for secondary sales.
 *
 * Spec: https://github.com/neo-project/proposals/blob/master/nep-24.mediawiki
 */

/**
 * @dev NEP-24 royalty interface.
 *
 * The NEP-24 spec models `royaltyInfo` as returning an Array whose elements are
 * 2-element `[royaltyRecipient (Hash160), royaltyAmount (Integer)]` tuples.
 *
 * DEVPACK DEVIATION (same shape as NEP-11 `properties`): a Solidity array/struct
 * return lowers to an EVM-canonical **ABI-encoded `bytes`** value (manifest type
 * `ByteArray`), not a native NeoVM Array — Solidity has no construct that yields
 * a NeoVM Array return. A spec-conformant NEP-24 consumer must therefore
 * `abi.decode` the returned bytes as `(address,uint256)[]` rather than iterate a
 * NeoVM Array. The bytecode and the declared manifest return type are mutually
 * consistent (`ByteArray`); only the wire *shape* differs from the spec's Array.
 */
interface INEP24Royalty {
    struct RoyaltyInfo {
        address royaltyRecipient;
        uint256 royaltyAmount;
    }

    function royaltyInfo(bytes memory tokenId, address royaltyToken, uint256 salePrice)
        external
        view
        returns (RoyaltyInfo[] memory royalties);

    event RoyaltiesTransferred(
        address indexed royaltyToken,
        address indexed royaltyRecipient,
        address indexed buyer,
        bytes tokenId,
        uint256 amount
    );
}

/**
 * @dev Minimal NEP-24 royalty implementation.
 *
 * - Supports a single recipient + basis points per token id, with an optional
 *   default royalty.
 * - Returns an empty array when no royalty is configured.
 *
 * Marketplaces should call `royaltyInfo` and transfer royalties themselves,
 * emitting `RoyaltiesTransferred` from the marketplace contract.
 */
abstract contract NEP24Royalty is INEP24Royalty {
    struct RoyaltyRule {
        address recipient;
        uint96 bps; // basis points: 10000 = 100%
        bool isSet;
    }

    mapping(bytes => RoyaltyRule) private _tokenRoyalty;
    RoyaltyRule private _defaultRoyalty;

    error NEP24InvalidRoyaltyRecipient(address recipient);
    error NEP24InvalidRoyaltyBps(uint96 bps);

    function _setDefaultRoyalty(address recipient, uint96 bps) internal {
        _setRoyaltyRule(_defaultRoyalty, recipient, bps);
    }

    function _setTokenRoyalty(bytes memory tokenId, address recipient, uint96 bps) internal {
        RoyaltyRule storage rule = _tokenRoyalty[tokenId];
        _setRoyaltyRule(rule, recipient, bps);
    }

    function _clearTokenRoyalty(bytes memory tokenId) internal {
        delete _tokenRoyalty[tokenId];
    }

    function _getRoyaltyRule(bytes memory tokenId) internal view returns (RoyaltyRule memory) {
        RoyaltyRule memory rule = _tokenRoyalty[tokenId];
        if (rule.isSet) return rule;
        return _defaultRoyalty;
    }

    function royaltyInfo(bytes memory tokenId, address royaltyToken, uint256 salePrice)
        public
        view
        virtual
        override
        returns (RoyaltyInfo[] memory royalties)
    {
        royaltyToken; // Reserved for token-specific royalty rules if desired.

        RoyaltyRule memory rule = _getRoyaltyRule(tokenId);
        if (!rule.isSet || rule.recipient == address(0) || rule.bps == 0) {
            return new RoyaltyInfo[](0);
        }

        uint256 amount = (salePrice * uint256(rule.bps)) / 10000;
        royalties = new RoyaltyInfo[](1);
        royalties[0].royaltyRecipient = rule.recipient;
        royalties[0].royaltyAmount = amount;
    }

    function _setRoyaltyRule(RoyaltyRule storage rule, address recipient, uint96 bps) private {
        if (recipient == address(0)) revert NEP24InvalidRoyaltyRecipient(recipient);
        if (bps > 10000) revert NEP24InvalidRoyaltyBps(bps);

        rule.recipient = recipient;
        rule.bps = bps;
        rule.isSet = true;
    }
}
