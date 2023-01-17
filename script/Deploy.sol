// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "forge-std/Script.sol";
import "../src/PaymentSplitter.sol";

contract PaymentSplitterScript is Script {
    address[] addresses;
    uint256[] shares;
    uint256 privateKey;
    uint256 totalShares;

    function setUp() public {
        string memory seedPhrase = vm.readFile(".secret");
        privateKey = vm.deriveKey(seedPhrase, 0);

        string memory payee = vm.readLine(".payees");
        while (keccak256(bytes(payee)) != keccak256("")) {
            address payeeAddr = vm.parseAddress(payee);
            addresses.push(payeeAddr);
            payee = vm.readLine(".payees");
        }
        console.log("read in payees");

        totalShares = 0;
        string memory share = vm.readLine(".shares");
        while (keccak256(bytes(share)) != keccak256("")) {
            uint256 shareInt = vm.parseUint(share);
            totalShares += shareInt;
            shares.push(shareInt);
            share = vm.readLine(".shares");
        }

        assert(addresses.length == shares.length);
        console.log("read in shares");

        // works for testing on a local EVM
        vm.deal(vm.addr(privateKey), totalShares);
    }

    function run() public {
        vm.startBroadcast(privateKey);
        PaymentSplitter splitter = new PaymentSplitter(addresses, shares);
        bool sent = payable(address(splitter)).send(totalShares);
        assert(sent);
        vm.stopBroadcast();
    }
}