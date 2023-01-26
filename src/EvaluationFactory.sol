// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./PaymentSplitter.sol";
import "../lib/openzeppelin-contracts/contracts/proxy/Clones.sol";
import "../lib/openzeppelin-contracts/contracts/access/AccessControl.sol";

contract EvaluationFactory is AccessControl {
    using Clones for address;

    // the total amount of shares allocated during the current evaluation round
    uint256 private _totalShares;
    // the index of the current evaluations
    uint256 private _payoutIndex;

    mapping(uint256 => mapping(address => uint256)) private _shares;
    address[] private _payees;

    // a dummy template for instantiating future splitting contracts
    address public immutable template = address(new PaymentSplitter());

    event SplitterCreated(address newSplitter);
    event RewardedPayee(address account, uint256 shares);

    /*
     * @dev Creates a new factory with an admin
     * @param admin The address of the factory admin.
     */
    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _payoutIndex = 0;
    }

    /**
     * @dev Getter for the total shares held by payees.
     */
    function totalShares() public view returns (uint256) {
        return _totalShares;
    }

    /**
     * @dev Getter for the amount of shares held by an account.
     */
    function shares(address account) public view returns (uint256) {
        return _shares[_payoutIndex][account];
    }

    /*
     * @dev Spins up a new payment splitter using the contract's _shares and _payees variables. By the end of process
     * both variables will be reset as they are set in stone by the new PaymentSplitter instance.
     */
    function payout()
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
        returns (address instance)
    {
        // create vesting instance
        instance = template.clone();

        uint256 numPayees = _payees.length;
        // initializes and locks in a payout
        uint256[] memory sharesSnapshot = new uint256[](numPayees);
        for (uint256 i = 0; i < numPayees; i++) {
            sharesSnapshot[i] = (_shares[_payoutIndex][_payees[i]]);
        }
        PaymentSplitter(payable(instance)).initialize(_payees, sharesSnapshot);

        // reset counters
        delete _payees;
        _payoutIndex += 1;

        // emit event
        emit SplitterCreated(instance);
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
            "EvaluationFactory: account is the zero address"
        );
        require(shares_ > 0, "EvaluationFactory: shares are 0");

        // means this is a new payee
        if (_shares[_payoutIndex][account] == 0) {
            _payees.push(account);
        }

        _shares[_payoutIndex][account] += shares_;
        _totalShares = _totalShares + shares_;
        emit RewardedPayee(account, shares_);
    }
}
