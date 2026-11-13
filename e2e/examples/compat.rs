use contagion_core::Config;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract};
use serde_json::{json, Value};
use std::path::PathBuf;

const NS: u64 = 1_000_000_000;
const STORAGE_MIN: u128 = 3_000_000_000_000_000_000_000;
const TIME_FIELDS: [&str; 7] = ["now_ns", "storage_usage", "balance", "at_ns", "last_spread_ns", "deadline_ns", "start_ns"];

fn config() -> Config {
    Config {
        base: 1_000 * 10u128.pow(18),
        decay_bps: 6_000,
        bonus_bps: 3_000,
        fanout: 3,
        incubation_ns: 3_600 * NS,
        burn_expired_bps: 5_000,
        burn_voluntary_bps: 1_000,
        cull_cut_bps: 1_000,
        vax_cost: 20 * 10u128.pow(18),
        herd_bps: 3_000,
        min_hosts: 1_000,
        season_max_ns: 86_400 * NS,
        min_fee: 10_000_000_000_000_000_000_000,
        fee_per_unit: 500,
        reserve_bps: 2_000,
    }
}

fn strip(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for field in TIME_FIELDS {
                map.remove(field);
            }
            map.values_mut().for_each(strip);
        }
        Value::Array(items) => items.iter_mut().for_each(strip),
        _ => {}
    }
}

async fn view(contract: &Contract, method: &str, args: Value) -> anyhow::Result<Value> {
    let mut value: Value = contract.view(method).args_json(args).await?.json()?;
    strip(&mut value);
    Ok(value)
}

async fn snapshot(contract: &Contract, accounts: &[&Account]) -> anyhow::Result<Value> {
    let mut balances = Vec::new();
    for who in accounts {
        let args = json!({ "account_id": who.id() });
        balances.push(json!({
            "id": who.id(),
            "ft_balance_of": view(contract, "ft_balance_of", args.clone()).await?,
            "storage_balance_of": view(contract, "storage_balance_of", args.clone()).await?,
            "get_host": view(contract, "get_host", args).await?,
        }));
    }
    Ok(json!({
        "get_config": view(contract, "get_config", json!({})).await?,
        "get_stats": view(contract, "get_stats", json!({})).await?,
        "get_hosts": view(contract, "get_hosts", json!({ "from": 0, "limit": 100 })).await?,
        "get_events": view(contract, "get_events", json!({ "from": 0, "limit": 100 })).await?,
        "ft_total_supply": view(contract, "ft_total_supply", json!({})).await?,
        "ft_metadata": view(contract, "ft_metadata", json!({})).await?,
        "storage_balance_bounds": view(contract, "storage_balance_bounds", json!({})).await?,
        "quote_catch": view(contract, "quote_catch", json!({ "carrier": accounts[0].id() })).await?,
        "balances": balances,
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let previous = std::env::args().nth(1).ok_or_else(|| anyhow::anyhow!("usage: compat <previous.wasm> [current.wasm]"))?;
    let current = std::env::args().nth(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/near/contagion_contract/contagion_contract.wasm")
    });
    let old_bytes = std::fs::read(&previous)?;
    let new_bytes = std::fs::read(&current)?;

    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&old_bytes).await?;
    let zero = worker.dev_create_account().await?;
    contract
        .call("new")
        .args_json(json!({ "config": config(), "patient_zeros": [zero.id()] }))
        .transact()
        .await?
        .into_result()?;

    let catch = |carrier: Account| {
        let worker = worker.clone();
        let contract = contract.clone();
        async move {
            let target = worker.dev_create_account().await?;
            let quote: Value = contract.view("quote_catch").args_json(json!({ "carrier": carrier.id() })).await?.json()?;
            let fee: u128 = quote["fee"].as_str().unwrap().parse()?;
            target
                .call(contract.id(), "catch")
                .args_json(json!({ "carrier": carrier.id() }))
                .deposit(NearToken::from_yoctonear(fee))
                .max_gas()
                .transact()
                .await?
                .into_result()?;
            anyhow::Ok(target)
        }
    };
    let alice = catch(zero.clone()).await?;
    let bob = catch(alice.clone()).await?;
    let carol = catch(zero.clone()).await?;
    let dave = worker.dev_create_account().await?;
    dave.call(contract.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_yoctonear(STORAGE_MIN))
        .transact()
        .await?
        .into_result()?;
    alice.call(contract.id(), "recover").max_gas().transact().await?.into_result()?;
    alice
        .call(contract.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": dave.id(), "amount": "1000", "memo": "thanks" }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let accounts = [&zero, &alice, &bob, &carol, &dave];
    let before = snapshot(&contract, &accounts).await?;
    let code_before = contract.view_code().await?.len();

    contract.as_account().deploy(&new_bytes).await?.into_result()?;
    let code_after = contract.view_code().await?.len();
    let after = snapshot(&contract, &accounts).await?;

    anyhow::ensure!(code_before != code_after, "the new code was not deployed");
    for key in before.as_object().unwrap().keys() {
        anyhow::ensure!(before[key] == after[key], "{key} differs across the redeploy:\nbefore {}\nafter  {}", before[key], after[key]);
        println!("same   {key}");
    }

    let erin = catch(zero.clone()).await?;
    bob.call(contract.id(), "recover").max_gas().transact().await?.into_result()?;
    let events: Value = contract.view("get_events").args_json(json!({ "from": 0, "limit": 100 })).await?.json()?;
    anyhow::ensure!(events.as_array().unwrap().len() == 6, "expected 6 logged events, got {}", events.as_array().unwrap().len());
    let host: Value = contract.view("get_host").args_json(json!({ "account_id": erin.id() })).await?.json()?;
    anyhow::ensure!(host["status"] == "contagious", "new code cannot continue the outbreak");
    println!("new code continued the outbreak on the old state: 6 events, erin is {}", host["status"]);
    println!("code {code_before} -> {code_after} bytes, every view identical");
    Ok(())
}
