// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "forge-std/Script.sol";
import "../src/PayoutFactory.sol";

contract FactoryDeployScript is Script {
    uint256 privateKey;

    function setUp() public {
        string memory seedPhrase = vm.readFile(".secret");
        privateKey = vm.deriveKey(seedPhrase, 0);
        // works for testing on a local EVM
        vm.deal(vm.addr(privateKey), 1000);
    }

    function run() public {
        vm.startBroadcast(privateKey);
        PayoutFactory factory = new PayoutFactory(vm.addr(privateKey));
        bool sent = payable(address(factory)).send(1);
        assert(sent);
        vm.stopBroadcast();
    }
}
