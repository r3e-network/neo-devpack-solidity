use super::*;
use neo_devpack_solidity::neo::NEF_SOURCE_MAX_BYTES;
use neo_devpack_solidity::solidity::StateMutability;
use sha3::{Digest, Keccak256};
use std::fs;
use tempfile::tempdir;

include!("standard_json/basic.rs");
include!("standard_json/input_errors.rs");
include!("standard_json/manifest_helpers.rs");
include!("standard_json/warnings.rs");
include!("standard_json/codes.rs");
include!("standard_json/contract_filter.rs");
