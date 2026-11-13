mod events;
mod ledger;
mod views;

pub use views::{HostView, LogView, QuoteView, StatsView};

use contagion_core::{apply, init, is_named, reserve_pool, vax_cost_now, Account, Action, Config, Event, Globals, Ledger};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::receiver::ext_ft_receiver;
use near_contract_standards::fungible_token::resolver::{ext_ft_resolver, FungibleTokenResolver};
use near_contract_standards::storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement};
use near_sdk::json_types::U128;
use near_sdk::store::{LookupMap, Vector};
use near_sdk::{
    assert_one_yocto, env, near, require, AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};

pub const STORAGE_MIN: u128 = 3_000_000_000_000_000_000_000;
pub const DECIMALS: u8 = 18;
const MAX_PAGE: u64 = 200;
const MAX_RESULT_LENGTH: usize = "\"+340282366920938463463374607431768211455\"".len();
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas::from_tgas(5);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas::from_tgas(30);
const ICON: &str = include_str!("icon.txt");

#[near(serializers = [borsh])]
pub struct LogEntry {
    pub at_ns: u64,
    pub event: Event,
}

#[near(
    contract_state,
    contract_metadata(
        standard(standard = "nep141", version = "1.0.0"),
        standard(standard = "nep145", version = "1.0.0"),
        standard(standard = "nep148", version = "1.0.0"),
        standard(standard = "nep297", version = "1.0.0")
    )
)]
#[derive(PanicOnDefault)]
pub struct Contract {
    pub(crate) config: Config,
    pub(crate) globals: Globals,
    pub(crate) accounts: LookupMap<String, Account>,
    pub(crate) index: Vector<String>,
    pub(crate) log: Vector<LogEntry>,
}

fn refund(to: &AccountId, yocto: u128) {
    if yocto > 0 {
        let _ = Promise::new(to.clone()).transfer(NearToken::from_yoctonear(yocto));
    }
}

impl Contract {
    fn run(&mut self, action: Action, memo: Option<String>) -> Vec<Event> {
        let cfg = self.config.clone();
        let now = env::block_timestamp();
        let events = apply(self, &cfg, action, now).unwrap_or_else(|error| env::panic_str(&error.to_string()));
        events::emit(&events, memo.as_deref());
        for event in &events {
            if events::is_logged(event) {
                self.log.push(LogEntry {
                    at_ns: now,
                    event: event.clone(),
                });
            }
        }
        events
    }

    fn registered(&self, id: &AccountId) -> bool {
        self.accounts.contains_key(id.as_str())
    }

    fn transfer_checked(&mut self, sender: &AccountId, receiver: &AccountId, amount: u128, memo: Option<String>) {
        require!(
            self.registered(receiver),
            "receiver is not registered, call storage_deposit first"
        );
        self.run(
            Action::Transfer {
                from: sender.to_string(),
                to: receiver.to_string(),
                amount,
            },
            memo,
        );
    }

    fn storage_balance() -> StorageBalance {
        StorageBalance {
            total: NearToken::from_yoctonear(STORAGE_MIN),
            available: NearToken::from_yoctonear(0),
        }
    }

    fn page(from: u64, limit: u64, len: u32) -> std::ops::Range<u32> {
        let start = from.min(len as u64) as u32;
        let end = (from.saturating_add(limit.min(MAX_PAGE))).min(len as u64) as u32;
        start..end
    }
}

#[near]
impl Contract {
    #[init]
    pub fn new(config: Config, patient_zeros: Vec<AccountId>) -> Self {
        config.validate().unwrap_or_else(|message| env::panic_str(&message));
        let mut contract = Self {
            config: config.clone(),
            globals: Globals::default(),
            accounts: LookupMap::new(b"a".to_vec()),
            index: Vector::new(b"i".to_vec()),
            log: Vector::new(b"l".to_vec()),
        };
        let zeros: Vec<String> = patient_zeros.iter().map(AccountId::to_string).collect();
        init(&mut contract, &config, &zeros, env::block_timestamp())
            .unwrap_or_else(|message| env::panic_str(&message));
        contract
    }

    #[payable]
    pub fn catch(&mut self, carrier: AccountId) {
        let target = env::predecessor_account_id();
        let deposit = env::attached_deposit().as_yoctonear();
        let events = self.run(
            Action::Catch {
                carrier: carrier.to_string(),
                target: target.to_string(),
                deposit,
            },
            None,
        );
        let fee = events
            .iter()
            .find_map(|event| match event {
                Event::Catch { fee, .. } => Some(*fee),
                _ => None,
            })
            .unwrap_or(0);
        refund(&target, deposit - fee);
    }

    pub fn recover(&mut self) {
        let account = env::predecessor_account_id();
        self.run(
            Action::Recover {
                account: account.to_string(),
            },
            None,
        );
    }

    pub fn cull(&mut self, account: AccountId) {
        let caller = env::predecessor_account_id();
        let registered = self.registered(&caller);
        let events = self.run(
            Action::Cull {
                account: account.to_string(),
                caller: caller.to_string(),
            },
            None,
        );
        let paid = events
            .iter()
            .any(|event| matches!(event, Event::Transfer { to, .. } if *to == caller.as_str()));
        require!(
            registered || !paid,
            "storage_deposit first to receive the plague doctor cut"
        );
    }

    #[payable]
    pub fn vaccinate(&mut self, account: AccountId) {
        let by = env::predecessor_account_id();
        let attached = env::attached_deposit().as_yoctonear();
        let needed = if self.registered(&account) { 0 } else { STORAGE_MIN };
        require!(attached >= needed, "attach the storage minimum for an unregistered target");
        self.run(
            Action::Vaccinate {
                account: account.to_string(),
                by: by.to_string(),
            },
            None,
        );
        refund(&by, attached - needed);
    }

    pub fn claim(&mut self) -> U128 {
        let account = env::predecessor_account_id();
        let events = self.run(
            Action::Claim {
                account: account.to_string(),
            },
            None,
        );
        let amount = events
            .iter()
            .find_map(|event| match event {
                Event::Claim { amount, .. } => Some(*amount),
                _ => None,
            })
            .unwrap_or(0);
        U128(amount)
    }

    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn get_stats(&self) -> StatsView {
        let now = env::block_timestamp();
        let effective = views::effective_globals(&self.config, &self.globals, now);
        StatsView {
            season_over: effective.ended_at.is_some(),
            season_ends_ns: effective
                .ended_at
                .unwrap_or_else(|| views::season_deadline(&self.config, &self.globals))
                .into(),
            reserve_pool: reserve_pool(&self.config, &self.globals).unwrap_or(0).into(),
            vax_cost: vax_cost_now(&self.config, &self.globals).unwrap_or(u128::MAX).into(),
            globals: effective,
            now_ns: now.into(),
            accounts: self.index.len() as u64,
            events: self.log.len() as u64,
            storage_usage: env::storage_usage(),
            balance: env::account_balance().as_yoctonear().into(),
        }
    }

    pub fn get_host(&self, account_id: AccountId) -> Option<HostView> {
        self.accounts
            .get(account_id.as_str())
            .map(|account| views::host_view(account_id.as_str(), account, &self.config, &self.globals))
    }

    pub fn get_hosts(&self, from: u64, limit: u64) -> Vec<HostView> {
        Self::page(from, limit, self.index.len())
            .filter_map(|position| {
                let id = self.index.get(position)?;
                let account = self.accounts.get(id.as_str())?;
                Some(views::host_view(id, account, &self.config, &self.globals))
            })
            .collect()
    }

    pub fn quote_catch(&self, carrier: AccountId) -> QuoteView {
        views::quote(&self.config, &self.globals, &self.account(carrier.as_str()))
    }

    pub fn get_events(&self, from: u64, limit: u64) -> Vec<LogView> {
        Self::page(from, limit, self.log.len())
            .filter_map(|position| {
                let entry = self.log.get(position)?;
                Some(LogView {
                    index: position as u64,
                    at_ns: entry.at_ns.into(),
                    event: entry.event.clone(),
                })
            })
            .collect()
    }

    pub fn is_named_account(&self, account_id: String) -> bool {
        is_named(&account_id)
    }
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        let sender = env::predecessor_account_id();
        self.transfer_checked(&sender, &receiver_id, amount.0, memo);
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        require!(env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL, "more gas is required");
        let sender = env::predecessor_account_id();
        self.transfer_checked(&sender, &receiver_id, amount.0, memo);
        let receiver_gas = env::prepaid_gas().saturating_sub(GAS_FOR_FT_TRANSFER_CALL);
        ext_ft_receiver::ext(receiver_id.clone())
            .with_static_gas(receiver_gas)
            .ft_on_transfer(sender.clone(), amount, msg)
            .then(
                ext_ft_resolver::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .with_unused_gas_weight(0)
                    .ft_resolve_transfer(sender, receiver_id, amount),
            )
            .into()
    }

    fn ft_total_supply(&self) -> U128 {
        U128(self.globals.supply)
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.account(account_id.as_str()).balance())
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> U128 {
        let amount = amount.0;
        let unused = env::promise_result_checked(0, MAX_RESULT_LENGTH)
            .ok()
            .and_then(|value| near_sdk::serde_json::from_slice::<U128>(&value).ok())
            .map_or(amount, |value| value.0.min(amount));
        let refundable = unused.min(self.account(receiver_id.as_str()).liquid);
        if refundable == 0 {
            return U128(amount);
        }
        self.run(
            Action::Transfer {
                from: receiver_id.to_string(),
                to: sender_id.to_string(),
                amount: refundable,
            },
            Some("refund".to_string()),
        );
        U128(amount - refundable)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Contagion".to_string(),
            symbol: "FLU".to_string(),
            icon: Some(ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: DECIMALS,
        }
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance {
        let _ = registration_only;
        let payer = env::predecessor_account_id();
        let account = account_id.unwrap_or_else(|| payer.clone());
        let attached = env::attached_deposit().as_yoctonear();
        if self.registered(&account) {
            refund(&payer, attached);
        } else {
            require!(attached >= STORAGE_MIN, "attach at least the storage minimum");
            self.put(account.as_str(), Account::default());
            refund(&payer, attached - STORAGE_MIN);
        }
        Self::storage_balance()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        assert_one_yocto();
        require!(
            self.registered(&env::predecessor_account_id()),
            "the account is not registered"
        );
        require!(
            amount.map_or(true, |requested| requested.is_zero()),
            "no storage balance is available to withdraw"
        );
        Self::storage_balance()
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let _ = force;
        if !self.registered(&env::predecessor_account_id()) {
            return false;
        }
        env::panic_str("registered accounts stay in the outbreak and cannot be unregistered")
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: NearToken::from_yoctonear(STORAGE_MIN),
            max: Some(NearToken::from_yoctonear(STORAGE_MIN)),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.registered(&account_id).then(Self::storage_balance)
    }
}
