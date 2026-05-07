// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title IndexedNFT
/// @notice ERC-721 used by the indexer demo. Open public mint with a per-wallet cap;
///         standard `Transfer` events drive the indexer; `Minted` adds the URI for
///         richer event payloads.
contract IndexedNFT is ERC721URIStorage, Ownable {
    error MintCapReached();
    error EmptyUri();

    event Minted(address indexed to, uint256 indexed tokenId, string uri);

    uint256 public nextTokenId;
    uint256 public mintCapPerWallet;

    mapping(address => uint256) public mintsPerWallet;

    constructor(string memory name_, string memory symbol_, address owner_, uint256 cap_)
        ERC721(name_, symbol_)
        Ownable(owner_)
    {
        mintCapPerWallet = cap_;
    }

    function setMintCap(uint256 cap_) external onlyOwner {
        mintCapPerWallet = cap_;
    }

    function mint(string calldata uri) external returns (uint256 tokenId) {
        if (bytes(uri).length == 0) revert EmptyUri();
        if (mintsPerWallet[msg.sender] >= mintCapPerWallet) revert MintCapReached();

        tokenId = nextTokenId++;
        mintsPerWallet[msg.sender] += 1;
        _safeMint(msg.sender, tokenId);
        _setTokenURI(tokenId, uri);
        emit Minted(msg.sender, tokenId, uri);
    }
}
