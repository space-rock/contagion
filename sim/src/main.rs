use contagion_sim::aggregate::batch;
use contagion_sim::{apply_override, base_config, presets, report, run};
use std::path::PathBuf;
use std::process::exit;

const USAGE: &str = "usage: sim <all|viral|stalled|sybil|vax_faction|whale_culler> [--seed N] [--runs N] [--pop N] [--out DIR] [--set key=value]...";

fn fail(message: &str) -> ! {
    eprintln!("{message}\n{USAGE}");
    exit(2)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(which) = args.next() else { fail("missing scenario") };
    let mut seed = 1u64;
    let mut runs = 1usize;
    let mut pop: Option<usize> = None;
    let mut out = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/out"));
    let mut cfg = base_config();

    while let Some(flag) = args.next() {
        let Some(value) = args.next() else { fail(&format!("{flag} needs a value")) };
        match flag.as_str() {
            "--seed" => seed = value.parse().unwrap_or_else(|_| fail("bad seed")),
            "--runs" => runs = value.parse().ok().filter(|n| *n > 0).unwrap_or_else(|| fail("bad runs")),
            "--pop" => pop = Some(value.parse().unwrap_or_else(|_| fail("bad pop"))),
            "--out" => out = PathBuf::from(value),
            "--set" => {
                let Some((key, val)) = value.split_once('=') else { fail("--set needs key=value") };
                if let Err(message) = apply_override(&mut cfg, key, val) {
                    fail(&message);
                }
            }
            _ => fail(&format!("unknown flag {flag}")),
        }
    }

    let all = presets();
    if which != "all" && !all.iter().any(|s| s.name == which) {
        fail(&format!("unknown scenario {which}"));
    }

    let mut outcomes = Vec::new();
    let mut aggregates = Vec::new();
    for (index, mut scenario) in all.into_iter().enumerate() {
        if which != "all" && scenario.name != which {
            continue;
        }
        if let Some(pop) = pop {
            scenario.pop = pop;
        }
        let scenario_seed = seed.wrapping_add(index as u64 * 7919);
        let outcome = if runs > 1 {
            let (first, summary) = batch(&scenario, &cfg, scenario_seed, runs);
            report::write_runs(&out, scenario.name, &summary)
                .unwrap_or_else(|e| fail(&format!("cannot write output: {e}")));
            aggregates.push(summary);
            first
        } else {
            run(&scenario, &cfg, scenario_seed)
        };
        report::write(&out, &outcome).unwrap_or_else(|e| fail(&format!("cannot write output: {e}")));
        outcomes.push(outcome);
    }

    if !aggregates.is_empty() {
        print!("{}", report::runs_table(&aggregates));
        println!();
    }
    print!("{}", report::table(&outcomes));
    println!("wrote {}", out.display());
}
