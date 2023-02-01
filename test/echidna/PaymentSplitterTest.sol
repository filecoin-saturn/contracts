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

    function payeeReleaseLessThanShares(address[] memory arr) private view returns (bool) {
        uint256 i;
        for (;i < arr.length; ) {
            if (released(arr[i]) > shares(arr[i])) {
                return false;
            }
            unchecked {
                 i++;
            }
        }
        return true;
    }


    function sharesEqualClaimableAndReleased(address[] memory arr) private view returns (bool) {
        uint256 i;
        for (;i < arr.length; ) {
            if (released(arr[i]) + releasable(arr[i]) != shares(arr[i])) {
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
        ) 
    private view returns (bool) {
        uint256 i;
        for (;i < payees.length;) {
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
        return totalReleased() <= totalShares();
    }

    function echidna_releasable_less_total() public view returns (bool) {
        return payeeReleaseLessThanShares(_payees);
    }

    function echidna_shares_equal_released_claimable() public view returns(bool) {
        return sharesEqualClaimableAndReleased(_payees);
    }

    function echidna_payees_have_shares() public view returns (bool) {
        return eachPayeeHasShare(_payees, _shares, _released);
    }

    function echidna_no_duplicate_payees() public returns (bool) {
        return hasNoDuplicates(_payees);
    }
}
