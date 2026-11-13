use crate::rng::Rng;
use crate::scenario::{Scenario, HOUR_NS, MILLI_NEAR, T0, TOKEN};
use contagion_core::{
    apply, init, is_expired, reserve_pool, vax_cost_now, Action, Config, EndReason, Event, Ledger, MemLedger, Status,
};
use serde_json::{json, Value};
use std::collections::VecDeque;

const RATE_MULT: [f64; 8] = [0.1, 0.3, 0.6, 1.0, 1.4, 2.0, 3.0, 5.0];
const INTEREST_MULT: [f64; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0];
const SAMPLE_HOURS: u64 = 6;
const ZEROS: usize = 3;
const SEEDER_RATE: f64 = 0.5;
const SEEDER_HOURS: u64 = 48;

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub hour: u64,
    pub hosts: u32,
    pub contagious: u32,
    pub recovered: u32,
    pub immune: u32,
    pub expired_waiting: u32,
    pub max_gen: u32,
    pub supply: f64,
    pub viral: f64,
    pub liquid: f64,
    pub burned: f64,
    pub catches: u64,
    pub culls: u64,
}

pub struct Outcome {
    pub scenario: Scenario,
    pub seed: u64,
    pub rows: Vec<Row>,
    pub summary: Value,
    pub ledger: MemLedger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Uninfected,
    Live,
    Done,
    Immune,
}

#[derive(Clone, Copy)]
struct Agent {
    state: State,
    rate: f64,
    interest_until: u64,
    diligent: bool,
    vaxxer: bool,
    deadline: u64,
    children: u32,
}

#[derive(Default)]
struct Tally {
    minted: u128,
    catches: u64,
    fees: u128,
    culls: u64,
    cut: u128,
}

impl Tally {
    fn add(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::Mint { amount, .. } => self.minted += amount,
                Event::Catch { fee, .. } => {
                    self.catches += 1;
                    self.fees += fee;
                }
                Event::Cull { cut, .. } => {
                    self.culls += 1;
                    self.cut += cut;
                }
                _ => {}
            }
        }
    }
}

struct Sybil {
    alts: Vec<String>,
    caught: usize,
    fees: u128,
}

fn catch_fees(events: &[Event]) -> u128 {
    events
        .iter()
        .map(|event| match event {
            Event::Catch { fee, .. } => *fee,
            _ => 0,
        })
        .sum()
}

fn mean(table: &[f64]) -> f64 {
    table.iter().sum::<f64>() / table.len() as f64
}

fn tokens(amount: u128) -> f64 {
    amount as f64 / TOKEN as f64
}

struct World<'a> {
    sc: &'a Scenario,
    cfg: &'a Config,
    rng: Rng,
    ledger: MemLedger,
    names: Vec<String>,
    doctors: Vec<String>,
    agents: Vec<Agent>,
    live: Vec<u32>,
    vaxxers: Vec<u32>,
    tally: Tally,
    sybil: Option<Sybil>,
}

impl<'a> World<'a> {
    fn new(sc: &'a Scenario, cfg: &'a Config, seed: u64) -> Self {
        let total = sc.pop + ZEROS;
        let names: Vec<String> = (0..total)
            .map(|i| {
                if i < sc.pop {
                    format!("u{i}")
                } else {
                    format!("zero{}", i - sc.pop)
                }
            })
            .collect();
        let doctors = (0..sc.doctors.max(1)).map(|i| format!("doc{i}")).collect();
        let mut ledger = MemLedger::default();
        init(&mut ledger, cfg, &names[sc.pop..], T0).expect("valid init");
        let mut world = World {
            sc,
            cfg,
            rng: Rng::new(seed),
            ledger,
            names,
            doctors,
            agents: vec![
                Agent {
                    state: State::Uninfected,
                    rate: 0.0,
                    interest_until: 0,
                    diligent: false,
                    vaxxer: false,
                    deadline: 0,
                    children: 0,
                };
                total
            ],
            live: Vec::new(),
            vaxxers: Vec::new(),
            tally: Tally::default(),
            sybil: None,
        };
        for i in sc.pop..total {
            world.infect(i, 0, T0);
            world.agents[i].rate = SEEDER_RATE;
            world.agents[i].interest_until = SEEDER_HOURS;
            world.agents[i].diligent = true;
            world.agents[i].vaxxer = false;
            world.live.push(i as u32);
        }
        world
    }

    fn infect(&mut self, index: usize, hour: u64, now: u64) {
        let rate = (self.sc.share_rate * self.rng.pick(&RATE_MULT) / mean(&RATE_MULT)).min(1.0);
        let span = (self.sc.interest_mean_h * self.rng.pick(&INTEREST_MULT) / mean(&INTEREST_MULT)).max(1.0) as u64;
        self.agents[index] = Agent {
            state: State::Live,
            rate,
            interest_until: hour + span,
            diligent: self.rng.chance(self.sc.diligence),
            vaxxer: self.rng.chance(self.sc.vaxxer_share),
            deadline: now + self.cfg.incubation_ns,
            children: 0,
        };
    }

    fn catch(&mut self, carrier: usize, target: usize, hour: u64, now: u64) -> bool {
        let action = Action::Catch {
            carrier: self.names[carrier].clone(),
            target: self.names[target].clone(),
            deposit: u128::MAX,
        };
        match apply(&mut self.ledger, self.cfg, action, now) {
            Ok(events) => {
                self.tally.add(&events);
                self.agents[carrier].children += 1;
                self.agents[carrier].deadline = now + self.cfg.incubation_ns;
                self.infect(target, hour, now);
                self.live.push(target as u32);
                true
            }
            Err(_) => false,
        }
    }

    fn settled(&mut self, index: usize) {
        self.agents[index].state = State::Done;
        if self.agents[index].vaxxer {
            self.vaxxers.push(index as u32);
        }
    }

    fn recover(&mut self, index: usize, now: u64) {
        let action = Action::Recover {
            account: self.names[index].clone(),
        };
        if let Ok(events) = apply(&mut self.ledger, self.cfg, action, now) {
            self.tally.add(&events);
            self.settled(index);
        }
    }

    fn cull(&mut self, index: usize, doctor: usize, now: u64) {
        let action = Action::Cull {
            account: self.names[index].clone(),
            caller: self.doctors[doctor].clone(),
        };
        if let Ok(events) = apply(&mut self.ledger, self.cfg, action, now) {
            self.tally.add(&events);
            self.settled(index);
        }
    }

    fn vaccinate(&mut self, target: usize, by: usize, now: u64) -> bool {
        let action = Action::Vaccinate {
            account: self.names[target].clone(),
            by: self.names[by].clone(),
        };
        match apply(&mut self.ledger, self.cfg, action, now) {
            Ok(events) => {
                self.tally.add(&events);
                self.agents[target].state = State::Immune;
                true
            }
            Err(_) => false,
        }
    }

    fn attack(&mut self, now: u64) {
        let alts: Vec<String> = (0..self.sc.sybil_alts).map(|i| format!("sy{i}")).collect();
        let entry = self.sc.pop;
        let mut caught = 0;
        let mut fees = 0u128;
        let first = Action::Catch {
            carrier: self.names[entry].clone(),
            target: alts[0].clone(),
            deposit: u128::MAX,
        };
        if let Ok(events) = apply(&mut self.ledger, self.cfg, first, now) {
            fees += catch_fees(&events);
            self.tally.add(&events);
            self.agents[entry].children += 1;
            self.agents[entry].deadline = now + self.cfg.incubation_ns;
            caught = 1;
        }
        let mut queue = VecDeque::from([0usize]);
        let mut next = 1;
        while let Some(carrier) = queue.pop_front() {
            for _ in 0..self.cfg.fanout {
                if next >= alts.len() || caught == 0 {
                    break;
                }
                let action = Action::Catch {
                    carrier: alts[carrier].clone(),
                    target: alts[next].clone(),
                    deposit: u128::MAX,
                };
                if let Ok(events) = apply(&mut self.ledger, self.cfg, action, now) {
                    fees += catch_fees(&events);
                    self.tally.add(&events);
                    caught += 1;
                    queue.push_back(next);
                }
                next += 1;
            }
        }
        for alt in alts.iter().take(next) {
            let action = Action::Recover { account: alt.clone() };
            if let Ok(events) = apply(&mut self.ledger, self.cfg, action, now) {
                self.tally.add(&events);
            }
        }
        self.sybil = Some(Sybil { alts, caught, fees });
    }

    fn spread(&mut self, hour: u64, now: u64, season_hours: u64) {
        let count = self.live.len();
        for k in 0..count {
            let a = self.live[k] as usize;
            let agent = self.agents[a];
            if agent.state != State::Live || now >= agent.deadline {
                continue;
            }
            let ending = agent.interest_until <= hour
                || agent.children >= self.cfg.fanout
                || hour + 24 >= season_hours;
            if ending {
                if agent.diligent {
                    self.recover(a, now);
                }
                continue;
            }
            if self.rng.chance(agent.rate) {
                let target = self.rng.below(self.sc.pop);
                if self.agents[target].state == State::Uninfected {
                    self.catch(a, target, hour, now);
                }
            }
        }
    }

    fn doctors_round(&mut self, now: u64) {
        let count = self.live.len();
        for k in 0..count {
            let a = self.live[k] as usize;
            let agent = self.agents[a];
            if agent.state == State::Live && now >= agent.deadline && self.rng.chance(self.sc.cull_prob) {
                let doctor = self.rng.below(self.doctors.len());
                self.cull(a, doctor, now);
            }
        }
    }

    fn vaccinate_round(&mut self, now: u64) {
        if self.sc.vaxxer_share <= 0.0 {
            return;
        }
        for k in 0..self.vaxxers.len() {
            let by = self.vaxxers[k] as usize;
            for _ in 0..self.sc.vax_per_hour {
                let liquid = self.ledger.accounts.get(&self.names[by]).map_or(0, |a| a.liquid);
                let cost = vax_cost_now(self.cfg, &self.ledger.globals).unwrap_or(u128::MAX);
                if liquid < cost {
                    break;
                }
                let mut target = None;
                for _ in 0..20 {
                    let t = self.rng.below(self.sc.pop);
                    if self.agents[t].state == State::Uninfected {
                        target = Some(t);
                        break;
                    }
                }
                let Some(t) = target else { break };
                if !self.vaccinate(t, by, now) || self.ledger.globals.ended_at.is_some() {
                    break;
                }
            }
        }
    }

    fn sample(&self, hour: u64, now: u64) -> Row {
        let g = &self.ledger.globals;
        let (mut viral, mut liquid) = (0u128, 0u128);
        let (mut contagious, mut recovered, mut expired) = (0u32, 0u32, 0u32);
        for account in self.ledger.accounts.values() {
            viral += account.viral;
            liquid += account.liquid;
            match account.status {
                Status::Contagious => {
                    contagious += 1;
                    if is_expired(account, self.cfg, g, now) {
                        expired += 1;
                    }
                }
                Status::Recovered => recovered += 1,
                _ => {}
            }
        }
        Row {
            hour,
            hosts: g.infected,
            contagious,
            recovered,
            immune: g.immune,
            expired_waiting: expired,
            max_gen: g.max_gen,
            supply: tokens(g.supply),
            viral: tokens(viral),
            liquid: tokens(liquid),
            burned: tokens(g.burned),
            catches: self.tally.catches,
            culls: self.tally.culls,
        }
    }

    fn sweep(&mut self, now: u64) -> u32 {
        let stragglers: Vec<usize> = self
            .live
            .iter()
            .map(|&a| a as usize)
            .filter(|&a| self.agents[a].state == State::Live)
            .collect();
        let left = stragglers.len() as u32;
        for a in stragglers {
            self.cull(a, 0, now);
        }
        let claimants: Vec<String> = self
            .ledger
            .accounts
            .iter()
            .filter(|(_, account)| account.points > 0)
            .map(|(name, _)| name.clone())
            .collect();
        for name in claimants {
            if let Ok(events) = apply(&mut self.ledger, self.cfg, Action::Claim { account: name }, now) {
                self.tally.add(&events);
            }
        }
        left
    }
}

fn first_day(rows: &[Row], hosts: u32) -> Value {
    rows.iter()
        .find(|row| row.hosts >= hosts)
        .map_or(Value::Null, |row| json!(row.hour as f64 / 24.0))
}

pub fn run(sc: &Scenario, cfg: &Config, seed: u64) -> Outcome {
    let mut world = World::new(sc, cfg, seed);
    let season_hours = cfg.season_max_ns / HOUR_NS;
    let mut rows = Vec::new();
    let mut hour = 0;
    let mut now = T0;

    while hour < season_hours {
        hour += 1;
        now = T0 + hour * HOUR_NS;
        if hour == 1 && sc.sybil_alts > 0 {
            world.attack(now);
        }
        world.spread(hour, now, season_hours);
        world.doctors_round(now);
        world.vaccinate_round(now);
        if hour % SAMPLE_HOURS == 0 {
            rows.push(world.sample(hour, now));
        }
        if world.ledger.globals.ended_at.is_some() {
            break;
        }
    }

    let end_hour = hour;
    let left = world.sweep(now + 1);
    rows.push(world.sample(end_hour + 1, now + 1));
    world.ledger.check_invariants().expect("ledger invariants hold");

    let g = world.ledger.globals.clone();
    let (end_day, end_reason) = match (g.ended_at, g.end_reason) {
        (Some(at), Some(reason)) => (
            (at - T0) as f64 / (24 * HOUR_NS) as f64,
            match reason {
                EndReason::Herd => "herd",
                EndReason::Time => "time",
            },
        ),
        _ => (season_hours as f64 / 24.0, "time"),
    };

    let mut balances: Vec<u128> = world.ledger.accounts.values().map(|a| a.balance()).collect();
    balances.sort_unstable_by(|a, b| b.cmp(a));
    let top10: u128 = balances.iter().take(10).sum();
    let recovered = world
        .ledger
        .accounts
        .values()
        .filter(|a| a.status == Status::Recovered)
        .count();

    let infection_minted = g.infection_minted;
    let pool = reserve_pool(cfg, &g).expect("pool fits");
    let near = |yocto: u128| yocto as f64 / (1000 * MILLI_NEAR) as f64;
    let per_near = |amount: u128, fees: u128| if fees > 0 { tokens(amount) / near(fees) } else { 0.0 };
    let reserve_share = |points: u64| pool * points as u128 / g.total_points.max(1) as u128;
    let posted_rate = if cfg.fee_per_unit > 0 {
        (1000 * MILLI_NEAR) as f64 / cfg.fee_per_unit as f64 / TOKEN as f64
    } else {
        0.0
    };
    let reserve_top3: u128 = {
        let mut paid: Vec<u128> = world
            .ledger
            .accounts
            .values()
            .filter(|a| a.claimed)
            .map(|a| reserve_share(a.points))
            .collect();
        paid.sort_unstable_by(|a, b| b.cmp(a));
        paid.iter().take(3).sum()
    };
    let sybil = world.sybil.as_ref().map(|s| {
        let held: u128 = s.alts.iter().map(|n| world.ledger.account(n).balance()).sum();
        let points: u64 = s.alts.iter().map(|n| world.ledger.account(n).points).sum();
        let from_reserve: u128 = s
            .alts
            .iter()
            .map(|n| world.ledger.account(n))
            .filter(|a| a.claimed)
            .map(|a| reserve_share(a.points))
            .sum();
        let from_infection = held - from_reserve;
        let sybil_rate = per_near(from_infection, s.fees);
        let honest_rate = per_near(
            infection_minted.saturating_sub(from_infection),
            world.tally.fees.saturating_sub(s.fees),
        );
        json!({
            "alts": s.alts.len(),
            "caught": s.caught,
            "fees_near": near(s.fees),
            "tokens": tokens(held),
            "tokens_from_infection": tokens(from_infection),
            "tokens_from_reserve": tokens(from_reserve),
            "share_of_supply_pct": held as f64 * 100.0 / g.supply.max(1) as f64,
            "share_of_infection_mint_pct": from_infection as f64 * 100.0 / infection_minted.max(1) as f64,
            "share_of_fees_pct": s.fees as f64 * 100.0 / world.tally.fees.max(1) as f64,
            "share_of_reserve_pct": points as f64 * 100.0 / g.total_points.max(1) as f64,
            "tokens_per_near": sybil_rate,
            "honest_tokens_per_near": honest_rate,
            "posted_tokens_per_near": posted_rate,
            "cost_advantage_vs_honest_avg": if honest_rate > 0.0 { json!(sybil_rate / honest_rate) } else { Value::Null },
            "cost_advantage_vs_posted": if posted_rate > 0.0 { json!(sybil_rate / posted_rate) } else { Value::Null },
        })
    });

    let doctor_balance: u128 = world.doctors.iter().map(|n| world.ledger.account(n).balance()).sum();
    let summary = json!({
        "scenario": sc.name,
        "seed": seed,
        "params": {
            "pop": sc.pop,
            "share_rate": sc.share_rate,
            "interest_mean_h": sc.interest_mean_h,
            "diligence": sc.diligence,
            "cull_prob": sc.cull_prob,
            "doctors": sc.doctors,
            "vaxxer_share": sc.vaxxer_share,
            "vax_per_hour": sc.vax_per_hour,
            "sybil_alts": sc.sybil_alts,
        },
        "config": serde_json::to_value(cfg).expect("config serializes"),
        "season": { "end_day": end_day, "end_reason": end_reason },
        "outcome": {
            "hosts": g.infected,
            "recovered": recovered,
            "immune": g.immune,
            "max_gen": g.max_gen,
            "carriers_unsettled_at_end": left,
            "minted": tokens(world.tally.minted),
            "infection_minted": tokens(infection_minted),
            "burned": tokens(g.burned),
            "supply": tokens(g.supply),
            "burned_pct_of_infection_mint": g.burned as f64 * 100.0 / infection_minted.max(1) as f64,
            "reserve_to_top3_pct": reserve_top3 as f64 * 100.0 / g.reserve_paid.max(1) as f64,
            "reserve_pool": tokens(pool),
            "reserve_paid": tokens(g.reserve_paid),
            "reserve_pct_of_supply": g.reserve_paid as f64 * 100.0 / g.supply.max(1) as f64,
            "fees_near": near(world.tally.fees),
            "tokens_per_near": per_near(infection_minted, world.tally.fees),
            "top10_holdings_pct": top10 as f64 * 100.0 / g.supply.max(1) as f64,
            "plague_doctor_cut": tokens(world.tally.cut),
            "plague_doctor_balance": tokens(doctor_balance),
            "culls": world.tally.culls,
            "catches": world.tally.catches,
            "days_to_100_hosts": first_day(&rows, 100),
            "days_to_1000_hosts": first_day(&rows, 1000),
        },
        "sybil": sybil,
    });

    Outcome {
        scenario: sc.clone(),
        seed,
        rows,
        summary,
        ledger: world.ledger,
    }
}
