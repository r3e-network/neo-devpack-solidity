use once_cell::sync::Lazy;
use regex::Regex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeCategory {
    AutoCompatible,
    ManualMigration,
    ManifestReview,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeFinding {
    pub phase: String,
    pub severity: UpgradeSeverity,
    pub category: UpgradeCategory,
    pub code: Option<String>,
    pub contract: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
}

impl UpgradeFinding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        phase: impl Into<String>,
        severity: UpgradeSeverity,
        category: UpgradeCategory,
        code: Option<impl Into<String>>,
        contract: Option<impl Into<String>>,
        message: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self {
            phase: phase.into(),
            severity,
            category,
            code: code.map(Into::into),
            contract: contract.map(Into::into),
            message: message.into(),
            suggestion: suggestion.map(Into::into),
        }
    }
}

pub fn analyze_upgrade_patterns(source: &str) -> Vec<UpgradeFinding> {
    static BLOCKHASH_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\bblockhash\s*\(").expect("valid regex pattern"));
    static SELFDESTRUCT_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\bselfdestruct\s*\(").expect("valid regex pattern"));
    static CODEHASH_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\.codehash\b").expect("valid regex pattern"));
    static TX_ORIGIN_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\btx\.origin\b").expect("valid regex pattern"));
    static MSG_SIG_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\bmsg\.sig\b").expect("valid regex pattern"));
    static DELEGATECALL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?:\.|\b)delegatecall\s*\(").expect("valid regex pattern"));
    static STATICCALL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?:\.|\b)staticcall\s*\(").expect("valid regex pattern"));
    static LOW_LEVEL_CALL_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\.\s*call\s*(?:\(|\{)").expect("valid regex pattern"));
    // M-FE4 fix — require the unit to follow a digit or `)` (the EVM
    // numeric-literal-suffix shape). The previous `\b(?:...)\b` matched the
    // word `ether` inside a comment (`// whether we proceed`), a variable
    // name (`uint ether = msg.value;`), or a function name — all false
    // positives that escalated to a hard error. Requiring a digit or `)`
    // before the unit excludes identifier/comment occurrences while still
    // catching the real `1 ether` / `(a + b) ether` forms.
    static ETHER_UNIT_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?:\d|\))\s*(?:wei|gwei|szabo|finney|ether)\b").expect("valid regex pattern")
    });
    static SUPPORTS_INTERFACE_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\bsupportsInterface\s*\(").expect("valid regex pattern"));

    let mut findings = Vec::new();

    if BLOCKHASH_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::AutoCompatible,
            Some("SCAN_BLOCKHASH_AUTO"),
            None::<String>,
            "blockhash() is compiler-compatible on Neo N3 and is auto-mapped to Ledger.getBlockHash().",
            Some("Prefer explicit Ledger.getBlockHash(height) when authoring Neo-native Solidity."),
        ));
    }

    if SELFDESTRUCT_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::AutoCompatible,
            Some("SCAN_SELFDESTRUCT_AUTO"),
            None::<String>,
            "selfdestruct() is auto-mapped to ContractManagement.destroy() during Neo compilation.",
            Some(
                "Review destruction and upgrade semantics before depending on EVM selfdestruct behavior.",
            ),
        ));
    }

    if CODEHASH_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::AutoCompatible,
            Some("SCAN_CODEHASH_AUTO"),
            None::<String>,
            "address.codehash is compiler-compatible on Neo N3 and resolves to the script hash identity.",
            Some("Prefer explicit script-hash based checks in Neo-native code."),
        ));
    }

    if TX_ORIGIN_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::ManualMigration,
            Some("SCAN_TX_ORIGIN"),
            None::<String>,
            "tx.origin keeps compiling on Neo N3, but its authorization semantics differ from EVM.",
            Some("Use Runtime.checkWitness(...) or msg.sender-oriented authorization instead."),
        ));
    }

    if MSG_SIG_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::ManualMigration,
            Some("SCAN_MSG_SIG"),
            None::<String>,
            "msg.sig compiles on Neo N3 as the current function selector, but this differs from EVM semantics across internal calls.",
            Some("Use explicit method names or interface IDs when you need dispatch identity that survives internal calls."),
        ));
    }

    if DELEGATECALL_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Error,
            UpgradeCategory::ManualMigration,
            Some("SCAN_DELEGATECALL"),
            None::<String>,
            "delegatecall-style upgrade and proxy flows do not exist on Neo N3.",
            Some(
                "Replace delegatecall proxies with ContractManagement.update() or explicit cross-contract calls.",
            ),
        ));
    }

    if STATICCALL_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::ManualMigration,
            Some("SCAN_STATICCALL"),
            None::<String>,
            "staticcall has no direct EVM-equivalent execution mode on Neo N3.",
            Some("Prefer typed view/pure calls or Syscalls.contractCallWithFlags(..., ReadOnly)."),
        ));
    }

    if LOW_LEVEL_CALL_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::ManifestReview,
            Some("SCAN_LOW_LEVEL_CALL"),
            None::<String>,
            "Low-level address.call(...) patterns can force broader Neo manifest permissions.",
            Some(
                "Prefer typed interfaces or fixed-target wrappers so manifest permissions stay narrow.",
            ),
        ));
    }

    if ETHER_UNIT_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Error,
            UpgradeCategory::ManualMigration,
            Some("SCAN_ETHER_UNITS"),
            None::<String>,
            "Ether-denominated value units are EVM-specific and do not map directly to Neo N3 execution.",
            Some("Use integer GAS fractions (10^8) or NEP-17 payment flows instead."),
        ));
    }

    if SUPPORTS_INTERFACE_RE.is_match(source) {
        findings.push(UpgradeFinding::new(
            "source_scan",
            UpgradeSeverity::Warning,
            UpgradeCategory::ManualMigration,
            Some("SCAN_SUPPORTS_INTERFACE"),
            None::<String>,
            "supportsInterface(bytes4) is usually unnecessary on Neo N3 because interfaces are advertised in the manifest.",
            Some(
                "Prefer manifest.supportedstandards instead of EIP-165 style runtime checks.",
            ),
        ));
    }

    findings
}
