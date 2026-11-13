use contagion_core::Event;
use near_contract_standards::fungible_token::events::{FtBurn, FtMint, FtTransfer};
use near_sdk::json_types::U128;
use near_sdk::serde_json::{json, Value};
use near_sdk::{env, AccountIdRef};

const STANDARD: &str = "contagion";
const VERSION: &str = "1.0.0";

fn account(id: &str) -> &AccountIdRef {
    AccountIdRef::new_or_panic(id)
}

fn log(event: &str, data: Value) {
    let payload = json!({
        "standard": STANDARD,
        "version": VERSION,
        "event": event,
        "data": [data],
    });
    env::log_str(&format!("EVENT_JSON:{payload}"));
}

pub fn is_logged(event: &Event) -> bool {
    !matches!(event, Event::Mint { .. } | Event::Burn { .. } | Event::Transfer { .. })
}

pub fn emit(events: &[Event], memo: Option<&str>) {
    for event in events {
        match event {
            Event::Mint { account: owner, amount } => FtMint {
                owner_id: account(owner),
                amount: U128(*amount),
                memo: None,
            }
            .emit(),
            Event::Burn { account: owner, amount } => FtBurn {
                owner_id: account(owner),
                amount: U128(*amount),
                memo: None,
            }
            .emit(),
            Event::Transfer { from, to, amount } => FtTransfer {
                old_owner_id: account(from),
                new_owner_id: account(to),
                amount: U128(*amount),
                memo,
            }
            .emit(),
            other => {
                let mut data = near_sdk::serde_json::to_value(other).expect("event serializes");
                let name = data
                    .as_object_mut()
                    .and_then(|fields| fields.remove("event"))
                    .and_then(|name| name.as_str().map(str::to_string))
                    .expect("event is tagged");
                log(&name, data);
            }
        }
    }
}
