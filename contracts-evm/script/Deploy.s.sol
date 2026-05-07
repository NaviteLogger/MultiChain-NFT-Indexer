// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console2} from "forge-std/Script.sol";
import {IndexedNFT} from "../src/IndexedNFT.sol";

/// @notice Deploy the IndexedNFT contract that the Rust indexer watches.
contract Deploy is Script {
    function run() external returns (IndexedNFT nft) {
        uint256 pk = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address owner = vm.addr(pk);
        uint256 cap = vm.envOr("MINT_CAP_PER_WALLET", uint256(10));

        vm.startBroadcast(pk);
        nft = new IndexedNFT("Indexed NFT", "IDX", owner, cap);
        vm.stopBroadcast();

        console2.log("IndexedNFT deployed:", address(nft));
        console2.log("Owner:", owner);
        console2.log("Mint cap per wallet:", cap);
        console2.log("Chain id:", block.chainid);
    }
}
