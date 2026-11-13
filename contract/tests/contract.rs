use contagion_contract::{Contract, STORAGE_MIN};
use contagion_core::{Config, EndReason, Status};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadataProvider;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::storage_management::StorageManagement;
use near_sdk::json_types::U128;
use near_sdk::mock::MockAction;
use near_sdk::serde_json::{self, json, Value};
use near_sdk::test_utils::{get_created_receipts, get_logs, VMContextBuilder};
use near_sdk::{test_vm_config, testing_env, AccountId, Gas, NearToken, PromiseResult, RuntimeFeesConfig};

const T0: u64 = 1_000;

fn id(name: &str) -> AccountId {
    format!("{name}.testnet").parse().unwrap()
}

fn context(caller: &str, deposit: u128, now: u64) -> near_sdk::VMContext {
    VMContextBuilder::new()
        .current_account_id(id("flu"))
        .predecessor_account_id(id(caller))
        .attached_deposit(NearToken::from_yoctonear(deposit))
        .block_timestamp(now)
        .prepaid_gas(Gas::from_tgas(300))
        .build()
}

fn as_user(caller: &str, deposit: u128, now: u64) {
    testing_env!(context(caller, deposit, now));
}

fn setup() -> Contract {
    as_user("deployer", 0, T0);
    Contract::new(Config::reference(), vec![id("zero")])
}

fn fee_for(contract: &Contract, carrier: &str) -> u128 {
    contract.quote_catch(id(carrier)).fee.0
}

fn catch(contract: &mut Contract, carrier: &str, target: &str, now: u64) {
    let fee = fee_for(contract, carrier);
    as_user(target, fee, now);
    contract.catch(id(carrier));
}

fn register(contract: &mut Contract, who: &str, now: u64) {
    as_user(who, STORAGE_MIN, now);
    contract.storage_deposit(None, Some(true));
}

fn refunded_to(who: &str) -> u128 {
    let account = id(who);
    get_created_receipts()
        .into_iter()
        .filter(|receipt| receipt.receiver_id == account)
        .flat_map(|receipt| receipt.actions)
        .map(|action| match action {
            MockAction::Transfer { deposit, .. } => deposit.as_yoctonear(),
            _ => 0,
        })
        .sum()
}

fn events_logged() -> Vec<Value> {
    get_logs()
        .iter()
        .filter_map(|line| line.strip_prefix("EVENT_JSON:"))
        .map(|json| serde_json::from_str(json).unwrap())
        .collect()
}

#[test]
fn init_makes_patient_zeros_contagious_and_config_readable() {
    let contract = setup();
    let zero = contract.get_host(id("zero")).unwrap();
    assert_eq!(zero.status, Status::Contagious);
    assert_eq!(zero.gen, 0);
    assert_eq!(contract.get_config(), Config::reference());
    assert_eq!(contract.ft_total_supply(), U128(0));
    let stats = contract.get_stats();
    assert_eq!(stats.globals.infected, 1);
    assert_eq!(stats.accounts, 1);
    assert!(!stats.season_over);
}

#[test]
#[should_panic(expected = "base must be above zero")]
fn init_rejects_an_invalid_config() {
    as_user("deployer", 0, T0);
    let mut config = Config::reference();
    config.base = 0;
    Contract::new(config, vec![id("zero")]);
}

#[test]
fn catch_mints_to_the_target_and_bonus_to_the_carrier() {
    let mut contract = setup();
    let quote = contract.quote_catch(id("zero"));
    assert!(quote.can_catch);
    assert_eq!((quote.mint.0, quote.fee.0, quote.gen), (600, 600, 1));
    catch(&mut contract, "zero", "alice", T0);
    assert_eq!(contract.ft_balance_of(id("alice")), U128(600));
    assert_eq!(contract.ft_balance_of(id("zero")), U128(180));
    assert_eq!(contract.ft_total_supply(), U128(780));
    let alice = contract.get_host(id("alice")).unwrap();
    assert_eq!(alice.parent.as_deref(), Some("zero.testnet"));
    assert_eq!(alice.viral.0, 600);
    assert_eq!(alice.liquid.0, 0);
    assert_eq!(contract.get_stats().globals.fees_locked, 600);
}

#[test]
fn catch_refunds_the_deposit_above_the_required_fee() {
    let mut contract = setup();
    as_user("alice", 600 + 5_000, T0);
    contract.catch(id("zero"));
    assert_eq!(refunded_to("alice"), 5_000);
}

#[test]
fn catch_with_the_exact_fee_creates_no_refund() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    assert_eq!(refunded_to("alice"), 0);
}

#[test]
#[should_panic(expected = "attached deposit is below the required fee")]
fn catch_with_a_short_deposit_panics() {
    let mut contract = setup();
    as_user("alice", 599, T0);
    contract.catch(id("zero"));
}

#[test]
#[should_panic(expected = "carrier incubation clock expired")]
fn catching_from_an_expired_carrier_panics() {
    let mut contract = setup();
    as_user("alice", 600, T0 + 100);
    contract.catch(id("zero"));
}

#[test]
fn quote_explains_why_a_carrier_cannot_be_caught_from() {
    let contract = setup();
    let stranger = contract.quote_catch(id("nobody"));
    assert!(!stranger.can_catch);
    assert_eq!(stranger.reason.as_deref(), Some("carrier is not contagious"));
    as_user("anyone", 0, T0 + 100);
    let expired = contract.quote_catch(id("zero"));
    assert!(!expired.can_catch);
    assert_eq!(expired.reason.as_deref(), Some("carrier incubation clock expired"));
}

#[test]
fn catch_emits_nep141_mint_events_and_a_contagion_event() {
    let mut contract = setup();
    as_user("alice", 600, T0);
    contract.catch(id("zero"));
    let events = events_logged();
    let names: Vec<(String, String)> = events
        .iter()
        .map(|e| (e["standard"].as_str().unwrap().to_string(), e["event"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("contagion".to_string(), "catch".to_string()),
            ("nep141".to_string(), "ft_mint".to_string()),
            ("nep141".to_string(), "ft_mint".to_string()),
        ]
    );
    assert_eq!(events[1]["data"][0]["owner_id"], "alice.testnet");
    assert_eq!(events[1]["data"][0]["amount"], "600");
    assert_eq!(events[0]["data"][0]["fee"], "600");
    assert_eq!(events[0]["version"], "1.0.0");
}

#[test]
fn log_stores_contagion_events_only_and_pages() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    catch(&mut contract, "zero", "bob", T0 + 1);
    as_user("alice", 0, T0 + 2);
    contract.recover();
    assert_eq!(contract.get_stats().events, 3);
    let page = contract.get_events(1, 10);
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].index, 1);
    assert_eq!(contract.get_events(0, 1).len(), 1);
    assert!(contract.get_events(50, 10).is_empty());
}

#[test]
fn hosts_page_in_registration_order() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    catch(&mut contract, "zero", "bob", T0);
    let all = contract.get_hosts(0, 10);
    let ids: Vec<&str> = all.iter().map(|h| h.account_id.as_str()).collect();
    assert_eq!(ids, vec!["zero.testnet", "alice.testnet", "bob.testnet"]);
    assert_eq!(contract.get_hosts(1, 1)[0].account_id, "alice.testnet");
    assert!(contract.get_hosts(10, 5).is_empty());
    assert!(contract.get_host(id("nobody")).is_none());
}

#[test]
fn views_flag_expired_hosts_without_a_transaction() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("anyone", 0, T0 + 50);
    assert!(!contract.get_host(id("alice")).unwrap().expired);
    as_user("anyone", 0, T0 + 100);
    let alice = contract.get_host(id("alice")).unwrap();
    assert!(alice.expired);
    assert_eq!(alice.deadline_ns.0, T0 + 100);
}

#[test]
#[should_panic(expected = "storage_deposit first")]
fn an_unregistered_plague_doctor_cannot_collect_a_cut() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("doc", 0, T0 + 100);
    contract.cull(id("alice"));
}

#[test]
fn a_registered_plague_doctor_collects_the_cut() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "doc", T0 + 100);
    as_user("doc", 0, T0 + 100);
    contract.cull(id("alice"));
    assert_eq!(contract.ft_balance_of(id("doc")), U128(30));
    let alice = contract.get_host(id("alice")).unwrap();
    assert_eq!((alice.status, alice.viral.0, alice.liquid.0), (Status::Recovered, 0, 300));
    assert_eq!(contract.ft_total_supply(), U128(180 + 600 - 270));
}

#[test]
#[should_panic(expected = "account is not expired")]
fn culling_a_live_carrier_panics() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "doc", T0 + 1);
    as_user("doc", 0, T0 + 50);
    contract.cull(id("alice"));
}

#[test]
fn storage_deposit_registers_once_and_refunds_the_rest() {
    let mut contract = setup();
    assert!(contract.storage_balance_of(id("carol")).is_none());
    as_user("carol", STORAGE_MIN + 777, T0);
    let balance = contract.storage_deposit(None, None);
    assert_eq!(balance.total, NearToken::from_yoctonear(STORAGE_MIN));
    assert_eq!(refunded_to("carol"), 777);
    assert!(contract.storage_balance_of(id("carol")).is_some());

    as_user("carol", 500, T0);
    contract.storage_deposit(None, None);
    assert_eq!(refunded_to("carol"), 500);
    assert_eq!(contract.get_stats().accounts, 2);
}

#[test]
fn storage_deposit_can_register_someone_else() {
    let mut contract = setup();
    as_user("carol", STORAGE_MIN, T0);
    contract.storage_deposit(Some(id("dave")), None);
    assert!(contract.storage_balance_of(id("dave")).is_some());
    assert!(contract.storage_balance_of(id("carol")).is_none());
}

#[test]
#[should_panic(expected = "attach at least the storage minimum")]
fn storage_deposit_requires_the_minimum() {
    let mut contract = setup();
    as_user("carol", STORAGE_MIN - 1, T0);
    contract.storage_deposit(None, None);
}

#[test]
fn bounds_and_metadata_describe_the_token() {
    let contract = setup();
    let bounds = contract.storage_balance_bounds();
    assert_eq!(bounds.min, NearToken::from_yoctonear(STORAGE_MIN));
    assert_eq!(bounds.max, Some(NearToken::from_yoctonear(STORAGE_MIN)));
    let metadata = contract.ft_metadata();
    assert_eq!(
        (metadata.symbol.as_str(), metadata.decimals, metadata.spec.as_str()),
        ("FLU", 18, "ft-1.0.0")
    );
    assert!(metadata.icon.unwrap().starts_with("data:image/svg+xml"));
}

#[test]
fn only_liquid_tokens_can_be_transferred() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "bob", T0);
    as_user("alice", 1, T0 + 1);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        contract.ft_transfer(id("bob"), U128(1), None);
    }));
    assert!(result.is_err(), "viral balance must be locked");

    as_user("alice", 0, T0 + 2);
    contract.recover();
    as_user("alice", 1, T0 + 3);
    contract.ft_transfer(id("bob"), U128(40), Some("thanks".to_string()));
    assert_eq!(contract.ft_balance_of(id("bob")), U128(40));
    assert_eq!(contract.ft_balance_of(id("alice")), U128(540 - 40));
    let events = events_logged();
    let transfer = events.iter().find(|e| e["event"] == "ft_transfer").unwrap();
    assert_eq!(transfer["data"][0]["old_owner_id"], "alice.testnet");
    assert_eq!(transfer["data"][0]["new_owner_id"], "bob.testnet");
    assert_eq!(transfer["data"][0]["memo"], "thanks");
}

#[test]
#[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
fn ft_transfer_requires_exactly_one_yocto() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "bob", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 0, T0 + 2);
    contract.ft_transfer(id("bob"), U128(1), None);
}

#[test]
#[should_panic(expected = "receiver is not registered")]
fn ft_transfer_to_an_unregistered_receiver_panics() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 1, T0 + 2);
    contract.ft_transfer(id("nobody"), U128(1), None);
}

#[test]
fn vaccinating_an_unregistered_target_needs_the_storage_minimum() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", STORAGE_MIN + 123, T0 + 2);
    contract.vaccinate(id("immune"));
    assert_eq!(contract.get_host(id("immune")).unwrap().status, Status::Immune);
    assert_eq!(refunded_to("alice"), 123);
    assert_eq!(contract.get_host(id("alice")).unwrap().liquid.0, 540 - 50);
}

#[test]
#[should_panic(expected = "attach the storage minimum")]
fn vaccinating_an_unregistered_target_without_a_deposit_panics() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 0, T0 + 2);
    contract.vaccinate(id("immune"));
}

#[test]
fn vaccinating_a_registered_target_needs_no_deposit() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "bob", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 0, T0 + 2);
    contract.vaccinate(id("bob"));
    assert_eq!(contract.get_host(id("bob")).unwrap().status, Status::Immune);
}

#[test]
fn claim_pays_the_reserve_after_the_season() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    let config = contract.get_config();
    let after = T0 + config.season_max_ns;
    as_user("zero", 0, after);
    let stats = contract.get_stats();
    assert!(stats.season_over);
    assert_eq!(stats.globals.ended_at, Some(after));
    assert_eq!(stats.globals.end_reason, Some(EndReason::Time));
    let paid = contract.claim();
    let pool = contract.get_stats().reserve_pool;
    assert_eq!(paid.0, pool.0);
    assert!(paid.0 > 0);
    assert_eq!(contract.get_stats().globals.reserve_paid, paid.0);
}

#[test]
#[should_panic(expected = "the season is still open")]
fn claim_before_the_season_ends_panics() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("zero", 0, T0 + 1);
    contract.claim();
}

#[test]
fn ft_transfer_call_schedules_the_receiver_call_and_a_resolver() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "dex", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 1, T0 + 2);
    let _ = contract.ft_transfer_call(id("dex"), U128(100), None, "swap".to_string());
    assert_eq!(contract.ft_balance_of(id("dex")), U128(100));
    let receipts = get_created_receipts();
    let receivers: Vec<String> = receipts.iter().map(|r| r.receiver_id.to_string()).collect();
    assert!(receivers.contains(&"dex.testnet".to_string()));
    assert!(receivers.contains(&"flu.testnet".to_string()));
}

fn resolve(contract: &mut Contract, outcome: PromiseResult, now: u64) -> U128 {
    testing_env!(
        context("flu", 0, now),
        test_vm_config(),
        RuntimeFeesConfig::test(),
        Default::default(),
        vec![outcome]
    );
    contract.ft_resolve_transfer(id("alice"), id("dex"), U128(100))
}

fn returned(unused: u128) -> PromiseResult {
    PromiseResult::Successful(format!("\"{unused}\"").into_bytes())
}

fn staged_transfer_call() -> Contract {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "dex", T0);
    as_user("alice", 0, T0 + 1);
    contract.recover();
    as_user("alice", 1, T0 + 2);
    let _ = contract.ft_transfer_call(id("dex"), U128(100), None, "swap".to_string());
    contract
}

#[test]
fn resolve_refunds_what_the_receiver_did_not_use() {
    let mut contract = staged_transfer_call();
    let used = resolve(&mut contract, returned(30), T0 + 3);
    assert_eq!(used, U128(70));
    assert_eq!(contract.ft_balance_of(id("dex")), U128(70));
    assert_eq!(contract.ft_balance_of(id("alice")), U128(540 - 100 + 30));
}

#[test]
fn resolve_refunds_everything_when_the_receiver_call_failed() {
    let mut contract = staged_transfer_call();
    let used = resolve(&mut contract, PromiseResult::Failed, T0 + 3);
    assert_eq!(used, U128(0));
    assert_eq!(contract.ft_balance_of(id("dex")), U128(0));
    assert_eq!(contract.ft_balance_of(id("alice")), U128(540));
}

#[test]
fn resolve_keeps_everything_when_the_receiver_used_it_all() {
    let mut contract = staged_transfer_call();
    let used = resolve(&mut contract, returned(0), T0 + 3);
    assert_eq!(used, U128(100));
    assert_eq!(contract.ft_balance_of(id("dex")), U128(100));
}

#[test]
fn resolve_cannot_refund_more_than_the_receiver_still_holds() {
    let mut contract = staged_transfer_call();
    as_user("dex", 1, T0 + 3);
    register(&mut contract, "elsewhere", T0 + 3);
    as_user("dex", 1, T0 + 3);
    contract.ft_transfer(id("elsewhere"), U128(80), None);
    let used = resolve(&mut contract, PromiseResult::Failed, T0 + 4);
    assert_eq!(used, U128(80));
    assert_eq!(contract.ft_balance_of(id("dex")), U128(0));
}

#[test]
fn the_token_icon_is_the_brand_coin_embedded_as_a_base64_svg() {
    use base64::Engine;
    let contract = setup();
    let icon = contract.ft_metadata().icon.expect("icon is set");
    let prefix = "data:image/svg+xml;base64,";
    assert!(icon.starts_with(prefix));
    assert!(icon.len() <= 4096, "icon is {} bytes", icon.len());
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&icon[prefix.len()..])
        .expect("valid base64");
    let svg = String::from_utf8(decoded).expect("utf-8 svg");
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("<image"));
    assert_eq!(svg.matches("http").count(), 1, "only the svg namespace may mention http");
    assert_eq!(svg, include_str!("../../brand/coin.svg"));
}

#[test]
fn nep141_logs_have_the_exact_standard_shape() {
    let mut contract = setup();
    as_user("alice", 600, T0);
    contract.catch(id("zero"));
    let logs = get_logs();
    assert_eq!(
        logs[1],
        r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_mint","data":[{"owner_id":"alice.testnet","amount":"600"}]}"#
    );
    assert_eq!(
        logs[2],
        r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_mint","data":[{"owner_id":"zero.testnet","amount":"180"}]}"#
    );
    assert!(logs[0].starts_with(r#"EVENT_JSON:{"standard":"contagion","version":"1.0.0","event":"catch","data":[{"#));

    as_user("alice", 0, T0 + 1);
    contract.recover();
    let logs = get_logs();
    assert_eq!(
        logs[0],
        r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_burn","data":[{"owner_id":"alice.testnet","amount":"60"}]}"#
    );

    register(&mut contract, "bob", T0 + 2);
    as_user("alice", 1, T0 + 3);
    contract.ft_transfer(id("bob"), U128(40), None);
    assert_eq!(
        get_logs()[0],
        r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_transfer","data":[{"old_owner_id":"alice.testnet","new_owner_id":"bob.testnet","amount":"40"}]}"#
    );
    as_user("alice", 1, T0 + 4);
    contract.ft_transfer(id("bob"), U128(1), Some("tip".to_string()));
    assert_eq!(
        get_logs()[0],
        r#"EVENT_JSON:{"standard":"nep141","version":"1.0.0","event":"ft_transfer","data":[{"old_owner_id":"alice.testnet","new_owner_id":"bob.testnet","amount":"1","memo":"tip"}]}"#
    );
}

#[test]
fn culling_and_vaccinating_and_claiming_log_their_token_movements() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    register(&mut contract, "doc", T0 + 100);
    as_user("doc", 0, T0 + 100);
    contract.cull(id("alice"));
    let kinds: Vec<(String, String)> = events_logged()
        .iter()
        .map(|e| (e["event"].as_str().unwrap().to_string(), e["data"][0]["amount"].as_str().unwrap_or("").to_string()))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("ft_transfer".to_string(), "30".to_string()),
            ("ft_burn".to_string(), "270".to_string()),
            ("cull".to_string(), String::new()),
        ]
    );

    as_user("zero", 0, T0 + 200);
    contract.recover();
    as_user("zero", STORAGE_MIN, T0 + 201);
    contract.vaccinate(id("immune"));
    let logged = events_logged();
    assert_eq!(logged[0]["event"], "ft_burn");
    assert_eq!(logged[0]["data"][0]["amount"], "50");
    assert_eq!(logged[1]["event"], "vaccinate");

    let after = T0 + contract.get_config().season_max_ns;
    as_user("zero", 0, after);
    let paid = contract.claim();
    let logged = events_logged();
    let mint = logged.iter().find(|e| e["event"] == "ft_mint").unwrap();
    assert_eq!(mint["data"][0]["amount"], paid.0.to_string());
    assert!(logged.iter().any(|e| e["event"] == "season_end"));
}

#[test]
fn view_json_keeps_decimal_strings_and_snake_case() {
    let mut contract = setup();
    catch(&mut contract, "zero", "alice", T0);
    as_user("anyone", 0, T0 + 5);
    let host = serde_json::to_value(contract.get_host(id("alice")).unwrap()).unwrap();
    assert_eq!(host["viral"], "600");
    assert_eq!(host["liquid"], "0");
    assert_eq!(host["mint"], "600");
    assert_eq!(host["status"], "contagious");
    assert_eq!(host["last_spread_ns"], T0.to_string());
    assert_eq!(host["deadline_ns"], (T0 + 100).to_string());
    assert_eq!(host["points"], 0);
    let quote = serde_json::to_value(contract.quote_catch(id("alice"))).unwrap();
    assert_eq!(quote["fee"], "360");
    assert_eq!(quote["can_catch"], true);
    let stats = serde_json::to_value(contract.get_stats()).unwrap();
    assert_eq!(stats["globals"]["supply"], "780");
    assert_eq!(stats["globals"]["ended_at"], Value::Null);
    assert_eq!(stats["globals"]["end_reason"], Value::Null);
    assert_eq!(stats["vax_cost"], "50");
    assert_eq!(stats["now_ns"], (T0 + 5).to_string());
    assert!(stats["reserve_pool"].is_string() && stats["balance"].is_string());
    assert!(stats["accounts"].is_u64() && stats["events"].is_u64() && stats["storage_usage"].is_u64());
    let events = serde_json::to_value(contract.get_events(0, 5)).unwrap();
    assert_eq!(events[0]["at_ns"], T0.to_string());
    assert_eq!(events[0]["event"]["event"], "catch");
}

#[test]
fn metadata_and_storage_json_match_the_standards() {
    let mut contract = setup();
    let metadata = serde_json::to_value(contract.ft_metadata()).unwrap();
    assert_eq!(metadata["spec"], "ft-1.0.0");
    assert_eq!(metadata["name"], "Contagion");
    assert_eq!(metadata["symbol"], "FLU");
    assert_eq!(metadata["decimals"], 18);
    assert_eq!(metadata["reference"], Value::Null);
    assert_eq!(metadata["reference_hash"], Value::Null);
    assert!(metadata["icon"].as_str().unwrap().starts_with("data:image/svg+xml;base64,"));
    let bounds = serde_json::to_value(contract.storage_balance_bounds()).unwrap();
    assert_eq!(bounds, json!({ "min": STORAGE_MIN.to_string(), "max": STORAGE_MIN.to_string() }));
    register(&mut contract, "carol", T0);
    let balance = serde_json::to_value(contract.storage_balance_of(id("carol"))).unwrap();
    assert_eq!(balance, json!({ "total": STORAGE_MIN.to_string(), "available": "0" }));
}

#[test]
fn storage_withdraw_returns_the_balance_and_never_moves_funds() {
    let mut contract = setup();
    register(&mut contract, "carol", T0);
    as_user("carol", 1, T0);
    let balance = contract.storage_withdraw(None);
    assert_eq!(balance.total, NearToken::from_yoctonear(STORAGE_MIN));
    assert_eq!(balance.available, NearToken::from_yoctonear(0));
    as_user("carol", 1, T0);
    contract.storage_withdraw(Some(NearToken::from_yoctonear(0)));
    assert!(get_created_receipts().is_empty());
}

#[test]
#[should_panic(expected = "no storage balance is available to withdraw")]
fn storage_withdraw_of_a_positive_amount_panics() {
    let mut contract = setup();
    register(&mut contract, "carol", T0);
    as_user("carol", 1, T0);
    contract.storage_withdraw(Some(NearToken::from_yoctonear(1)));
}

#[test]
#[should_panic(expected = "the account is not registered")]
fn storage_withdraw_requires_registration() {
    let mut contract = setup();
    as_user("stranger", 1, T0);
    contract.storage_withdraw(None);
}

#[test]
#[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
fn storage_withdraw_requires_exactly_one_yocto() {
    let mut contract = setup();
    register(&mut contract, "carol", T0);
    as_user("carol", 0, T0);
    contract.storage_withdraw(None);
}

#[test]
fn storage_unregister_is_false_for_strangers_and_refused_for_the_registered() {
    let mut contract = setup();
    as_user("stranger", 1, T0);
    assert!(!contract.storage_unregister(None));
    register(&mut contract, "carol", T0);
    as_user("carol", 1, T0);
    let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| contract.storage_unregister(Some(true))));
    assert!(attempt.is_err());
    assert!(contract.storage_balance_of(id("carol")).is_some());
    assert_eq!(contract.get_stats().accounts, 2);
}

#[test]
#[should_panic(expected = "Requires attached deposit of exactly 1 yoctoNEAR")]
fn storage_unregister_requires_exactly_one_yocto() {
    let mut contract = setup();
    as_user("carol", 0, T0);
    contract.storage_unregister(None);
}

#[test]
fn resolve_treats_an_unreadable_receiver_result_as_everything_unused() {
    let mut contract = staged_transfer_call();
    let used = resolve(&mut contract, PromiseResult::Successful(b"not a number".to_vec()), T0 + 3);
    assert_eq!(used, U128(0));
    assert_eq!(contract.ft_balance_of(id("alice")), U128(540));
}

#[test]
fn resolve_caps_a_receiver_that_claims_more_unused_than_was_sent() {
    let mut contract = staged_transfer_call();
    let used = resolve(&mut contract, returned(10_000), T0 + 3);
    assert_eq!(used, U128(0));
    assert_eq!(contract.ft_balance_of(id("alice")), U128(540));
    assert_eq!(contract.ft_balance_of(id("dex")), U128(0));
}
