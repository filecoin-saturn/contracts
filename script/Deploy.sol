// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "forge-std/Script.sol";
import "../src/PaymentSplitter.sol";

contract PaymentSplitterScript is Script {
    function setUp() public {}

    function run() public {
        string memory seedPhrase = vm.readFile(".secret");
        uint256 privateKey = vm.deriveKey(seedPhrase, 0);

        address[] memory addresses = new address[](1);
        uint256[] memory shares = new uint256[](1);

        string memory payee = vm.readLine(".payees");
        while (keccak256(bytes(payee)) != keccak256("")) {
            address payeeAddr = vm.parseAddress(payee);
            addresses[0] = payeeAddr;
            payee = vm.readLine(".payees");
        }
        console.log("read in payees");

        string memory share = vm.readLine(".shares");
        while (keccak256(bytes(share)) != keccak256("")) {
            uint256 shareInt = vm.parseUint(share);
            shares[0] = shareInt;
            share = vm.readLine(".shares");
        }
        console.log("read in shares");

        assert(addresses.length == shares.length);

        vm.startBroadcast(privateKey);
        PaymentSplitter spacebear = new PaymentSplitter(addresses, shares);
        vm.deal(address(spacebear), 5);

        vm.stopBroadcast();
    }
}
