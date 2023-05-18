// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/PayoutFactory.sol";
import "../src/PaymentSplitter.sol";

contract PayoutFactoryTest is Test {
    PayoutFactory public factory;
    PaymentSplitter public splitter;

    address[] testAddr = [makeAddr("Test")];
    mapping(address => bool) Addr;

    function setUp() public {}

    function hasNoDuplicates(address[] memory arr) private returns (bool) {
        uint256 i;
        mapping(address => bool) storage localAddr = Addr;

        for (; i < arr.length; ) {
            if (localAddr[arr[i]] == true) {
                return false;
            }
            localAddr[arr[i]] = true;

            unchecked {
                i++;
            }
        }

        return true;
    }

    // NOTE: we don't fuzz  payout amounts as this causes too many rejections for forge to extract statistically significant numbers
    function testPayout_withFuzzing(address[] calldata addresses) public {
        vm.assume(
            addresses.length > 0 &&
                // has no duplicates
                hasNoDuplicates(addresses)
        );

        factory = new PayoutFactory(address(this));

        uint256[] memory shares = new uint256[](addresses.length);

        vm.deal(address(factory), addresses.length * 10001);

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010) &&
                    // not the factory address
                    addresses[i] != address(factory) &&
                    // not the current address
                    addresses[i] != address(this)
            );
            shares[i] = 1;
        }

        address payoutAddress = factory.payout(
            addresses,
            shares,
            addresses.length
        );
        splitter = PaymentSplitter(payable(payoutAddress));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            assert(addresses[i].balance == 1);
        }

        // now payout again to check we can create a new contract
        for (uint256 i = 0; i < addresses.length; i++) {
            shares[i] = 10000;
        }

        address payoutAddress2 = factory.payout(
            addresses,
            shares,
            addresses.length * 10000
        );
        // make sure the current variable has updated accordingly
        assert(payoutAddress != payoutAddress2);
        splitter = PaymentSplitter(payable(payoutAddress2));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            // release 1 + release 2 balance
            assert(addresses[i].balance == 10001);
        }
    }

    // NOTE: we don't fuzz  payout amounts as this causes too many rejections for forge to extract statistically significant numbers
    function testRelease_withFuzzing(address[] calldata addresses) public {
        vm.assume(
            addresses.length > 0 &&
                // has no duplicates
                hasNoDuplicates(addresses)
        );

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010) &&
                    // not the factory address
                    addresses[i] != address(factory) &&
                    // not the current address
                    addresses[i] != address(this)
            );
        }

        factory = new PayoutFactory(address(this));
        vm.deal(address(factory), addresses.length * 12);

        uint256[] memory shares = new uint256[](addresses.length);
        for (uint256 j = 0; j < addresses.length; j++) {
            shares[j] = 1;
        }

        for (uint256 i = 0; i < 12; i++) {
            // now payout again to check we can create a new contract
            address payoutAddr = factory.payout(
                addresses,
                shares,
                addresses.length
            );
            for (uint256 j = 0; j < addresses.length; j++) {
                vm.assume(addresses[j] != payoutAddr);
            }
        }

        for (uint256 i = 0; i < addresses.length; i++) {
            assert(factory.releasable(addresses[i]) == 12);
            factory.releaseAll(addresses[i], 0);
            // 12 releases balance
            assert(addresses[i].balance == 12);
        }
    }
}
