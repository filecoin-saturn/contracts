// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../../src/PayoutFactory.sol";
import "../../src/PaymentSplitter.sol";

contract TestPayoutFactory is PayoutFactory {

   constructor()  PayoutFactory(address(0x00a329c0648769A73afAc7F9381E08FB43dBEA72)) payable {
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x0000000000000000000000000000000000010000));
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x0000000000000000000000000000000000020000));
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x0000000000000000000000000000000000030000));
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x00a329c0648769A73afAc7F9381E08FB43dBEA72));
    }

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

    function contractReleaseLessThanTotal(address[] memory arr)
        private
        view
        returns (bool)
    {
        uint256 i;
        for (; i < arr.length; ) {
            PaymentSplitter splitter = PaymentSplitter(payable(arr[i]));
            address[] memory payees = splitter.payees();
            uint256 contractReleased = 0;
            uint256 j;
            for(; j < payees.length;) {

                address payee = payees[j];
                if (splitter.released(payee) > splitter.totalReleased()) {
                    return false;
                }

                unchecked {
                    j++;
                    contractReleased += splitter.released(payee);
                }
            }

            if (contractReleased > this.totalReleased()) {
                return false;
            }

            unchecked {
                i++;
            }
        }
        return true;
    }

    function contractSharesLessThanTotal(address[] memory arr)
        private
        view
        returns (bool)
    {
        uint256 i;
        for (; i < arr.length; ) {
            PaymentSplitter splitter = PaymentSplitter(payable(arr[i]));
            address[] memory payees = splitter.payees();
            uint256 contractShares = 0;
            uint256 j;
            for(; j < payees.length;) {

                address payee = payees[j];
                if (splitter.shares(payee) > splitter.totalShares()) {
                    return false;
                }

                unchecked {
                    j++;
                    contractShares += splitter.shares(payee);
                }
            }

            if (contractShares > this.totalShares()) {
                return false;
            }

            unchecked {
                i++;
            }
        }
        return true;
    }


    function eachContractHasPayees(
        address[] memory arr
    ) private view returns (bool) {
        uint256 i;
        for (; i < arr.length; ) {

            PaymentSplitter splitter = PaymentSplitter(payable(arr[i]));
            address[] memory payees = splitter.payees();

            if (payees.length == 0) {
                return false;
            }

            unchecked {
                i++;
            }
        }
        return true;
    }

    // This function acts as a sanity check that the PayoutFactory
    // is generating new PaymentSplitter contracts. It should always fail
    // function echidna_payout_length() public view returns(bool) {
    //     return _payouts.length == 0;
    // }

    function echidna_released_less_total() public view returns (bool) {
        return contractReleaseLessThanTotal(_payouts);
    }

    function echidna_shares_less_total() public view returns (bool) {
        return contractSharesLessThanTotal(_payouts);
    }

    function echidna_contracts_have_payees() public view returns (bool) {
        return eachContractHasPayees(_payouts);
    }

    function echidna_no_duplicate_splitters() public returns (bool) {
        return hasNoDuplicates(_payouts);
    }
}
