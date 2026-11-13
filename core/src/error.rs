use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    SeasonOver,
    SeasonOpen,
    SelfCatch,
    InvalidName,
    DepositTooLow,
    CarrierNotContagious,
    CarrierExpired,
    FanoutReached,
    TargetNotUninfected,
    MintExhausted,
    NotContagious,
    NotExpired,
    NotVaccinable,
    InsufficientLiquid,
    NothingToClaim,
    ZeroAmount,
    SelfTransfer,
    Overflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Error::SeasonOver => "the season is over",
            Error::SeasonOpen => "the season is still open",
            Error::SelfCatch => "an account cannot catch from itself",
            Error::InvalidName => "target must be a named account",
            Error::DepositTooLow => "attached deposit is below the required fee",
            Error::CarrierNotContagious => "carrier is not contagious",
            Error::CarrierExpired => "carrier incubation clock expired",
            Error::FanoutReached => "carrier reached its fanout cap",
            Error::TargetNotUninfected => "target is not uninfected",
            Error::MintExhausted => "mint decayed to zero",
            Error::NotContagious => "account is not contagious",
            Error::NotExpired => "account is not expired",
            Error::NotVaccinable => "account is not uninfected",
            Error::InsufficientLiquid => "not enough liquid balance",
            Error::NothingToClaim => "nothing to claim",
            Error::ZeroAmount => "amount must be above zero",
            Error::SelfTransfer => "sender and receiver must differ",
            Error::Overflow => "arithmetic overflow",
        };
        f.write_str(text)
    }
}

impl std::error::Error for Error {}
