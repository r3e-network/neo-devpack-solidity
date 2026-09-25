use super::*;

mod contract;
mod crypto;
mod dispatch;
mod iterators;
mod runtime;
mod storage;

// S6 fix — re-export the CallFlags bitmask constants so every syscall handler
// (storage / runtime / contract) and the contract-call dispatcher can gate on
// them without each file reaching into the `storage` submodule. `pub(super)`
// makes them visible to the `execution` module too, where
// `handle_contract_call` lives. (`READ_STATES`/`WRITE_STATES` stay private to
// `storage` — they have no cross-module consumer.)
pub(super) use storage::{CALL_FLAG_ALL, CALL_FLAG_ALLOW_CALL, CALL_FLAG_ALLOW_NOTIFY};
