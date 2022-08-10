#![allow(unused_parens)]
#![allow(non_snake_case)]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::convert::TryInto;
use error::Error;

use contract::{
    contract_api::{
        runtime,
        storage::{self, create_contract_package_at_hash},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    system::CallStackElement,
    ApiError, CLTyped, CLValue, Key, URef, U256,
};
pub mod entry_points;
pub mod error;
pub mod events;
use events::ERC20Event;

/// # Purpose
/// * Returns the `name` property.
#[no_mangle]
pub extern "C" fn name() {
    let name: String = get_key("name");
    ret(name)
}

/// # Purpose
/// * Returns the `symbol` property.
#[no_mangle]
pub extern "C" fn symbol() {
    let symbol: String = get_key("symbol");
    ret(symbol)
}

/// # Purpose
/// * Returns the `decimals` property.
#[no_mangle]
pub extern "C" fn decimals() {
    let decimals: u8 = get_key("decimals");
    ret(decimals)
}

/// # Purpose
/// * Returns the `total_supply` of the token.
#[no_mangle]
pub extern "C" fn total_supply() {
    let total_supply: U256 = get_key("total_supply");
    ret(total_supply)
}

/// # Purpose
/// * Returns how much tokens the given `address` owns.
/// # Arguments
/// * `address` - `Key` -> Address that we are looking for it's token balance.
/// # Returns
/// * `balance` - `U256` -> The given `address`'s balance.
#[no_mangle]
pub extern "C" fn balance_of() {
    let address: Key = runtime::get_named_arg("address");
    let balance: U256 = get("balances", &key_to_str(&address));
    ret(balance)
}

/// # Purpose
/// * Returns how much allowance the `owner` has given to the `spender`.
/// # Arguments
/// * `owner` - `Key` -> Address of the owner.
/// * `spender` - `Key` -> Address of the spender.
/// # Returns
/// * `amount` - `U256` -> Amount of the allowance.
#[no_mangle]
pub extern "C" fn allowance() {
    let owner: Key = runtime::get_named_arg("owner");
    let spender: Key = runtime::get_named_arg("spender");
    let amount: U256 = read_allowance(owner, spender);
    ret(amount)
}

/// # Purpose
/// * Grants an address the liberty to spend an amount of the caller's tokens.
/// # Arguments
/// * `spender` - `Key` -> Address of the spender.
/// * `amount` - `U256` -> Amount of the allowance.
#[no_mangle]
pub extern "C" fn approve() {
    let spender: Key = runtime::get_named_arg("spender");
    let amount: U256 = runtime::get_named_arg("amount");

    _approve(get_caller(), spender, amount);
}

/// # Purpose
/// * Transfers an amount of the caller's tokens to the given address.
/// # Arguments
/// * `recipient` - `Key` -> Address of the recipient.
/// * `amount` - `U256` -> Amount of the tokens to be sent.
#[no_mangle]
pub extern "C" fn transfer() {
    let recipient: Key = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");

    _transfer(get_caller(), recipient, amount);
}

/// # Purpose
/// * Transfers an `amount` of tokens from `owner` to `recipient`.
/// # Arguments
/// * `owner` - `Key` -> Address of the owner.
/// * `recipient` - `Key` -> Address of the recipient.
/// * `amount` - `U256` -> Amount of the tokens to be sent.
#[no_mangle]
pub extern "C" fn transfer_from() {
    let owner: Key = runtime::get_named_arg("owner");
    let recipient: Key = runtime::get_named_arg("recipient");
    let amount: U256 = runtime::get_named_arg("amount");

    _transfer_from(owner, recipient, amount);
}

/// # Purpose
/// * Creates an `amount` of tokens for the given address.
/// # Arguments
/// * `owner` - `Key` -> Address of the owner.
/// * `amount` - `U256` -> Amount of the tokens to be created.
#[no_mangle]
pub extern "C" fn mint() {
    let owner: Key = runtime::get_named_arg("owner");
    let amount: U256 = runtime::get_named_arg("amount");

    if (owner == Key::Hash([0u8; 32]) || owner == Key::Account(AccountHash::new([0u8; 32]))) {
        runtime::revert(Error::CannotMintToZeroHash);
    }

    let total_supply = get_key::<U256>("total_supply");

    set_key(
        "total_supply",
        total_supply.checked_add(amount).unwrap_or_revert(),
    );

    let balance = get::<U256>("balances", &key_to_str(&owner));

    set(
        "balances",
        &key_to_str(&owner),
        balance.checked_add(amount).unwrap_or_revert(),
    );

    events::emit(&ERC20Event::Transfer {
        from: Key::Hash([0u8; 32]),
        to: owner,
        value: amount,
    });
}

/// # Purpose
/// * Destroys an `amount` of tokens from the given address.
/// # Arguments
/// * `owner` - `Key` -> Address of the owner.
/// * `amount` - `U256` -> Amount of the tokens to be destroyed.
#[no_mangle]
pub extern "C" fn burn() {
    let owner: Key = runtime::get_named_arg("owner");
    let amount: U256 = runtime::get_named_arg("amount");

    if (owner == Key::Hash([0u8; 32]) || owner == Key::Account(AccountHash::new([0u8; 32]))) {
        runtime::revert(Error::CannotBurnFromZeroHash);
    }

    let balance = get::<U256>("balances", &key_to_str(&owner));

    if (balance < amount) {
        runtime::revert(Error::BurnAmountExceedsBalance);
    }

    set(
        "balances",
        &key_to_str(&owner),
        balance.checked_sub(amount).unwrap_or_revert(),
    );

    let total_supply = get_key::<U256>("total_supply");

    set_key(
        "total_supply",
        total_supply.checked_sub(amount).unwrap_or_revert(),
    );

    events::emit(&ERC20Event::Transfer {
        from: owner,
        to: Key::Hash([0u8; 32]),
        value: amount,
    });
}

#[no_mangle]
pub extern "C" fn call() {
    let token_name: String = runtime::get_named_arg("token_name");
    let token_symbol: String = runtime::get_named_arg("token_symbol");
    let token_decimals: u8 = runtime::get_named_arg("token_decimals");
    let token_total_supply: U256 = runtime::get_named_arg("token_total_supply");

    let entry_points = entry_points::default();

    let balances_seed_uref = storage::new_dictionary("balances").unwrap_or_revert();

    storage::dictionary_put(
        balances_seed_uref,
        &key_to_str(&Key::Account(runtime::get_caller())),
        token_total_supply,
    );

    let allowances_seed_uref = storage::new_dictionary("allowances").unwrap_or_revert();
    let mut named_keys = NamedKeys::new();

    named_keys.insert(
        "name".to_string(),
        storage::new_uref(token_name.clone()).into(),
    );
    named_keys.insert("symbol".to_string(), storage::new_uref(token_symbol).into());
    named_keys.insert(
        "decimals".to_string(),
        storage::new_uref(token_decimals).into(),
    );
    named_keys.insert(
        "total_supply".to_string(),
        storage::new_uref(token_total_supply).into(),
    );
    named_keys.insert("balances".to_string(), balances_seed_uref.into());
    named_keys.insert("allowances".to_string(), allowances_seed_uref.into());

    let (contract_package_hash, access_uref) = create_contract_package_at_hash();
    named_keys.insert(
        "contract_package_hash".to_string(),
        storage::new_uref(contract_package_hash).into(),
    );

    // Add new version to the package.
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);
    runtime::put_key(&token_name, contract_hash.into());
    runtime::put_key(
        [&token_name, "_hash"].join("").as_str(),
        storage::new_uref(contract_hash).into(),
    );
    runtime::put_key(
        [&token_name, "_package_hash"].join("").as_str(),
        contract_package_hash.into(),
    );
    runtime::put_key(
        [&token_name, "_access_token"].join("").as_str(),
        access_uref.into(),
    );
}

fn _transfer(sender: Key, recipient: Key, amount: U256) {
    _check_keys_not_null(sender, recipient);

    let new_sender_balance: U256 = get::<U256>("balances", &key_to_str(&sender))
        .checked_sub(amount)
        .ok_or(Error::InsufficientBalance)
        .unwrap_or_revert();

    set("balances", &key_to_str(&sender), new_sender_balance);

    let new_recipient_balance: U256 = get::<U256>("balances", &key_to_str(&recipient))
        .checked_add(amount)
        .ok_or(Error::Overflow)
        .unwrap_or_revert();

    set("balances", &key_to_str(&recipient), new_recipient_balance);

    events::emit(&ERC20Event::Transfer {
        from: sender,
        to: recipient,
        value: amount,
    });
}

fn _transfer_from(owner: Key, recipient: Key, amount: U256) {
    _check_keys_not_null(owner, recipient);

    let spender_allowance = read_allowance(owner, get_caller());
    let new_spender_allowance = spender_allowance
        .checked_sub(amount)
        .ok_or(Error::InsufficientAllowance)
        .unwrap_or_revert();

    _transfer(owner, recipient, amount);

    _approve(owner, get_caller(), new_spender_allowance);
}

fn _approve(owner: Key, spender: Key, amount: U256) {
    _check_keys_not_null(owner, spender);

    write_allowance(owner, spender, amount);

    events::emit(&ERC20Event::Approval {
        owner,
        spender,
        value: amount,
    });
}

fn _mint(to: Key, value: U256) {
    let total_supply: U256 = get_key::<U256>("total_supply")
        .checked_add(value)
        .ok_or(Error::Overflow)
        .unwrap_or_revert();

    set_key("total_supply", total_supply);

    let old_balance = get::<U256>("balances", &key_to_str(&to));
    let new_to_balance: U256 = old_balance
        .checked_add(value)
        .ok_or(Error::Overflow)
        .unwrap_or_revert();

    set("balances", &key_to_str(&to), new_to_balance);

    events::emit(&ERC20Event::Transfer {
        from: Key::Hash([0u8; 32]),
        to,
        value,
    });
}

fn _burn(from: Key, value: U256) {
    let from_balance = get::<U256>("balances", &key_to_str(&from));

    let new_from_balance: U256 = from_balance
        .checked_sub(value)
        .ok_or(Error::InsufficientBalance)
        .unwrap_or_revert();

    set("balances", &key_to_str(&from), new_from_balance);

    let total_supply: U256 = get_key::<U256>("total_supply")
        .checked_sub(value)
        .ok_or(Error::Overflow)
        .unwrap_or_revert();

    set_key("total_supply", total_supply);

    events::emit(&ERC20Event::Transfer {
        from,
        to: Key::Hash([0u8; 32]),
        value,
    });
}

fn _check_keys_not_null(x: Key, y: Key) {
    if x == Key::Account(AccountHash::default())
        || x == Key::Hash([0u8; 32])
        || y == Key::Account(AccountHash::default())
        || y == Key::Hash([0u8; 32])
    {
        runtime::revert(Error::ZeroAddress);
    }
}

fn ret<T: CLTyped + ToBytes>(value: T) {
    runtime::ret(CLValue::from_t(value).unwrap_or_revert())
}

fn key_to_str(key: &Key) -> String {
    let preimage = key.to_bytes().unwrap_or_revert();
    base64::encode(&preimage)
}

fn get_dictionary_seed_uref(name: &str) -> URef {
    match runtime::get_key(name) {
        Some(key) => key.into_uref().unwrap_or_revert(),
        None => {
            let new_dict = storage::new_dictionary(name).unwrap_or_revert();
            let key = storage::new_uref(new_dict).into();
            runtime::put_key(name, key);
            new_dict
        }
    }
}

fn get_key<T: FromBytes + CLTyped + Default>(name: &str) -> T {
    match runtime::get_key(name) {
        None => Default::default(),
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            storage::read(key).unwrap_or_revert().unwrap_or_revert()
        }
    }
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

fn get<T: FromBytes + CLTyped + Default>(dictionary_name: &str, key: &str) -> T {
    let dictionary_seed_uref = get_dictionary_seed_uref(dictionary_name);
    storage::dictionary_get(dictionary_seed_uref, key)
        .unwrap_or_default()
        .unwrap_or_default()
}

fn set<T: ToBytes + CLTyped>(dictionary_name: &str, key: &str, value: T) {
    let dictionary_seed_uref = get_dictionary_seed_uref(dictionary_name);
    storage::dictionary_put(dictionary_seed_uref, key, value)
}

/// Returns the immediate caller address, whether it's an account or a contract.
fn get_caller() -> Key {
    let mut callstack = runtime::get_call_stack();
    callstack.pop();
    match callstack
        .last()
        .ok_or(Error::InvalidContext)
        .unwrap_or_revert()
    {
        CallStackElement::Session { account_hash } => (*account_hash).into(),
        CallStackElement::StoredSession {
            account_hash,
            contract_package_hash: _,
            contract_hash: _,
        } => (*account_hash).into(),
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => Key::from(*contract_package_hash),
    }
}

/// Returns the `allowances` dictionary [`URef`].
#[inline]
fn allowances_uref() -> URef {
    _get_uref("allowances")
}

/// Returns the allowance that `owner` has given to `spender`.
fn read_allowance(owner: Key, spender: Key) -> U256 {
    _read_allowance_from(allowances_uref(), owner, spender)
}

/// Sets the allowance that `owner` is giving to `spender` to `amount`.
fn write_allowance(owner: Key, spender: Key, amount: U256) {
    _write_allowance_to(allowances_uref(), owner, spender, amount)
}

/// Creates a dictionary item key for an (owner, spender) pair.
fn make_dictionary_item_key(owner: Key, spender: Key) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut owner.to_bytes().unwrap_or_revert());
    preimage.append(&mut spender.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

/// Writes an allowance for owner and spender for a specific amount.
fn _write_allowance_to(allowances_uref: URef, owner: Key, spender: Key, amount: U256) {
    let dictionary_item_key = make_dictionary_item_key(owner, spender);
    storage::dictionary_put(allowances_uref, &dictionary_item_key, amount)
}

/// Reads an allowance for an owner and spender.
fn _read_allowance_from(allowances_uref: URef, owner: Key, spender: Key) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(owner, spender);
    storage::dictionary_get(allowances_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}

/// Gets [`URef`] under a name.
fn _get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();

    key.try_into().unwrap_or_revert()
}
