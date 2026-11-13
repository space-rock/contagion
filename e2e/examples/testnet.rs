use contagion_core::Config;
use near_workspaces::network::Testnet;
use near_workspaces::types::{Gas, NearToken, SecretKey};
use near_workspaces::{Account, AccountId, Contract, Worker};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

const STORAGE_MIN: u128 = 3_000_000_000_000_000_000_000;
const SEEDS: usize = 24;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn wasm() -> anyhow::Result<Vec<u8>> {
    let path = root().join("target/near/contagion_contract/contagion_contract.wasm");
    std::fs::read(&path).map_err(|_| anyhow::anyhow!("wasm not found at {}; run scripts/build-contract.sh", path.display()))
}

fn account(worker: &Worker<Testnet>, name: &str) -> anyhow::Result<Account> {
    let path = root().join(".testnet").join(format!("{name}.json"));
    let keys: Value = serde_json::from_str(
        &std::fs::read_to_string(&path).map_err(|_| anyhow::anyhow!("no keys at {}; run scripts/testnet-account.sh {name}", path.display()))?,
    )?;
    let id: AccountId = format!("{name}.testnet").parse()?;
    let key: SecretKey = keys["private_key"].as_str().unwrap().parse()?;
    Ok(Account::from_secret_key(id, key, worker))
}

fn preset(name: &str) -> anyhow::Result<Config> {
    let path = root().join("deploy/presets").join(format!("{name}.json"));
    let config: Config = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    config.validate().map_err(anyhow::Error::msg)?;
    Ok(config)
}

async fn view(contract: &Contract, method: &str, args: Value) -> anyhow::Result<Value> {
    Ok(contract.view(method).args_json(args).await?.json()?)
}

async fn deploy(worker: &Worker<Testnet>, preset_name: &str, contract_name: &str, zeros: &[String]) -> anyhow::Result<()> {
    let config = preset(preset_name)?;
    let owner = account(worker, contract_name)?;
    let patient_zeros: Vec<AccountId> = zeros.iter().map(|z| format!("{z}.testnet").parse()).collect::<Result<_, _>>()?;
    for zero in &patient_zeros {
        worker.view_account(zero).await.map_err(|_| anyhow::anyhow!("patient zero {zero} does not exist"))?;
    }
    let bytes = wasm()?;
    println!("deploying {} bytes to {}", bytes.len(), owner.id());
    let contract = owner.deploy(&bytes).await?.into_result()?;
    contract
        .call("new")
        .args_json(json!({ "config": config, "patient_zeros": patient_zeros }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let stored = view(&contract, "get_config", json!({})).await?;
    anyhow::ensure!(serde_json::from_value::<Config>(stored)? == config, "stored config differs");
    let stats = view(&contract, "get_stats", json!({})).await?;
    println!("initialized: preset {preset_name}, {} patient zeros, infected {}", patient_zeros.len(), stats["globals"]["infected"]);
    println!("explorer: https://testnet.nearblocks.io/address/{}", contract.id());
    Ok(())
}

async fn retry<T, F, Fut>(mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut delay = 3;
    for attempt in 0..8 {
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < 7 && format!("{error:#}").to_lowercase().contains("rate limit") => {
                tokio::time::sleep(Duration::from_secs(delay)).await;
                delay = (delay * 2).min(30);
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}

async fn all_hosts(contract: &Contract) -> anyhow::Result<Vec<Value>> {
    let mut all: Vec<Value> = Vec::new();
    loop {
        let from = all.len();
        let page = retry(|| view(contract, "get_hosts", json!({ "from": from, "limit": 200 }))).await?;
        let page = page.as_array().cloned().unwrap_or_default();
        let done = page.len() < 200;
        all.extend(page);
        if done {
            return Ok(all);
        }
    }
}

fn count(hosts: &[Value], statuses: &[&str]) -> usize {
    hosts.iter().filter(|h| statuses.contains(&h["status"].as_str().unwrap_or(""))).count()
}

async fn seed(worker: &Worker<Testnet>, contract_name: &str, zeros: &[String]) -> anyhow::Result<()> {
    let contract_id: AccountId = format!("{contract_name}.testnet").parse()?;
    let contract = Contract::from_secret_key(
        contract_id.clone(),
        account(worker, contract_name)?.secret_key().clone(),
        worker,
    );
    let creators: Vec<Account> = zeros.iter().map(|z| account(worker, z)).collect::<Result<_, _>>()?;
    let prefix = std::env::var("SEED_PREFIX").unwrap_or_else(|_| "a".to_string());
    let config: Config = serde_json::from_value(retry(|| view(&contract, "get_config", json!({}))).await?)?;
    let cap = config.fanout.min(3);
    let wanted = creators.len() + SEEDS;

    let mut hosts = all_hosts(&contract).await?;
    let mut mine: Vec<Account> = Vec::new();
    let mut counter = 0usize;
    while count(&hosts, &["contagious", "recovered"]) < wanted {
        let carrier = hosts
            .iter()
            .filter(|h| h["status"] == "contagious" && h["expired"] == false && h["children"].as_u64().unwrap_or(0) < cap as u64)
            .min_by_key(|h| h["gen"].as_u64().unwrap_or(0))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no carrier with spare fanout"))?;
        let carrier_id = carrier["account_id"].as_str().unwrap().to_string();

        let creator = &creators[counter % creators.len()];
        let (name, id) = loop {
            counter += 1;
            let name = format!("{prefix}{counter}");
            let id: AccountId = format!("{name}.{}", creator.id()).parse()?;
            if worker.view_account(&id).await.is_err() {
                break (name, id);
            }
        };
        let target = retry(|| async {
            Ok(creator
                .create_subaccount(&name)
                .initial_balance(NearToken::from_millinear(300))
                .transact()
                .await?
                .into_result()?)
        })
        .await?;
        let quote = retry(|| view(&contract, "quote_catch", json!({ "carrier": carrier_id }))).await?;
        anyhow::ensure!(quote["can_catch"] == true, "cannot seed from {carrier_id}: {quote}");
        let fee: u128 = quote["fee"].as_str().unwrap().parse()?;
        retry(|| async {
            Ok(target
                .call(&contract_id, "catch")
                .args_json(json!({ "carrier": carrier_id }))
                .deposit(NearToken::from_yoctonear(fee))
                .gas(Gas::from_tgas(60))
                .transact()
                .await?
                .into_result()?)
        })
        .await?;
        println!("seeded {id} from {carrier_id} (gen {}, fee {fee} yocto)", quote["gen"]);

        for host in hosts.iter_mut() {
            if host["account_id"] == carrier_id.as_str() {
                host["children"] = json!(host["children"].as_u64().unwrap_or(0) + 1);
            }
        }
        hosts.push(json!({
            "account_id": id.to_string(), "status": "contagious", "expired": false,
            "gen": quote["gen"], "children": 0
        }));
        mine.push(target);
    }

    let wanted_recovered = 3;
    let recovered = count(&hosts, &["recovered"]);
    if recovered < wanted_recovered {
        for who in mine.iter().take(wanted_recovered - recovered) {
            retry(|| async { Ok(who.call(&contract_id, "recover").gas(Gas::from_tgas(60)).transact().await?.into_result()?) }).await?;
            println!("{} recovered", who.id());
        }
    }
    let immune = count(&hosts, &["immune"]);
    let stats = retry(|| view(&contract, "get_stats", json!({}))).await?;
    let mut already = stats["globals"]["immune"].as_u64().unwrap_or(0) as usize;
    already = already.max(immune);
    if already < 2 {
        if let Some(vaccinator) = mine.first() {
            for n in already + 1..=2 {
                let stranger = format!("vaccinated{n}.{}", vaccinator.id());
                retry(|| async {
                    Ok(vaccinator
                        .call(&contract_id, "vaccinate")
                        .args_json(json!({ "account": stranger }))
                        .deposit(NearToken::from_yoctonear(STORAGE_MIN))
                        .gas(Gas::from_tgas(60))
                        .transact()
                        .await?
                        .into_result()?)
                })
                .await?;
                println!("{} vaccinated {stranger}", vaccinator.id());
            }
        }
    }
    let stats = retry(|| view(&contract, "get_stats", json!({}))).await?;
    let g = &stats["globals"];
    println!("done: infected {} immune {} max gen {} supply {}", g["infected"], g["immune"], g["max_gen"], g["supply"]);
    Ok(())
}

fn strip_clock(mut hosts: Value) -> Value {
    if let Some(list) = hosts.as_array_mut() {
        for host in list {
            if let Some(fields) = host.as_object_mut() {
                fields.remove("expired");
            }
        }
    }
    hosts
}

async fn redeploy(worker: &Worker<Testnet>, contract_name: &str) -> anyhow::Result<()> {
    let owner = account(worker, contract_name)?;
    let contract = Contract::from_secret_key(owner.id().clone(), owner.secret_key().clone(), worker);

    let snapshot = |stats: &Value| {
        json!({
            "globals": stats["globals"],
            "accounts": stats["accounts"],
            "events": stats["events"],
        })
    };
    let stats_before = snapshot(&retry(|| view(&contract, "get_stats", json!({}))).await?);
    let config_before = retry(|| view(&contract, "get_config", json!({}))).await?;
    let metadata_before = retry(|| view(&contract, "ft_metadata", json!({}))).await?;
    let icon_before = metadata_before["icon"].clone();
    let bounds_before = retry(|| view(&contract, "storage_balance_bounds", json!({}))).await?;
    let events_before = retry(|| view(&contract, "get_events", json!({ "from": 0, "limit": 200 }))).await?;
    let hosts_before = retry(|| view(&contract, "get_hosts", json!({ "from": 0, "limit": 200 }))).await?;

    let bytes = wasm()?;
    println!("redeploying {} bytes to {} (code only, `new` is not called)", bytes.len(), owner.id());
    retry(|| async { Ok(owner.deploy(&bytes).await?.into_result()?) }).await?;

    let stats_after = snapshot(&retry(|| view(&contract, "get_stats", json!({}))).await?);
    let config_after = retry(|| view(&contract, "get_config", json!({}))).await?;
    let metadata = retry(|| view(&contract, "ft_metadata", json!({}))).await?;
    let bounds_after = retry(|| view(&contract, "storage_balance_bounds", json!({}))).await?;
    let events_after = retry(|| view(&contract, "get_events", json!({ "from": 0, "limit": 200 }))).await?;
    let hosts_after = retry(|| view(&contract, "get_hosts", json!({ "from": 0, "limit": 200 }))).await?;
    let source = retry(|| view(&contract, "contract_source_metadata", json!({}))).await?;

    anyhow::ensure!(stats_before == stats_after, "state changed across the redeploy:\nbefore {stats_before}\nafter  {stats_after}");
    anyhow::ensure!(config_before == config_after, "config changed across the redeploy");
    anyhow::ensure!(bounds_before == bounds_after, "storage_balance_bounds changed across the redeploy");
    anyhow::ensure!(events_before == events_after, "the event log changed across the redeploy");
    anyhow::ensure!(strip_clock(hosts_before) == strip_clock(hosts_after), "hosts changed across the redeploy");
    let (mut without_icon_before, mut without_icon_after) = (metadata_before.clone(), metadata.clone());
    without_icon_before["icon"] = Value::Null;
    without_icon_after["icon"] = Value::Null;
    anyhow::ensure!(without_icon_before == without_icon_after, "ft_metadata changed across the redeploy");
    let standards: Vec<String> = source["standards"]
        .as_array()
        .map(|list| list.iter().filter_map(|entry| entry["standard"].as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    for expected in ["nep141", "nep145", "nep148", "nep297", "nep330"] {
        anyhow::ensure!(standards.iter().any(|name| name == expected), "contract_source_metadata does not declare {expected}: {source}");
    }
    let expected = std::fs::read_to_string(root().join("contract/src/icon.txt"))?;
    anyhow::ensure!(metadata["icon"] == expected.as_str(), "ft_metadata.icon does not match contract/src/icon.txt");

    println!("state unchanged: {stats_after}");
    println!("standards declared: {}", standards.join(", "));
    println!("source metadata: {source}");
    println!("icon before: {} characters, after: {} characters", icon_before.as_str().map_or(0, str::len), expected.len());
    println!("explorer: https://testnet.nearblocks.io/address/{}", owner.id());
    Ok(())
}

async fn burn(worker: &Worker<Testnet>, name: &str) -> anyhow::Result<()> {
    let owner = account(worker, name)?;
    let keys = worker.view_access_keys(owner.id()).await?;
    for info in &keys {
        owner
            .batch(owner.id())
            .delete_key(info.public_key.clone())
            .transact()
            .await?
            .into_result()?;
        println!("deleted {}", info.public_key);
    }
    let left = worker.view_access_keys(owner.id()).await?;
    anyhow::ensure!(left.is_empty(), "{} keys remain", left.len());
    println!("{} now has no access keys: the contract can never be changed", owner.id());
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: testnet deploy <preset> <contract> <zero>... | seed <contract> <zero>... | redeploy <contract> | burn <contract>";
    let rpc = std::env::var("NEAR_RPC").unwrap_or_else(|_| "https://test.rpc.fastnear.com".to_string());
    let worker = near_workspaces::testnet().rpc_addr(&rpc).await?;
    match args.first().map(String::as_str) {
        Some("deploy") if args.len() >= 4 => deploy(&worker, &args[1], &args[2], &args[3..]).await,
        Some("seed") if args.len() >= 3 => seed(&worker, &args[1], &args[2..]).await,
        Some("burn") if args.len() == 2 => burn(&worker, &args[1]).await,
        Some("redeploy") if args.len() == 2 => redeploy(&worker, &args[1]).await,
        _ => anyhow::bail!(usage),
    }
}
