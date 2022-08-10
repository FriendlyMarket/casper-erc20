use crate::utilities::{blake2b256, get_current_time, key_to_str};
use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_AUCTION_DELAY,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_GENESIS_TIMESTAMP_MILLIS,
    DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS, DEFAULT_PAYMENT, DEFAULT_PROPOSER_PUBLIC_KEY,
    DEFAULT_PROTOCOL_VERSION, DEFAULT_ROUND_SEIGNIORAGE_RATE, DEFAULT_SYSTEM_CONFIG,
    DEFAULT_UNBONDING_DELAY, DEFAULT_VALIDATOR_SLOTS, DEFAULT_WASM_CONFIG,
};
use casper_execution_engine::core::engine_state::{
    genesis::{ExecConfig, GenesisAccount},
    run_genesis_request::RunGenesisRequest,
};
use casper_types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    runtime_args, CLTyped, ContractHash, Key, Motes, PublicKey, RuntimeArgs, SecretKey,
    StoredValue, U256,
};
use rand::Rng;
use std::path::PathBuf;

// contains methods that can simulate a real-world deployment (storing the contract in the blockchain)
// and transactions to invoke the methods in the contract.

pub mod token_cfg {
    use super::*;
    pub const NAME: &str = "ERC20";
    pub const SYMBOL: &str = "ERC";
    pub const DECIMALS: u8 = 8;
    pub fn total_supply() -> U256 {
        1_000.into()
    }
}

pub const ERC20_TOKEN_CONTRACT_KEY_NAME: &str = "erc20_token_contract";

const BALANCES_DICT: &str = "balances";
const ALLOWANCES_DICT: &str = "allowances";

pub struct Sender(pub AccountHash);
pub type Hash = [u8; 32];

pub struct Config {}

impl Config {
    /// Creates a vector of [`GenesisAccount`] out of a vector of [`PublicKey`].
    pub fn set_custom_accounts(public_keys: Vec<PublicKey>) -> Vec<GenesisAccount> {
        let mut genesis_accounts = Vec::new();

        // add default and proposer accounts.
        let genesis_account = GenesisAccount::account(
            DEFAULT_ACCOUNT_PUBLIC_KEY.clone(),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            None,
        );
        genesis_accounts.push(genesis_account);
        let proposer_account = GenesisAccount::account(
            DEFAULT_PROPOSER_PUBLIC_KEY.clone(),
            Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
            None,
        );
        genesis_accounts.push(proposer_account);

        // add custom accounts.
        for public_key in public_keys {
            let genesis_account = GenesisAccount::account(
                public_key.clone(),
                Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE.into()),
                None,
            );
            genesis_accounts.push(genesis_account);
        }
        genesis_accounts
    }

    /// Creates an [`ExecConfig`] out of the given `genesis_accounts`
    /// and uses default values for the other params.
    pub fn set_custom_exec_config(genesis_accounts: Vec<GenesisAccount>) -> ExecConfig {
        ExecConfig::new(
            genesis_accounts,
            *DEFAULT_WASM_CONFIG,
            *DEFAULT_SYSTEM_CONFIG,
            DEFAULT_VALIDATOR_SLOTS,
            DEFAULT_AUCTION_DELAY,
            DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS,
            DEFAULT_ROUND_SEIGNIORAGE_RATE,
            DEFAULT_UNBONDING_DELAY,
            DEFAULT_GENESIS_TIMESTAMP_MILLIS,
        )
    }

    /// Creates a new [`RunGenesisRequest`] given a custom [`ExecConfig`].
    pub fn set_custom_run_genesis_request(custom_exec_config: ExecConfig) -> RunGenesisRequest {
        RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            *DEFAULT_PROTOCOL_VERSION,
            custom_exec_config,
        )
    }

    /// Deploys a contract and returns the `contract_hash` and the updated `builder`.
    pub fn deploy_contract(
        mut builder: InMemoryWasmTestBuilder,
        session_code: PathBuf,
        session_args: RuntimeArgs,
        deployer: PublicKey,
        contract_hash_key: String,
    ) -> (InMemoryWasmTestBuilder, Hash) {
        let mut rng = rand::thread_rng();

        let deploy_item = DeployItemBuilder::new()
            // .with_payment_bytes(module_bytes, args)
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_deploy_hash(rng.gen())
            .with_authorization_keys(&[deployer.to_account_hash()])
            .with_address(deployer.to_account_hash())
            .build();

        // prepare the execute request.
        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(get_current_time())
            .build();

        // pre-assertion before the contract deployment.
        let contract_hash = builder.query(
            None,
            Key::Account(deployer.to_account_hash()),
            &[contract_hash_key.clone()],
        );

        assert!(contract_hash.is_err());

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();

        // retrieving hashes & post-assertions after the contract deployment.
        let contract_hash = builder
            .get_account(deployer.to_account_hash())
            .expect("should have account")
            .named_keys()
            .get(&contract_hash_key)
            .and_then(|key| key.into_hash())
            .map(ContractHash::new)
            .expect("should have contract hash")
            .value();

        assert_ne!(contract_hash, [0u8; 32]);

        (builder, contract_hash)
    }
}

pub struct Token {
    pub name: String,
    pub symbol: String,
    pub builder: InMemoryWasmTestBuilder,
    pub hash: Hash,
    pub ali: AccountHash,
    pub bob: AccountHash,
    pub joe: AccountHash,
}

impl Token {
    pub fn deployed(name: &str, symbol: &str) -> Token {
        // ====================== ACCOUNTS SETUP ======================
        let ali = PublicKey::from(&SecretKey::ed25519_from_bytes([3u8; 32]).unwrap());
        let bob = PublicKey::from(&SecretKey::ed25519_from_bytes([6u8; 32]).unwrap());
        let joe = PublicKey::from(&SecretKey::ed25519_from_bytes([9u8; 32]).unwrap());

        // ====================== BLOCKCHAIN SETUP ======================
        // create our WasmBuilder.
        let mut builder = InMemoryWasmTestBuilder::default();

        // initialize the blockchain network to get our first block.

        // implement custom accounts.
        let genesis_accounts: Vec<GenesisAccount> =
            Config::set_custom_accounts(vec![ali.clone(), bob.clone()]);

        // implement custom exec config.
        let custom_exec_config: ExecConfig = Config::set_custom_exec_config(genesis_accounts);

        // implement custom run genesis request.
        let custom_run_genesis_request: RunGenesisRequest =
            Config::set_custom_run_genesis_request(custom_exec_config);

        // run genesis request with the custom exec config.
        builder.run_genesis(&custom_run_genesis_request).commit();

        // ====================== CONTRACT DEPLOYMENT ======================
        let session_code = PathBuf::from("erc20_token.wasm");
        let session_args = runtime_args! {
            "name" => name,
            "symbol" => symbol,
            "decimals" => token_cfg::DECIMALS,
            "total_supply" => token_cfg::total_supply(),
        };

        let (builder, hash) = Config::deploy_contract(
            builder,
            session_code,
            session_args,
            ali.clone(),
            ERC20_TOKEN_CONTRACT_KEY_NAME.to_string(),
        );

        // ====================== FUNCTION RETURN ======================
        Token {
            name: name.to_string(),
            symbol: symbol.to_string(),
            builder,
            hash,
            ali: ali.to_account_hash(),
            bob: bob.to_account_hash(),
            joe: joe.to_account_hash(),
        }
    }

    /// query a contract's named key.
    fn query_contract<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        match self.builder.query(
            None,
            Key::Account(self.ali),
            &[ERC20_TOKEN_CONTRACT_KEY_NAME.to_string(), name.to_string()],
        ) {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .as_cl_value()
                    .expect("should be cl value.")
                    .clone()
                    .into_t()
                    .expect("should have the correct type.");
                Some(value)
            }
        }
    }

    fn query_dictionary_value<T: CLTyped + FromBytes>(
        &self,
        dict_name: &str,
        key: String,
    ) -> Option<T> {
        // prepare the dictionary seed uref.
        let stored_value = self
            .builder
            .query(None, Key::Hash(self.hash), &[])
            .map_err(|_| "error")
            .unwrap();

        // get the named keys of the given Key.
        let named_keys = match &stored_value {
            StoredValue::Account(account) => account.named_keys(),
            StoredValue::Contract(contract) => contract.named_keys(),
            _ => return None,
        };

        // get the dictionary uref.
        let dictionary_uref = named_keys.get(dict_name).and_then(Key::as_uref).unwrap();

        let dictionary_key_bytes = key.as_bytes();

        let _address = Key::dictionary(*dictionary_uref, dictionary_key_bytes);

        // query the dictionary.
        match self
            .builder
            .query_dictionary_item(None, *dictionary_uref, &key)
        {
            Err(_) => None,
            Ok(maybe_value) => {
                let value = maybe_value
                    .as_cl_value()
                    .expect("should be cl value.")
                    .clone()
                    .into_t()
                    .expect("should have the correct type.");
                Some(value)
            }
        }
    }

    /// call a contract's specific entry point.
    fn call(&mut self, sender: Sender, method: &str, args: RuntimeArgs) {
        let Sender(address) = sender;

        // prepare the deploy item.
        let deploy_item = DeployItemBuilder::new()
            // .with_payment_bytes(module_bytes, args)
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_stored_session_hash(self.hash.into(), method, args)
            .with_authorization_keys(&[address])
            .with_address(address)
            .build();

        // prepare the execute request.
        // we can use .with_block_time() when setting the execute request.
        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        // executes the execute_request.
        self.builder.exec(execute_request).commit().expect_success();
    }

    pub fn name(&self) -> String {
        self.query_contract("name").unwrap()
    }

    pub fn symbol(&self) -> String {
        self.query_contract("symbol").unwrap()
    }

    pub fn decimals(&self) -> u8 {
        self.query_contract("decimals").unwrap()
    }

    pub fn total_supply(&self) -> U256 {
        self.query_contract("total_supply").unwrap()
    }

    pub fn balance_of(&self, address: Key) -> U256 {
        self.query_dictionary_value(BALANCES_DICT, key_to_str(&address))
            .unwrap()
    }

    pub fn allowance(&self, owner: Key, spender: Key) -> U256 {
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap());
        preimage.append(&mut spender.to_bytes().unwrap());
        let key_bytes = blake2b256(&preimage);
        let allowance_item_key = hex::encode(&key_bytes);

        self.query_dictionary_value(ALLOWANCES_DICT, allowance_item_key)
            .unwrap()
    }

    pub fn transfer(&mut self, recipient: Key, amount: U256, sender: Sender) {
        self.call(
            sender,
            "transfer",
            runtime_args! {
                "recipient" => recipient,
                "amount" => amount
            },
        );
    }

    pub fn approve(&mut self, spender: Key, amount: U256, sender: Sender) {
        self.call(
            sender,
            "approve",
            runtime_args! {
                "spender" => spender,
                "amount" => amount
            },
        );
    }

    pub fn transfer_from(&mut self, owner: Key, recipient: Key, amount: U256, sender: Sender) {
        self.call(
            sender,
            "transfer_from",
            runtime_args! {
                "owner" => owner,
                "recipient" => recipient,
                "amount" => amount
            },
        );
    }
}
