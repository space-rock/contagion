use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Uninfected,
    Contagious,
    Recovered,
    Immune,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    Herd,
    Time,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
pub struct Account {
    pub status: Status,
    pub parent: Option<String>,
    pub gen: u32,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub mint: u128,
    pub children: u32,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub last_spread_ns: u64,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub viral: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub liquid: u128,
    pub points: u64,
    pub claimed: bool,
}

impl Account {
    pub fn balance(&self) -> u128 {
        self.viral + self.liquid
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
pub struct Globals {
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub start_ns: u64,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub supply: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub burned: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub fees_locked: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub infection_minted: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub reserve_paid: u128,
    pub infected: u32,
    pub immune: u32,
    pub max_gen: u32,
    pub total_points: u64,
    #[serde(with = "crate::strnum::opt")]
    #[cfg_attr(feature = "abi", schemars(with = "Option<String>"))]
    pub ended_at: Option<u64>,
    pub end_reason: Option<EndReason>,
}
