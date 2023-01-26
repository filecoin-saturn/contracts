// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/EvaluationFactory.sol";
import "../src/PaymentSplitter.sol";

contract EvaluationFactoryTest is Test {
    EvaluationFactory public factory;
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

        factory = new EvaluationFactory(address(this));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010)
            );
            factory.rewardPayee(addresses[i], 1);
        }

        address payoutAddress = factory.payout();
        splitter = PaymentSplitter(payable(payoutAddress));

        vm.deal(address(splitter), splitter.totalShares());
        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            assert(addresses[i].balance == 1);
            assert(factory.shares(addresses[i]) == 0);
        }

        // now payout again to check we can create a new contract
        for (uint256 i = 0; i < addresses.length; i++) {
            factory.rewardPayee(addresses[i], 10000);
        }

        address payoutAddress2 = factory.payout();
        // make sure the current variable has updated accordingly
        assert(payoutAddress != payoutAddress2);
        splitter = PaymentSplitter(payable(payoutAddress2));

        vm.deal(address(splitter), splitter.totalShares());
        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            // release 1 + release 2 balance
            assert(addresses[i].balance == 10001);
            assert(factory.shares(addresses[i]) == 0);
        }
    }
}
