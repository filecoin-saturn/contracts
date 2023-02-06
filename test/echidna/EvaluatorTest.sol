// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "../../src/PayoutFactory.sol";
import "../../src/Evaluator.sol";

contract TestEvaluator is Evaluator {
    constructor()
        payable
        Evaluator(address(0x00a329c0648769A73afAc7F9381E08FB43dBEA72))
    {
        _grantRole(
            DEFAULT_ADMIN_ROLE,
            address(0x0000000000000000000000000000000000010000)
        );
        _grantRole(
            DEFAULT_ADMIN_ROLE,
            address(0x00a329c0648769A73afAc7F9381E08FB43dBEA72)
        );
    }

    mapping(address => bool) Addr;

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

    function echidna_shares_less_total() public view returns (bool) {
        return payeeSharesLessThanTotal(payees());
    }
}
