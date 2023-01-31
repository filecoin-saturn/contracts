// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../../src/PaymentSplitter.sol";

contract TestPaymentSplitter is PaymentSplitter {
    mapping(address => bool) Addr;

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

    function echidna_released_less_total() public view returns (bool) {
        return totalReleased() <= totalShares();
    }

    function echidna_no_duplicate_payees() public returns (bool) {
        return hasNoDuplicates(_payees);
    }
}
