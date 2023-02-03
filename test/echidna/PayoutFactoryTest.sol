// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../../src/PayoutFactory.sol";
import "../../src/PaymentSplitter.sol";

contract TestPayoutFactory is PayoutFactory {

   address private deployerAddr;
   constructor() PayoutFactory(deployerAddr) {
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x2000));
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x3000));
        _grantRole(DEFAULT_ADMIN_ROLE, address(0x1000));
        _grantRole(DEFAULT_ADMIN_ROLE, deployerAddr);
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

    function echidna_payout_length() public view returns(bool) {
        return _payouts.length == 0;
    }

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