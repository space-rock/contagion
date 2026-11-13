pub mod account;
pub mod config;
pub mod engine;
pub mod error;
pub mod event;
pub mod ledger;
pub mod strnum;

pub use account::{Account, EndReason, Globals, Status};
pub use config::Config;
pub use engine::{apply, child_mint, init, is_expired, is_named, required_fee, reserve_pool, vax_cost_now, Action};
pub use error::Error;
pub use event::Event;
pub use ledger::{Ledger, MemLedger};

pub const BPS: u128 = 10_000;
pub const POINTS_CARRIER: u64 = 4;
pub const POINTS_PARENT: u64 = 2;
pub const POINTS_GRANDPARENT: u64 = 1;
pub const VAX_STEP_BPS: u128 = 10;
