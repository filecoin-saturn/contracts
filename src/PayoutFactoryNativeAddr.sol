// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import "./PaymentSplitterNativeAddr.sol";
import "../lib/openzeppelin-contracts/contracts/proxy/Clones.sol";
import "../lib/openzeppelin-contracts/contracts/access/AccessControl.sol";

/**
 * @title Payout factory
 * @dev This contract is a factory contract for generating new PaymentSplitterNativeAddr contracts and tracking old instantiations of these contracts.
 * function.
 */
contract PayoutFactoryNativeAddr is AccessControl {
    event SplitterCreated(address newSplitter);
    event PaymentReleased(CommonTypes.FilAddress to, uint256 amount);
    event PaymentReceived(address from, uint256 amount);
    using Clones for address;

    // past payout contracts
    address[] private _payouts;
    // a dummy template for instantiating future splitting contracts
    address public immutable template =
        address(new PaymentSplitterNativeAddr());

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
        CommonTypes.FilAddress[] memory payees,
        uint256[] memory shares_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) returns (address instance) {
        // create new payout instance
        instance = template.clone();

        // register
        _payouts.push(instance);

        // emit event
        emit SplitterCreated(instance);

        // initializes and locks in a payout
        PaymentSplitterNativeAddr splitter = PaymentSplitterNativeAddr(
            payable(instance)
        );
        splitter.initialize(payees, shares_);

        // if tokens were pre-added to this contract here's where we'd fund the new contract
        bool sent = payable(instance).send(splitter.totalShares());
        require(sent, "PayoutFactory: Failed to send FIL");
    }

    /**
     * @dev Returns a list of all previously generated PaymentSplitterNativeAddr contract addresses.
     */
    function payouts() external view returns (address[] memory) {
        return _payouts;
    }

    /**
     * @dev Returns the total shares over all previously generated payout contracts.
     */
    function totalShares() external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
            totalValue += rewards.totalShares();
        }
    }

    /**
     * @dev Returns the total released funds over all previously generated payout contracts.
     */
    function totalReleased() external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
            totalValue += rewards.totalReleased();
        }
    }

    /**
     * @dev Returns the total claimable amount for an account
            over all previously generated payout contracts.
     * @param account Address of the payee.  
     */
    function releasablePerContract(
        CommonTypes.FilAddress memory account
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
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
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
        CommonTypes.FilAddress memory account
    ) external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
            totalValue += rewards.releasable(account);
        }
    }

    /**
     * @dev Returns the total released amount over all previously generated payout contracts.
     * @param account The address of the payee.
     */
    function released(
        CommonTypes.FilAddress memory account
    ) external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
            totalValue += rewards.released(account);
        }
    }

    /**
     * @dev Returns the total shares amount over all previously generated payout contracts.
     * @param account The address of the payee.
     */
    function shares(
        CommonTypes.FilAddress memory account
    ) external view returns (uint256 totalValue) {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            PaymentSplitterNativeAddr rewards = PaymentSplitterNativeAddr(
                payable(_payouts[i])
            );
            totalValue += rewards.shares(account);
        }
    }

    /**
     * @dev Releases all available funds in previously generated payout contracts.
     */
    function releaseAll(CommonTypes.FilAddress memory account) external {
        uint256 length = _payouts.length;
        for (uint256 i = 0; i < length; i++) {
            _releasePayout(account, i);
        }
    }

    /**
     * @dev Releases all available funds in selected generated payout contracts.
     * @param account The address of the payee.
     * @param selectPayouts List of selected PaymentSplitterNativeAddr addresses.
     */
    function releaseSelect(
        CommonTypes.FilAddress memory account,
        address[] memory selectPayouts
    ) external {
        uint256 length = selectPayouts.length;
        for (uint256 i = 0; i < length; i++) {
            address paymentSplitterAddress = selectPayouts[i];
            uint256 claimable = PaymentSplitterNativeAddr(
                payable(paymentSplitterAddress)
            ).releasable(account);

            if (claimable > 0) {
                emit PaymentReleased(account, claimable);
                PaymentSplitterNativeAddr(payable(paymentSplitterAddress))
                    .release(account);
            }
        }
    }

    /**
     * @dev Releases all available funds in a single previously generated payout contract.
     * @param account The address of the payee.
     * @param index Index of the payout contract.
     */
    function _releasePayout(
        CommonTypes.FilAddress memory account,
        uint256 index
    ) private {
        PaymentSplitterNativeAddr splitter = PaymentSplitterNativeAddr(
            payable(_payouts[index])
        );
        uint256 claimable = splitter.releasable(account);

        if (claimable > 0) {
            emit PaymentReleased(account, claimable);
            splitter.release(account);
        }
    }
}
