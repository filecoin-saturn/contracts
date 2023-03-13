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
* @dev Creates an instance of `PaymentSplitter` where each account in `payees` is assigned the number of shares at
* the matching position in the `shares` array.
*
* All addresses in `payees` must be non-zero. Both arrays must have the same non-zero length, and there must be no
* duplicates in `payees`.
*/
initialize(address[] memory payees, uint256[] memory shares_) payable {...}

```

The release function can then be triggered to release payments owed to a specific account.

```solidity
/**
* @dev Triggers a transfer to `account` of the amount of Ether they are owed, according to their percentage of the
* total shares and their previous withdrawals.
*/
function release(address payable account) public virtual {...}

```


The second contract is a payout factory contract. Its core functionality, which is to instantiate and fund a new payment splitter contract, can only be invoked by the admin user of the contract.

```solidity
/**
* @dev Spins up a new payment splitter.
*/
function payout(address[] memory payees, uint256[] memory shares_)
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
function releasable(address account)
	external
	view
	returns (uint256 totalValue) {...}

/**
* @dev Releases all available funds in previously generated payout contracts.
* @param account The address of the payee.
*/
function releaseAll(address account) external {...}

/**
* @dev Releases all available funds in a single previously generated payout contract.
* @param account The address of the payee.
* @param index Index of the payout contract.
*/
function _releasePayout(address account, uint256 index) private {...}
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

### Deployment

> TODO: Implement a deployment script for the evaluator contract.


To deploy a Factory contract to the hyperspace testnet you need to create a list of payees in a `.payees`. This is a new line delineated list of address we want to send payouts to. For instance:
```bash
0x1FE76102978Dbb20566BA59E29e6256715a7de4d
0xd3043090211ebd3dabc74d0eb9cc468e1dfbb700
...
```

Conversely `.shares` represents the payments owed (in attoFil and in order) for each of those addresses:
```bash
10
12
...
```

Finally you need a `.secret` file which includes the list of mnemonic words for a throwaway testnet account that has been pre-filled with [test FIL](https://hyperspace.yoga/#faucet). 🚨🚨 Note that this should not be an address that is of import to you, this is solely for testing purposes. 🚨🚨

You need to set an environment variable for the Filecoin testnet we want to use. This can be done via an environment variable or a `.env` file that is then sourced. For instance:
```bash
HYPERSPACE_RPC_URL="https://api.hyperspace.node.glif.io/rpc/v0"
```

To deploy the factory contract with a local instance of the EVM, and pre-fill it with ETH you can run the deployment script.
```bash
forge script script/Deploy.sol:FactoryDeployScript --broadcast --verify
```

To deploy it to the fEVM and prefill it with test FIL you can run it with the environment variable set previously. You also want to increase the gas estimate multiplier (TODO: finetune this value), and allow for many retries to fetch receipts from the RPC endpoint. You need to ensure that the address you provided the secret for previously has sufficient test FIL to payout _all the owed shares_ defined in `.shares`.
```bash
forge script script/Deploy.sol:FactoryDeployScript --broadcast --verify --rpc-url ${HYPERSPACE_RPC_URL} --gas-estimate-multiplier 10000 --slow
```


You can then spin out a new payment splitter contract. First set `FACTORY_ADDRESS` to the deployed factory contract's ETH address. Then run:
```bash
forge script script/Payout.sol:PaymentSplitterScript --broadcast --verify --rpc-url ${HYPERSPACE_RPC_URL} --gas-estimate-multiplier 10000 --slow
```

This command will return the testnet address, which you can check out on an [explorer](https://hyperspace.filfox.info/en). If you want to interact with the contract we recommend using `cast` (installed with forge).

```bash
cast send ${CONTRACT_ADDRESS} "release(address)" ${ADDRESS_TO_PAY}  --rpc-url ${HYPERSPACE_RPC_URL} --private-key=${HYPERSPACE_PRIVKEY}
```


### Bindings

Foundry generates bindings for solidity contracts that allow for programmatic interactions with the generated contracts through Rust. This comes in handy for writing deployment scripts, gas estimation, etc.

To generate the bindings we use the `forge bind` commmand and select desired contracts as follows:

```bash
forge bind  --select "(?:^|\W)PayoutFactory|PaymentSplitter(?:$|\W)" --crate-name contract-bindings -b ./cli/bindings
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

Run:
```bash
cd ./cli
cargo run --bin saturn-contracts -- -S secrets/.secret -U https://api.hyperspace.node.glif.io/rpc/v1 --retries=10 new-payout -F $FACTORY_ADDRESS -P ./secrets/payouts.csv 
```

You can then claim funds for a specific payee using the cli: 
```bash
cd ./cli
cargo run --bin saturn-contracts -- -S secrets/.secret -U https://api.hyperspace.node.glif.io/rpc/v1 --retries=10 claim -F $FACTORY_ADDRESS -A $CLAIM_ADDRESS 
```

## Hardhat Integration

This Foundry project has been integrated with Hardhat using the
`hardhat-foundry` plugin available at https://hardhat.org/hardhat-runner/docs/advanced/hardhat-and-foundry for those who prefer developing using
Hardhat rather than Foundry. This integration enables using Hardhat
on top of the Foundry project structure.

Note that dependencies still need to be installed using forge(`forge install`) as we want to manage dependencies as git modules rather than npm modules.

Please use the following steps to start developing with Hardhat after you have installed the project dependencies using `forge install`:
1. Install all hardhat deps and packages using `npm i`.
2. Compile contracts using `npx hardhat compile`.
3. Add tests in Javascript/Typescript to the `test` directory and run them using `npx hardhat test`.
4. There's a **sample** `hardhat_deploy.js` script in the `script/` dir that can be used to deploy the `Evaluator` contract to a network of your choosing. To deploy it to the `Goerli` network, please rename the `env.example` file to `.env`, fill in the required varaiables, uncomment the `goerli` network config in `hardhat.config.js` and run `npx hardhat run script/hardhat_deploy.js --network goerli`.
5. Add more tasks/config etc to the `hardhat.config.js` file.

For more, see the [Hardhat Docs](https://hardhat.org/hardhat-runner/docs/getting-started#overview).