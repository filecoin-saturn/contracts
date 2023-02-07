// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.16;

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

    function eachPayeeHasShare(
        address[] memory payees,
        mapping(address => uint256) storage sharesMap,
        mapping(address => uint256) storage releasedMap
    ) private view returns (bool) {
        uint256 i;
        for (; i < payees.length; ) {
            if (releasedMap[payees[i]] == 0 && sharesMap[payees[i]] == 0) {
                return false;
            }
            unchecked {
                i++;
            }
        }
        return true;
    }

    function echidna_released_less_total() public view returns (bool) {
        return payeeReleaseLessThanTotal(_payees);
    }

    function echidna_shares_less_total() public view returns (bool) {
        return payeeSharesLessThanTotal(_payees);
    }

    function echidna_payees_have_shares() public view returns (bool) {
        return eachPayeeHasShare(_payees, _shares, _released);
    }

    function echidna_no_duplicate_payees() public returns (bool) {
        return hasNoDuplicates(_payees);
    }
}
