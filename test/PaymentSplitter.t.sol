// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/PaymentSplitter.sol";

contract PaymentSplitterTest is Test {
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

    function testRelease_withFuzzing(address[] calldata addresses) public {
        vm.assume(
            addresses.length > 0 &&
                // has no duplicates
                hasNoDuplicates(addresses)
        );

        uint256[] memory shares = new uint256[](addresses.length);

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010)
            );
            shares[i] = 1;
        }

        splitter = new PaymentSplitter(addresses, shares);
        vm.deal(address(splitter), addresses.length);
        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            splitter.release(payable(addresses[i]));
            assert(addresses[i].balance == 1);
        }
    }
}
