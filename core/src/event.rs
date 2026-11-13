use crate::account::EndReason;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    Mint {
        account: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        amount: u128,
    },
    Burn {
        account: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        amount: u128,
    },
    Transfer {
        from: String,
        to: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        amount: u128,
    },
    Catch {
        carrier: String,
        target: String,
        gen: u32,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        mint: u128,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        bonus: u128,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        fee: u128,
    },
    Recover {
        account: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        burned: u128,
        expired: bool,
    },
    Cull {
        account: String,
        caller: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        burned: u128,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        cut: u128,
    },
    Vaccinate {
        account: String,
        by: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        cost: u128,
    },
    Claim {
        account: String,
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        amount: u128,
    },
    SeasonEnd {
        #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
        at_ns: u64,
        reason: EndReason,
    },
}
