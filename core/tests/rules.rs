use contagion_core::{
    apply, child_mint, init, is_named, required_fee, reserve_pool, vax_cost_now, Account, Action, Config, EndReason,
    Error, Event, Ledger, MemLedger, Status, BPS, VAX_STEP_BPS,
};

const T0: u64 = 1_000;
const DEPOSIT: u128 = 1_000_000;

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|name| name.to_string()).collect()
}

fn world_with(cfg: Config, zeros: &[&str]) -> (MemLedger, Config) {
    let mut ledger = MemLedger::default();
    init(&mut ledger, &cfg, &names(zeros), T0).unwrap();
    ledger.check_invariants().unwrap();
    (ledger, cfg)
}

fn world() -> (MemLedger, Config) {
    world_with(Config::reference(), &["zero"])
}

fn act(ledger: &mut MemLedger, cfg: &Config, action: Action, now: u64) -> Result<Vec<Event>, Error> {
    let result = apply(ledger, cfg, action, now);
    ledger.check_invariants().unwrap();
    result
}

fn catch(ledger: &mut MemLedger, cfg: &Config, carrier: &str, target: &str, now: u64) -> Result<Vec<Event>, Error> {
    act(
        ledger,
        cfg,
        Action::Catch {
            carrier: carrier.into(),
            target: target.into(),
            deposit: DEPOSIT,
        },
        now,
    )
}

fn recover(ledger: &mut MemLedger, cfg: &Config, id: &str, now: u64) -> Result<Vec<Event>, Error> {
    act(ledger, cfg, Action::Recover { account: id.into() }, now)
}

fn cull(ledger: &mut MemLedger, cfg: &Config, id: &str, caller: &str, now: u64) -> Result<Vec<Event>, Error> {
    act(
        ledger,
        cfg,
        Action::Cull {
            account: id.into(),
            caller: caller.into(),
        },
        now,
    )
}

fn vaccinate(ledger: &mut MemLedger, cfg: &Config, id: &str, by: &str, now: u64) -> Result<Vec<Event>, Error> {
    act(
        ledger,
        cfg,
        Action::Vaccinate {
            account: id.into(),
            by: by.into(),
        },
        now,
    )
}

fn claim(ledger: &mut MemLedger, cfg: &Config, id: &str, now: u64) -> Result<Vec<Event>, Error> {
    act(ledger, cfg, Action::Claim { account: id.into() }, now)
}

fn transfer(ledger: &mut MemLedger, cfg: &Config, from: &str, to: &str, amount: u128, now: u64) -> Result<Vec<Event>, Error> {
    act(
        ledger,
        cfg,
        Action::Transfer {
            from: from.into(),
            to: to.into(),
            amount,
        },
        now,
    )
}

fn get(ledger: &MemLedger, id: &str) -> Account {
    ledger.account(id)
}

#[test]
fn patient_zeros_start_contagious_with_no_balance() {
    let (ledger, cfg) = world_with(Config::reference(), &["zero", "zero2", "zero3"]);
    for id in ["zero", "zero2", "zero3"] {
        let a = get(&ledger, id);
        assert_eq!(a.status, Status::Contagious);
        assert_eq!(a.gen, 0);
        assert_eq!(a.mint, cfg.base);
        assert_eq!(a.balance(), 0);
        assert_eq!(a.last_spread_ns, T0);
    }
    assert_eq!(ledger.globals.infected, 3);
    assert_eq!(ledger.globals.start_ns, T0);
}

#[test]
fn catch_mints_decay_per_generation_and_pays_carrier_bonus() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let a = get(&l, "a");
    assert_eq!((a.status, a.gen, a.mint, a.viral), (Status::Contagious, 1, 600, 600));
    assert_eq!(a.parent.as_deref(), Some("zero"));
    assert_eq!(get(&l, "zero").viral, 180);
    assert_eq!(get(&l, "zero").children, 1);

    catch(&mut l, &cfg, "a", "b", T0).unwrap();
    let b = get(&l, "b");
    assert_eq!((b.gen, b.mint, b.viral), (2, 360, 360));
    assert_eq!(get(&l, "a").viral, 600 + 108);
    assert_eq!(l.globals.supply, 180 + 600 + 108 + 360);
    assert_eq!(l.globals.max_gen, 2);
    assert_eq!(l.globals.infection_minted, 180 + 600 + 108 + 360);
    assert_eq!(l.globals.fees_locked, 600 + 360);
}

#[test]
fn catch_emits_catch_and_mint_events() {
    let (mut l, cfg) = world();
    let events = catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(
        events,
        vec![
            Event::Catch {
                carrier: "zero".into(),
                target: "a".into(),
                gen: 1,
                mint: 600,
                bonus: 180,
                fee: 600
            },
            Event::Mint {
                account: "a".into(),
                amount: 600
            },
            Event::Mint {
                account: "zero".into(),
                amount: 180
            },
        ]
    );
}

#[test]
fn catch_rejects_invalid_requests() {
    let (mut l, cfg) = world();
    assert_eq!(catch(&mut l, &cfg, "zero", "zero", T0), Err(Error::SelfCatch));
    let implicit = "a".repeat(64);
    assert_eq!(catch(&mut l, &cfg, "zero", &implicit, T0), Err(Error::InvalidName));
    let eth = format!("0x{}", "ab".repeat(20));
    assert_eq!(catch(&mut l, &cfg, "zero", &eth, T0), Err(Error::InvalidName));
    assert_eq!(
        act(
            &mut l,
            &cfg,
            Action::Catch {
                carrier: "zero".into(),
                target: "a".into(),
                deposit: 599
            },
            T0
        ),
        Err(Error::DepositTooLow)
    );
    assert_eq!(catch(&mut l, &cfg, "nobody", "a", T0), Err(Error::CarrierNotContagious));
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(catch(&mut l, &cfg, "zero", "a", T0), Err(Error::TargetNotUninfected));
    assert_eq!(catch(&mut l, &cfg, "zero", "zero", T0), Err(Error::SelfCatch));
}

#[test]
fn immune_and_recovered_accounts_cannot_be_caught() {
    let (mut l, cfg) = world_with(Config::reference(), &["zero"]);
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    recover(&mut l, &cfg, "a", T0).unwrap();
    vaccinate(&mut l, &cfg, "i", "a", T0).unwrap();
    assert_eq!(catch(&mut l, &cfg, "zero", "i", T0), Err(Error::TargetNotUninfected));
    assert_eq!(catch(&mut l, &cfg, "zero", "a", T0), Err(Error::TargetNotUninfected));
}

#[test]
fn fanout_caps_children_per_carrier() {
    let (mut l, cfg) = world();
    for i in 0..cfg.fanout {
        catch(&mut l, &cfg, "zero", &format!("a{i}"), T0).unwrap();
    }
    assert_eq!(catch(&mut l, &cfg, "zero", "extra", T0), Err(Error::FanoutReached));
    assert_eq!(get(&l, "zero").children, cfg.fanout);
}

#[test]
fn catching_restarts_the_incubation_clock() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0 + 90).unwrap();
    assert_eq!(get(&l, "zero").last_spread_ns, T0 + 90);
    catch(&mut l, &cfg, "zero", "b", T0 + 150).unwrap();
    assert_eq!(catch(&mut l, &cfg, "zero", "c", T0 + 250), Err(Error::CarrierExpired));
}

#[test]
fn expired_carrier_cannot_catch() {
    let (mut l, cfg) = world();
    assert_eq!(catch(&mut l, &cfg, "zero", "a", T0 + cfg.incubation_ns), Err(Error::CarrierExpired));
    catch(&mut l, &cfg, "zero", "a", T0 + cfg.incubation_ns - 1).unwrap();
}

#[test]
fn cull_burns_expired_viral_and_pays_the_plague_doctor_from_the_burn() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let supply_before = l.globals.supply;
    assert_eq!(cull(&mut l, &cfg, "a", "doc", T0 + 99), Err(Error::NotExpired));

    let events = cull(&mut l, &cfg, "a", "doc", T0 + 100).unwrap();
    let a = get(&l, "a");
    assert_eq!((a.status, a.viral, a.liquid), (Status::Recovered, 0, 300));
    assert_eq!(get(&l, "doc").liquid, 30);
    assert_eq!(l.globals.supply, supply_before - 270);
    assert_eq!(l.globals.burned, 270);
    assert_eq!(
        events,
        vec![
            Event::Transfer {
                from: "a".into(),
                to: "doc".into(),
                amount: 30
            },
            Event::Burn {
                account: "a".into(),
                amount: 270
            },
            Event::Cull {
                account: "a".into(),
                caller: "doc".into(),
                burned: 270,
                cut: 30
            },
        ]
    );
    assert_eq!(cull(&mut l, &cfg, "a", "doc", T0 + 200), Err(Error::NotContagious));
}

#[test]
fn culling_yourself_pays_no_cut() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    cull(&mut l, &cfg, "a", "a", T0 + 100).unwrap();
    let a = get(&l, "a");
    assert_eq!((a.liquid, a.viral), (300, 0));
    assert_eq!(l.globals.burned, 300);
}

#[test]
fn timely_recover_burns_the_voluntary_share() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let events = recover(&mut l, &cfg, "a", T0 + 50).unwrap();
    let a = get(&l, "a");
    assert_eq!((a.status, a.viral, a.liquid), (Status::Recovered, 0, 540));
    assert_eq!(l.globals.burned, 60);
    assert!(events.contains(&Event::Recover {
        account: "a".into(),
        burned: 60,
        expired: false
    }));
    assert_eq!(recover(&mut l, &cfg, "a", T0 + 60), Err(Error::NotContagious));
    assert_eq!(catch(&mut l, &cfg, "a", "b", T0 + 60), Err(Error::CarrierNotContagious));
}

#[test]
fn recovering_late_pays_the_expired_rate_so_waiting_does_not_dodge_the_burn() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let events = recover(&mut l, &cfg, "a", T0 + 100).unwrap();
    let a = get(&l, "a");
    assert_eq!(a.liquid, 300);
    assert_eq!(l.globals.burned, 300);
    assert!(events.contains(&Event::Recover {
        account: "a".into(),
        burned: 300,
        expired: true
    }));
}

#[test]
fn viral_balance_cannot_be_transferred_but_counts_in_the_balance() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(get(&l, "a").balance(), 600);
    assert_eq!(transfer(&mut l, &cfg, "a", "b", 1, T0), Err(Error::InsufficientLiquid));
    recover(&mut l, &cfg, "a", T0).unwrap();
    transfer(&mut l, &cfg, "a", "b", 40, T0).unwrap();
    assert_eq!(get(&l, "a").liquid, 500);
    assert_eq!(get(&l, "b").liquid, 40);
    assert_eq!(transfer(&mut l, &cfg, "a", "a", 1, T0), Err(Error::SelfTransfer));
    assert_eq!(transfer(&mut l, &cfg, "a", "b", 0, T0), Err(Error::ZeroAmount));
}

#[test]
fn vaccinate_burns_liquid_and_makes_the_target_immune() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(vaccinate(&mut l, &cfg, "i", "a", T0), Err(Error::InsufficientLiquid));
    recover(&mut l, &cfg, "a", T0).unwrap();
    vaccinate(&mut l, &cfg, "i", "a", T0).unwrap();
    assert_eq!(get(&l, "i").status, Status::Immune);
    assert_eq!(get(&l, "a").liquid, 540 - cfg.vax_cost);
    assert_eq!(l.globals.immune, 1);
    assert_eq!(l.globals.burned, 60 + cfg.vax_cost);
    assert_eq!(vaccinate(&mut l, &cfg, "i", "a", T0), Err(Error::NotVaccinable));
    assert_eq!(vaccinate(&mut l, &cfg, "zero", "a", T0), Err(Error::NotVaccinable));
}

#[test]
fn each_vaccination_costs_more_than_the_last() {
    let mut cfg = Config::reference();
    cfg.base = 1_000_000;
    cfg.vax_cost = 10_000;
    cfg.fee_per_unit = 0;
    let (mut l, cfg) = world_with(cfg, &["zero"]);
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    recover(&mut l, &cfg, "a", T0).unwrap();
    let step = cfg.vax_cost * VAX_STEP_BPS / BPS;
    assert!(step > 0);
    let mut liquid = get(&l, "a").liquid;
    let burned = l.globals.burned;
    let mut spent = 0;
    for (n, target) in ["i0", "i1", "i2"].into_iter().enumerate() {
        let cost = cfg.vax_cost + step * n as u128;
        assert_eq!(vax_cost_now(&cfg, &l.globals), Ok(cost));
        let events = vaccinate(&mut l, &cfg, target, "a", T0).unwrap();
        assert!(events.contains(&Event::Vaccinate {
            account: target.into(),
            by: "a".into(),
            cost,
        }));
        liquid -= cost;
        spent += cost;
        assert_eq!(get(&l, "a").liquid, liquid);
    }
    assert_eq!(l.globals.burned, burned + spent);
    assert_eq!(l.globals.immune, 3);
}

#[test]
fn vaccinating_yourself_works_once_you_hold_liquid() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    recover(&mut l, &cfg, "a", T0).unwrap();
    transfer(&mut l, &cfg, "a", "b", 100, T0).unwrap();
    vaccinate(&mut l, &cfg, "b", "b", T0).unwrap();
    assert_eq!(get(&l, "b").status, Status::Immune);
    assert_eq!(get(&l, "b").liquid, 100 - cfg.vax_cost);
}

#[test]
fn herd_immunity_ends_the_season() {
    let (mut l, cfg) = world_with(Config::reference(), &["z1", "z2", "z3"]);
    for i in 1..=5 {
        catch(&mut l, &cfg, "z1", &format!("a{i}"), T0 + 1).unwrap();
        catch(&mut l, &cfg, "z2", &format!("b{i}"), T0 + 1).unwrap();
    }
    assert_eq!(l.globals.infected, 13);
    recover(&mut l, &cfg, "a1", T0 + 2).unwrap();
    for i in 1..=6 {
        vaccinate(&mut l, &cfg, &format!("v{i}"), "a1", T0 + 3).unwrap();
        assert_eq!(l.globals.ended_at, None);
    }
    let events = vaccinate(&mut l, &cfg, "v7", "a1", T0 + 4).unwrap();
    assert_eq!(l.globals.ended_at, Some(T0 + 4));
    assert_eq!(l.globals.end_reason, Some(EndReason::Herd));
    assert!(events.contains(&Event::SeasonEnd {
        at_ns: T0 + 4,
        reason: EndReason::Herd
    }));
    assert_eq!(catch(&mut l, &cfg, "z3", "late", T0 + 5), Err(Error::SeasonOver));
    assert_eq!(vaccinate(&mut l, &cfg, "v8", "a1", T0 + 5), Err(Error::SeasonOver));
}

#[test]
fn herd_immunity_needs_min_hosts() {
    let mut cfg = Config::reference();
    cfg.min_hosts = 50;
    let (mut l, cfg) = world_with(cfg, &["z1"]);
    catch(&mut l, &cfg, "z1", "a", T0).unwrap();
    recover(&mut l, &cfg, "a", T0).unwrap();
    for i in 0..5 {
        vaccinate(&mut l, &cfg, &format!("v{i}"), "a", T0).unwrap();
    }
    assert_eq!(l.globals.ended_at, None);
}

#[test]
fn season_ends_on_time_and_everyone_counts_as_expired() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0 + 10).unwrap();
    let end = T0 + cfg.season_max_ns;
    let events = cull(&mut l, &cfg, "a", "doc", end + 7).unwrap();
    assert!(events.contains(&Event::SeasonEnd {
        at_ns: end,
        reason: EndReason::Time
    }));
    assert_eq!(l.globals.ended_at, Some(end));
    assert_eq!(l.globals.end_reason, Some(EndReason::Time));
    assert_eq!(get(&l, "a").status, Status::Recovered);
    assert_eq!(catch(&mut l, &cfg, "zero", "b", end + 8), Err(Error::SeasonOver));
}

#[test]
fn season_end_cuts_a_live_clock_short() {
    let mut cfg = Config::reference();
    cfg.incubation_ns = 9_000;
    let (mut l, cfg) = world_with(cfg, &["zero"]);
    let end = T0 + cfg.season_max_ns;
    catch(&mut l, &cfg, "zero", "a", T0 + 8_000).unwrap();
    assert!(get(&l, "a").last_spread_ns + cfg.incubation_ns > end);
    let events = recover(&mut l, &cfg, "a", end + 1).unwrap();
    assert!(events.contains(&Event::Recover {
        account: "a".into(),
        burned: 300,
        expired: true
    }));
}

#[test]
fn spread_points_climb_two_levels_up() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(get(&l, "zero").points, 4);
    catch(&mut l, &cfg, "a", "b", T0).unwrap();
    catch(&mut l, &cfg, "b", "c", T0).unwrap();
    catch(&mut l, &cfg, "c", "d", T0).unwrap();
    let points: Vec<u64> = ["zero", "a", "b", "c", "d"].iter().map(|id| get(&l, id).points).collect();
    assert_eq!(points, vec![7, 7, 6, 4, 0]);
    assert_eq!(l.globals.total_points, 24);
}

#[test]
fn claim_pays_the_reserve_pro_rata_after_the_season_only() {
    let (mut l, cfg) = world();
    for (carrier, target) in [("zero", "a"), ("a", "b"), ("b", "c"), ("c", "d")] {
        catch(&mut l, &cfg, carrier, target, T0).unwrap();
    }
    assert_eq!(claim(&mut l, &cfg, "zero", T0 + 1), Err(Error::SeasonOpen));
    let end = T0 + cfg.season_max_ns;
    claim(&mut l, &cfg, "zero", end).unwrap();
    claim(&mut l, &cfg, "a", end).unwrap();
    claim(&mut l, &cfg, "b", end).unwrap();
    claim(&mut l, &cfg, "c", end).unwrap();
    let pool = l.globals.infection_minted * cfg.reserve_bps as u128 / 10_000;
    assert_eq!(pool, reserve_pool(&cfg, &l.globals).unwrap());
    assert_eq!(get(&l, "zero").liquid, pool * 7 / 24);
    assert_eq!(get(&l, "b").liquid, pool * 6 / 24);
    assert_eq!(get(&l, "c").liquid, pool * 4 / 24);
    assert_eq!(l.globals.reserve_paid, 2 * (pool * 7 / 24) + pool * 6 / 24 + pool * 4 / 24);
    assert!(l.globals.reserve_paid <= pool);
    assert_eq!(claim(&mut l, &cfg, "zero", end), Err(Error::NothingToClaim));
    assert_eq!(claim(&mut l, &cfg, "d", end), Err(Error::NothingToClaim));
}

#[test]
fn mint_that_decays_to_zero_stops_the_chain() {
    let mut cfg = Config::reference();
    cfg.base = 10;
    cfg.decay_bps = 1_000;
    let (mut l, cfg) = world_with(cfg, &["zero"]);
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(get(&l, "a").mint, 1);
    assert_eq!(catch(&mut l, &cfg, "a", "b", T0), Err(Error::MintExhausted));
    assert_eq!(get(&l, "b").status, Status::Uninfected);
}

#[test]
fn config_validation_rejects_nonsense() {
    let ok = Config::reference();
    ok.validate().unwrap();
    let mut c = ok.clone();
    c.decay_bps = 10_000;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.decay_bps = 0;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.burn_expired_bps = 10_001;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.fanout = 0;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.season_max_ns = c.incubation_ns;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.base = 0;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.reserve_bps = 10_001;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.vax_cost = 0;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.burn_voluntary_bps = c.burn_expired_bps + 1;
    assert!(c.validate().is_err());
    let mut c = ok.clone();
    c.burn_voluntary_bps = c.burn_expired_bps;
    c.validate().unwrap();
    let mut c = ok;
    c.herd_bps = 0;
    assert!(c.validate().is_err());
}

#[test]
fn init_rejects_bad_patient_zeros() {
    let cfg = Config::reference();
    let mut l = MemLedger::default();
    assert!(init(&mut l, &cfg, &[], T0).is_err());
    assert!(init(&mut l, &cfg, &names(&["a", "a"]), T0).is_err());
    assert!(init(&mut l, &cfg, &["f".repeat(64)], T0).is_err());
}

#[test]
fn naming_rules() {
    assert!(is_named("alice.testnet"));
    assert!(is_named("a"));
    assert!(!is_named(""));
    assert!(!is_named(&"0".repeat(64)));
    assert!(!is_named(&format!("0x{}", "Ab".repeat(20))));
    assert!(is_named(&"g".repeat(64)));
}

#[test]
fn json_amounts_are_strings_and_accept_numbers() {
    let cfg = Config::reference();
    let json = serde_json::to_value(&cfg).unwrap();
    assert_eq!(json["base"], "1000");
    assert_eq!(json["incubation_ns"], "100");
    assert_eq!(json["decay_bps"], 6000);
    let back: Config = serde_json::from_value(json).unwrap();
    assert_eq!(back, cfg);

    let mut numeric = serde_json::to_value(&cfg).unwrap();
    numeric["base"] = serde_json::json!(1000);
    let back: Config = serde_json::from_value(numeric).unwrap();
    assert_eq!(back, cfg);

    let huge = Config {
        min_fee: 340_282_366_920_938_463_463_374_607_431_768_211_455,
        ..cfg
    };
    let json = serde_json::to_string(&huge).unwrap();
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_fee, u128::MAX);
}

#[test]
fn borsh_round_trips_state() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let a = get(&l, "a");
    let bytes = borsh::to_vec(&a).unwrap();
    assert_eq!(borsh::from_slice::<Account>(&bytes).unwrap(), a);
    let globals = l.globals.clone();
    let bytes = borsh::to_vec(&globals).unwrap();
    assert_eq!(borsh::from_slice::<contagion_core::Globals>(&bytes).unwrap(), globals);
    let bytes = borsh::to_vec(&cfg).unwrap();
    assert_eq!(borsh::from_slice::<Config>(&bytes).unwrap(), cfg);
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn random_play_keeps_every_invariant() {
    for seed in 1..=40u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut cfg = Config::reference();
        cfg.fanout = 1 + rng.below(6) as u32;
        cfg.incubation_ns = 20 + rng.below(200);
        cfg.season_max_ns = cfg.incubation_ns * (5 + rng.below(40));
        cfg.min_hosts = 3 + rng.below(12) as u32;
        cfg.herd_bps = 1_000 + rng.below(6_000) as u32;
        cfg.vax_cost = 1 + rng.below(80) as u128;
        cfg.burn_expired_bps = rng.below(10_001) as u32;
        cfg.burn_voluntary_bps = rng.below(cfg.burn_expired_bps as u64 + 1) as u32;
        cfg.cull_cut_bps = rng.below(10_001) as u32;
        cfg.decay_bps = 3_000 + rng.below(6_500) as u32;
        cfg.reserve_bps = rng.below(10_001) as u32;
        cfg.fee_per_unit = rng.below(4) as u128;
        cfg.min_fee = rng.below(40) as u128;
        cfg.validate().unwrap();

        let (mut l, cfg) = world_with(cfg, &["a0", "a1", "a2"]);
        let pool: Vec<String> = (0..40).map(|i| format!("a{i}")).collect();
        let mut now = T0;
        let mut minted: u128 = 0;
        let mut burned: u128 = 0;
        let mut infection: u128 = 0;
        let mut fees: u128 = 0;

        for _ in 0..2_500 {
            now += rng.below(25);
            let pick = |rng: &mut Rng| pool[rng.below(pool.len() as u64) as usize].clone();
            let action = match rng.below(100) {
                0..=44 => Action::Catch {
                    carrier: pick(&mut rng),
                    target: pick(&mut rng),
                    deposit: if rng.below(10) == 0 { 0 } else { DEPOSIT },
                },
                45..=52 => Action::Recover { account: pick(&mut rng) },
                53..=67 => Action::Cull {
                    account: pick(&mut rng),
                    caller: pick(&mut rng),
                },
                68..=79 => Action::Vaccinate {
                    account: pick(&mut rng),
                    by: pick(&mut rng),
                },
                80..=87 => Action::Claim { account: pick(&mut rng) },
                _ => Action::Transfer {
                    from: pick(&mut rng),
                    to: pick(&mut rng),
                    amount: rng.below(300) as u128,
                },
            };
            let accounts_before = l.accounts.clone();
            match act(&mut l, &cfg, action, now) {
                Ok(events) => {
                    for event in events {
                        match event {
                            Event::Mint { amount, .. } => minted += amount,
                            Event::Burn { amount, .. } => burned += amount,
                            Event::Catch { mint, bonus, fee, .. } => {
                                infection += mint + bonus;
                                fees += fee;
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => assert_eq!(l.accounts, accounts_before, "failed action changed accounts, seed {seed}"),
            }
            assert_eq!(l.globals.supply, minted - burned, "event totals drifted, seed {seed}");
            assert_eq!(l.globals.burned, burned, "burned drifted, seed {seed}");
            assert_eq!(l.globals.infection_minted, infection, "infection_minted drifted, seed {seed}");
            assert_eq!(l.globals.fees_locked, fees, "fees drifted, seed {seed}");
            assert_eq!(minted, infection + l.globals.reserve_paid, "mint split drifted, seed {seed}");
            assert!(l.globals.reserve_paid <= reserve_pool(&cfg, &l.globals).unwrap());
        }
    }
}

#[test]
fn required_fee_scales_with_mint_and_never_drops_below_the_floor() {
    let cfg = Config::reference();
    assert_eq!(required_fee(&cfg, 600).unwrap(), 600);
    assert_eq!(required_fee(&cfg, 360).unwrap(), 360);
    assert_eq!(required_fee(&cfg, 3).unwrap(), cfg.min_fee);
    let pricey = Config {
        fee_per_unit: 7,
        ..cfg.clone()
    };
    assert_eq!(required_fee(&pricey, 600).unwrap(), 4_200);
    let flat = Config {
        fee_per_unit: 0,
        ..cfg
    };
    assert_eq!(required_fee(&flat, 600).unwrap(), flat.min_fee);
    assert_eq!(required_fee(&flat, 1).unwrap(), flat.min_fee);
}

#[test]
fn catch_charges_exactly_the_required_fee_for_each_generation() {
    let (mut l, cfg) = world();
    let mint = child_mint(&cfg, get(&l, "zero").mint).unwrap();
    let fee = required_fee(&cfg, mint).unwrap();
    let short = Action::Catch {
        carrier: "zero".into(),
        target: "a".into(),
        deposit: fee - 1,
    };
    assert_eq!(act(&mut l, &cfg, short, T0), Err(Error::DepositTooLow));
    assert_eq!(l.globals.fees_locked, 0);
    let exact = Action::Catch {
        carrier: "zero".into(),
        target: "a".into(),
        deposit: fee,
    };
    act(&mut l, &cfg, exact, T0).unwrap();
    assert_eq!(l.globals.fees_locked, fee);

    let deeper = required_fee(&cfg, child_mint(&cfg, get(&l, "a").mint).unwrap()).unwrap();
    assert!(deeper < fee);
    let overpay = Action::Catch {
        carrier: "a".into(),
        target: "b".into(),
        deposit: deeper * 10,
    };
    let events = act(&mut l, &cfg, overpay, T0).unwrap();
    assert!(events.iter().any(|e| matches!(e, Event::Catch { fee: charged, .. } if *charged == deeper)));
    assert_eq!(l.globals.fees_locked, fee + deeper);
}

#[test]
fn reserve_is_a_share_of_infection_mint_and_empty_when_nobody_spreads() {
    let (mut l, cfg) = world();
    let end = T0 + cfg.season_max_ns;
    assert_eq!(claim(&mut l, &cfg, "zero", end), Err(Error::NothingToClaim));
    assert_eq!(l.globals.reserve_paid, 0);

    let (mut l, cfg) = world_with(
        Config {
            reserve_bps: 10_000,
            ..Config::reference()
        },
        &["zero"],
    );
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    assert_eq!(reserve_pool(&cfg, &l.globals).unwrap(), l.globals.infection_minted);
    let end = T0 + cfg.season_max_ns;
    claim(&mut l, &cfg, "zero", end).unwrap();
    assert!(l.globals.reserve_paid <= l.globals.infection_minted);
    assert!(l.globals.reserve_paid > 0);
}

#[test]
fn reserve_does_not_count_its_own_claims_or_burns() {
    let (mut l, cfg) = world();
    catch(&mut l, &cfg, "zero", "a", T0).unwrap();
    let before = l.globals.infection_minted;
    recover(&mut l, &cfg, "a", T0 + 1).unwrap();
    cull(&mut l, &cfg, "zero", "doc", T0 + 500).unwrap();
    assert_eq!(l.globals.infection_minted, before);
    let end = T0 + cfg.season_max_ns;
    claim(&mut l, &cfg, "zero", end).unwrap();
    assert_eq!(l.globals.infection_minted, before);
}
