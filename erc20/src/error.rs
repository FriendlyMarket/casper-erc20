//! Error handling on the casper platform.
use types::ApiError;

/// Errors which can be returned by the library.
///
/// When an `Error` is returned from a smart contract, it is converted to an [`ApiError::User`].
///
/// Where a smart contract consuming this library needs to define further error variants, it can
/// return those via the [`Error::User`] variant or equivalently via the [`ApiError::User`]
/// variant.
///
/// Such a user error should be in the range `[0..(u16::MAX - 27)]` (i.e. [0, 65508]) to avoid
/// conflicting with the other `Error` variants.
pub enum Error {
    /// ERC20 contract called from within an invalid context.
    InvalidContext,
    /// Spender does not have enough balance.
    InsufficientBalance,
    /// Spender does not have enough allowance approved.
    InsufficientAllowance,
    /// Operation would cause an integer overflow.
    Overflow,
    /// Tokens address is null.
    ZeroAddress,
    /// Cannot mint tokens to zero hash address.
    CannotMintToZeroHash,
    /// Cannot burn tokens from zero hash address.
    CannotBurnFromZeroHash,
    /// Trying to burn an amount that surpasses the owner's balance.
    BurnAmountExceedsBalance,
    /// Called a pair's function with the wrong emergency_mode.
    InadequateEmergencyMode,
    /// User error.
    User(u16),
}

// u16::MAX = 65535
const ERROR_INVALID_CONTEXT: u16 = u16::MAX; // 65535
const ERROR_INSUFFICIENT_BALANCE: u16 = u16::MAX - 1; // 65534
const ERROR_INSUFFICIENT_ALLOWANCE: u16 = u16::MAX - 2; // 65533
const ERROR_OVERFLOW: u16 = u16::MAX - 3; // 65532
const ERROR_ZERO_ADDRESS: u16 = u16::MAX - 4; // 65531
const ERROR_CANNOT_MINT_TO_ZERO_HASH: u16 = u16::MAX - 5; // 65530
const ERROR_CANNOT_BURN_FROM_ZERO_HASH: u16 = u16::MAX - 6; // 65529
const ERROR_BURN_AMOUNT_EXCEEDS_BALANCE: u16 = u16::MAX - 7; // 65528
const ERROR_INADEQUATE_EMERGENCY_MODE: u16 = u16::MAX - 8; // 65527

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let user_error = match error {
            Error::InvalidContext => ERROR_INVALID_CONTEXT,
            Error::InsufficientBalance => ERROR_INSUFFICIENT_BALANCE,
            Error::InsufficientAllowance => ERROR_INSUFFICIENT_ALLOWANCE,
            Error::Overflow => ERROR_OVERFLOW,
            Error::ZeroAddress => ERROR_ZERO_ADDRESS,
            Error::CannotMintToZeroHash => ERROR_CANNOT_MINT_TO_ZERO_HASH,
            Error::CannotBurnFromZeroHash => ERROR_CANNOT_BURN_FROM_ZERO_HASH,
            Error::BurnAmountExceedsBalance => ERROR_BURN_AMOUNT_EXCEEDS_BALANCE,
            Error::InadequateEmergencyMode => ERROR_INADEQUATE_EMERGENCY_MODE,
            Error::User(user_error) => user_error,
        };
        ApiError::User(user_error)
    }
}
