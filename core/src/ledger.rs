use crate::account::{Account, Globals, Status};
use std::collections::BTreeMap;

pub trait Ledger {
    fn account(&self, id: &str) -> Account;
    fn put(&mut self, id: &str, account: Account);
    fn globals(&self) -> Globals;
    fn set_globals(&mut self, globals: Globals);
}

#[derive(Clone, Debug, Default)]
pub struct MemLedger {
    pub accounts: BTreeMap<String, Account>,
    pub globals: Globals,
}

impl Ledger for MemLedger {
    fn account(&self, id: &str) -> Account {
        self.accounts.get(id).cloned().unwrap_or_default()
    }

    fn put(&mut self, id: &str, account: Account) {
        self.accounts.insert(id.to_string(), account);
    }

    fn globals(&self) -> Globals {
        self.globals.clone()
    }

    fn set_globals(&mut self, globals: Globals) {
        self.globals = globals;
    }
}

impl MemLedger {
    pub fn check_invariants(&self) -> Result<(), String> {
        let g = &self.globals;
        let balances: u128 = self.accounts.values().map(Account::balance).sum();
        if g.supply != balances {
            return Err(format!("supply {} != sum of balances {}", g.supply, balances));
        }
        let count = |wanted: &[Status]| {
            self.accounts.values().filter(|a| wanted.contains(&a.status)).count() as u32
        };
        let infected = count(&[Status::Contagious, Status::Recovered]);
        if g.infected != infected {
            return Err(format!("infected {} != counted {}", g.infected, infected));
        }
        let immune = count(&[Status::Immune]);
        if g.immune != immune {
            return Err(format!("immune {} != counted {}", g.immune, immune));
        }
        let points: u64 = self.accounts.values().map(|a| a.points).sum();
        if g.total_points != points {
            return Err(format!("total_points {} != sum {}", g.total_points, points));
        }
        let children: u32 = self.accounts.values().map(|a| a.children).sum();
        let with_parent = self.accounts.values().filter(|a| a.parent.is_some()).count() as u32;
        if children != with_parent {
            return Err(format!("children {children} != accounts with parent {with_parent}"));
        }
        let max_gen = self.accounts.values().map(|a| a.gen).max().unwrap_or(0);
        if g.max_gen != max_gen {
            return Err(format!("max_gen {} != {}", g.max_gen, max_gen));
        }
        if g.reserve_paid > 0 && g.ended_at.is_none() {
            return Err("reserve paid before season end".into());
        }
        Ok(())
    }
}
