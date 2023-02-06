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

    function payeeReleaseLessThanTotal(address[] memory arr)
        private
        view
        returns (bool)
    {
        uint256 i;
        for (; i < arr.length; ) {
            if (released(arr[i]) > totalReleased()) {
                return false;
            }
            unchecked {
                i++;
            }
        }
        return true;
    }

    function payeeSharesLessThanTotal(address[] memory arr)
        private
        view
        returns (bool)
    {
        uint256 i;
        for (; i < arr.length; ) {
            if (shares(arr[i]) > totalShares()) {
                return false;
            }
            unchecked {
                i++;
            }
        }
        return true;
    }

    function eachPayeeHasShare(address[] memory payees)
        private
        view
        returns (bool)
    {
        uint256 i;
        for (; i < payees.length; ) {
            if (released(payees[i]) == 0 && shares(payees[i]) == 0) {
                return false;
            }
            unchecked {
                i++;
            }
        }
        return true;
    }

    function echidna_released_less_total() public view returns (bool) {
        return payeeReleaseLessThanTotal(payees());
    }

    function echidna_shares_less_total() public view returns (bool) {
        return payeeSharesLessThanTotal(payees());
    }

    function echidna_payees_have_shares() public view returns (bool) {
        return eachPayeeHasShare(payees());
    }

    function echidna_no_duplicate_payees() public returns (bool) {
        return hasNoDuplicates(payees());
    }
}
