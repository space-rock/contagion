use contagion_core::Config;
use contagion_sim::aggregate::batch;
use contagion_sim::{apply_override, base_config, presets, report, run, Scenario};

const POP: usize = 1_500;

fn scenario(name: &str) -> Scenario {
    let mut found = presets().into_iter().find(|s| s.name == name).expect("known scenario");
    found.pop = POP;
    found
}

fn short() -> Config {
    let mut cfg = base_config();
    apply_override(&mut cfg, "season_days", "20").unwrap();
    cfg
}

#[test]
fn same_seed_gives_identical_output() {
    let cfg = short();
    let a = run(&scenario("viral"), &cfg, 7);
    let b = run(&scenario("viral"), &cfg, 7);
    assert_eq!(a.rows, b.rows);
    assert_eq!(a.summary, b.summary);
    assert_eq!(report::csv(&a.rows), report::csv(&b.rows));
}

#[test]
fn different_seeds_diverge() {
    let cfg = short();
    let a = run(&scenario("viral"), &cfg, 1);
    let b = run(&scenario("viral"), &cfg, 2);
    assert_ne!(a.rows, b.rows);
}

#[test]
fn every_preset_runs_and_keeps_ledger_invariants() {
    let cfg = short();
    for sc in presets() {
        let mut sc = sc;
        sc.pop = POP;
        let outcome = run(&sc, &cfg, 3);
        outcome.ledger.check_invariants().unwrap();
        assert!(!outcome.rows.is_empty());
        assert!(outcome.ledger.globals.supply > 0, "{} minted nothing", sc.name);
    }
}

#[test]
fn stalled_outbreak_dies_while_viral_takes_off() {
    let cfg = short();
    let viral = run(&scenario("viral"), &cfg, 1).ledger.globals.infected;
    let stalled = run(&scenario("stalled"), &cfg, 1).ledger.globals.infected;
    assert!(viral > 500, "viral reached only {viral}");
    assert!(stalled < 200, "stalled reached {stalled}");
}

#[test]
fn sybil_scenario_builds_every_alt_and_reports_costs() {
    let cfg = short();
    let outcome = run(&scenario("sybil"), &cfg, 1);
    let sybil = &outcome.summary["sybil"];
    assert_eq!(sybil["caught"], 500);
    assert!(sybil["tokens_from_infection"].as_f64().unwrap() > 0.0);
    assert!(sybil["fees_near"].as_f64().unwrap() > 1.0);
    assert!(sybil["share_of_infection_mint_pct"].as_f64().unwrap() > 0.0);
    assert!(sybil["share_of_fees_pct"].as_f64().unwrap() > 0.0);
}

#[test]
fn proportional_fee_prices_sybil_alts_near_the_posted_rate() {
    let proportional = run(&scenario("sybil"), &short(), 1).summary["sybil"].clone();
    let posted = proportional["cost_advantage_vs_posted"].as_f64().unwrap();
    assert!((0.9..1.6).contains(&posted), "advantage vs posted was {posted}");

    let mut flat_cfg = short();
    apply_override(&mut flat_cfg, "fee_per_unit", "0").unwrap();
    apply_override(&mut flat_cfg, "min_fee_milli_near", "20").unwrap();
    let flat = run(&scenario("sybil"), &flat_cfg, 1).summary["sybil"].clone();
    assert!(flat["cost_advantage_vs_posted"].is_null());
    let flat_adv = flat["cost_advantage_vs_honest_avg"].as_f64().unwrap();
    let prop_adv = proportional["cost_advantage_vs_honest_avg"].as_f64().unwrap();
    assert!(flat_adv > 1.2 * prop_adv, "flat {flat_adv} vs proportional {prop_adv}");
}

#[test]
fn reserve_is_a_share_of_infection_mint_and_scales_with_the_outbreak() {
    let cfg = short();
    let viral = run(&scenario("viral"), &cfg, 1).summary["outcome"].clone();
    let stalled = run(&scenario("stalled"), &cfg, 1).summary["outcome"].clone();
    for outcome in [&viral, &stalled] {
        let pool = outcome["reserve_pool"].as_f64().unwrap();
        let minted = outcome["infection_minted"].as_f64().unwrap();
        let expected = minted * cfg.reserve_bps as f64 / 10_000.0;
        assert!((pool - expected).abs() <= expected * 1e-6 + 1e-9, "pool {pool} vs {expected}");
        assert!(outcome["reserve_paid"].as_f64().unwrap() <= pool + 1e-9);
    }
    let viral_paid = viral["reserve_paid"].as_f64().unwrap();
    let stalled_paid = stalled["reserve_paid"].as_f64().unwrap();
    assert!(stalled_paid * 5.0 < viral_paid, "stalled {stalled_paid} vs viral {viral_paid}");
}

#[test]
fn honest_scenarios_have_no_sybil_block() {
    let outcome = run(&scenario("viral"), &short(), 1);
    assert!(outcome.summary["sybil"].is_null());
}

#[test]
fn whale_culler_out_earns_each_scattered_doctor() {
    let cfg = short();
    let whale = run(&scenario("whale_culler"), &cfg, 1).summary["outcome"]["plague_doctor_balance"]
        .as_f64()
        .unwrap();
    let scattered_total = run(&scenario("viral"), &cfg, 1).summary["outcome"]["plague_doctor_balance"]
        .as_f64()
        .unwrap();
    let per_doctor = scattered_total / scenario("viral").doctors as f64;
    assert!(whale > 0.0 && per_doctor > 0.0);
    assert!(whale > 2.0 * per_doctor, "whale {whale} vs per doctor {per_doctor}");
}

#[test]
fn vaccine_faction_can_end_the_season_early() {
    let mut cfg = short();
    apply_override(&mut cfg, "min_hosts", "50").unwrap();
    let (_, summary) = batch(&scenario("vax_faction"), &cfg, 1, 12);
    assert!(summary["herd_end_rate"].as_f64().unwrap() > 0.0);
}

#[test]
fn csv_has_a_header_and_fixed_width_rows() {
    let outcome = run(&scenario("viral"), &short(), 1);
    let csv = report::csv(&outcome.rows);
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), outcome.rows.len() + 1);
    let width = lines[0].split(',').count();
    assert!(lines.iter().all(|line| line.split(',').count() == width));
    assert!(lines[0].starts_with("hour,day,hosts"));
}

#[test]
fn batch_reports_quantiles_over_runs() {
    let (first, summary) = batch(&scenario("viral"), &short(), 1, 3);
    assert_eq!(summary["runs"], 3);
    let rate = summary["takeoff_rate"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&rate));
    let hosts = &summary["metrics"]["hosts"];
    assert!(hosts["p10"].as_f64().unwrap() <= hosts["p50"].as_f64().unwrap());
    assert!(hosts["p50"].as_f64().unwrap() <= hosts["p90"].as_f64().unwrap());
    assert_eq!(first.seed, 1);
}

#[test]
fn overrides_validate_and_reject_unknown_keys() {
    let mut cfg = base_config();
    apply_override(&mut cfg, "fanout", "7").unwrap();
    assert_eq!(cfg.fanout, 7);
    assert!(apply_override(&mut cfg, "decay_bps", "10000").is_err());
    assert!(apply_override(&mut cfg, "nope", "1").is_err());
    assert!(apply_override(&mut cfg, "fanout", "many").is_err());
}
