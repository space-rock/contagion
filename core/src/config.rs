use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema, borsh::BorshSchema))]
pub struct Config {
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub base: u128,
    pub decay_bps: u32,
    pub bonus_bps: u32,
    pub fanout: u32,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub incubation_ns: u64,
    pub burn_expired_bps: u32,
    pub burn_voluntary_bps: u32,
    pub cull_cut_bps: u32,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub vax_cost: u128,
    pub herd_bps: u32,
    pub min_hosts: u32,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub season_max_ns: u64,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub min_fee: u128,
    #[serde(with = "crate::strnum")]
    #[cfg_attr(feature = "abi", schemars(with = "String"))]
    pub fee_per_unit: u128,
    pub reserve_bps: u32,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        let bps = |name: &str, value: u32, max: u32| {
            if value > max {
                Err(format!("{name} must be at most {max}, got {value}"))
            } else {
                Ok(())
            }
        };
        if self.base == 0 {
            return Err("base must be above zero".into());
        }
        if self.decay_bps == 0 || self.decay_bps >= 10_000 {
            return Err("decay_bps must be in 1..=9999 so mint shrinks every generation".into());
        }
        bps("bonus_bps", self.bonus_bps, 10_000)?;
        bps("burn_expired_bps", self.burn_expired_bps, 10_000)?;
        bps("burn_voluntary_bps", self.burn_voluntary_bps, 10_000)?;
        bps("cull_cut_bps", self.cull_cut_bps, 10_000)?;
        bps("reserve_bps", self.reserve_bps, 10_000)?;
        if self.burn_voluntary_bps > self.burn_expired_bps {
            return Err("burn_voluntary_bps must not exceed burn_expired_bps so settling on time is never worse".into());
        }
        if self.vax_cost == 0 {
            return Err("vax_cost must be above zero so vaccination is never free".into());
        }
        if self.herd_bps == 0 || self.herd_bps > 10_000 {
            return Err("herd_bps must be in 1..=10000".into());
        }
        if self.fanout == 0 {
            return Err("fanout must be at least 1".into());
        }
        if self.min_hosts == 0 {
            return Err("min_hosts must be at least 1".into());
        }
        if self.incubation_ns == 0 {
            return Err("incubation_ns must be above zero".into());
        }
        if self.season_max_ns <= self.incubation_ns {
            return Err("season_max_ns must be longer than incubation_ns".into());
        }
        Ok(())
    }

    pub fn reference() -> Self {
        Config {
            base: 1_000,
            decay_bps: 6_000,
            bonus_bps: 3_000,
            fanout: 5,
            incubation_ns: 100,
            burn_expired_bps: 5_000,
            burn_voluntary_bps: 1_000,
            cull_cut_bps: 1_000,
            vax_cost: 50,
            herd_bps: 5_000,
            min_hosts: 10,
            season_max_ns: 10_000,
            min_fee: 10,
            fee_per_unit: 1,
            reserve_bps: 5_000,
        }
    }
}
