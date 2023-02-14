// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/PayoutFactory.sol";
import "../src/Evaluator.sol";
import "../src/PaymentSplitter.sol";

contract EvaluatorTest is Test {
    Evaluator public evaluator;
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
    function testEvaluation_withFuzzing(address[] calldata addresses) public {
        vm.assume(
            addresses.length > 0 &&
                // has no duplicates
                hasNoDuplicates(addresses)
        );

        evaluator = new Evaluator(address(this));
        vm.deal(address(evaluator), addresses.length * 10001);

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010)
            );
            evaluator.rewardPayee(addresses[i], 1);
        }

        address payoutAddress = evaluator.payout();
        assert(evaluator.totalShares() == 0);
        splitter = PaymentSplitter(payable(payoutAddress));

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            assert(addresses[i].balance == 1);
            assert(evaluator.shares(addresses[i]) == 0);
        }

        // now payout again to check we can create a new contract
        for (uint256 i = 0; i < addresses.length; i++) {
            evaluator.rewardPayee(addresses[i], 10000);
        }

        address payoutAddress2 = evaluator.payout();
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
    function testClaim_withFuzzing(address[] calldata addresses) public {
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

        evaluator = new Evaluator(address(this));

        vm.deal(address(evaluator), addresses.length * 12);

        for (uint256 i = 0; i < 12; i++) {
            for (uint256 j = 0; j < addresses.length; j++) {
                evaluator.rewardPayee(addresses[j], 1);
            }
            // now payout again to check we can create a new contract
            address payoutAddr = evaluator.payout();
            assert(evaluator.totalShares() == 0);
            for (uint256 j = 0; j < addresses.length; j++) {
                vm.assume(addresses[j] != payoutAddr);
            }
            console.log(i);
        }

        for (uint256 i = 0; i < addresses.length; i++) {
            console.log(evaluator.releasable(addresses[i]));
            assert(evaluator.releasable(addresses[i]) == 12);
            evaluator.claim(addresses[i]);
            // 12 releases balance
            assert(addresses[i].balance == 12);
            assert(evaluator.shares(addresses[i]) == 0);
        }
    }
}
