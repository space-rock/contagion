use crate::Contract;
use contagion_core::{Account, Globals, Ledger};

impl Ledger for Contract {
    fn account(&self, id: &str) -> Account {
        self.accounts.get(id).cloned().unwrap_or_default()
    }

    fn put(&mut self, id: &str, account: Account) {
        if self.accounts.insert(id.to_string(), account).is_none() {
            self.index.push(id.to_string());
        }
    }

    fn globals(&self) -> Globals {
        self.globals.clone()
    }

    fn set_globals(&mut self, globals: Globals) {
        self.globals = globals;
    }
}
