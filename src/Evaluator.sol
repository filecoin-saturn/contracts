// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./PayoutFactory.sol";
import "../lib/openzeppelin-contracts/contracts/proxy/Clones.sol";
import "../lib/openzeppelin-contracts/contracts/access/AccessControl.sol";

/**
 * @title Evaluator contract
 * @dev This contract is an evaluator contract for tracking rewards for individual payees and then generating payouts.
 */
contract Evaluator is AccessControl {
    event Payout(address newSplitter);
    event PaymentReceived(address from, uint256 amount);
    event RewardedPayee(address account, uint256 shares);
    using Clones for address;

    // the total amount of shares allocated during the current evaluation round
    uint256 private _totalShares;
    // the index of the current evaluations
    uint256 private _payoutIndex;

    mapping(uint256 => mapping(address => uint256)) private _shares;
    address[] private _payees;

    PayoutFactory factory;

    /**
     * @dev Creates a new evaluator with an admin
     * @param admin The address of the evaluator admin.
     **/
    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        // create new factory
        factory = new PayoutFactory(address(this));
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
     * @dev Spins up a new payment splitter using _shares as input to the factory contract payout function.
     */
    function payout()
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        returns (address instance)
    {
        uint256 numPayees = _payees.length;

        // initializes and locks in a payout
        uint256[] memory sharesSnapshot = new uint256[](numPayees);
        for (uint256 i = 0; i < numPayees; i++) {
            sharesSnapshot[i] = (_shares[_payoutIndex][_payees[i]]);
        }

        bool sent = payable(factory).send(_totalShares);
        require(sent, "Evaluator: Failed to send FIL");

        instance = factory.payout(_payees, sharesSnapshot);

        // reset counters
        delete _payees;
        delete _totalShares;
        _payoutIndex += 1;

        // emit event
        emit Payout(instance);
    }

    /**
     * @dev Returns the total claimable amount from factory
     * @param account The address of the payee.
     */
    function releasable(address account)
        external
        view
        returns (uint256 totalValue)
    {
        totalValue = factory.releasable(account);
    }

    /**
     * @dev Releases all available funds from factory.
     * @param account The address of the payee.
     */
    function claim(address account) external {
        factory.releaseAll(payable(account));
    }

    /**
     * @dev Reward a payee.
     * @param account The address of the payee.
     * @param shares_ The number of shares owned by the payee.
     */
    function rewardPayee(address account, uint256 shares_)
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        require(
            account != address(0),
            "Evaluator: account is the zero address"
        );
        require(shares_ > 0, "Evaluator: shares are 0");

        // means this is a new payee
        if (_shares[_payoutIndex][account] == 0) {
            _payees.push(account);
        }

        _shares[_payoutIndex][account] =
            _shares[_payoutIndex][account] +
            shares_;
        _totalShares = _totalShares + shares_;
        emit RewardedPayee(account, shares_);
    }

    /**
     * @dev Getter for the total shares held by payees.
     */
    function totalShares() public view returns (uint256) {
        return _totalShares;
    }

    /**
     * @dev Getter for the amount of shares held by an account in current eval period.
     */
    function shares(address account) public view returns (uint256) {
        return _shares[_payoutIndex][account];
    }
}
