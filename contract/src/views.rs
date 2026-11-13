use contagion_core::{child_mint, is_expired, required_fee, Account, Config, EndReason, Event, Globals, Status};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near};

#[near(serializers = [json])]
pub struct HostView {
    pub account_id: String,
    pub status: Status,
    pub parent: Option<String>,
    pub gen: u32,
    pub mint: U128,
    pub children: u32,
    pub fanout_left: u32,
    pub viral: U128,
    pub liquid: U128,
    pub points: u64,
    pub claimed: bool,
    pub last_spread_ns: U64,
    pub deadline_ns: U64,
    pub expired: bool,
}

#[near(serializers = [json])]
pub struct QuoteView {
    pub can_catch: bool,
    pub reason: Option<String>,
    pub gen: u32,
    pub mint: U128,
    pub fee: U128,
}

#[near(serializers = [json])]
pub struct StatsView {
    pub globals: Globals,
    pub now_ns: U64,
    pub season_over: bool,
    pub season_ends_ns: U64,
    pub reserve_pool: U128,
    pub vax_cost: U128,
    pub accounts: u64,
    pub events: u64,
    pub storage_usage: u64,
    pub balance: U128,
}

#[near(serializers = [json])]
pub struct LogView {
    pub index: u64,
    pub at_ns: U64,
    pub event: Event,
}

pub fn season_deadline(cfg: &Config, globals: &Globals) -> u64 {
    globals.start_ns.saturating_add(cfg.season_max_ns)
}

pub fn effective_globals(cfg: &Config, globals: &Globals, now: u64) -> Globals {
    let mut effective = globals.clone();
    let deadline = season_deadline(cfg, globals);
    if effective.ended_at.is_none() && now >= deadline {
        effective.ended_at = Some(deadline);
        effective.end_reason = Some(EndReason::Time);
    }
    effective
}

pub fn host_view(id: &str, account: &Account, cfg: &Config, globals: &Globals) -> HostView {
    let now = env::block_timestamp();
    let effective = effective_globals(cfg, globals, now);
    HostView {
        account_id: id.to_string(),
        status: account.status,
        parent: account.parent.clone(),
        gen: account.gen,
        mint: account.mint.into(),
        children: account.children,
        fanout_left: if account.status == Status::Contagious {
            cfg.fanout.saturating_sub(account.children)
        } else {
            0
        },
        viral: account.viral.into(),
        liquid: account.liquid.into(),
        points: account.points,
        claimed: account.claimed,
        last_spread_ns: account.last_spread_ns.into(),
        deadline_ns: account.last_spread_ns.saturating_add(cfg.incubation_ns).into(),
        expired: is_expired(account, cfg, &effective, now),
    }
}

pub fn quote(cfg: &Config, globals: &Globals, carrier: &Account) -> QuoteView {
    let now = env::block_timestamp();
    let effective = effective_globals(cfg, globals, now);
    let mint = child_mint(cfg, carrier.mint).unwrap_or(0);
    let fee = required_fee(cfg, mint).unwrap_or(0);
    let reason = if effective.ended_at.is_some() {
        Some("the season is over")
    } else if carrier.status != Status::Contagious {
        Some("carrier is not contagious")
    } else if is_expired(carrier, cfg, &effective, now) {
        Some("carrier incubation clock expired")
    } else if carrier.children >= cfg.fanout {
        Some("carrier reached its fanout cap")
    } else if mint == 0 {
        Some("mint decayed to zero")
    } else {
        None
    };
    QuoteView {
        can_catch: reason.is_none(),
        reason: reason.map(str::to_string),
        gen: carrier.gen + 1,
        mint: mint.into(),
        fee: fee.into(),
    }
}
