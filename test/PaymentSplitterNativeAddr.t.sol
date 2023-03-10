// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "../src/PaymentSplitterNativeAddr.sol";
import "../lib/filecoin-solidity/contracts/v0.8/types/CommonTypes.sol";
import "../lib/filecoin-solidity/contracts/v0.8/utils/FilAddresses.sol";

contract PaymentSplitterTestNativeAddr is Test {
    PaymentSplitterNativeAddr public splitter;

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
    function testRelease_withFuzzing(address[] calldata addresses) public {
        vm.assume(
            addresses.length > 0 &&
                // has no duplicates
                hasNoDuplicates(addresses)
        );

        uint256[] memory shares = new uint256[](addresses.length);

        CommonTypes.FilAddress[]
            memory fil_addresses = new CommonTypes.FilAddress[](
                addresses.length
            );

        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(
                // zero address
                addresses[i] != address(0) &&
                    // reserved addresses
                    uint160(addresses[i]) >
                    uint160(0x0000000000000000000000000000000000000010)
            );
            shares[i] = 1;
            fil_addresses[i] = FilAddresses.fromEthAddress(addresses[i]);
        }

        splitter = new PaymentSplitterNativeAddr();
        splitter.initialize(fil_addresses, shares);
        vm.deal(address(splitter), splitter.totalShares());
        for (uint256 i = 0; i < addresses.length; i++) {
            vm.assume(addresses[i] != address(splitter));
            // splitter.release(fil_addresses[i]);
            assert(addresses[i].balance == 0);
        }
    }
}
