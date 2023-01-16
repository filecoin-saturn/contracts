// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "forge-std/Script.sol";
import "../src/PaymentSplitter.sol";

contract PaymentSplitterScript is Script {
    address[] addresses;
    uint256[] shares;

    function setUp() public {}

    function run() public {
        string memory seedPhrase = vm.readFile(".secret");
        uint256 privateKey = vm.deriveKey(seedPhrase, 0);

        string memory payee = vm.readLine(".payees");
        while (keccak256(bytes(payee)) != keccak256("")) {
            payee = vm.readLine(".payees");
            if (keccak256(bytes(payee)) != keccak256("")) {
                addresses.push(vm.parseAddress(payee));
            }
        }
        string memory share = vm.readLine(".shares");
        while (keccak256(bytes(share)) != keccak256("")) {
            share = vm.readLine(".shares");
            if (keccak256(bytes(share)) != keccak256("")) {
                shares.push(vm.parseUint(share));
            }
        }

        // assert(addresses.length == shares.length);

        vm.startBroadcast(privateKey);
        PaymentSplitter spacebear = new PaymentSplitter(addresses, shares);
        vm.deal(address(spacebear), 5);

        vm.stopBroadcast();
    }
}
