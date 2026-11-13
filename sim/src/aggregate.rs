use crate::run::{run, Outcome};
use crate::scenario::Scenario;
use contagion_core::Config;
use serde_json::{json, Map, Value};

const SEED_STEP: u64 = 104_729;
const TAKEOFF_HOSTS: f64 = 1_000.0;

const METRICS: [(&str, &[&str]); 16] = [
    ("fees_near", &["outcome", "fees_near"]),
    ("sybil_fees_near", &["sybil", "fees_near"]),
    ("reserve_pct_of_supply", &["outcome", "reserve_pct_of_supply"]),
    ("tokens_per_near", &["outcome", "tokens_per_near"]),
    ("sybil_cost_advantage_vs_posted", &["sybil", "cost_advantage_vs_posted"]),
    ("sybil_cost_advantage_vs_honest_avg", &["sybil", "cost_advantage_vs_honest_avg"]),
    ("sybil_share_of_fees_pct", &["sybil", "share_of_fees_pct"]),
    ("hosts", &["outcome", "hosts"]),
    ("max_gen", &["outcome", "max_gen"]),
    ("end_day", &["season", "end_day"]),
    ("burned_pct_of_infection_mint", &["outcome", "burned_pct_of_infection_mint"]),
    ("top10_holdings_pct", &["outcome", "top10_holdings_pct"]),
    ("reserve_to_top3_pct", &["outcome", "reserve_to_top3_pct"]),
    ("infection_minted", &["outcome", "infection_minted"]),
    ("plague_doctor_cut", &["outcome", "plague_doctor_cut"]),
    ("sybil_share_of_infection_mint_pct", &["sybil", "share_of_infection_mint_pct"]),
];

fn dig(value: &Value, path: &[&str]) -> Option<f64> {
    path.iter().try_fold(value, |v, key| v.get(*key))?.as_f64()
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * q).round() as usize]
}

pub fn batch(sc: &Scenario, cfg: &Config, seed: u64, runs: usize) -> (Outcome, Value) {
    let mut first = None;
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); METRICS.len()];
    let mut takeoffs = 0usize;
    let mut herd = 0usize;

    for k in 0..runs {
        let outcome = run(sc, cfg, seed.wrapping_add(k as u64 * SEED_STEP));
        for (column, (_, path)) in columns.iter_mut().zip(METRICS.iter()) {
            if let Some(value) = dig(&outcome.summary, path) {
                column.push(value);
            }
        }
        if dig(&outcome.summary, &["outcome", "hosts"]).unwrap_or(0.0) >= TAKEOFF_HOSTS {
            takeoffs += 1;
        }
        if outcome.summary["season"]["end_reason"] == "herd" {
            herd += 1;
        }
        if first.is_none() {
            first = Some(outcome);
        }
    }

    let mut metrics = Map::new();
    for (column, (name, _)) in columns.iter_mut().zip(METRICS.iter()) {
        if column.is_empty() {
            continue;
        }
        column.sort_by(|a, b| a.partial_cmp(b).expect("finite metric"));
        metrics.insert(
            name.to_string(),
            json!({
                "p10": quantile(column, 0.1),
                "p50": quantile(column, 0.5),
                "p90": quantile(column, 0.9),
            }),
        );
    }

    let summary = json!({
        "scenario": sc.name,
        "runs": runs,
        "takeoff_hosts": TAKEOFF_HOSTS,
        "takeoff_rate": takeoffs as f64 / runs as f64,
        "herd_end_rate": herd as f64 / runs as f64,
        "metrics": Value::Object(metrics),
    });
    (first.expect("at least one run"), summary)
}
