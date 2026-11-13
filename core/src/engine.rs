use crate::account::{Account, EndReason, Globals, Status};
use crate::config::Config;
use crate::error::Error;
use crate::event::Event;
use crate::ledger::Ledger;
use crate::{BPS, POINTS_CARRIER, POINTS_GRANDPARENT, POINTS_PARENT, VAX_STEP_BPS};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Catch {
        carrier: String,
        target: String,
        deposit: u128,
    },
    Recover {
        account: String,
    },
    Cull {
        account: String,
        caller: String,
    },
    Vaccinate {
        account: String,
        by: String,
    },
    Claim {
        account: String,
    },
    Transfer {
        from: String,
        to: String,
        amount: u128,
    },
}

pub fn is_named(id: &str) -> bool {
    let near_implicit = id.len() == 64 && id.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    let eth_implicit = id.len() == 42 && id.starts_with("0x") && id[2..].bytes().all(|b| b.is_ascii_hexdigit());
    !id.is_empty() && !near_implicit && !eth_implicit
}

pub fn is_expired(account: &Account, cfg: &Config, globals: &Globals, now: u64) -> bool {
    account.status == Status::Contagious
        && (globals.ended_at.is_some() || now >= account.last_spread_ns.saturating_add(cfg.incubation_ns))
}

fn share(amount: u128, bps: u32) -> Result<u128, Error> {
    amount
        .checked_mul(bps as u128)
        .map(|scaled| scaled / BPS)
        .ok_or(Error::Overflow)
}

pub fn child_mint(cfg: &Config, carrier_mint: u128) -> Result<u128, Error> {
    share(carrier_mint, cfg.decay_bps)
}

pub fn required_fee(cfg: &Config, mint: u128) -> Result<u128, Error> {
    let proportional = mint.checked_mul(cfg.fee_per_unit).ok_or(Error::Overflow)?;
    Ok(proportional.max(cfg.min_fee))
}

pub fn reserve_pool(cfg: &Config, globals: &Globals) -> Result<u128, Error> {
    share(globals.infection_minted, cfg.reserve_bps)
}

pub fn vax_cost_now(cfg: &Config, globals: &Globals) -> Result<u128, Error> {
    let factor = VAX_STEP_BPS
        .checked_mul(globals.immune as u128)
        .and_then(|step| step.checked_add(BPS))
        .ok_or(Error::Overflow)?;
    cfg.vax_cost
        .checked_mul(factor)
        .map(|scaled| scaled / BPS)
        .ok_or(Error::Overflow)
}

pub fn init<L: Ledger>(ledger: &mut L, cfg: &Config, zeros: &[String], now: u64) -> Result<(), String> {
    cfg.validate()?;
    if zeros.is_empty() {
        return Err("at least one patient zero is required".into());
    }
    let mut seen = BTreeSet::new();
    for zero in zeros {
        if !is_named(zero) {
            return Err(format!("patient zero {zero} must be a named account"));
        }
        if !seen.insert(zero) {
            return Err(format!("patient zero {zero} listed twice"));
        }
    }
    let mut globals = Globals {
        start_ns: now,
        ..Default::default()
    };
    for zero in zeros {
        ledger.put(
            zero,
            Account {
                status: Status::Contagious,
                mint: cfg.base,
                last_spread_ns: now,
                ..Default::default()
            },
        );
        globals.infected += 1;
    }
    ledger.set_globals(globals);
    Ok(())
}

pub fn apply<L: Ledger>(ledger: &mut L, cfg: &Config, action: Action, now: u64) -> Result<Vec<Event>, Error> {
    let mut events = Vec::new();
    sync_season(ledger, cfg, now, &mut events);
    match action {
        Action::Catch {
            carrier,
            target,
            deposit,
        } => catch(ledger, cfg, &carrier, &target, deposit, now, &mut events)?,
        Action::Recover { account } => recover(ledger, cfg, &account, now, &mut events)?,
        Action::Cull { account, caller } => cull(ledger, cfg, &account, &caller, now, &mut events)?,
        Action::Vaccinate { account, by } => vaccinate(ledger, cfg, &account, &by, now, &mut events)?,
        Action::Claim { account } => claim(ledger, cfg, &account, &mut events)?,
        Action::Transfer { from, to, amount } => transfer(ledger, &from, &to, amount, &mut events)?,
    }
    Ok(events)
}

fn sync_season<L: Ledger>(ledger: &mut L, cfg: &Config, now: u64, events: &mut Vec<Event>) {
    let mut globals = ledger.globals();
    if globals.ended_at.is_some() {
        return;
    }
    let deadline = globals.start_ns.saturating_add(cfg.season_max_ns);
    if now >= deadline {
        globals.ended_at = Some(deadline);
        globals.end_reason = Some(EndReason::Time);
        ledger.set_globals(globals);
        events.push(Event::SeasonEnd {
            at_ns: deadline,
            reason: EndReason::Time,
        });
    }
}

fn catch<L: Ledger>(
    ledger: &mut L,
    cfg: &Config,
    carrier_id: &str,
    target_id: &str,
    deposit: u128,
    now: u64,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    let mut globals = ledger.globals();
    if globals.ended_at.is_some() {
        return Err(Error::SeasonOver);
    }
    if carrier_id == target_id {
        return Err(Error::SelfCatch);
    }
    if !is_named(target_id) {
        return Err(Error::InvalidName);
    }
    let mut carrier = ledger.account(carrier_id);
    if carrier.status != Status::Contagious {
        return Err(Error::CarrierNotContagious);
    }
    if is_expired(&carrier, cfg, &globals, now) {
        return Err(Error::CarrierExpired);
    }
    if carrier.children >= cfg.fanout {
        return Err(Error::FanoutReached);
    }
    let target = ledger.account(target_id);
    if target.status != Status::Uninfected {
        return Err(Error::TargetNotUninfected);
    }
    let mint = child_mint(cfg, carrier.mint)?;
    if mint == 0 {
        return Err(Error::MintExhausted);
    }
    let fee = required_fee(cfg, mint)?;
    if deposit < fee {
        return Err(Error::DepositTooLow);
    }
    let bonus = share(mint, cfg.bonus_bps)?;
    let minted = mint.checked_add(bonus).ok_or(Error::Overflow)?;
    let supply = globals.supply.checked_add(minted).ok_or(Error::Overflow)?;

    let gen = carrier.gen + 1;
    let mut awarded = POINTS_CARRIER;
    carrier.points += POINTS_CARRIER;
    if let Some(parent_id) = carrier.parent.clone() {
        let mut parent = ledger.account(&parent_id);
        parent.points += POINTS_PARENT;
        awarded += POINTS_PARENT;
        if let Some(grandparent_id) = parent.parent.clone() {
            let mut grandparent = ledger.account(&grandparent_id);
            grandparent.points += POINTS_GRANDPARENT;
            awarded += POINTS_GRANDPARENT;
            ledger.put(&grandparent_id, grandparent);
        }
        ledger.put(&parent_id, parent);
    }
    carrier.children += 1;
    carrier.last_spread_ns = now;
    carrier.viral += bonus;
    ledger.put(carrier_id, carrier);

    ledger.put(
        target_id,
        Account {
            status: Status::Contagious,
            parent: Some(carrier_id.to_string()),
            gen,
            mint,
            last_spread_ns: now,
            viral: mint,
            ..Default::default()
        },
    );

    globals.infected += 1;
    globals.max_gen = globals.max_gen.max(gen);
    globals.supply = supply;
    globals.infection_minted += minted;
    globals.fees_locked += fee;
    globals.total_points += awarded;
    ledger.set_globals(globals);

    events.push(Event::Catch {
        carrier: carrier_id.to_string(),
        target: target_id.to_string(),
        gen,
        mint,
        bonus,
        fee,
    });
    events.push(Event::Mint {
        account: target_id.to_string(),
        amount: mint,
    });
    if bonus > 0 {
        events.push(Event::Mint {
            account: carrier_id.to_string(),
            amount: bonus,
        });
    }
    Ok(())
}

fn recover<L: Ledger>(
    ledger: &mut L,
    cfg: &Config,
    id: &str,
    now: u64,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    let mut globals = ledger.globals();
    let mut account = ledger.account(id);
    if account.status != Status::Contagious {
        return Err(Error::NotContagious);
    }
    let expired = is_expired(&account, cfg, &globals, now);
    let bps = if expired {
        cfg.burn_expired_bps
    } else {
        cfg.burn_voluntary_bps
    };
    let burn = share(account.viral, bps)?;
    account.liquid += account.viral - burn;
    account.viral = 0;
    account.status = Status::Recovered;
    globals.supply -= burn;
    globals.burned += burn;
    ledger.put(id, account);
    ledger.set_globals(globals);

    if burn > 0 {
        events.push(Event::Burn {
            account: id.to_string(),
            amount: burn,
        });
    }
    events.push(Event::Recover {
        account: id.to_string(),
        burned: burn,
        expired,
    });
    Ok(())
}

fn cull<L: Ledger>(
    ledger: &mut L,
    cfg: &Config,
    id: &str,
    caller: &str,
    now: u64,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    let mut globals = ledger.globals();
    let mut account = ledger.account(id);
    if account.status != Status::Contagious {
        return Err(Error::NotContagious);
    }
    if !is_expired(&account, cfg, &globals, now) {
        return Err(Error::NotExpired);
    }
    let burn = share(account.viral, cfg.burn_expired_bps)?;
    let cut = if caller == id { 0 } else { share(burn, cfg.cull_cut_bps)? };
    let destroyed = burn - cut;
    account.liquid += account.viral - burn;
    account.viral = 0;
    account.status = Status::Recovered;
    if cut > 0 {
        let mut doctor = ledger.account(caller);
        doctor.liquid += cut;
        ledger.put(caller, doctor);
    }
    globals.supply -= destroyed;
    globals.burned += destroyed;
    ledger.put(id, account);
    ledger.set_globals(globals);

    if cut > 0 {
        events.push(Event::Transfer {
            from: id.to_string(),
            to: caller.to_string(),
            amount: cut,
        });
    }
    if destroyed > 0 {
        events.push(Event::Burn {
            account: id.to_string(),
            amount: destroyed,
        });
    }
    events.push(Event::Cull {
        account: id.to_string(),
        caller: caller.to_string(),
        burned: destroyed,
        cut,
    });
    Ok(())
}

fn vaccinate<L: Ledger>(
    ledger: &mut L,
    cfg: &Config,
    id: &str,
    by: &str,
    now: u64,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    let mut globals = ledger.globals();
    if globals.ended_at.is_some() {
        return Err(Error::SeasonOver);
    }
    let mut target = ledger.account(id);
    if target.status != Status::Uninfected {
        return Err(Error::NotVaccinable);
    }
    let cost = vax_cost_now(cfg, &globals)?;
    if id == by {
        if target.liquid < cost {
            return Err(Error::InsufficientLiquid);
        }
        target.liquid -= cost;
    } else {
        let mut payer = ledger.account(by);
        if payer.liquid < cost {
            return Err(Error::InsufficientLiquid);
        }
        payer.liquid -= cost;
        ledger.put(by, payer);
    }
    target.status = Status::Immune;
    ledger.put(id, target);

    globals.supply -= cost;
    globals.burned += cost;
    globals.immune += 1;

    if cost > 0 {
        events.push(Event::Burn {
            account: by.to_string(),
            amount: cost,
        });
    }
    events.push(Event::Vaccinate {
        account: id.to_string(),
        by: by.to_string(),
        cost,
    });

    let herd = globals.infected >= cfg.min_hosts
        && (globals.immune as u128) * BPS >= (cfg.herd_bps as u128) * (globals.infected as u128);
    if herd {
        globals.ended_at = Some(now);
        globals.end_reason = Some(EndReason::Herd);
        events.push(Event::SeasonEnd {
            at_ns: now,
            reason: EndReason::Herd,
        });
    }
    ledger.set_globals(globals);
    Ok(())
}

fn claim<L: Ledger>(ledger: &mut L, cfg: &Config, id: &str, events: &mut Vec<Event>) -> Result<(), Error> {
    let mut globals = ledger.globals();
    if globals.ended_at.is_none() {
        return Err(Error::SeasonOpen);
    }
    let mut account = ledger.account(id);
    if account.claimed || account.points == 0 || globals.total_points == 0 {
        return Err(Error::NothingToClaim);
    }
    let amount = reserve_pool(cfg, &globals)?
        .checked_mul(account.points as u128)
        .ok_or(Error::Overflow)?
        / globals.total_points as u128;
    if amount == 0 {
        return Err(Error::NothingToClaim);
    }
    account.claimed = true;
    account.liquid += amount;
    globals.supply += amount;
    globals.reserve_paid += amount;
    ledger.put(id, account);
    ledger.set_globals(globals);

    events.push(Event::Mint {
        account: id.to_string(),
        amount,
    });
    events.push(Event::Claim {
        account: id.to_string(),
        amount,
    });
    Ok(())
}

fn transfer<L: Ledger>(
    ledger: &mut L,
    from: &str,
    to: &str,
    amount: u128,
    events: &mut Vec<Event>,
) -> Result<(), Error> {
    if amount == 0 {
        return Err(Error::ZeroAmount);
    }
    if from == to {
        return Err(Error::SelfTransfer);
    }
    let mut sender = ledger.account(from);
    if sender.liquid < amount {
        return Err(Error::InsufficientLiquid);
    }
    let mut receiver = ledger.account(to);
    sender.liquid -= amount;
    receiver.liquid += amount;
    ledger.put(from, sender);
    ledger.put(to, receiver);
    events.push(Event::Transfer {
        from: from.to_string(),
        to: to.to_string(),
        amount,
    });
    Ok(())
}
