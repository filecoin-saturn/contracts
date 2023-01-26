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
constructor(address[] memory payees, uint256[] memory shares_) payable {...}

```

The release function can then be triggered to release payments owed to a specific account. 

```solidity 
/**
* @dev Triggers a transfer to `account` of the amount of Ether they are owed, according to their percentage of the
* total shares and their previous withdrawals.
*/
function release(address payable account) public virtual {...}

```


The second contract is an evaluation factory contract, which can incrementally keep a record ofhow much payees are owed over an epoch. The core functionality of this contract such as rewarding a payee more shares or spinning up a new payment splitter contract can only be called by an admin user of the contract, defined when constructing it. 

```solidity 
 /**
* @dev Reward a payee.
* @param account The address of the payee.
* @param shares_ The number of shares owned by the payee.
*/
function rewardPayee(address account, uint256 shares_)
	external
	onlyRole(DEFAULT_ADMIN_ROLE) {...}
```

The payout function sets the mapping of payees to rewards in stone by spinning up a new `PaymentSplitter` contract which holds this map. As  `PaymentSplitter` cannot be updated after instantiation we effectively fix the rewards for that time period permanently. On the `EvaluationFactory` contract the variable and mapping which kept a running log of which payee is owed what are reset. The method returns the address of the newly generated `PaymentSplitter`.

```solidity 
/*
* @dev Spins up a new payment splitter using the contract's _shares and _payees variables. By the end of process
* both variables will be reset as they are set in stone by the new PaymentSplitter instance.
*/
function payout()
	external
	onlyRole(DEFAULT_ADMIN_ROLE)
	returns (address instance) {...}
```
    

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
### Deployment

> TODO: create a deployment script for the factory contract

#### PaymentSplitter

To deploy the contracts to the hyperspace testnet you need to create a list of payees in a `.payees`. This is a new line delineated list of address we want to send payouts to. For instance: 
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

To deploy the contract with a local instance of the EVM, and pre-fill it with ETH you can run the deployment script. 
```
forge script script/Deploy.sol:PaymentSplitterScript --broadcast --verify 

```

To deploy it to the fEVM and prefill it with test FIL you can run it with the environment variable set previously. You also want to increase the gas estimate multiplier (TODO: finetune this value), and allow for many retries to fetch receipts from the RPC endpoint. You need to ensure that the address you provided the secret for previously has sufficient test FIL to payout _all the owed shares_ defined in `.shares`. 
```
forge script script/Deploy.sol:PaymentSplitterScript --broadcast --verify --rpc-url ${HYPERSPACE_RPC_URL} --gas-estimate-multiplier 10000 --retries 10 --delay 30 --slow

```

This command will return the testnet address, which you can check out on an [explorer](https://hyperspace.filfox.info/en). If you want to interact with the contract we recommend using `cast` (installed with forge). 

```bash
cast send ${CONTRACT_ADDRESS} "release(address)" ${ADDRESS_TO_PAY}  --rpc-url ${HYPERSPACE_RPC_URL} --private-key=${HYPERSPACE_PRIVKEY} 
```
