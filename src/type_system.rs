//! Basic Neo type representations derived from Solidity type strings.

mod parse;
mod types;

pub(crate) use types::*;

#[cfg(test)]
mod tests;
