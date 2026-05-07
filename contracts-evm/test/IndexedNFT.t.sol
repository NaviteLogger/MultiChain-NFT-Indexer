// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {IndexedNFT} from "../src/IndexedNFT.sol";
import {IERC721} from "@openzeppelin/contracts/token/ERC721/IERC721.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract IndexedNFTTest is Test {
    IndexedNFT internal nft;

    address internal owner = address(0xA11CE);
    address internal alice = address(0xCAFE);
    address internal bob = address(0xBEEF);

    string internal constant URI_ONE = "ipfs://QmExample/1.json";
    string internal constant URI_TWO = "ipfs://QmExample/2.json";

    event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
    event Minted(address indexed to, uint256 indexed tokenId, string uri);

    function setUp() public {
        nft = new IndexedNFT("Indexed NFT", "IDX", owner, 3);
    }

    function test_Mint_EmitsTransferAndMintedEvents() public {
        vm.expectEmit(true, true, true, false, address(nft));
        emit Transfer(address(0), alice, 0);
        vm.expectEmit(true, true, false, true, address(nft));
        emit Minted(alice, 0, URI_ONE);

        vm.prank(alice);
        uint256 tokenId = nft.mint(URI_ONE);

        assertEq(tokenId, 0);
        assertEq(nft.ownerOf(0), alice);
        assertEq(nft.tokenURI(0), URI_ONE);
        assertEq(nft.balanceOf(alice), 1);
        assertEq(nft.mintsPerWallet(alice), 1);
    }

    function test_Mint_RevertsOnEmptyUri() public {
        vm.prank(alice);
        vm.expectRevert(IndexedNFT.EmptyUri.selector);
        nft.mint("");
    }

    function test_Mint_RevertsAtCap() public {
        vm.startPrank(alice);
        nft.mint(URI_ONE);
        nft.mint(URI_TWO);
        nft.mint(URI_ONE);
        vm.expectRevert(IndexedNFT.MintCapReached.selector);
        nft.mint(URI_ONE);
        vm.stopPrank();
    }

    function test_Transfer() public {
        vm.prank(alice);
        nft.mint(URI_ONE);

        vm.expectEmit(true, true, true, false, address(nft));
        emit Transfer(alice, bob, 0);
        vm.prank(alice);
        nft.transferFrom(alice, bob, 0);
        assertEq(nft.ownerOf(0), bob);
    }

    function test_SetMintCap_RevertsIfNotOwner() public {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        nft.setMintCap(99);
    }

    function testFuzz_MintAssignsSequentialIds(uint8 perWallet) public {
        perWallet = uint8(bound(uint256(perWallet), 1, 5));
        vm.prank(owner);
        nft.setMintCap(perWallet);

        vm.startPrank(alice);
        for (uint256 i = 0; i < perWallet; i++) {
            uint256 id = nft.mint(URI_ONE);
            assertEq(id, i);
        }
        vm.stopPrank();
        assertEq(nft.balanceOf(alice), perWallet);
    }
}
