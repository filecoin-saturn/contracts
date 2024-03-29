// SPDX-License-Identifier: MIT

pragma solidity ^0.8.17;

import "../lib/openzeppelin-contracts/contracts/proxy/utils/Initializable.sol";

import {SendAPI} from "../lib/filecoin-solidity/contracts/v0.8/SendAPI.sol";

import "../lib/filecoin-solidity/contracts/v0.8/types/CommonTypes.sol";

import "../lib/filecoin-solidity/contracts/v0.8/utils/FilAddresses.sol";

/**
 * @title PaymentSplitterNativeAddr
 * @dev This contract allows to split FIL payments among a group of accounts. The sender does not need to be aware
 * that the FIL will be split in this way, since it is handled transparently by the contract.
 *
 * The split can be in equal parts or in any other arbitrary proportion. The way this is specified is by assigning each
 * account to a number of shares. Of all the FIL that this contract receives, each account will then be able to claim
 * an amount proportional to the percentage of total shares they were assigned. The distribution of shares is set at the
 * time of contract deployment and can't be updated thereafter.
 *
 * `PaymentSplitterNativeAddr` follows a _pull payment_ model. This means that payments are not automatically forwarded to the
 * accounts but kept in this contract, and the actual transfer is triggered as a separate step by calling the {release}
 * function.
 *
 * [CAUTION]
 * ====
 * Avoid leaving a contract uninitialized !
 *
 * An uninitialized contract can be taken over by an attacker
 *
 */
contract PaymentSplitterNativeAddr is Initializable {
    event PayeeAdded(CommonTypes.FilAddress account, uint256 shares);
    event PaymentReleased(CommonTypes.FilAddress to, uint256 amount);
    event PaymentReceived(address from, uint256 amount);

    uint256 private _totalShares;
    uint256 private _totalReleased;

    mapping(bytes => uint256) private _shares;
    mapping(bytes => uint256) private _released;
    CommonTypes.FilAddress[] private _payees;

    /**
     * @dev Creates an instance of `PaymentSplitter` where each account in `payees` is assigned the number of shares at
     * the matching position in the `shares` array.
     *
     * All addresses in `payees` must be non-zero. Both arrays must have the same non-zero length, and there must be no
     * duplicates in `payees`.
     */
    function initialize(
        CommonTypes.FilAddress[] memory payees_,
        uint256[] memory shares_
    ) external payable initializer {
        require(
            payees_.length == shares_.length,
            "PaymentSplitter: payees and shares length mismatch"
        );
        require(payees_.length > 0, "PaymentSplitter: no payees");

        for (uint256 i = 0; i < payees_.length; i++) {
            require(payees_[i].data.length > 0, "PaymentSplitter: null payee");
            require(
                FilAddresses.validate(payees_[i]),
                "PaymentSplitter: invalid Filecoin address"
            );
            require(
                keccak256(payees_[i].data) !=
                    keccak256(FilAddresses.fromEthAddress(address(this)).data)
            );
            _addPayee(payees_[i], shares_[i]);
        }
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
     * @dev Getter for the total shares held by payees.
     */
    function totalShares() public view returns (uint256) {
        return _totalShares;
    }

    /**
     * @dev Getter for the total amount of FIL already released.
     */
    function totalReleased() public view returns (uint256) {
        return _totalReleased;
    }

    /**
     * @dev Getter for the payees.
     */
    function payees() public view returns (CommonTypes.FilAddress[] memory) {
        return _payees;
    }

    /**
     * @dev Getter for the amount of shares held by an account.
     */
    function shares(
        CommonTypes.FilAddress memory account
    ) public view returns (uint256) {
        return _shares[account.data];
    }

    /**
     * @dev Getter for the amount of FIL already released to a payee.
     */
    function released(
        CommonTypes.FilAddress memory account
    ) public view returns (uint256) {
        return _released[account.data];
    }

    /**
     * @dev Getter for the amount of payee's releasable FIL.
     */
    function releasable(
        CommonTypes.FilAddress memory account
    ) public view returns (uint256) {
        uint256 totalReceived = address(this).balance + totalReleased();
        return _pendingPayment(account, totalReceived, released(account));
    }

    /**
     * @dev Triggers a transfer to `account` of the amount of FIL they are owed, according to their percentage of the
     * total shares and their previous withdrawals.
     */
    function release(CommonTypes.FilAddress memory account) public virtual {
        require(
            _shares[account.data] > 0,
            "PaymentSplitter: account has no shares"
        );

        require(
            FilAddresses.validate(account),
            "PaymentSplitter: invalid Filecoin address"
        );

        uint256 payment = releasable(account);

        require(payment != 0, "PaymentSplitter: account is not due payment");

        // _totalReleased is the sum of all values in _released.
        // If "_totalReleased += payment" does not overflow, then "_released[account] += payment" cannot overflow.
        _totalReleased += payment;
        unchecked {
            _released[account.data] += payment;
        }
        emit PaymentReleased(account, payment);

        // will revert
        SendAPI.send(account, payment);
    }

    /**
     * @dev internal logic for computing the pending payment of an `account` given the token historical balances and
     * already released amounts.
     */
    function _pendingPayment(
        CommonTypes.FilAddress memory account,
        uint256 totalReceived,
        uint256 alreadyReleased
    ) private view returns (uint256) {
        return
            (totalReceived * _shares[account.data]) /
            _totalShares -
            alreadyReleased;
    }

    /**
     * @dev Add a new payee to the contract.
     * @param account The address of the payee to add.
     * @param shares_ The number of shares owned by the payee.
     */
    function _addPayee(
        CommonTypes.FilAddress memory account,
        uint256 shares_
    ) private {
        require(
            keccak256(account.data) !=
                keccak256(FilAddresses.fromEthAddress(address(0)).data),
            "PaymentSplitter: account is the zero address"
        );
        require(shares_ > 0, "PaymentSplitter: shares are 0");
        require(
            _shares[account.data] == 0,
            "PaymentSplitter: account already has shares"
        );

        _payees.push(account);
        _shares[account.data] = shares_;
        _totalShares = _totalShares + shares_;
        emit PayeeAdded(account, shares_);
    }
}
