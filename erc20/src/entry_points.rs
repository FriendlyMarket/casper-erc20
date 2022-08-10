//! Contains definition of the ERC20 contract entry points.
use types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter};

pub fn endpoint(name: &str, param: Vec<Parameter>, ret: CLType) -> EntryPoint {
    EntryPoint::new(
        String::from(name),
        param,
        ret,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `name` entry point.
pub fn name() -> EntryPoint {
    endpoint("name", vec![], CLType::String)
}

/// Returns the `symbol` entry point.
pub fn symbol() -> EntryPoint {
    endpoint("symbol", vec![], CLType::String)
}

/// Returns the `decimals` entry point.
pub fn decimals() -> EntryPoint {
    endpoint("decimals", vec![], CLType::U8)
}

/// Returns the `total_supply` entry point.
pub fn total_supply() -> EntryPoint {
    endpoint("total_supply", vec![], CLType::U256)
}

/// Returns the `transfer` entry point.
pub fn transfer() -> EntryPoint {
    endpoint(
        "transfer",
        vec![
            Parameter::new("recipient", CLType::Key),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    )
}

/// Returns the `balance_of` entry point.
pub fn balance_of() -> EntryPoint {
    endpoint(
        "balance_of",
        vec![Parameter::new("address", CLType::Key)],
        CLType::U256,
    )
}

/// Returns the `allowance` entry point.
pub fn allowance() -> EntryPoint {
    endpoint(
        "allowance",
        vec![
            Parameter::new("owner", CLType::Key),
            Parameter::new("spender", CLType::Key),
        ],
        CLType::U256,
    )
}

/// Returns the `approve` entry point.
pub fn approve() -> EntryPoint {
    endpoint(
        "approve",
        vec![
            Parameter::new("spender", CLType::Key),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    )
}

/// Returns the `transfer_from` entry point.
pub fn transfer_from() -> EntryPoint {
    endpoint(
        "transfer_from",
        vec![
            Parameter::new("owner", CLType::Key),
            Parameter::new("recipient", CLType::Key),
            Parameter::new("amount", CLType::U256),
        ],
        CLType::Unit,
    )
}

/// Returns the default set of ERC20 entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(name());
    entry_points.add_entry_point(symbol());
    entry_points.add_entry_point(decimals());
    entry_points.add_entry_point(total_supply());
    entry_points.add_entry_point(transfer());
    entry_points.add_entry_point(balance_of());
    entry_points.add_entry_point(allowance());
    entry_points.add_entry_point(approve());
    entry_points.add_entry_point(transfer_from());
    entry_points
}
