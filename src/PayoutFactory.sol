// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./PaymentSplitter.sol";
import "../lib/openzeppelin-contracts/contracts/proxy/Clones.sol";
import "../lib/openzeppelin-contracts/contracts/access/AccessControl.sol";

/**
 * @title Payout factory
 * @dev This contract is a factory contract for generating new PaymentSplitter contracts and tracking old instantiations of these contracts.
 * function.
 */
contract PayoutFactory is AccessControl {
    event SplitterCreated(address newSplitter);
    event PaymentReleased(address to, uint256 amount);
    event PaymentReceived(address from, uint256 amount);
    using Clones for address;

    // past payout contracts
    address[] internal _payouts;
    // a dummy template for instantiating future splitting contracts
    address public immutable template = address(new PaymentSplitter());

    /**
     * @dev Creates a new factory with an admin
     * @param admin The address of the factory admin.
     **/
    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /**
     * @dev The FIL received will be logged with {PaymentReceived} events. Note that these events are not fully
     * reliable: it's possible for a contract to receive FIL without triggering this function. This only affects the
     * reliability of the events, and not the actual splitting of FIL.
     *
     * To learn more about this see the Solidity documentation for
     * https://solidity.readthedocs.io/en/latest/contracts.html#fallback-function[fallback
     * functions].
     */
    receive() external payable virtual {
        emit PaymentReceived(msg.sender, msg.value);
    }

    /**
     * @dev Spins up a new payment splitter where each account in `payees` is assigned the number of shares at
     * the matching position in the `shares` array.
     *
     * All addresses in `payees` must be non-zero. Both arrays must have the same non-zero length, and there must be no
     * duplicates in `payees`.
     */
    function payout(address[] memory payees, uint256[] memory shares_)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        returns (address instance)
    {
        // create new payout instance
        instance = template.clone();

        // register
        _payouts.push(instance);

        // initializes and locks in a payout
        PaymentSplitter splitter = PaymentSplitter(payable(instance));
        splitter.initialize(payees, shares_);

        // if tokens were pre-added to this contract here's where we'd fund the new contract
        bool sent = payable(instance).send(splitter.totalShares());
        require(sent, "PayoutFactory: Failed to send FIL");

        // emit event
        emit SplitterCreated(instance);
    }

    /**
     * @dev Returns the total claimable amount over all previously generated payout contracts.
     * @param account The address of the payee.
     */
    function releasable(address account)
        external
        view
        returns (uint256 totalValue)
    {
        for (uint256 i = 0; i < _payouts.length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            totalValue += rewards.releasable(account);
        }
    }

    /**
     * @dev Releases all available funds in previously generated payout contracts.
     * @param account The address of the payee.
     */
    function releaseAll(address account) external {
        uint256 claimable = this.releasable(account);
        require(
            claimable > 0,
            "PaymentSplitter: account has no shares to claim"
        );
        for (uint256 i = 0; i < _payouts.length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            if (rewards.releasable(account) > 0) {
                PaymentSplitter(payable(_payouts[i])).release(payable(account));
            }
        }
        emit PaymentReleased(account, claimable);
    }

    /**
     * @dev Releases all available funds in a previously generated payout contract.
     * @param account The address of the payee.
     * @param index Index of the payout contract.
     */
    function releasePayout(address account, uint256 index) external {
        uint256 claimable = PaymentSplitter(payable(_payouts[index]))
            .releasable(account);
        require(
            claimable > 0,
            "PaymentSplitter: account has no shares to claim"
        );
        PaymentSplitter(payable(_payouts[index])).release(payable(account));
        emit PaymentReleased(account, claimable);
    }
}
