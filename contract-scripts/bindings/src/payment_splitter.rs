pub use payment_splitter::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod payment_splitter {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    ///PaymentSplitter was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
    use std::sync::Arc;
    use ::ethers::core::{
        abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
        types::*,
    };
    use ::ethers::contract::{
        Contract, builders::{ContractCall, Event},
        Lazy,
    };
    use ::ethers::providers::Middleware;
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"Initialized\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"shares\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PayeeAdded\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PaymentReceived\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PaymentReleased\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"payees_\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"shares_\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"initialize\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"payees\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"releasable\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"release\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"released\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"shares\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalReleased\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalShares\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"receive\",\"outputs\":[]}]";
    /// The parsed JSON-ABI of the contract.
    pub static PAYMENTSPLITTER_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi"));
    /// Bytecode of the #name contract
    pub static PAYMENTSPLITTER_BYTECODE: ::ethers::contract::Lazy<
        ::ethers::core::types::Bytes,
    > = ::ethers::contract::Lazy::new(|| {
        "0x608060405234801561001057600080fd5b50610f01806100206000396000f3fe60806040526004361061007f5760003560e01c80639852595c1161004e5780639852595c1461013e578063a3f8eace14610181578063ce7c2ac2146101a1578063e33b7de3146101e457600080fd5b806319165587146100c35780633a98ef39146100e55780634e8086aa146101095780637fbbe46f1461012b57600080fd5b366100be57604080513381523460208201527f6ef95f06320e7a25a04a175ca677b7052bdd97131872c2192525a629f51be770910160405180910390a1005b600080fd5b3480156100cf57600080fd5b506100e36100de366004610b4f565b6101f9565b005b3480156100f157600080fd5b506001545b6040519081526020015b60405180910390f35b34801561011557600080fd5b5061011e61048a565b6040516101009190610b6a565b6100e3610139366004610cd1565b6104f9565b34801561014a57600080fd5b506100f6610159366004610b4f565b73ffffffffffffffffffffffffffffffffffffffff1660009081526004602052604090205490565b34801561018d57600080fd5b506100f661019c366004610b4f565b6107dc565b3480156101ad57600080fd5b506100f66101bc366004610b4f565b73ffffffffffffffffffffffffffffffffffffffff1660009081526003602052604090205490565b3480156101f057600080fd5b506002546100f6565b73ffffffffffffffffffffffffffffffffffffffff81166000908152600360205260409020546102b0576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602660248201527f5061796d656e7453706c69747465723a206163636f756e7420686173206e6f2060448201527f736861726573000000000000000000000000000000000000000000000000000060648201526084015b60405180910390fd5b60006102bb826107dc565b90508060000361034d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602b60248201527f5061796d656e7453706c69747465723a206163636f756e74206973206e6f742060448201527f647565207061796d656e7400000000000000000000000000000000000000000060648201526084016102a7565b806002600082825461035f9190610dc0565b909155505073ffffffffffffffffffffffffffffffffffffffff8216600081815260046020908152604091829020805485019055815192835282018390527fdf20fd1e76bc69d672e4814fafb2c449bba3a5369d8359adf9e05e6fde87b056910160405180910390a160405173ffffffffffffffffffffffffffffffffffffffff83169082156108fc029083906000818181858888f19350505050610486576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602360248201527f5061796d656e7453706c69747465723a204661696c656420746f2073656e642060448201527f46494c000000000000000000000000000000000000000000000000000000000060648201526084016102a7565b5050565b606060058054806020026020016040519081016040528092919081815260200182805480156104ef57602002820191906000526020600020905b815473ffffffffffffffffffffffffffffffffffffffff1681526001909101906020018083116104c4575b5050505050905090565b600054610100900460ff16158080156105195750600054600160ff909116105b806105335750303b158015610533575060005460ff166001145b6105bf576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602e60248201527f496e697469616c697a61626c653a20636f6e747261637420697320616c72656160448201527f647920696e697469616c697a656400000000000000000000000000000000000060648201526084016102a7565b600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00166001179055801561061d57600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00ff166101001790555b81518351146106ae576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603260248201527f5061796d656e7453706c69747465723a2070617965657320616e64207368617260448201527f6573206c656e677468206d69736d61746368000000000000000000000000000060648201526084016102a7565b6000835111610719576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f5061796d656e7453706c69747465723a206e6f2070617965657300000000000060448201526064016102a7565b60005b83518110156107735761076184828151811061073a5761073a610dd9565b602002602001015184838151811061075457610754610dd9565b6020026020010151610831565b8061076b81610e08565b91505061071c565b5080156107d757600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00ff169055604051600181527f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb38474024989060200160405180910390a15b505050565b6000806107e860025490565b6107f29047610dc0565b905061082a83826108258673ffffffffffffffffffffffffffffffffffffffff1660009081526004602052604090205490565b610ad3565b9392505050565b73ffffffffffffffffffffffffffffffffffffffff82166108d4576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602c60248201527f5061796d656e7453706c69747465723a206163636f756e74206973207468652060448201527f7a65726f2061646472657373000000000000000000000000000000000000000060648201526084016102a7565b6000811161093e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601d60248201527f5061796d656e7453706c69747465723a2073686172657320617265203000000060448201526064016102a7565b73ffffffffffffffffffffffffffffffffffffffff8216600090815260036020526040902054156109f1576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602b60248201527f5061796d656e7453706c69747465723a206163636f756e7420616c726561647960448201527f206861732073686172657300000000000000000000000000000000000000000060648201526084016102a7565b6005805460018082019092557f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db00180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff8516908117909155600090815260036020526040902082905554610a7d908290610dc0565b6001556040805173ffffffffffffffffffffffffffffffffffffffff84168152602081018390527f40c340f65e17194d14ddddb073d3c9f888e3cb52b5aae0c6c7706b4fbc905fac910160405180910390a15050565b60015473ffffffffffffffffffffffffffffffffffffffff841660009081526003602052604081205490918391610b0a9086610e40565b610b149190610e7d565b610b1e9190610eb8565b949350505050565b803573ffffffffffffffffffffffffffffffffffffffff81168114610b4a57600080fd5b919050565b600060208284031215610b6157600080fd5b61082a82610b26565b6020808252825182820181905260009190848201906040850190845b81811015610bb857835173ffffffffffffffffffffffffffffffffffffffff1683529284019291840191600101610b86565b50909695505050505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b604051601f82017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe016810167ffffffffffffffff81118282101715610c3a57610c3a610bc4565b604052919050565b600067ffffffffffffffff821115610c5c57610c5c610bc4565b5060051b60200190565b600082601f830112610c7757600080fd5b81356020610c8c610c8783610c42565b610bf3565b82815260059290921b84018101918181019086841115610cab57600080fd5b8286015b84811015610cc65780358352918301918301610caf565b509695505050505050565b60008060408385031215610ce457600080fd5b823567ffffffffffffffff80821115610cfc57600080fd5b818501915085601f830112610d1057600080fd5b81356020610d20610c8783610c42565b82815260059290921b84018101918181019089841115610d3f57600080fd5b948201945b83861015610d6457610d5586610b26565b82529482019490820190610d44565b96505086013592505080821115610d7a57600080fd5b50610d8785828601610c66565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b80820180821115610dd357610dd3610d91565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203610e3957610e39610d91565b5060010190565b6000817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0483118215151615610e7857610e78610d91565b500290565b600082610eb3577f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b500490565b81810381811115610dd357610dd3610d9156fea2646970667358221220b8fe0357dbe6558925bd78fccca5b6b2a6f6e429597778291e9fde4512f8781b64736f6c63430008100033"
            .parse()
            .expect("invalid bytecode")
    });
    pub struct PaymentSplitter<M>(::ethers::contract::Contract<M>);
    impl<M> Clone for PaymentSplitter<M> {
        fn clone(&self) -> Self {
            PaymentSplitter(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PaymentSplitter<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PaymentSplitter<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PaymentSplitter)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> PaymentSplitter<M> {
        /// Creates a new contract instance with the specified `ethers`
        /// client at the given `Address`. The contract derefs to a `ethers::Contract`
        /// object
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    PAYMENTSPLITTER_ABI.clone(),
                    client,
                ),
            )
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// 1. If there are no constructor arguments, you should pass `()` as the argument.
        /// 1. The default poll duration is 7 seconds.
        /// 1. The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter,"../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::std::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                PAYMENTSPLITTER_ABI.clone(),
                PAYMENTSPLITTER_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `initialize` (0x7fbbe46f) function
        pub fn initialize(
            &self,
            payees: ::std::vec::Vec<::ethers::core::types::Address>,
            shares: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([127, 187, 228, 111], (payees, shares))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `payees` (0x4e8086aa) function
        pub fn payees(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::Address>,
        > {
            self.0
                .method_hash([78, 128, 134, 170], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `releasable` (0xa3f8eace) function
        pub fn releasable(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([163, 248, 234, 206], account)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `release` (0x19165587) function
        pub fn release(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([25, 22, 85, 135], account)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `released` (0x9852595c) function
        pub fn released(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([152, 82, 89, 92], account)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `shares` (0xce7c2ac2) function
        pub fn shares(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([206, 124, 42, 194], account)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalReleased` (0xe33b7de3) function
        pub fn total_released(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([227, 59, 125, 227], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalShares` (0x3a98ef39) function
        pub fn total_shares(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([58, 152, 239, 57], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `Initialized` event
        pub fn initialized_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, InitializedFilter> {
            self.0.event()
        }
        ///Gets the contract's `PayeeAdded` event
        pub fn payee_added_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, PayeeAddedFilter> {
            self.0.event()
        }
        ///Gets the contract's `PaymentReceived` event
        pub fn payment_received_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, PaymentReceivedFilter> {
            self.0.event()
        }
        ///Gets the contract's `PaymentReleased` event
        pub fn payment_released_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, PaymentReleasedFilter> {
            self.0.event()
        }
        /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<M, PaymentSplitterEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for PaymentSplitter<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "Initialized", abi = "Initialized(uint8)")]
    pub struct InitializedFilter {
        pub version: u8,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "PayeeAdded", abi = "PayeeAdded(address,uint256)")]
    pub struct PayeeAddedFilter {
        pub account: ::ethers::core::types::Address,
        pub shares: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "PaymentReceived", abi = "PaymentReceived(address,uint256)")]
    pub struct PaymentReceivedFilter {
        pub from: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(name = "PaymentReleased", abi = "PaymentReleased(address,uint256)")]
    pub struct PaymentReleasedFilter {
        pub to: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ::ethers::contract::EthAbiType)]
    pub enum PaymentSplitterEvents {
        InitializedFilter(InitializedFilter),
        PayeeAddedFilter(PayeeAddedFilter),
        PaymentReceivedFilter(PaymentReceivedFilter),
        PaymentReleasedFilter(PaymentReleasedFilter),
    }
    impl ::ethers::contract::EthLogDecode for PaymentSplitterEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(PaymentSplitterEvents::InitializedFilter(decoded));
            }
            if let Ok(decoded) = PayeeAddedFilter::decode_log(log) {
                return Ok(PaymentSplitterEvents::PayeeAddedFilter(decoded));
            }
            if let Ok(decoded) = PaymentReceivedFilter::decode_log(log) {
                return Ok(PaymentSplitterEvents::PaymentReceivedFilter(decoded));
            }
            if let Ok(decoded) = PaymentReleasedFilter::decode_log(log) {
                return Ok(PaymentSplitterEvents::PaymentReleasedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for PaymentSplitterEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PaymentSplitterEvents::InitializedFilter(element) => element.fmt(f),
                PaymentSplitterEvents::PayeeAddedFilter(element) => element.fmt(f),
                PaymentSplitterEvents::PaymentReceivedFilter(element) => element.fmt(f),
                PaymentSplitterEvents::PaymentReleasedFilter(element) => element.fmt(f),
            }
        }
    }
    ///Container type for all input parameters for the `initialize` function with signature `initialize(address[],uint256[])` and selector `0x7fbbe46f`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "initialize", abi = "initialize(address[],uint256[])")]
    pub struct InitializeCall {
        pub payees: ::std::vec::Vec<::ethers::core::types::Address>,
        pub shares: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `payees` function with signature `payees()` and selector `0x4e8086aa`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "payees", abi = "payees()")]
    pub struct PayeesCall;
    ///Container type for all input parameters for the `releasable` function with signature `releasable(address)` and selector `0xa3f8eace`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "releasable", abi = "releasable(address)")]
    pub struct ReleasableCall {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `release` function with signature `release(address)` and selector `0x19165587`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "release", abi = "release(address)")]
    pub struct ReleaseCall {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `released` function with signature `released(address)` and selector `0x9852595c`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "released", abi = "released(address)")]
    pub struct ReleasedCall {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `shares` function with signature `shares(address)` and selector `0xce7c2ac2`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "shares", abi = "shares(address)")]
    pub struct SharesCall {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `totalReleased` function with signature `totalReleased()` and selector `0xe33b7de3`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "totalReleased", abi = "totalReleased()")]
    pub struct TotalReleasedCall;
    ///Container type for all input parameters for the `totalShares` function with signature `totalShares()` and selector `0x3a98ef39`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "totalShares", abi = "totalShares()")]
    pub struct TotalSharesCall;
    #[derive(Debug, Clone, PartialEq, Eq, ::ethers::contract::EthAbiType)]
    pub enum PaymentSplitterCalls {
        Initialize(InitializeCall),
        Payees(PayeesCall),
        Releasable(ReleasableCall),
        Release(ReleaseCall),
        Released(ReleasedCall),
        Shares(SharesCall),
        TotalReleased(TotalReleasedCall),
        TotalShares(TotalSharesCall),
    }
    impl ::ethers::core::abi::AbiDecode for PaymentSplitterCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::AbiError> {
            if let Ok(decoded)
                = <InitializeCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::Initialize(decoded));
            }
            if let Ok(decoded)
                = <PayeesCall as ::ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PaymentSplitterCalls::Payees(decoded));
            }
            if let Ok(decoded)
                = <ReleasableCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::Releasable(decoded));
            }
            if let Ok(decoded)
                = <ReleaseCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::Release(decoded));
            }
            if let Ok(decoded)
                = <ReleasedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::Released(decoded));
            }
            if let Ok(decoded)
                = <SharesCall as ::ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PaymentSplitterCalls::Shares(decoded));
            }
            if let Ok(decoded)
                = <TotalReleasedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::TotalReleased(decoded));
            }
            if let Ok(decoded)
                = <TotalSharesCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PaymentSplitterCalls::TotalShares(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PaymentSplitterCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                PaymentSplitterCalls::Initialize(element) => element.encode(),
                PaymentSplitterCalls::Payees(element) => element.encode(),
                PaymentSplitterCalls::Releasable(element) => element.encode(),
                PaymentSplitterCalls::Release(element) => element.encode(),
                PaymentSplitterCalls::Released(element) => element.encode(),
                PaymentSplitterCalls::Shares(element) => element.encode(),
                PaymentSplitterCalls::TotalReleased(element) => element.encode(),
                PaymentSplitterCalls::TotalShares(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PaymentSplitterCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PaymentSplitterCalls::Initialize(element) => element.fmt(f),
                PaymentSplitterCalls::Payees(element) => element.fmt(f),
                PaymentSplitterCalls::Releasable(element) => element.fmt(f),
                PaymentSplitterCalls::Release(element) => element.fmt(f),
                PaymentSplitterCalls::Released(element) => element.fmt(f),
                PaymentSplitterCalls::Shares(element) => element.fmt(f),
                PaymentSplitterCalls::TotalReleased(element) => element.fmt(f),
                PaymentSplitterCalls::TotalShares(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<InitializeCall> for PaymentSplitterCalls {
        fn from(var: InitializeCall) -> Self {
            PaymentSplitterCalls::Initialize(var)
        }
    }
    impl ::std::convert::From<PayeesCall> for PaymentSplitterCalls {
        fn from(var: PayeesCall) -> Self {
            PaymentSplitterCalls::Payees(var)
        }
    }
    impl ::std::convert::From<ReleasableCall> for PaymentSplitterCalls {
        fn from(var: ReleasableCall) -> Self {
            PaymentSplitterCalls::Releasable(var)
        }
    }
    impl ::std::convert::From<ReleaseCall> for PaymentSplitterCalls {
        fn from(var: ReleaseCall) -> Self {
            PaymentSplitterCalls::Release(var)
        }
    }
    impl ::std::convert::From<ReleasedCall> for PaymentSplitterCalls {
        fn from(var: ReleasedCall) -> Self {
            PaymentSplitterCalls::Released(var)
        }
    }
    impl ::std::convert::From<SharesCall> for PaymentSplitterCalls {
        fn from(var: SharesCall) -> Self {
            PaymentSplitterCalls::Shares(var)
        }
    }
    impl ::std::convert::From<TotalReleasedCall> for PaymentSplitterCalls {
        fn from(var: TotalReleasedCall) -> Self {
            PaymentSplitterCalls::TotalReleased(var)
        }
    }
    impl ::std::convert::From<TotalSharesCall> for PaymentSplitterCalls {
        fn from(var: TotalSharesCall) -> Self {
            PaymentSplitterCalls::TotalShares(var)
        }
    }
    ///Container type for all return fields from the `payees` function with signature `payees()` and selector `0x4e8086aa`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct PayeesReturn(pub ::std::vec::Vec<::ethers::core::types::Address>);
    ///Container type for all return fields from the `releasable` function with signature `releasable(address)` and selector `0xa3f8eace`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct ReleasableReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `released` function with signature `released(address)` and selector `0x9852595c`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct ReleasedReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `shares` function with signature `shares(address)` and selector `0xce7c2ac2`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct SharesReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `totalReleased` function with signature `totalReleased()` and selector `0xe33b7de3`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct TotalReleasedReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `totalShares` function with signature `totalShares()` and selector `0x3a98ef39`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct TotalSharesReturn(pub ::ethers::core::types::U256);
}
