use contagion_core::Config;
use near_workspaces::network::Sandbox;
use near_workspaces::operations::CallTransaction;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, Contract, Worker};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

const NS: u64 = 1_000_000_000;
const STORAGE_MIN: u128 = 3_000_000_000_000_000_000_000;
const BYTE_COST: u128 = 10_000_000_000_000_000_000;

fn wasm() -> Vec<u8> {
    let path = std::env::var("CONTAGION_WASM").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/near/contagion_contract/contagion_contract.wasm")
    });
    std::fs::read(&path).unwrap_or_else(|_| {
        panic!(
            "contract wasm not found at {}; run scripts/build-contract.sh first or set CONTAGION_WASM",
            path.display()
        )
    })
}

fn config(incubation_s: u64, season_s: u64) -> Config {
    Config {
        base: 1_000 * 10u128.pow(18),
        decay_bps: 6_000,
        bonus_bps: 3_000,
        fanout: 3,
        incubation_ns: incubation_s * NS,
        burn_expired_bps: 5_000,
        burn_voluntary_bps: 1_000,
        cull_cut_bps: 1_000,
        vax_cost: 20 * 10u128.pow(18),
        herd_bps: 3_000,
        min_hosts: 1_000,
        season_max_ns: season_s * NS,
        min_fee: 10_000_000_000_000_000_000_000,
        fee_per_unit: 500,
        reserve_bps: 2_000,
    }
}

struct World {
    worker: Worker<Sandbox>,
    contract: Contract,
    zero: Account,
}

async fn deploy(config: Config) -> anyhow::Result<World> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(&wasm()).await?;
    let zero = worker.dev_create_account().await?;
    contract
        .call("new")
        .args_json(json!({ "config": config, "patient_zeros": [zero.id()] }))
        .transact()
        .await?
        .into_result()?;
    Ok(World { worker, contract, zero })
}

async fn view(contract: &Contract, method: &str, args: Value) -> anyhow::Result<Value> {
    Ok(contract.view(method).args_json(args).await?.json()?)
}

async fn fee(contract: &Contract, carrier: &Account) -> anyhow::Result<u128> {
    let quote = view(contract, "quote_catch", json!({ "carrier": carrier.id() })).await?;
    anyhow::ensure!(quote["can_catch"] == true, "cannot catch: {quote}");
    Ok(quote["fee"].as_str().unwrap().parse()?)
}

async fn catch(world: &World, carrier: &Account) -> anyhow::Result<Account> {
    let target = world.worker.dev_create_account().await?;
    let price = fee(&world.contract, carrier).await?;
    target
        .call(world.contract.id(), "catch")
        .args_json(json!({ "carrier": carrier.id() }))
        .deposit(NearToken::from_yoctonear(price))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    Ok(target)
}

async fn balance(contract: &Contract, who: &Account) -> anyhow::Result<u128> {
    let value = view(contract, "ft_balance_of", json!({ "account_id": who.id() })).await?;
    Ok(value.as_str().unwrap().parse()?)
}

async fn storage_usage(contract: &Contract) -> anyhow::Result<u64> {
    Ok(contract.view_account().await?.storage_usage)
}

async fn register(world: &World, who: &Account) -> anyhow::Result<()> {
    who.call(world.contract.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_yoctonear(STORAGE_MIN))
        .transact()
        .await?
        .into_result()?;
    Ok(())
}

async fn send(logs: &mut Vec<Value>, tx: CallTransaction) -> anyhow::Result<()> {
    let outcome = tx.max_gas().transact().await?.into_result()?;
    for line in outcome.logs() {
        if let Some(json) = line.strip_prefix("EVENT_JSON:") {
            logs.push(serde_json::from_str(json)?);
        }
    }
    Ok(())
}

async fn catch_logged(
    world: &World,
    carrier: &Account,
    logs: &mut Vec<Value>,
    touched: &mut Vec<Account>,
) -> anyhow::Result<Account> {
    let target = world.worker.dev_create_account().await?;
    let price = fee(&world.contract, carrier).await?;
    send(
        logs,
        target
            .call(world.contract.id(), "catch")
            .args_json(json!({ "carrier": carrier.id() }))
            .deposit(NearToken::from_yoctonear(price)),
    )
    .await?;
    touched.push(target.clone());
    Ok(target)
}

async fn wait_until(contract: &Contract, method: &str, args: Value, field: &str, seconds: u64) -> anyhow::Result<()> {
    for _ in 0..seconds {
        if view(contract, method, args.clone()).await?[field] == true {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    anyhow::bail!("{field} never became true")
}

#[tokio::test]
async fn outbreak_runs_on_chain_and_supply_matches_balances() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let alice = catch(&world, &world.zero).await?;
    let bob = catch(&world, &alice).await?;

    let price = fee(&world.contract, &world.zero).await?;
    let carol = world.worker.dev_create_account().await?;
    let outcome = carol
        .call(world.contract.id(), "catch")
        .args_json(json!({ "carrier": world.zero.id() }))
        .deposit(NearToken::from_yoctonear(price + 1_000_000_000_000_000_000_000_000))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let logs: Vec<&str> = outcome.logs().into_iter().filter(|l| l.starts_with("EVENT_JSON:")).collect();
    assert_eq!(logs.len(), 3, "{logs:?}");
    assert!(logs[0].contains("\"standard\":\"contagion\"") && logs[0].contains("\"event\":\"catch\""));
    assert!(logs[1].contains("ft_mint") && logs[2].contains("ft_mint"));

    let carol_balance = world.worker.view_account(carol.id()).await?.balance;
    assert!(
        carol_balance.as_yoctonear() > 99 * 10u128.pow(24) - price - 2 * 10u128.pow(21),
        "excess deposit was not refunded"
    );

    let supply: u128 = view(&world.contract, "ft_total_supply", json!({})).await?.as_str().unwrap().parse()?;
    let mut sum = 0;
    for who in [&world.zero, &alice, &bob, &carol] {
        sum += balance(&world.contract, who).await?;
    }
    assert_eq!(supply, sum);

    let stats = view(&world.contract, "get_stats", json!({})).await?;
    assert_eq!(stats["globals"]["infected"], 4);
    assert_eq!(stats["accounts"], 4);
    assert_eq!(view(&world.contract, "get_hosts", json!({ "from": 0, "limit": 10 })).await?.as_array().unwrap().len(), 4);
    Ok(())
}

#[tokio::test]
async fn host_lifecycle_storage_is_covered_by_the_fee_floor() -> anyhow::Result<()> {
    const CLAIM_LOG_SLACK: u128 = 120;
    let world = deploy(config(600, 3_000)).await?;
    let long_name = "x".repeat(50);
    let target = world.worker.root_account()?.create_subaccount(&long_name).initial_balance(NearToken::from_near(5)).transact().await?.into_result()?;
    assert_eq!(target.id().as_str().len(), 60);

    let before = storage_usage(&world.contract).await?;
    let price = fee(&world.contract, &world.zero).await?;
    target
        .call(world.contract.id(), "catch")
        .args_json(json!({ "carrier": world.zero.id() }))
        .deposit(NearToken::from_yoctonear(price))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let after_catch = storage_usage(&world.contract).await?;
    target.call(world.contract.id(), "recover").max_gas().transact().await?.into_result()?;
    let after_recover = storage_usage(&world.contract).await?;

    let lifecycle = (after_recover - before) as u128 + CLAIM_LOG_SLACK;
    let floor = config(1, 2).min_fee;
    println!(
        "catch {} B, recover {} B, with claim slack {} B = {} yocto, floor {} yocto",
        after_catch - before,
        after_recover - after_catch,
        lifecycle,
        lifecycle * BYTE_COST,
        floor
    );
    assert!(lifecycle * BYTE_COST <= floor, "fee floor is below the storage a host costs");

    let bob = world.worker.dev_create_account().await?;
    let before_register = storage_usage(&world.contract).await?;
    register(&world, &bob).await?;
    let register_bytes = (storage_usage(&world.contract).await? - before_register) as u128;
    println!("registration {register_bytes} B");
    assert!(register_bytes * BYTE_COST <= STORAGE_MIN, "storage minimum is below a registration");
    Ok(())
}

#[tokio::test]
async fn expired_carrier_is_culled_and_the_doctor_is_paid() -> anyhow::Result<()> {
    let world = deploy(config(6, 600)).await?;
    let alice = catch(&world, &world.zero).await?;
    let doc = world.worker.dev_create_account().await?;
    register(&world, &doc).await?;

    let early = doc
        .call(world.contract.id(), "cull")
        .args_json(json!({ "account": alice.id() }))
        .max_gas()
        .transact()
        .await?;
    assert!(early.is_failure(), "culling a live carrier must fail");

    tokio::time::sleep(Duration::from_secs(8)).await;
    let host = view(&world.contract, "get_host", json!({ "account_id": alice.id() })).await?;
    assert_eq!(host["expired"], true);

    doc.call(world.contract.id(), "cull")
        .args_json(json!({ "account": alice.id() }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let alice_host = view(&world.contract, "get_host", json!({ "account_id": alice.id() })).await?;
    assert_eq!(alice_host["status"], "recovered");
    assert_eq!(alice_host["viral"], "0");
    assert_eq!(balance(&world.contract, &doc).await?, 600 * 10u128.pow(18) / 2 / 10);
    Ok(())
}

#[tokio::test]
async fn season_ends_and_the_reserve_is_claimed_by_spread_points() -> anyhow::Result<()> {
    let world = deploy(config(5, 20)).await?;
    let alice = catch(&world, &world.zero).await?;
    let _bob = catch(&world, &alice).await?;

    let early = world.zero.call(world.contract.id(), "claim").max_gas().transact().await?;
    assert!(early.is_failure(), "claim before the season ends must fail");

    tokio::time::sleep(Duration::from_secs(22)).await;
    let stats = view(&world.contract, "get_stats", json!({})).await?;
    assert_eq!(stats["season_over"], true);
    let pool: u128 = stats["reserve_pool"].as_str().unwrap().parse()?;
    assert!(pool > 0);

    let before = balance(&world.contract, &world.zero).await?;
    world.zero.call(world.contract.id(), "claim").max_gas().transact().await?.into_result()?;
    let gained = balance(&world.contract, &world.zero).await? - before;
    assert!(gained > 0 && gained <= pool);
    let again = world.zero.call(world.contract.id(), "claim").max_gas().transact().await?;
    assert!(again.is_failure(), "claim twice must fail");
    Ok(())
}

#[tokio::test]
async fn only_liquid_tokens_move_and_receivers_must_register() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let alice = catch(&world, &world.zero).await?;
    let bob = world.worker.dev_create_account().await?;

    let locked = alice
        .call(world.contract.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": bob.id(), "amount": "1" }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(locked.is_failure(), "viral balance must be locked");

    alice.call(world.contract.id(), "recover").max_gas().transact().await?.into_result()?;
    let unregistered = alice
        .call(world.contract.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": bob.id(), "amount": "1" }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(unregistered.is_failure(), "unregistered receiver must be rejected");

    register(&world, &bob).await?;
    let no_yocto = alice
        .call(world.contract.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": bob.id(), "amount": "1" }))
        .max_gas()
        .transact()
        .await?;
    assert!(no_yocto.is_failure(), "transfer without 1 yocto must be rejected");

    alice
        .call(world.contract.id(), "ft_transfer")
        .args_json(json!({ "receiver_id": bob.id(), "amount": "1000" }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    assert_eq!(balance(&world.contract, &bob).await?, 1000);
    Ok(())
}

#[tokio::test]
async fn the_contract_cannot_be_initialized_twice_and_the_callback_is_private() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let again = world
        .zero
        .call(world.contract.id(), "new")
        .args_json(json!({ "config": config(1, 2), "patient_zeros": [world.zero.id()] }))
        .max_gas()
        .transact()
        .await?;
    assert!(again.is_failure());
    let direct = world
        .zero
        .call(world.contract.id(), "ft_resolve_transfer")
        .args_json(json!({ "sender_id": world.zero.id(), "receiver_id": world.zero.id(), "amount": "1" }))
        .max_gas()
        .transact()
        .await?;
    assert!(direct.is_failure());
    assert!(format!("{:?}", direct.failures()).contains("private"));
    Ok(())
}

fn exported_functions() -> Vec<String> {
    let bytes = wasm();
    let mut exported = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(&bytes) {
        if let Ok(wasmparser::Payload::ExportSection(section)) = payload {
            for export in section {
                let export = export.unwrap();
                if matches!(export.kind, wasmparser::ExternalKind::Func) {
                    exported.push(export.name.to_string());
                }
            }
        }
    }
    exported.sort();
    exported
}

#[test]
fn the_wasm_exports_no_admin_upgrade_or_withdraw_surface() {
    let exported = exported_functions();
    let allowed = [
        "__contract_abi",
        "catch",
        "claim",
        "contract_source_metadata",
        "cull",
        "ft_balance_of",
        "ft_metadata",
        "ft_resolve_transfer",
        "ft_total_supply",
        "ft_transfer",
        "ft_transfer_call",
        "get_config",
        "get_events",
        "get_host",
        "get_hosts",
        "get_stats",
        "is_named_account",
        "new",
        "quote_catch",
        "recover",
        "storage_balance_bounds",
        "storage_balance_of",
        "storage_deposit",
        "storage_unregister",
        "storage_withdraw",
        "vaccinate",
    ];
    let unexpected: Vec<&String> = exported.iter().filter(|name| !allowed.contains(&name.as_str())).collect();
    assert!(unexpected.is_empty(), "unexpected exports: {unexpected:?}");
    for forbidden in ["upgrade", "migrate", "withdraw", "set_owner", "set_config", "update_config", "owner"] {
        assert!(
            !exported.iter().any(|name| name != "storage_withdraw" && name.contains(forbidden)),
            "found {forbidden}"
        );
    }
}

#[tokio::test]
async fn every_balance_change_has_a_nep141_log_so_a_replay_matches_the_chain() -> anyhow::Result<()> {
    let world = deploy(config(10, 70)).await?;
    let contract = &world.contract;
    let mut logs: Vec<Value> = Vec::new();
    let mut touched: Vec<Account> = vec![world.zero.clone()];

    let alice = catch_logged(&world, &world.zero, &mut logs, &mut touched).await?;
    let bob = catch_logged(&world, &alice, &mut logs, &mut touched).await?;
    let carol = catch_logged(&world, &world.zero, &mut logs, &mut touched).await?;

    let doc = world.worker.dev_create_account().await?;
    let dave = world.worker.dev_create_account().await?;
    let eve = world.worker.dev_create_account().await?;
    for who in [&doc, &dave] {
        register(&world, who).await?;
        touched.push(who.clone());
    }

    send(&mut logs, alice.call(contract.id(), "recover")).await?;
    send(
        &mut logs,
        alice
            .call(contract.id(), "ft_transfer")
            .args_json(json!({ "receiver_id": dave.id(), "amount": "1000", "memo": "thanks" }))
            .deposit(NearToken::from_yoctonear(1)),
    )
    .await?;
    send(
        &mut logs,
        alice
            .call(contract.id(), "ft_transfer_call")
            .args_json(json!({ "receiver_id": dave.id(), "amount": "500", "msg": "swap" }))
            .deposit(NearToken::from_yoctonear(1)),
    )
    .await?;
    send(
        &mut logs,
        alice
            .call(contract.id(), "vaccinate")
            .args_json(json!({ "account": eve.id() }))
            .deposit(NearToken::from_yoctonear(STORAGE_MIN)),
    )
    .await?;
    touched.push(eve.clone());

    wait_until(contract, "get_host", json!({ "account_id": bob.id() }), "expired", 40).await?;
    send(&mut logs, doc.call(contract.id(), "cull").args_json(json!({ "account": bob.id() }))).await?;
    send(&mut logs, carol.call(contract.id(), "recover")).await?;

    wait_until(contract, "get_stats", json!({}), "season_over", 70).await?;
    for claimant in [&world.zero, &alice] {
        send(&mut logs, claimant.call(contract.id(), "claim")).await?;
    }

    let nep141: Vec<&Value> = logs.iter().filter(|log| log["standard"] == "nep141").collect();
    let kinds = |name: &str| nep141.iter().filter(|log| log["event"] == name).count();
    assert!(kinds("ft_mint") >= 8, "mints: {}", kinds("ft_mint"));
    assert!(kinds("ft_burn") >= 4, "burns: {}", kinds("ft_burn"));
    assert!(kinds("ft_transfer") >= 3, "transfers: {}", kinds("ft_transfer"));
    assert!(
        nep141.iter().any(|log| log["data"][0]["memo"] == "refund"),
        "the failed ft_transfer_call must log its refund"
    );
    assert!(nep141.iter().any(|log| log["data"][0]["memo"] == "thanks"));

    let amount = |data: &Value| -> i128 { data["amount"].as_str().unwrap().parse().unwrap() };
    let mut replay: BTreeMap<String, i128> = BTreeMap::new();
    for log in &nep141 {
        for data in log["data"].as_array().unwrap() {
            match log["event"].as_str().unwrap() {
                "ft_mint" => *replay.entry(data["owner_id"].as_str().unwrap().to_string()).or_default() += amount(data),
                "ft_burn" => *replay.entry(data["owner_id"].as_str().unwrap().to_string()).or_default() -= amount(data),
                "ft_transfer" => {
                    *replay.entry(data["old_owner_id"].as_str().unwrap().to_string()).or_default() -= amount(data);
                    *replay.entry(data["new_owner_id"].as_str().unwrap().to_string()).or_default() += amount(data);
                }
                other => panic!("unexpected nep141 event {other}"),
            }
        }
    }

    for who in &touched {
        let on_chain = balance(contract, who).await? as i128;
        let replayed = replay.get(who.id().as_str()).copied().unwrap_or(0);
        assert_eq!(replayed, on_chain, "replayed balance differs for {}", who.id());
    }
    assert!(replay.keys().all(|id| touched.iter().any(|who| who.id().as_str() == id)));
    let supply: i128 = view(contract, "ft_total_supply", json!({})).await?.as_str().unwrap().parse()?;
    assert_eq!(replay.values().sum::<i128>(), supply);
    assert!(supply > 0);
    Ok(())
}

fn published_abi() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../deploy/abi.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("{} is missing; run scripts/build-contract.sh", path.display()));
    serde_json::from_str(&text).unwrap()
}

#[test]
fn the_published_abi_lists_exactly_the_exported_methods() {
    let abi = published_abi();
    let mut described: Vec<String> = abi["body"]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|function| function["name"].as_str().unwrap().to_string())
        .collect();
    described.sort();
    let exported: Vec<String> = exported_functions().into_iter().filter(|name| name != "__contract_abi").collect();
    assert_eq!(described, exported);

    let function = |name: &str| abi["body"]["functions"].as_array().unwrap().iter().find(|f| f["name"] == name).unwrap().clone();
    assert_eq!(function("new")["modifiers"], json!(["init"]));
    assert_eq!(function("ft_resolve_transfer")["modifiers"], json!(["private"]));
    for payable in ["catch", "vaccinate", "ft_transfer", "ft_transfer_call", "storage_deposit", "storage_withdraw", "storage_unregister"] {
        assert_eq!(function(payable)["modifiers"], json!(["payable"]), "{payable}");
    }
    for view in ["get_config", "get_stats", "get_host", "get_hosts", "get_events", "quote_catch", "ft_metadata", "ft_balance_of"] {
        assert_eq!(function(view)["kind"], "view", "{view}");
    }
    let definitions = abi["body"]["root_schema"]["definitions"].as_object().unwrap();
    for name in ["Config", "Globals", "Event", "HostView", "StatsView", "StorageBalance", "FungibleTokenMetadata"] {
        assert!(definitions.contains_key(name), "{name} missing from the ABI schema");
    }
}

#[tokio::test]
async fn the_abi_embedded_in_the_wasm_matches_the_published_file() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let compressed = world.contract.view("__contract_abi").await?.result;
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(compressed.as_slice())?;
    let mut text = String::new();
    std::io::Read::read_to_string(&mut decoder, &mut text)?;
    let embedded: Value = serde_json::from_str(&text)?;
    assert_eq!(embedded["body"], published_abi()["body"]);
    assert_eq!(embedded["metadata"]["name"], "contagion-contract");
    Ok(())
}

#[tokio::test]
async fn contract_source_metadata_declares_the_implemented_standards() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let metadata = view(&world.contract, "contract_source_metadata", json!({})).await?;
    let standards: Vec<(String, String)> = metadata["standards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| (entry["standard"].as_str().unwrap().to_string(), entry["version"].as_str().unwrap().to_string()))
        .collect();
    for expected in [("nep141", "1.0.0"), ("nep145", "1.0.0"), ("nep148", "1.0.0"), ("nep297", "1.0.0"), ("nep330", "1.3.0")] {
        assert!(standards.contains(&(expected.0.to_string(), expected.1.to_string())), "{expected:?} missing in {standards:?}");
    }
    assert_eq!(metadata["version"], "0.1.0");
    Ok(())
}

#[tokio::test]
async fn storage_management_keeps_registered_accounts_and_answers_withdraw() -> anyhow::Result<()> {
    let world = deploy(config(600, 3_000)).await?;
    let bob = world.worker.dev_create_account().await?;
    register(&world, &bob).await?;

    let withdrawn = bob
        .call(world.contract.id(), "storage_withdraw")
        .args_json(json!({}))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?;
    let balance: Value = withdrawn.json()?;
    assert_eq!(balance, json!({ "total": STORAGE_MIN.to_string(), "available": "0" }));

    let refused = bob
        .call(world.contract.id(), "storage_unregister")
        .args_json(json!({ "force": true }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert!(refused.is_failure());
    let still = view(&world.contract, "storage_balance_of", json!({ "account_id": bob.id() })).await?;
    assert_eq!(still["total"], STORAGE_MIN.to_string());

    let stranger = world.worker.dev_create_account().await?;
    let nothing: bool = stranger
        .call(world.contract.id(), "storage_unregister")
        .args_json(json!({}))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?
        .into_result()?
        .json()?;
    assert!(!nothing);
    Ok(())
}
