use casper_types::{account::AccountHash, Key, U256};

use crate::erc20::{token_cfg, Sender, Token};

fn to_key(account: AccountHash) -> Key {
    Key::Account(account)
}

// ------------ START - ERC20 Tests ------------

#[test]
fn should_deploy_erc20() {
    let t = Token::deployed("ERC20", "ERC");
    assert_eq!(t.name(), token_cfg::NAME);
    assert_eq!(t.symbol(), token_cfg::SYMBOL);
    assert_eq!(t.decimals(), token_cfg::DECIMALS);
    assert_eq!(t.balance_of(to_key(t.ali)), token_cfg::total_supply());
}

#[test]
fn should_transfer_erc20() {
    let amount = 10.into();
    let mut t = Token::deployed("ERC20", "ERC");
    t.transfer(to_key(t.bob), amount, Sender(t.ali));
    assert_eq!(
        t.balance_of(to_key(t.ali)),
        token_cfg::total_supply() - amount
    );
    assert_eq!(t.balance_of(to_key(t.bob)), amount);
}

#[test]
#[should_panic = "65534"]
fn should_not_transfer_too_much_erc20() {
    let amount = 1.into();
    let mut t = Token::deployed("ERC20", "ERC");
    t.transfer(to_key(t.ali), amount, Sender(t.bob));
}

#[test]
fn should_approve_erc20() {
    let amount = 10.into();
    let mut t = Token::deployed("ERC20", "ERC");
    t.approve(to_key(t.bob), amount, Sender(t.ali));
    assert_eq!(t.balance_of(to_key(t.ali)), token_cfg::total_supply());
    assert_eq!(t.allowance(to_key(t.ali), to_key(t.bob)), amount);
}

#[test]
fn should_approve_erc20_to_zero_address() {
    let amount = 10.into();
    let mut t = Token::deployed("ERC20", "ERC");
    let zero_key = Key::Account(AccountHash::default());
    t.approve(zero_key, amount, Sender(t.ali));
}

#[test]
fn should_transfer_erc20_from() {
    let allowance = 10.into();
    let amount = 3.into();
    let mut t = Token::deployed("ERC20", "ERC");
    t.approve(to_key(t.bob), allowance, Sender(t.ali));
    assert_eq!(t.allowance(to_key(t.ali), to_key(t.bob)), allowance);
    t.transfer_from(to_key(t.ali), to_key(t.joe), amount, Sender(t.bob));
    assert_eq!(
        t.balance_of(to_key(t.ali)),
        token_cfg::total_supply() - amount
    );
    assert_eq!(t.balance_of(to_key(t.joe)), amount);
    assert_eq!(
        t.allowance(to_key(t.ali), to_key(t.bob)),
        allowance - amount
    );
}

#[test]
#[should_panic = "65534"]
fn should_not_transfer_from_too_much_erc20() {
    let amount = token_cfg::total_supply().checked_add(1.into()).unwrap();
    let mut t = Token::deployed("ERC20", "ERC");
    t.approve(to_key(t.bob), amount, Sender(t.ali));
    t.transfer_from(to_key(t.ali), to_key(t.joe), amount, Sender(t.bob));
}

#[test]
fn should_transfer_erc20_to_zero_address() {
    let amount = 1.into();
    let mut t = Token::deployed("ERC20", "ERC");
    let zero_key = Key::Account(AccountHash::default());
    t.transfer(zero_key, amount, Sender(t.ali));
}

#[test]
#[should_panic = "User(65533)"]
fn should_not_transfer_from_erc20_without_approval() {
    let amount = U256::from(1);
    let mut t = Token::deployed("ERC20", "ERC");
    t.transfer_from(to_key(t.ali), to_key(t.joe), amount, Sender(t.bob));
}

#[test]
#[should_panic = "User(65533)"]
fn should_not_transfer_from_erc20_with_low_allowance() {
    let allowance = 1.into();
    let amount = 3.into();
    let mut t = Token::deployed("ERC20", "ERC");
    t.approve(to_key(t.bob), allowance, Sender(t.ali));
    t.transfer_from(to_key(t.ali), to_key(t.joe), amount, Sender(t.bob));
}
