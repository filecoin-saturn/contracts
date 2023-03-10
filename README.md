<h1 align="center">
	<br>
	 :black_nib: :scroll:
	<br>
	<br>
	Contracts
	<br>
	<br>
	<br>
</h1>

> Repo for Saturn Payouts Solidity Contracts


## What this

The first contract implements a simple payment splitter which, when instantiated, takes in a list of payees and shares owed.

```solidity
/**
* @dev Creates an instance of `PaymentSplitterNativeAddr` where each account in `payees` is assigned the number of shares at
* the matching position in the `shares` array.
*
* All addresses in `payees` must be non-zero. Both arrays must have the same non-zero length, and there must be no
* duplicates in `payees`.
*/
function initialize(
        CommonTypes.FilAddress[] memory payees_,
        uint256[] memory shares_
    ) external payable

```

The release function can then be triggered to release payments owed to a specific account.

```solidity
/**
* @dev Triggers a transfer to `account` of the amount of FIL they are owed, according to their percentage of the
* total shares and their previous withdrawals.
*/
function release(CommonTypes.FilAddress memory account) public virtual {...}

```


The second contract is a payout factory contract. Its core functionality, which is to instantiate and fund a new payment splitter contract, can only be invoked by the admin user of the contract.

```solidity
/**
* @dev Spins up a new payment splitter.
*/
function payout(CommonTypes.FilAddress[] memory payees_, uint256[] memory shares_)
	external
	onlyRole(DEFAULT_ADMIN_ROLE)
	returns (address instance) {...}
```

The contract also keeps track of all past payout contracts such that we can query it for all unclaimed tokens linked to a specific address. If this value is non nil we can also release all funds linked to an address.

```solidity
/**
* @dev Returns the total claimable amount over all previously generated payout contracts.
* @param account The address of the payee.
*/
function releasable(CommonTypes.FilAddress memory account)
	external
	view
	returns (uint256 totalValue) {...}

/**
* @dev Releases all available funds in previously generated payout contracts.
* @param account The address of the payee.
*/
function releaseAll(CommonTypes.FilAddress memory account) external {...}

/**
* @dev Releases all available funds in a single previously generated payout contract.
* @param account The fil address of the payee.
* @param index Index of the payout contract.
*/
function _releasePayout(CommonTypes.FilAddress memory account, uint256 index) private {...}
```

The third contract, is a simple evaluator which keeps track of how much each payee is owed in a given evaluation period. For now this value can only be _incremented_ by way of `rewardPayee`. This contract is also bundled with a  `PayoutFactory` upon construction. The evaluator contract is the admin of the factory which enables it to generate new payouts at the end of a given evaluation period. It uses an internal mapping (`mapping(uint256 => mapping(address => uint256)) private _shares`) as values for the payout. Whereby this mapping is reset after the call. )



## How to


We recommend installing `foundry` from source:
```bash
git clone https://github.com/foundry-rs/foundry
cd foundry
# install cast + forge
cargo install --path ./cli --profile local --bins --locked --force
# install anvil
cargo install --path ./anvil --profile local --locked --force
```

To run tests:
```bash
forge test
```
> Note: TO run these tests with commonly used solidity framework we have duplicated contracts which use Ethereum-based addressing for payouts -- the tests are run over these contracts, not the Filecoin-based addressing contracts. 

To run slither analyzer:
```bash
conda create -n contracts python=3.9
conda activate contracts
pip3 install slither-analyzer
slither .
```

To run echidna fuzzing tests, first install [echidna](https://github.com/crytic/echidna), then you can find echidna specific tests within `test/echidna` and run:

```bash
echidna-test test/echidna/PaymentSplitterTest.sol --contract TestPaymentSplitter --config echidnaconfig.yaml
```


### Bindings

Foundry generates bindings for solidity contracts that allow for programmatic interactions with the generated contracts through Rust. This comes in handy for writing deployment scripts, gas estimation, etc.

To generate the bindings we use the `forge bind` commmand and select desired contracts as follows:

```bash
forge bind  --select "(?:^|\W)PayoutFactoryNativeAddr|PaymentSplitterNativeAddr(?:$|\W)" --crate-name contract-bindings -b ./cli/bindings
```

## Cli

To use the bindings as scripts to deploy and interact with contracts first create a `./secrets/secret` file within `./cli` containing your mnemonic string (note this should only be used for testing purposes !).

```bash
cd ./cli
cargo run --bin saturn-contracts -- -S secrets/.secret -U https://api.hyperspace.node.glif.io/rpc/v1 --retries=10 deploy 

```

> **Note:** The `--retries` parameter sets a number of times to poll a pending transaction before considering it as having failed. Because of the differences in block times between Filecoin / Hyperspace and Ethereum, `ethers-rs` can sometimes timeout prematurely _before_ a transaction has truly failed or succeeded (`ethers-rs` has been built with Ethereum in mind). `--retries` has a default value of 10, which empirically we have found to result in successful transactions.

To deploy a new `PaymentSplitter` from a deployed `PayoutFactory` contract:
- Set an env var called `FACTORY_ADDRESS` with the address of the deployed `PayoutFactory`.
- Generate a csv file with the headers `payee,shares` and fill out the rows with pairs of addresses and shares.

For instance: 

```csv
payee,shares
t1ypi542zmmgaltijzw4byonei5c267ev5iif2liy,1
t410f4bmm756u5kft2czgqll4oybvtch3jj5v64yjeya,1
```

Now run:
```bash
cd ./cli
cargo run --bin saturn-contracts -- -S secrets/.secret -U https://api.hyperspace.node.glif.io/rpc/v1 --retries=10 new-payout -F $FACTORY_ADDRESS -P ./secrets/payouts.csv 
```

You can then claim funds for a specific payee using the cli: 
```bash
cd ./cli
cargo run --bin saturn-contracts -- -S secrets/.secret -U https://api.hyperspace.node.glif.io/rpc/v1 --retries=10 claim -F $FACTORY_ADDRESS -A $CLAIM_ADDRESS 
```
