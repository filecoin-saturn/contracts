// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

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
    address[] private _payouts;
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
    function payout(
        address[] memory payees,
        uint256[] memory shares_,
        uint256 totalValue
    ) external onlyRole(DEFAULT_ADMIN_ROLE) returns (address instance) {
        // create new payout instance
        require(
            payees.length <= 700,
            "PayoutFactory: payees is longer than 700 not met"
        );

        instance = template.clone();

        // register
        _payouts.push(instance);

        // emit event
        emit SplitterCreated(instance);

        // initializes and locks in a payout
        PaymentSplitter splitter = PaymentSplitter(payable(instance));
        splitter.initialize(payees, shares_);

        // if tokens were pre-added to this contract here's where we'd fund the new contract
        bool sent = payable(instance).send(totalValue);
        require(sent, "PayoutFactory: Failed to send FIL");
    }

    /**
     * @dev Returns a list of all previously generated PaymentSplitter contract addresses.
     */
    function payouts() external view returns (address[] memory) {
        return _payouts;
    }

    /**
     * @dev Returns the total released funds over all previously generated payout contracts.
     */
    function totalReleased() external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            totalValue += rewards.totalReleased();
        }
    }

    /**
     * @dev Returns the total claimable amount for an account
            over all previously generated payout contracts.
     * @param account Address of the payee.  
     */
    function releasablePerContract(
        address account
    )
        external
        view
        returns (address[] memory, uint256[] memory, uint256[] memory)
    {
        uint256 length = _payouts.length;
        address[] memory contracts = new address[](length);
        uint256[] memory releasableFunds = new uint256[](length);
        uint256[] memory releasedFunds = new uint256[](length);

        for (uint256 i = 0; i < length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            uint256 payeeShares = rewards.shares(account);

            if (payeeShares > 0) {
                contracts[i] = _payouts[i];
                releasableFunds[i] = rewards.releasable(account);
                releasedFunds[i] = rewards.released(account);
            }
        }
        return (contracts, releasableFunds, releasedFunds);
    }

    /**
     * @dev Returns the total claimable amount over all previously generated payout contracts.
     * @param account The address of the payee.
     */
    function releasable(
        address account
    ) external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            totalValue += rewards.releasable(account);
        }
    }

    /**
     * @dev Returns the total released amount over all previously generated payout contracts.
     * @param account The address of the payee.
     */
    function released(
        address account
    ) external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitter rewards = PaymentSplitter(payable(_payouts[i]));
            totalValue += rewards.released(account);
        }
    }

    /**
     * @dev Releases all available funds in previously generated payout contracts subsequent to a given offset.
     * @param offset The index of the first payout contract to release. At most 12 contracts can be released in a single call.
     */
    function releaseAll(address account, uint256 offset) external {
        uint256 length = _payouts.length;
        require(offset <= length);
        uint limit = 12;
        // min
        uint stop = length <= offset + limit ? length : offset + limit;
        for (uint i = offset; i < stop; i++) {
            _releasePayout(account, i);
        }
    }

    /**
     * @dev Releases all available funds in selected generated payout contracts.
     * @param account The address of the payee.
     * @param indices List of indices of the payout contracts to release. At most 12 contracts can be released in a single call.
     */
    function releaseSelect(address account, uint256[] memory indices) external {
        uint256 length = indices.length;
        require(length <= 12, "PayoutFactory: Too many contracts to release");
        for (uint256 i = 0; i < length; i++) {
            uint256 index = indices[i];
            require(index < _payouts.length);
            address paymentSplitterAddress = _payouts[index];
            uint256 claimable = PaymentSplitter(payable(paymentSplitterAddress))
                .releasable(account);

            if (claimable > 0) {
                emit PaymentReleased(account, claimable);
                PaymentSplitter(payable(paymentSplitterAddress)).release(
                    payable(account)
                );
            }
        }
    }

    /**
     * @dev Releases all available funds in a single previously generated payout contract.
     * @param account The address of the payee.
     * @param index Index of the payout contract.
     */
    function _releasePayout(address account, uint256 index) private {
        PaymentSplitter splitter = PaymentSplitter(payable(_payouts[index]));
        uint256 claimable = splitter.releasable(account);

        if (claimable > 0) {
            emit PaymentReleased(account, claimable);
            splitter.release(account);
        }
    }
}
