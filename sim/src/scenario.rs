use contagion_core::Config;

pub const TOKEN: u128 = 1_000_000_000_000_000_000;
pub const MILLI_NEAR: u128 = 1_000_000_000_000_000_000_000;
pub const HOUR_NS: u64 = 3_600_000_000_000;
pub const T0: u64 = 1_000 * HOUR_NS;

#[derive(Clone, Debug)]
pub struct Scenario {
    pub name: &'static str,
    pub pop: usize,
    pub share_rate: f64,
    pub interest_mean_h: f64,
    pub diligence: f64,
    pub cull_prob: f64,
    pub doctors: usize,
    pub vaxxer_share: f64,
    pub vax_per_hour: u32,
    pub sybil_alts: usize,
}

pub fn base_config() -> Config {
    Config {
        base: 1_000 * TOKEN,
        decay_bps: 6_000,
        bonus_bps: 3_000,
        fanout: 5,
        incubation_ns: 72 * HOUR_NS,
        burn_expired_bps: 5_000,
        burn_voluntary_bps: 1_000,
        cull_cut_bps: 1_000,
        vax_cost: 20 * TOKEN,
        herd_bps: 3_000,
        min_hosts: 1_000,
        season_max_ns: 30 * 24 * HOUR_NS,
        min_fee: 10 * MILLI_NEAR,
        fee_per_unit: 500,
        reserve_bps: 2_000,
    }
}

fn organic() -> Scenario {
    Scenario {
        name: "viral",
        pop: 20_000,
        share_rate: 0.08,
        interest_mean_h: 36.0,
        diligence: 0.5,
        cull_prob: 0.25,
        doctors: 5,
        vaxxer_share: 0.02,
        vax_per_hour: 1,
        sybil_alts: 0,
    }
}

pub fn presets() -> Vec<Scenario> {
    vec![
        organic(),
        Scenario {
            name: "stalled",
            share_rate: 0.015,
            ..organic()
        },
        Scenario {
            name: "sybil",
            sybil_alts: 500,
            ..organic()
        },
        Scenario {
            name: "vax_faction",
            vaxxer_share: 0.25,
            vax_per_hour: 3,
            ..organic()
        },
        Scenario {
            name: "whale_culler",
            doctors: 1,
            cull_prob: 1.0,
            ..organic()
        },
    ]
}

fn parse<T: std::str::FromStr>(key: &str, value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("bad value {value} for {key}"))
}

pub fn apply_override(cfg: &mut Config, key: &str, value: &str) -> Result<(), String> {
    match key {
        "base_tokens" => cfg.base = parse::<u128>(key, value)? * TOKEN,
        "vax_cost_tokens" => cfg.vax_cost = parse::<u128>(key, value)? * TOKEN,
        "min_fee_milli_near" => cfg.min_fee = parse::<u128>(key, value)? * MILLI_NEAR,
        "fee_per_unit" => cfg.fee_per_unit = parse(key, value)?,
        "reserve_bps" => cfg.reserve_bps = parse(key, value)?,
        "decay_bps" => cfg.decay_bps = parse(key, value)?,
        "bonus_bps" => cfg.bonus_bps = parse(key, value)?,
        "burn_expired_bps" => cfg.burn_expired_bps = parse(key, value)?,
        "burn_voluntary_bps" => cfg.burn_voluntary_bps = parse(key, value)?,
        "cull_cut_bps" => cfg.cull_cut_bps = parse(key, value)?,
        "herd_bps" => cfg.herd_bps = parse(key, value)?,
        "fanout" => cfg.fanout = parse(key, value)?,
        "min_hosts" => cfg.min_hosts = parse(key, value)?,
        "incubation_h" => cfg.incubation_ns = parse::<u64>(key, value)? * HOUR_NS,
        "season_days" => cfg.season_max_ns = parse::<u64>(key, value)? * 24 * HOUR_NS,
        _ => return Err(format!("unknown key {key}")),
    }
    cfg.validate()
}
