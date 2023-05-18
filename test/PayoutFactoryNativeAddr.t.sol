// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/PayoutFactoryNativeAddr.sol";
import "../src/PaymentSplitterNativeAddr.sol";

contract PayoutFactoryTestNativeAddr is Test {
    PayoutFactoryNativeAddr public factory;
    PaymentSplitterNativeAddr public splitter;

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

        factory = new PayoutFactoryNativeAddr(address(this));

        uint256[] memory shares = new uint256[](addresses.length);
        CommonTypes.FilAddress[]
            memory fil_addresses = new CommonTypes.FilAddress[](
                addresses.length
            );

        vm.deal(address(factory), addresses.length * 10001);

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010)
            );
            shares[i] = 1;
            fil_addresses[i] = FilAddresses.fromEthAddress(addresses[i]);
        }

        address payoutAddress = factory.payout(
            fil_addresses,
            shares,
            // each address gets 1 Fil
            addresses.length
        );
        splitter = PaymentSplitterNativeAddr(payable(payoutAddress));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
        }

        // now payout again to check we can create a new contract
        for (uint256 i = 0; i < addresses.length; i++) {
            shares[i] = 10000;
        }

        address payoutAddress2 = factory.payout(
            fil_addresses,
            shares,
            // each address gets 10000 Fil
            addresses.length * 10000
        );
        // make sure the current variable has updated accordingly
        assert(payoutAddress != payoutAddress2);
        splitter = PaymentSplitterNativeAddr(payable(payoutAddress2));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
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
                    uint160(0x0000000000000000000000000000000000000010)
            );
        }

        factory = new PayoutFactoryNativeAddr(address(this));
        vm.deal(address(factory), addresses.length * 1000);

        uint256[] memory shares = new uint256[](addresses.length);
        CommonTypes.FilAddress[]
            memory fil_addresses = new CommonTypes.FilAddress[](
                addresses.length
            );

        for (uint256 j = 0; j < addresses.length; j++) {
            shares[j] = 1;
            fil_addresses[j] = FilAddresses.fromEthAddress(addresses[j]);
        }

        for (uint256 i = 0; i < 12; i++) {
            // now payout again to check we can create a new contract
            address payoutAddr = factory.payout(
                fil_addresses,
                shares,
                addresses.length
            );
            for (uint256 j = 0; j < addresses.length; j++) {
                vm.assume(addresses[j] != payoutAddr);
            }
        }

        for (uint256 i = 0; i < addresses.length; i++) {
            assert(factory.releasable(fil_addresses[i]) == 12);
            // factory.releaseAll(fil_addresses[i]);
            // 12 releases balance
            // assert(addresses[i].balance == 12);
        }
    }
}
