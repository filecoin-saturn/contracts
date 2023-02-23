pub use payout_factory::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod payout_factory {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    ///PayoutFactory was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs
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
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"admin\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PaymentReceived\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"PaymentReleased\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"previousAdminRole\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"bytes32\",\"name\":\"newAdminRole\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RoleAdminChanged\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RoleGranted\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"RoleRevoked\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newSplitter\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SplitterCreated\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"DEFAULT_ADMIN_ROLE\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getRoleAdmin\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"grantRole\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"hasRole\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"payees\",\"type\":\"address[]\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"shares_\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"payout\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"instance\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"payouts\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"releasable\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"totalValue\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"releaseAll\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"released\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"totalValue\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceRole\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"role\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"revokeRole\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"shares\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"totalValue\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"interfaceId\",\"type\":\"bytes4\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"supportsInterface\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"template\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalReleased\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"totalValue\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalShares\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"totalValue\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"receive\",\"outputs\":[]}]";
    /// The parsed JSON-ABI of the contract.
    pub static PAYOUTFACTORY_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi"));
    /// Bytecode of the #name contract
    pub static PAYOUTFACTORY_BYTECODE: ::ethers::contract::Lazy<
        ::ethers::core::types::Bytes,
    > = ::ethers::contract::Lazy::new(|| {
        "0x60a0604052604051620000129062000121565b604051809103906000f0801580156200002f573d6000803e3d6000fd5b506001600160a01b03166080523480156200004957600080fd5b5060405162002aa738038062002aa78339810160408190526200006c916200012f565b6200007960008262000080565b5062000161565b6000828152602081815260408083206001600160a01b038516845290915290205460ff166200011d576000828152602081815260408083206001600160a01b03851684529091529020805460ff19166001179055620000dc3390565b6001600160a01b0316816001600160a01b0316837f2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d60405160405180910390a45b5050565b610f218062001b8683390190565b6000602082840312156200014257600080fd5b81516001600160a01b03811681146200015a57600080fd5b9392505050565b608051611a02620001846000396000818161023701526108960152611a026000f3fe6080604052600436106100f75760003560e01c80639852595c1161008a578063c176e63911610059578063c176e63914610346578063ce7c2ac214610366578063d547741f14610386578063e33b7de3146103a657600080fd5b80639852595c146102cf578063a217fddf146102ef578063a247bc3214610304578063a3f8eace1461032657600080fd5b80633a98ef39116100c65780633a98ef39146101f0578063580fc80a146102055780636f2ddd931461022557806391d148541461027e57600080fd5b806301ffc9a71461013b578063248a9ca3146101705780632f2ff15d146101ae57806336568abe146101d057600080fd5b3661013657604080513381523460208201527f6ef95f06320e7a25a04a175ca677b7052bdd97131872c2192525a629f51be770910160405180910390a1005b600080fd5b34801561014757600080fd5b5061015b610156366004611454565b6103bb565b60405190151581526020015b60405180910390f35b34801561017c57600080fd5b506101a061018b366004611496565b60009081526020819052604090206001015490565b604051908152602001610167565b3480156101ba57600080fd5b506101ce6101c93660046114d3565b610454565b005b3480156101dc57600080fd5b506101ce6101eb3660046114d3565b61047e565b3480156101fc57600080fd5b506101a0610536565b34801561021157600080fd5b506101ce6102203660046114ff565b61061b565b34801561023157600080fd5b506102597f000000000000000000000000000000000000000000000000000000000000000081565b60405173ffffffffffffffffffffffffffffffffffffffff9091168152602001610167565b34801561028a57600080fd5b5061015b6102993660046114d3565b60009182526020828152604080842073ffffffffffffffffffffffffffffffffffffffff93909316845291905290205460ff1690565b3480156102db57600080fd5b506101a06102ea3660046114ff565b610645565b3480156102fb57600080fd5b506101a0600081565b34801561031057600080fd5b50610319610731565b604051610167919061156b565b34801561033257600080fd5b506101a06103413660046114ff565b6107a0565b34801561035257600080fd5b5061025961036136600461168b565b610885565b34801561037257600080fd5b506101a06103813660046114ff565b610b3a565b34801561039257600080fd5b506101ce6103a13660046114d3565b610c1f565b3480156103b257600080fd5b506101a0610c44565b60007fffffffff0000000000000000000000000000000000000000000000000000000082167f7965db0b00000000000000000000000000000000000000000000000000000000148061044e57507f01ffc9a7000000000000000000000000000000000000000000000000000000007fffffffff000000000000000000000000000000000000000000000000000000008316145b92915050565b60008281526020819052604090206001015461046f81610d24565b6104798383610d31565b505050565b73ffffffffffffffffffffffffffffffffffffffff81163314610528576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602f60248201527f416363657373436f6e74726f6c3a2063616e206f6e6c792072656e6f756e636560448201527f20726f6c657320666f722073656c66000000000000000000000000000000000060648201526084015b60405180910390fd5b6105328282610e21565b5050565b600154600090815b818110156106165760006001828154811061055b5761055b61174b565b60009182526020918290200154604080517f3a98ef39000000000000000000000000000000000000000000000000000000008152905173ffffffffffffffffffffffffffffffffffffffff90921693508392633a98ef39926004808401938290030181865afa1580156105d2573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906105f6919061177a565b61060090856117c2565b935050808061060e906117d5565b91505061053e565b505090565b60015460005b81811015610479576106338382610ed8565b8061063d816117d5565b915050610621565b600154600090815b8181101561072a5760006001828154811061066a5761066a61174b565b6000918252602090912001546040517f9852595c00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff878116600483015290911691508190639852595c90602401602060405180830381865afa1580156106e6573d6000803e3d6000fd5b505050506040513d601f19601f8201168201806040525081019061070a919061177a565b61071490856117c2565b9350508080610722906117d5565b91505061064d565b5050919050565b6060600180548060200260200160405190810160405280929190818152602001828054801561079657602002820191906000526020600020905b815473ffffffffffffffffffffffffffffffffffffffff16815260019091019060200180831161076b575b5050505050905090565b600154600090815b8181101561072a576000600182815481106107c5576107c561174b565b6000918252602090912001546040517fa3f8eace00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff87811660048301529091169150819063a3f8eace90602401602060405180830381865afa158015610841573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610865919061177a565b61086f90856117c2565b935050808061087d906117d5565b9150506107a8565b60008061089181610d24565b6108d07f000000000000000000000000000000000000000000000000000000000000000073ffffffffffffffffffffffffffffffffffffffff1661106b565b6001805480820182556000919091527fb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf60180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff83169081179091556040519081529092507f6499ba0dcde9970f83357fecd0782918cffb6a9a97a10957ded519f11907efe59060200160405180910390a16040517f7fbbe46f000000000000000000000000000000000000000000000000000000008152829073ffffffffffffffffffffffffffffffffffffffff821690637fbbe46f906109ce908890889060040161180d565b600060405180830381600087803b1580156109e857600080fd5b505af11580156109fc573d6000803e3d6000fd5b5050505060008373ffffffffffffffffffffffffffffffffffffffff166108fc8373ffffffffffffffffffffffffffffffffffffffff16633a98ef396040518163ffffffff1660e01b8152600401602060405180830381865afa158015610a67573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610a8b919061177a565b6040518115909202916000818181858888f19350505050905080610b31576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602160248201527f5061796f7574466163746f72793a204661696c656420746f2073656e6420464960448201527f4c00000000000000000000000000000000000000000000000000000000000000606482015260840161051f565b50505092915050565b600154600090815b8181101561072a57600060018281548110610b5f57610b5f61174b565b6000918252602090912001546040517fce7c2ac200000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff87811660048301529091169150819063ce7c2ac290602401602060405180830381865afa158015610bdb573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610bff919061177a565b610c0990856117c2565b9350508080610c17906117d5565b915050610b42565b600082815260208190526040902060010154610c3a81610d24565b6104798383610e21565b600154600090815b8181101561061657600060018281548110610c6957610c6961174b565b60009182526020918290200154604080517fe33b7de3000000000000000000000000000000000000000000000000000000008152905173ffffffffffffffffffffffffffffffffffffffff9092169350839263e33b7de3926004808401938290030181865afa158015610ce0573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610d04919061177a565b610d0e90856117c2565b9350508080610d1c906117d5565b915050610c4c565b610d2e8133611133565b50565b60008281526020818152604080832073ffffffffffffffffffffffffffffffffffffffff8516845290915290205460ff166105325760008281526020818152604080832073ffffffffffffffffffffffffffffffffffffffff85168452909152902080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00166001179055610dc33390565b73ffffffffffffffffffffffffffffffffffffffff168173ffffffffffffffffffffffffffffffffffffffff16837f2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d60405160405180910390a45050565b60008281526020818152604080832073ffffffffffffffffffffffffffffffffffffffff8516845290915290205460ff16156105325760008281526020818152604080832073ffffffffffffffffffffffffffffffffffffffff8516808552925280832080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0016905551339285917ff6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b9190a45050565b600060018281548110610eed57610eed61174b565b60009182526020822001546040517fa3f8eace00000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff86811660048301529091169250829063a3f8eace90602401602060405180830381865afa158015610f68573d6000803e3d6000fd5b505050506040513d601f19601f82011682018060405250810190610f8c919061177a565b90508015611065576040805173ffffffffffffffffffffffffffffffffffffffff86168152602081018390527fdf20fd1e76bc69d672e4814fafb2c449bba3a5369d8359adf9e05e6fde87b056910160405180910390a16040517f1916558700000000000000000000000000000000000000000000000000000000815273ffffffffffffffffffffffffffffffffffffffff8581166004830152831690631916558790602401600060405180830381600087803b15801561104c57600080fd5b505af1158015611060573d6000803e3d6000fd5b505050505b50505050565b6000763d602d80600a3d3981f3363d3d373d3d3d363d730000008260601b60e81c176000526e5af43d82803e903d91602b57fd5bf38260781b17602052603760096000f0905073ffffffffffffffffffffffffffffffffffffffff811661112e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601660248201527f455243313136373a20637265617465206661696c656400000000000000000000604482015260640161051f565b919050565b60008281526020818152604080832073ffffffffffffffffffffffffffffffffffffffff8516845290915290205460ff1661053257611171816111eb565b61117c83602061120a565b60405160200161118d929190611888565b604080517fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0818403018152908290527f08c379a000000000000000000000000000000000000000000000000000000000825261051f91600401611909565b606061044e73ffffffffffffffffffffffffffffffffffffffff831660145b6060600061121983600261195a565b6112249060026117c2565b67ffffffffffffffff81111561123c5761123c61157e565b6040519080825280601f01601f191660200182016040528015611266576020820181803683370190505b5090507f30000000000000000000000000000000000000000000000000000000000000008160008151811061129d5761129d61174b565b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a9053507f7800000000000000000000000000000000000000000000000000000000000000816001815181106113005761130061174b565b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a905350600061133c84600261195a565b6113479060016117c2565b90505b60018111156113e4577f303132333435363738396162636465660000000000000000000000000000000085600f16601081106113885761138861174b565b1a60f81b82828151811061139e5761139e61174b565b60200101907effffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff1916908160001a90535060049490941c936113dd81611997565b905061134a565b50831561144d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820181905260248201527f537472696e67733a20686578206c656e67746820696e73756666696369656e74604482015260640161051f565b9392505050565b60006020828403121561146657600080fd5b81357fffffffff000000000000000000000000000000000000000000000000000000008116811461144d57600080fd5b6000602082840312156114a857600080fd5b5035919050565b803573ffffffffffffffffffffffffffffffffffffffff8116811461112e57600080fd5b600080604083850312156114e657600080fd5b823591506114f6602084016114af565b90509250929050565b60006020828403121561151157600080fd5b61144d826114af565b600081518084526020808501945080840160005b8381101561156057815173ffffffffffffffffffffffffffffffffffffffff168752958201959082019060010161152e565b509495945050505050565b60208152600061144d602083018461151a565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b604051601f82017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe016810167ffffffffffffffff811182821017156115f4576115f461157e565b604052919050565b600067ffffffffffffffff8211156116165761161661157e565b5060051b60200190565b600082601f83011261163157600080fd5b81356020611646611641836115fc565b6115ad565b82815260059290921b8401810191818101908684111561166557600080fd5b8286015b848110156116805780358352918301918301611669565b509695505050505050565b6000806040838503121561169e57600080fd5b823567ffffffffffffffff808211156116b657600080fd5b818501915085601f8301126116ca57600080fd5b813560206116da611641836115fc565b82815260059290921b840181019181810190898411156116f957600080fd5b948201945b8386101561171e5761170f866114af565b825294820194908201906116fe565b9650508601359250508082111561173457600080fd5b5061174185828601611620565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60006020828403121561178c57600080fd5b5051919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b8082018082111561044e5761044e611793565b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff820361180657611806611793565b5060010190565b604081526000611820604083018561151a565b82810360208481019190915284518083528582019282019060005b818110156118575784518352938301939183019160010161183b565b5090979650505050505050565b60005b8381101561187f578181015183820152602001611867565b50506000910152565b7f416363657373436f6e74726f6c3a206163636f756e74200000000000000000008152600083516118c0816017850160208801611864565b7f206973206d697373696e6720726f6c652000000000000000000000000000000060179184019182015283516118fd816028840160208801611864565b01602801949350505050565b6020815260008251806020840152611928816040850160208701611864565b601f017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0169190910160400192915050565b6000817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff048311821515161561199257611992611793565b500290565b6000816119a6576119a6611793565b507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff019056fea2646970667358221220d71f37bc13dddf49535df19a22b3d0d05aaf4aa712077e6030f2057d44090bf864736f6c63430008100033608060405234801561001057600080fd5b50610f01806100206000396000f3fe60806040526004361061007f5760003560e01c80639852595c1161004e5780639852595c1461013e578063a3f8eace14610181578063ce7c2ac2146101a1578063e33b7de3146101e457600080fd5b806319165587146100c35780633a98ef39146100e55780634e8086aa146101095780637fbbe46f1461012b57600080fd5b366100be57604080513381523460208201527f6ef95f06320e7a25a04a175ca677b7052bdd97131872c2192525a629f51be770910160405180910390a1005b600080fd5b3480156100cf57600080fd5b506100e36100de366004610b4f565b6101f9565b005b3480156100f157600080fd5b506001545b6040519081526020015b60405180910390f35b34801561011557600080fd5b5061011e61048a565b6040516101009190610b6a565b6100e3610139366004610cd1565b6104f9565b34801561014a57600080fd5b506100f6610159366004610b4f565b73ffffffffffffffffffffffffffffffffffffffff1660009081526004602052604090205490565b34801561018d57600080fd5b506100f661019c366004610b4f565b6107dc565b3480156101ad57600080fd5b506100f66101bc366004610b4f565b73ffffffffffffffffffffffffffffffffffffffff1660009081526003602052604090205490565b3480156101f057600080fd5b506002546100f6565b73ffffffffffffffffffffffffffffffffffffffff81166000908152600360205260409020546102b0576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602660248201527f5061796d656e7453706c69747465723a206163636f756e7420686173206e6f2060448201527f736861726573000000000000000000000000000000000000000000000000000060648201526084015b60405180910390fd5b60006102bb826107dc565b90508060000361034d576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602b60248201527f5061796d656e7453706c69747465723a206163636f756e74206973206e6f742060448201527f647565207061796d656e7400000000000000000000000000000000000000000060648201526084016102a7565b806002600082825461035f9190610dc0565b909155505073ffffffffffffffffffffffffffffffffffffffff8216600081815260046020908152604091829020805485019055815192835282018390527fdf20fd1e76bc69d672e4814fafb2c449bba3a5369d8359adf9e05e6fde87b056910160405180910390a160405173ffffffffffffffffffffffffffffffffffffffff83169082156108fc029083906000818181858888f19350505050610486576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602360248201527f5061796d656e7453706c69747465723a204661696c656420746f2073656e642060448201527f46494c000000000000000000000000000000000000000000000000000000000060648201526084016102a7565b5050565b606060058054806020026020016040519081016040528092919081815260200182805480156104ef57602002820191906000526020600020905b815473ffffffffffffffffffffffffffffffffffffffff1681526001909101906020018083116104c4575b5050505050905090565b600054610100900460ff16158080156105195750600054600160ff909116105b806105335750303b158015610533575060005460ff166001145b6105bf576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602e60248201527f496e697469616c697a61626c653a20636f6e747261637420697320616c72656160448201527f647920696e697469616c697a656400000000000000000000000000000000000060648201526084016102a7565b600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00166001179055801561061d57600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00ff166101001790555b81518351146106ae576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152603260248201527f5061796d656e7453706c69747465723a2070617965657320616e64207368617260448201527f6573206c656e677468206d69736d61746368000000000000000000000000000060648201526084016102a7565b6000835111610719576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601a60248201527f5061796d656e7453706c69747465723a206e6f2070617965657300000000000060448201526064016102a7565b60005b83518110156107735761076184828151811061073a5761073a610dd9565b602002602001015184838151811061075457610754610dd9565b6020026020010151610831565b8061076b81610e08565b91505061071c565b5080156107d757600080547fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00ff169055604051600181527f7f26b83ff96e1f2b6a682f133852f6798a09c465da95921460cefb38474024989060200160405180910390a15b505050565b6000806107e860025490565b6107f29047610dc0565b905061082a83826108258673ffffffffffffffffffffffffffffffffffffffff1660009081526004602052604090205490565b610ad3565b9392505050565b73ffffffffffffffffffffffffffffffffffffffff82166108d4576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602c60248201527f5061796d656e7453706c69747465723a206163636f756e74206973207468652060448201527f7a65726f2061646472657373000000000000000000000000000000000000000060648201526084016102a7565b6000811161093e576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152601d60248201527f5061796d656e7453706c69747465723a2073686172657320617265203000000060448201526064016102a7565b73ffffffffffffffffffffffffffffffffffffffff8216600090815260036020526040902054156109f1576040517f08c379a000000000000000000000000000000000000000000000000000000000815260206004820152602b60248201527f5061796d656e7453706c69747465723a206163636f756e7420616c726561647960448201527f206861732073686172657300000000000000000000000000000000000000000060648201526084016102a7565b6005805460018082019092557f036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db00180547fffffffffffffffffffffffff00000000000000000000000000000000000000001673ffffffffffffffffffffffffffffffffffffffff8516908117909155600090815260036020526040902082905554610a7d908290610dc0565b6001556040805173ffffffffffffffffffffffffffffffffffffffff84168152602081018390527f40c340f65e17194d14ddddb073d3c9f888e3cb52b5aae0c6c7706b4fbc905fac910160405180910390a15050565b60015473ffffffffffffffffffffffffffffffffffffffff841660009081526003602052604081205490918391610b0a9086610e40565b610b149190610e7d565b610b1e9190610eb8565b949350505050565b803573ffffffffffffffffffffffffffffffffffffffff81168114610b4a57600080fd5b919050565b600060208284031215610b6157600080fd5b61082a82610b26565b6020808252825182820181905260009190848201906040850190845b81811015610bb857835173ffffffffffffffffffffffffffffffffffffffff1683529284019291840191600101610b86565b50909695505050505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b604051601f82017fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe016810167ffffffffffffffff81118282101715610c3a57610c3a610bc4565b604052919050565b600067ffffffffffffffff821115610c5c57610c5c610bc4565b5060051b60200190565b600082601f830112610c7757600080fd5b81356020610c8c610c8783610c42565b610bf3565b82815260059290921b84018101918181019086841115610cab57600080fd5b8286015b84811015610cc65780358352918301918301610caf565b509695505050505050565b60008060408385031215610ce457600080fd5b823567ffffffffffffffff80821115610cfc57600080fd5b818501915085601f830112610d1057600080fd5b81356020610d20610c8783610c42565b82815260059290921b84018101918181019089841115610d3f57600080fd5b948201945b83861015610d6457610d5586610b26565b82529482019490820190610d44565b96505086013592505080821115610d7a57600080fd5b50610d8785828601610c66565b9150509250929050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b80820180821115610dd357610dd3610d91565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052603260045260246000fd5b60007fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8203610e3957610e39610d91565b5060010190565b6000817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0483118215151615610e7857610e78610d91565b500290565b600082610eb3577f4e487b7100000000000000000000000000000000000000000000000000000000600052601260045260246000fd5b500490565b81810381811115610dd357610dd3610d9156fea2646970667358221220b8fe0357dbe6558925bd78fccca5b6b2a6f6e429597778291e9fde4512f8781b64736f6c63430008100033"
            .parse()
            .expect("invalid bytecode")
    });
    pub struct PayoutFactory<M>(::ethers::contract::Contract<M>);
    impl<M> Clone for PayoutFactory<M> {
        fn clone(&self) -> Self {
            PayoutFactory(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for PayoutFactory<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for PayoutFactory<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(PayoutFactory)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> PayoutFactory<M> {
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
                    PAYOUTFACTORY_ABI.clone(),
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
                PAYOUTFACTORY_ABI.clone(),
                PAYOUTFACTORY_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `DEFAULT_ADMIN_ROLE` (0xa217fddf) function
        pub fn default_admin_role(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([162, 23, 253, 223], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRoleAdmin` (0x248a9ca3) function
        pub fn get_role_admin(
            &self,
            role: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([36, 138, 156, 163], role)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `grantRole` (0x2f2ff15d) function
        pub fn grant_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([47, 47, 241, 93], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasRole` (0x91d14854) function
        pub fn has_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([145, 209, 72, 84], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `payout` (0xc176e639) function
        pub fn payout(
            &self,
            payees: ::std::vec::Vec<::ethers::core::types::Address>,
            shares: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([193, 118, 230, 57], (payees, shares))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `payouts` (0xa247bc32) function
        pub fn payouts(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::Address>,
        > {
            self.0
                .method_hash([162, 71, 188, 50], ())
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
        ///Calls the contract's `releaseAll` (0x580fc80a) function
        pub fn release_all(
            &self,
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([88, 15, 200, 10], account)
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
        ///Calls the contract's `renounceRole` (0x36568abe) function
        pub fn renounce_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 86, 138, 190], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `revokeRole` (0xd547741f) function
        pub fn revoke_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([213, 71, 116, 31], (role, account))
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
        ///Calls the contract's `supportsInterface` (0x01ffc9a7) function
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `template` (0x6f2ddd93) function
        pub fn template(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([111, 45, 221, 147], ())
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
        ///Gets the contract's `RoleAdminChanged` event
        pub fn role_admin_changed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, RoleAdminChangedFilter> {
            self.0.event()
        }
        ///Gets the contract's `RoleGranted` event
        pub fn role_granted_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, RoleGrantedFilter> {
            self.0.event()
        }
        ///Gets the contract's `RoleRevoked` event
        pub fn role_revoked_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, RoleRevokedFilter> {
            self.0.event()
        }
        ///Gets the contract's `SplitterCreated` event
        pub fn splitter_created_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<M, SplitterCreatedFilter> {
            self.0.event()
        }
        /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<M, PayoutFactoryEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for PayoutFactory<M> {
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
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethevent(
        name = "RoleAdminChanged",
        abi = "RoleAdminChanged(bytes32,bytes32,bytes32)"
    )]
    pub struct RoleAdminChangedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub previous_admin_role: [u8; 32],
        #[ethevent(indexed)]
        pub new_admin_role: [u8; 32],
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
    #[ethevent(name = "RoleGranted", abi = "RoleGranted(bytes32,address,address)")]
    pub struct RoleGrantedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub account: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub sender: ::ethers::core::types::Address,
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
    #[ethevent(name = "RoleRevoked", abi = "RoleRevoked(bytes32,address,address)")]
    pub struct RoleRevokedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub account: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub sender: ::ethers::core::types::Address,
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
    #[ethevent(name = "SplitterCreated", abi = "SplitterCreated(address)")]
    pub struct SplitterCreatedFilter {
        pub new_splitter: ::ethers::core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ::ethers::contract::EthAbiType)]
    pub enum PayoutFactoryEvents {
        PaymentReceivedFilter(PaymentReceivedFilter),
        PaymentReleasedFilter(PaymentReleasedFilter),
        RoleAdminChangedFilter(RoleAdminChangedFilter),
        RoleGrantedFilter(RoleGrantedFilter),
        RoleRevokedFilter(RoleRevokedFilter),
        SplitterCreatedFilter(SplitterCreatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for PayoutFactoryEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = PaymentReceivedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::PaymentReceivedFilter(decoded));
            }
            if let Ok(decoded) = PaymentReleasedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::PaymentReleasedFilter(decoded));
            }
            if let Ok(decoded) = RoleAdminChangedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::RoleAdminChangedFilter(decoded));
            }
            if let Ok(decoded) = RoleGrantedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::RoleGrantedFilter(decoded));
            }
            if let Ok(decoded) = RoleRevokedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::RoleRevokedFilter(decoded));
            }
            if let Ok(decoded) = SplitterCreatedFilter::decode_log(log) {
                return Ok(PayoutFactoryEvents::SplitterCreatedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for PayoutFactoryEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PayoutFactoryEvents::PaymentReceivedFilter(element) => element.fmt(f),
                PayoutFactoryEvents::PaymentReleasedFilter(element) => element.fmt(f),
                PayoutFactoryEvents::RoleAdminChangedFilter(element) => element.fmt(f),
                PayoutFactoryEvents::RoleGrantedFilter(element) => element.fmt(f),
                PayoutFactoryEvents::RoleRevokedFilter(element) => element.fmt(f),
                PayoutFactoryEvents::SplitterCreatedFilter(element) => element.fmt(f),
            }
        }
    }
    ///Container type for all input parameters for the `DEFAULT_ADMIN_ROLE` function with signature `DEFAULT_ADMIN_ROLE()` and selector `0xa217fddf`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "DEFAULT_ADMIN_ROLE", abi = "DEFAULT_ADMIN_ROLE()")]
    pub struct DefaultAdminRoleCall;
    ///Container type for all input parameters for the `getRoleAdmin` function with signature `getRoleAdmin(bytes32)` and selector `0x248a9ca3`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "getRoleAdmin", abi = "getRoleAdmin(bytes32)")]
    pub struct GetRoleAdminCall {
        pub role: [u8; 32],
    }
    ///Container type for all input parameters for the `grantRole` function with signature `grantRole(bytes32,address)` and selector `0x2f2ff15d`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "grantRole", abi = "grantRole(bytes32,address)")]
    pub struct GrantRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `hasRole` function with signature `hasRole(bytes32,address)` and selector `0x91d14854`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "hasRole", abi = "hasRole(bytes32,address)")]
    pub struct HasRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `payout` function with signature `payout(address[],uint256[])` and selector `0xc176e639`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "payout", abi = "payout(address[],uint256[])")]
    pub struct PayoutCall {
        pub payees: ::std::vec::Vec<::ethers::core::types::Address>,
        pub shares: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `payouts` function with signature `payouts()` and selector `0xa247bc32`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "payouts", abi = "payouts()")]
    pub struct PayoutsCall;
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
    ///Container type for all input parameters for the `releaseAll` function with signature `releaseAll(address)` and selector `0x580fc80a`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "releaseAll", abi = "releaseAll(address)")]
    pub struct ReleaseAllCall {
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
    ///Container type for all input parameters for the `renounceRole` function with signature `renounceRole(bytes32,address)` and selector `0x36568abe`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "renounceRole", abi = "renounceRole(bytes32,address)")]
    pub struct RenounceRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `revokeRole` function with signature `revokeRole(bytes32,address)` and selector `0xd547741f`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "revokeRole", abi = "revokeRole(bytes32,address)")]
    pub struct RevokeRoleCall {
        pub role: [u8; 32],
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
    ///Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    ///Container type for all input parameters for the `template` function with signature `template()` and selector `0x6f2ddd93`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
    )]
    #[derive(Default)]
    #[ethcall(name = "template", abi = "template()")]
    pub struct TemplateCall;
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
    pub enum PayoutFactoryCalls {
        DefaultAdminRole(DefaultAdminRoleCall),
        GetRoleAdmin(GetRoleAdminCall),
        GrantRole(GrantRoleCall),
        HasRole(HasRoleCall),
        Payout(PayoutCall),
        Payouts(PayoutsCall),
        Releasable(ReleasableCall),
        ReleaseAll(ReleaseAllCall),
        Released(ReleasedCall),
        RenounceRole(RenounceRoleCall),
        RevokeRole(RevokeRoleCall),
        Shares(SharesCall),
        SupportsInterface(SupportsInterfaceCall),
        Template(TemplateCall),
        TotalReleased(TotalReleasedCall),
        TotalShares(TotalSharesCall),
    }
    impl ::ethers::core::abi::AbiDecode for PayoutFactoryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ::ethers::core::abi::AbiError> {
            if let Ok(decoded)
                = <DefaultAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::DefaultAdminRole(decoded));
            }
            if let Ok(decoded)
                = <GetRoleAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::GetRoleAdmin(decoded));
            }
            if let Ok(decoded)
                = <GrantRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::GrantRole(decoded));
            }
            if let Ok(decoded)
                = <HasRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::HasRole(decoded));
            }
            if let Ok(decoded)
                = <PayoutCall as ::ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PayoutFactoryCalls::Payout(decoded));
            }
            if let Ok(decoded)
                = <PayoutsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::Payouts(decoded));
            }
            if let Ok(decoded)
                = <ReleasableCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::Releasable(decoded));
            }
            if let Ok(decoded)
                = <ReleaseAllCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::ReleaseAll(decoded));
            }
            if let Ok(decoded)
                = <ReleasedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::Released(decoded));
            }
            if let Ok(decoded)
                = <RenounceRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::RenounceRole(decoded));
            }
            if let Ok(decoded)
                = <RevokeRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::RevokeRole(decoded));
            }
            if let Ok(decoded)
                = <SharesCall as ::ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(PayoutFactoryCalls::Shares(decoded));
            }
            if let Ok(decoded)
                = <SupportsInterfaceCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::SupportsInterface(decoded));
            }
            if let Ok(decoded)
                = <TemplateCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::Template(decoded));
            }
            if let Ok(decoded)
                = <TotalReleasedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::TotalReleased(decoded));
            }
            if let Ok(decoded)
                = <TotalSharesCall as ::ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                ) {
                return Ok(PayoutFactoryCalls::TotalShares(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PayoutFactoryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                PayoutFactoryCalls::DefaultAdminRole(element) => element.encode(),
                PayoutFactoryCalls::GetRoleAdmin(element) => element.encode(),
                PayoutFactoryCalls::GrantRole(element) => element.encode(),
                PayoutFactoryCalls::HasRole(element) => element.encode(),
                PayoutFactoryCalls::Payout(element) => element.encode(),
                PayoutFactoryCalls::Payouts(element) => element.encode(),
                PayoutFactoryCalls::Releasable(element) => element.encode(),
                PayoutFactoryCalls::ReleaseAll(element) => element.encode(),
                PayoutFactoryCalls::Released(element) => element.encode(),
                PayoutFactoryCalls::RenounceRole(element) => element.encode(),
                PayoutFactoryCalls::RevokeRole(element) => element.encode(),
                PayoutFactoryCalls::Shares(element) => element.encode(),
                PayoutFactoryCalls::SupportsInterface(element) => element.encode(),
                PayoutFactoryCalls::Template(element) => element.encode(),
                PayoutFactoryCalls::TotalReleased(element) => element.encode(),
                PayoutFactoryCalls::TotalShares(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for PayoutFactoryCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                PayoutFactoryCalls::DefaultAdminRole(element) => element.fmt(f),
                PayoutFactoryCalls::GetRoleAdmin(element) => element.fmt(f),
                PayoutFactoryCalls::GrantRole(element) => element.fmt(f),
                PayoutFactoryCalls::HasRole(element) => element.fmt(f),
                PayoutFactoryCalls::Payout(element) => element.fmt(f),
                PayoutFactoryCalls::Payouts(element) => element.fmt(f),
                PayoutFactoryCalls::Releasable(element) => element.fmt(f),
                PayoutFactoryCalls::ReleaseAll(element) => element.fmt(f),
                PayoutFactoryCalls::Released(element) => element.fmt(f),
                PayoutFactoryCalls::RenounceRole(element) => element.fmt(f),
                PayoutFactoryCalls::RevokeRole(element) => element.fmt(f),
                PayoutFactoryCalls::Shares(element) => element.fmt(f),
                PayoutFactoryCalls::SupportsInterface(element) => element.fmt(f),
                PayoutFactoryCalls::Template(element) => element.fmt(f),
                PayoutFactoryCalls::TotalReleased(element) => element.fmt(f),
                PayoutFactoryCalls::TotalShares(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<DefaultAdminRoleCall> for PayoutFactoryCalls {
        fn from(var: DefaultAdminRoleCall) -> Self {
            PayoutFactoryCalls::DefaultAdminRole(var)
        }
    }
    impl ::std::convert::From<GetRoleAdminCall> for PayoutFactoryCalls {
        fn from(var: GetRoleAdminCall) -> Self {
            PayoutFactoryCalls::GetRoleAdmin(var)
        }
    }
    impl ::std::convert::From<GrantRoleCall> for PayoutFactoryCalls {
        fn from(var: GrantRoleCall) -> Self {
            PayoutFactoryCalls::GrantRole(var)
        }
    }
    impl ::std::convert::From<HasRoleCall> for PayoutFactoryCalls {
        fn from(var: HasRoleCall) -> Self {
            PayoutFactoryCalls::HasRole(var)
        }
    }
    impl ::std::convert::From<PayoutCall> for PayoutFactoryCalls {
        fn from(var: PayoutCall) -> Self {
            PayoutFactoryCalls::Payout(var)
        }
    }
    impl ::std::convert::From<PayoutsCall> for PayoutFactoryCalls {
        fn from(var: PayoutsCall) -> Self {
            PayoutFactoryCalls::Payouts(var)
        }
    }
    impl ::std::convert::From<ReleasableCall> for PayoutFactoryCalls {
        fn from(var: ReleasableCall) -> Self {
            PayoutFactoryCalls::Releasable(var)
        }
    }
    impl ::std::convert::From<ReleaseAllCall> for PayoutFactoryCalls {
        fn from(var: ReleaseAllCall) -> Self {
            PayoutFactoryCalls::ReleaseAll(var)
        }
    }
    impl ::std::convert::From<ReleasedCall> for PayoutFactoryCalls {
        fn from(var: ReleasedCall) -> Self {
            PayoutFactoryCalls::Released(var)
        }
    }
    impl ::std::convert::From<RenounceRoleCall> for PayoutFactoryCalls {
        fn from(var: RenounceRoleCall) -> Self {
            PayoutFactoryCalls::RenounceRole(var)
        }
    }
    impl ::std::convert::From<RevokeRoleCall> for PayoutFactoryCalls {
        fn from(var: RevokeRoleCall) -> Self {
            PayoutFactoryCalls::RevokeRole(var)
        }
    }
    impl ::std::convert::From<SharesCall> for PayoutFactoryCalls {
        fn from(var: SharesCall) -> Self {
            PayoutFactoryCalls::Shares(var)
        }
    }
    impl ::std::convert::From<SupportsInterfaceCall> for PayoutFactoryCalls {
        fn from(var: SupportsInterfaceCall) -> Self {
            PayoutFactoryCalls::SupportsInterface(var)
        }
    }
    impl ::std::convert::From<TemplateCall> for PayoutFactoryCalls {
        fn from(var: TemplateCall) -> Self {
            PayoutFactoryCalls::Template(var)
        }
    }
    impl ::std::convert::From<TotalReleasedCall> for PayoutFactoryCalls {
        fn from(var: TotalReleasedCall) -> Self {
            PayoutFactoryCalls::TotalReleased(var)
        }
    }
    impl ::std::convert::From<TotalSharesCall> for PayoutFactoryCalls {
        fn from(var: TotalSharesCall) -> Self {
            PayoutFactoryCalls::TotalShares(var)
        }
    }
    ///Container type for all return fields from the `DEFAULT_ADMIN_ROLE` function with signature `DEFAULT_ADMIN_ROLE()` and selector `0xa217fddf`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct DefaultAdminRoleReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getRoleAdmin` function with signature `getRoleAdmin(bytes32)` and selector `0x248a9ca3`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct GetRoleAdminReturn(pub [u8; 32]);
    ///Container type for all return fields from the `hasRole` function with signature `hasRole(bytes32,address)` and selector `0x91d14854`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct HasRoleReturn(pub bool);
    ///Container type for all return fields from the `payout` function with signature `payout(address[],uint256[])` and selector `0xc176e639`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct PayoutReturn {
        pub instance: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `payouts` function with signature `payouts()` and selector `0xa247bc32`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct PayoutsReturn(pub ::std::vec::Vec<::ethers::core::types::Address>);
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
    pub struct ReleasableReturn {
        pub total_value: ::ethers::core::types::U256,
    }
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
    pub struct ReleasedReturn {
        pub total_value: ::ethers::core::types::U256,
    }
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
    pub struct SharesReturn {
        pub total_value: ::ethers::core::types::U256,
    }
    ///Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct SupportsInterfaceReturn(pub bool);
    ///Container type for all return fields from the `template` function with signature `template()` and selector `0x6f2ddd93`
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
    )]
    #[derive(Default)]
    pub struct TemplateReturn(pub ::ethers::core::types::Address);
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
    pub struct TotalReleasedReturn {
        pub total_value: ::ethers::core::types::U256,
    }
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
    pub struct TotalSharesReturn {
        pub total_value: ::ethers::core::types::U256,
    }
}
