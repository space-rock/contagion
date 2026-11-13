use contagion_core::Config;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

const NS: u64 = 1_000_000_000;
const TOKEN: u128 = 1_000_000_000_000_000_000;
const STORAGE_MIN: u128 = 3_000_000_000_000_000_000_000;
const HOSTS: usize = 22;
const INCUBATION_S: u64 = 30;

fn wasm() -> anyhow::Result<Vec<u8>> {
    let path = std::env::var("CONTAGION_WASM").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/near/contagion_contract/contagion_contract.wasm")
    });
    std::fs::read(&path).map_err(|_| anyhow::anyhow!("wasm not found at {}; run scripts/build-contract.sh", path.display()))
}

fn tokens(raw: &str) -> f64 {
    raw.parse::<u128>().unwrap_or(0) as f64 / TOKEN as f64
}

async fn view(contract: &Contract, method: &str, args: Value) -> anyhow::Result<Value> {
    Ok(contract.view(method).args_json(args).await?.json()?)
}

async fn hosts(contract: &Contract) -> anyhow::Result<Vec<Value>> {
    let mut all = Vec::new();
    loop {
        let page = view(contract, "get_hosts", json!({ "from": all.len(), "limit": 200 })).await?;
        let page = page.as_array().cloned().unwrap_or_default();
        let done = page.len() < 200;
        all.extend(page);
        if done {
            return Ok(all);
        }
    }
}

fn glyph(host: &Value) -> &'static str {
    match (host["status"].as_str().unwrap_or(""), host["expired"].as_bool().unwrap_or(false)) {
        ("contagious", false) => "●",
        ("contagious", true) => "◌",
        ("recovered", _) => "✓",
        ("immune", _) => "◆",
        _ => "·",
    }
}

type Labels = BTreeMap<String, String>;

fn short(labels: &Labels, id: &str) -> String {
    labels.get(id).cloned().unwrap_or_else(|| id.chars().take(10).collect())
}

fn draw(hosts: &[Value], labels: &Labels) {
    let mut children: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    let mut roots = Vec::new();
    for host in hosts {
        match host["parent"].as_str() {
            Some(parent) => children.entry(parent.to_string()).or_default().push(host),
            None if host["status"] != "uninfected" && host["status"] != "immune" => roots.push(host),
            None => {}
        }
    }
    fn walk(host: &Value, children: &BTreeMap<String, Vec<&Value>>, labels: &Labels, prefix: &str, last: bool, root: bool) {
        let line = format!(
            "{} {:<5} gen {} viral {:>5.0} liquid {:>5.0}",
            glyph(host),
            short(labels, host["account_id"].as_str().unwrap()),
            host["gen"],
            tokens(host["viral"].as_str().unwrap_or("0")),
            tokens(host["liquid"].as_str().unwrap_or("0"))
        );
        if root {
            println!("{line}");
        } else {
            println!("{prefix}{}{line}", if last { "└─ " } else { "├─ " });
        }
        let next = if root { String::new() } else { format!("{prefix}{}", if last { "   " } else { "│  " }) };
        if let Some(kids) = children.get(host["account_id"].as_str().unwrap()) {
            for (i, kid) in kids.iter().enumerate() {
                walk(kid, children, labels, &next, i + 1 == kids.len(), false);
            }
        }
    }
    for root in roots {
        walk(root, &children, labels, "", true, true);
    }
}

async fn stats_line(contract: &Contract, label: &str) -> anyhow::Result<()> {
    let stats = view(contract, "get_stats", json!({})).await?;
    let g = &stats["globals"];
    println!(
        "{label}: infected {} immune {} max gen {} supply {:.0} burned {:.0} reserve paid {:.0} season over {}",
        g["infected"],
        g["immune"],
        g["max_gen"],
        tokens(g["supply"].as_str().unwrap_or("0")),
        tokens(g["burned"].as_str().unwrap_or("0")),
        tokens(g["reserve_paid"].as_str().unwrap_or("0")),
        stats["season_over"]
    );
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config {
        base: 1_000 * TOKEN,
        decay_bps: 6_000,
        bonus_bps: 3_000,
        fanout: 3,
        incubation_ns: INCUBATION_S * NS,
        burn_expired_bps: 5_000,
        burn_voluntary_bps: 1_000,
        cull_cut_bps: 1_000,
        vax_cost: 20 * TOKEN,
        herd_bps: 3_000,
        min_hosts: 10,
        season_max_ns: 600 * NS,
        min_fee: 10_000_000_000_000_000_000_000,
        fee_per_unit: 500,
        reserve_bps: 2_000,
    };
    println!("starting a local NEAR sandbox and deploying the contract");
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&wasm()?).await?;
    let zero = worker.dev_create_account().await?;
    contract
        .call("new")
        .args_json(json!({ "config": cfg, "patient_zeros": [zero.id()] }))
        .transact()
        .await?
        .into_result()?;
    let mut labels = Labels::new();
    labels.insert(zero.id().to_string(), "zero".to_string());
    println!("contract {} patient zero {}\n", contract.id(), zero.id());

    println!("== 1. outbreak: {HOSTS} accounts catch it, each from the one before in the tree");
    let mut members: Vec<Account> = vec![zero.clone()];
    let mut carrier = 0usize;
    let mut slots = 0u32;
    for n in 1..=HOSTS {
        if slots == cfg.fanout {
            carrier += 1;
            slots = 0;
        }
        let from = members[carrier].clone();
        let quote = view(&contract, "quote_catch", json!({ "carrier": from.id() })).await?;
        let fee: u128 = quote["fee"].as_str().unwrap().parse()?;
        let target = worker.dev_create_account().await?;
        let outcome = target
            .call(contract.id(), "catch")
            .args_json(json!({ "carrier": from.id() }))
            .deposit(NearToken::from_yoctonear(fee))
            .max_gas()
            .transact()
            .await?
            .into_result()?;
        if n == 1 {
            for log in outcome.logs().iter().filter(|l| l.starts_with("EVENT_JSON:")) {
                println!("   event {log}");
            }
        }
        labels.insert(target.id().to_string(), format!("h{n}"));
        members.push(target);
        slots += 1;
    }
    stats_line(&contract, "after outbreak").await?;
    draw(&hosts(&contract).await?, &labels);

    println!("\n== 2. five early carriers recover voluntarily (10% burn) and get liquid tokens");
    for who in members.iter().skip(1).take(5) {
        who.call(contract.id(), "recover").max_gas().transact().await?.into_result()?;
    }
    stats_line(&contract, "after recoveries").await?;

    println!("\n== 3. everyone else stops spreading; waiting {}s for incubation clocks to run out", INCUBATION_S + 2);
    tokio::time::sleep(Duration::from_secs(INCUBATION_S + 2)).await;
    let late = worker.dev_create_account().await?;
    let stale = &members[HOSTS];
    let rejected = late
        .call(contract.id(), "catch")
        .args_json(json!({ "carrier": stale.id() }))
        .deposit(NearToken::from_near(1))
        .max_gas()
        .transact()
        .await?;
    println!("catching from the expired carrier {} fails: {}", short(&labels, stale.id().as_str()), rejected.is_failure());

    println!("\n== 4. a plague doctor culls expired carriers (50% burn, doctor keeps 10% of it)");
    let doctor = worker.dev_create_account().await?;
    labels.insert(doctor.id().to_string(), "doctor".to_string());
    doctor
        .call(contract.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_yoctonear(STORAGE_MIN))
        .transact()
        .await?
        .into_result()?;
    let mut culled = 0;
    for host in hosts(&contract).await? {
        if host["expired"] == true && host["status"] == "contagious" {
            doctor
                .call(contract.id(), "cull")
                .args_json(json!({ "account": host["account_id"] }))
                .max_gas()
                .transact()
                .await?
                .into_result()?;
            culled += 1;
        }
    }
    let cut: Value = view(&contract, "ft_balance_of", json!({ "account_id": doctor.id() })).await?;
    println!("culled {culled} carriers, doctor earned {:.1} FLU", tokens(cut.as_str().unwrap()));
    stats_line(&contract, "after culling").await?;

    println!("\n== 5. recovered holders vaccinate strangers until immunity passes {}% of infected", cfg.herd_bps / 100);
    let vaccinator = &members[1];
    let mut shots = 0;
    while !view(&contract, "get_stats", json!({})).await?["season_over"].as_bool().unwrap_or(false) {
        let stranger = worker.dev_create_account().await?;
        labels.insert(stranger.id().to_string(), format!("v{}", shots + 1));
        let outcome = vaccinator
            .call(contract.id(), "vaccinate")
            .args_json(json!({ "account": stranger.id() }))
            .deposit(NearToken::from_yoctonear(STORAGE_MIN))
            .max_gas()
            .transact()
            .await?;
        if outcome.is_failure() {
            println!("vaccinator ran out of liquid tokens after {shots} shots");
            break;
        }
        shots += 1;
    }
    stats_line(&contract, "after vaccination").await?;

    println!("\n== 6. season over: spread points pay out the reserve");
    let pool = view(&contract, "get_stats", json!({})).await?["reserve_pool"].clone();
    println!("reserve pool {:.0} FLU (a share of infection mint)", tokens(pool.as_str().unwrap_or("0")));
    let mut ranked = Vec::new();
    for host in hosts(&contract).await? {
        if host["points"].as_u64().unwrap_or(0) > 0 {
            ranked.push((host["points"].as_u64().unwrap(), host["account_id"].as_str().unwrap().to_string()));
        }
    }
    ranked.sort_by(|a, b| b.cmp(a));
    for (points, id) in ranked.iter().take(5) {
        let account = members.iter().find(|m| m.id().as_str() == id).unwrap();
        let outcome = account.call(contract.id(), "claim").max_gas().transact().await?.into_result()?;
        println!("{} had {points} spread points and claimed {:.1} FLU", short(&labels, id), tokens(&outcome.json::<String>()?));
    }
    stats_line(&contract, "final").await?;
    println!("\nfinal tree (● live, ◌ expired, ✓ recovered, ◆ immune):");
    draw(&hosts(&contract).await?, &labels);
    Ok(())
}
