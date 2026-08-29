use super::*;

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            // S2 fix — align the default gas budget with Neo N3 mainnet.
            // `MaxTransactionSystemFee` on mainnet is 2000 GAS = 2×10^9 gas
            // units; the previous 10M default was ~200× too tight, so any
            // contract that wrote a handful of storage slots (now charged at
            // the mainnet-aligned 100_000 gas/byte rate) exhausted the budget.
            // 1_000_000_000 sits comfortably under the mainnet cap while
            // leaving realistic headroom for storage-heavy contracts. Tests
            // that explicitly assert a tight gas budget construct their own
            // `RuntimeConfig` and are unaffected.
            gas_limit: 1_000_000_000,
            call_stack_limit: 1024,
            memory_limit: 1024 * 1024,       // 1MB
            storage_limit: 10 * 1024 * 1024, // 10MB
            network_magic: 0x4F454E,         // "NEO" magic
            enable_debugging: false,
            enable_tracing: false,
            strict_mode: true,
            // Target Neo N3 node version. v3.10.0 is the current MainNet/T5
            // TestNet release (Gorgon-hardfork preparation; v3.10.0 itself
            // activates NO hardfork, so opcode/syscall/gas/NEF stay
            // consensus-compatible with v3.7.x). This field is informational
            // metadata only — there are zero runtime consumers (no version-
            // gated logic, no manifest emission, no syscall return reads it).
            neo_version: "3.10.0".to_string(),
            contract_account: "0x0000000000000000000000000000000000000000".to_string(),
            default_block_height: 0,
            // Task #105 — 1_704_067_200 = 2024-01-01T00:00:00Z. Using 0
            // produced an unrealistic `block.timestamp == 0` whenever a
            // contract ran without an explicit `override_timestamp`, which
            // made timestamp-sensitive Solidity (deadlines, `require(now >=
            // x)`, TWAP checks, etc.) trivially passable/rejectable against
            // any positive constant. A pinned realistic epoch is the Neo
            // MainNet-genesis-equivalent fallback.
            default_timestamp: 1_704_067_200,
        }
    }
}

impl RuntimeConfig {
    /// Create a new builder
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::default()
    }

    /// Create config for testing with debugging enabled
    pub fn for_testing() -> Self {
        Self {
            enable_debugging: true,
            enable_tracing: true,
            strict_mode: false,
            ..Default::default()
        }
    }
}

/// Builder for RuntimeConfig
#[derive(Debug, Clone, Default)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl RuntimeConfigBuilder {
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.config.gas_limit = limit;
        self
    }

    pub fn debugging(mut self, enabled: bool) -> Self {
        self.config.enable_debugging = enabled;
        self
    }

    pub fn tracing(mut self, enabled: bool) -> Self {
        self.config.enable_tracing = enabled;
        self
    }

    pub fn build(self) -> RuntimeConfig {
        self.config
    }
}
