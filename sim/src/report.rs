use crate::run::{Outcome, Row};
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;

const BARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn csv(rows: &[Row]) -> String {
    let mut out = String::from(
        "hour,day,hosts,contagious,recovered,immune,expired_waiting,max_gen,supply,viral,liquid,burned,catches,culls\n",
    );
    for r in rows {
        let _ = writeln!(
            out,
            "{},{:.2},{},{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{},{}",
            r.hour,
            r.hour as f64 / 24.0,
            r.hosts,
            r.contagious,
            r.recovered,
            r.immune,
            r.expired_waiting,
            r.max_gen,
            r.supply,
            r.viral,
            r.liquid,
            r.burned,
            r.catches,
            r.culls
        );
    }
    out
}

pub fn write(dir: &Path, outcome: &Outcome) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let name = outcome.scenario.name;
    fs::write(dir.join(format!("{name}.csv")), csv(&outcome.rows))?;
    let json = serde_json::to_string_pretty(&outcome.summary).expect("summary serializes");
    fs::write(dir.join(format!("{name}.json")), json + "\n")
}

pub fn sparkline(values: &[f64], width: usize) -> String {
    if values.is_empty() {
        return String::new();
    }
    let max = values.iter().cloned().fold(0.0, f64::max);
    (0..width)
        .map(|i| {
            let v = values[i * (values.len() - 1) / (width - 1).max(1)];
            if max <= 0.0 {
                BARS[0]
            } else {
                BARS[((v / max) * 7.0).round() as usize]
            }
        })
        .collect()
}

fn num(value: &serde_json::Value, path: &[&str]) -> f64 {
    path.iter()
        .fold(value, |v, key| &v[*key])
        .as_f64()
        .unwrap_or(0.0)
}

pub fn write_runs(dir: &Path, name: &str, summary: &serde_json::Value) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(summary).expect("summary serializes");
    fs::write(dir.join(format!("{name}.runs.json")), json + "\n")
}

pub fn runs_table(summaries: &[serde_json::Value]) -> String {
    let mut out = String::new();
    let q = |s: &serde_json::Value, metric: &str, key: &str| s["metrics"][metric][key].as_f64();
    let cell = |value: Option<f64>, precision: usize| value.map_or("-".to_string(), |v| format!("{v:.precision$}"));
    let _ = writeln!(
        out,
        "{:<13} {:>5} {:>8} {:>6} {:>20} {:>8} {:>8} {:>10} {:>9} {:>10} {:>9}",
        "scenario",
        "runs",
        "takeoff",
        "herd",
        "hosts p10/p50/p90",
        "end day",
        "burn %",
        "res->top3%",
        "res %sup",
        "sybil %mnt",
        "adv posted"
    );
    for s in summaries {
        let _ = writeln!(
            out,
            "{:<13} {:>5} {:>7.0}% {:>5.0}% {:>20} {:>8} {:>8} {:>10} {:>9} {:>10} {:>9}",
            s["scenario"].as_str().unwrap_or("?"),
            s["runs"],
            s["takeoff_rate"].as_f64().unwrap_or(0.0) * 100.0,
            s["herd_end_rate"].as_f64().unwrap_or(0.0) * 100.0,
            format!(
                "{}/{}/{}",
                cell(q(s, "hosts", "p10"), 0),
                cell(q(s, "hosts", "p50"), 0),
                cell(q(s, "hosts", "p90"), 0)
            ),
            cell(q(s, "end_day", "p50"), 1),
            cell(q(s, "burned_pct_of_infection_mint", "p50"), 1),
            cell(q(s, "reserve_to_top3_pct", "p50"), 1),
            cell(q(s, "reserve_pct_of_supply", "p50"), 1),
            cell(q(s, "sybil_share_of_infection_mint_pct", "p50"), 1),
            cell(q(s, "sybil_cost_advantage_vs_posted", "p50"), 2),
        );
    }
    out
}

pub fn table(outcomes: &[Outcome]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<13} {:>6} {:>4} {:>11} {:>11} {:>8} {:>8} {:>9}  hosts over time",
        "scenario", "hosts", "gen", "season end", "supply", "burn %", "top10 %", "unsettled"
    );
    for o in outcomes {
        let s = &o.summary;
        let hosts: Vec<f64> = o.rows.iter().map(|r| r.hosts as f64).collect();
        let _ = writeln!(
            out,
            "{:<13} {:>6} {:>4} {:>6.1}d {:<4} {:>11.0} {:>8.1} {:>8.1} {:>9}  {}",
            o.scenario.name,
            num(s, &["outcome", "hosts"]) as u64,
            num(s, &["outcome", "max_gen"]) as u64,
            num(s, &["season", "end_day"]),
            s["season"]["end_reason"].as_str().unwrap_or("?"),
            num(s, &["outcome", "supply"]),
            num(s, &["outcome", "burned_pct_of_infection_mint"]),
            num(s, &["outcome", "top10_holdings_pct"]),
            num(s, &["outcome", "carriers_unsettled_at_end"]) as u64,
            sparkline(&hosts, 30),
        );
        if let Some(sybil) = s["sybil"].as_object() {
            let _ = writeln!(
                out,
                "  sybil: {} alts paid {:.2} NEAR ({:.1}% of all fees) for {:.0} infection tokens ({:.1}% of infection mint): {:.0} tokens/NEAR vs {:.0} honest avg and {:.0} posted, advantage {} vs honest avg and {} vs posted",
                sybil["caught"],
                sybil["fees_near"].as_f64().unwrap_or(0.0),
                sybil["share_of_fees_pct"].as_f64().unwrap_or(0.0),
                sybil["tokens_from_infection"].as_f64().unwrap_or(0.0),
                sybil["share_of_infection_mint_pct"].as_f64().unwrap_or(0.0),
                sybil["tokens_per_near"].as_f64().unwrap_or(0.0),
                sybil["honest_tokens_per_near"].as_f64().unwrap_or(0.0),
                sybil["posted_tokens_per_near"].as_f64().unwrap_or(0.0),
                sybil["cost_advantage_vs_honest_avg"].as_f64().map_or("-".into(), |v| format!("{v:.1}x")),
                sybil["cost_advantage_vs_posted"].as_f64().map_or("-".into(), |v| format!("{v:.2}x")),
            );
        }
        let _ = writeln!(
            out,
            "  plague doctors: {} culls, {:.0} tokens of cut, {:.0} tokens held at end",
            num(s, &["outcome", "culls"]) as u64,
            num(s, &["outcome", "plague_doctor_cut"]),
            num(s, &["outcome", "plague_doctor_balance"]),
        );
    }
    out
}
