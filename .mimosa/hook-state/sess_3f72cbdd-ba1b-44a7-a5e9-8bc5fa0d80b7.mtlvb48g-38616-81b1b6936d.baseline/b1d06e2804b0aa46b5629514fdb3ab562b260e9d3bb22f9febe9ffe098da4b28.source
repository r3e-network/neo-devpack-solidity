use super::*;

/// Address type for account identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Address(pub String);

/// Transaction hash type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionHash(pub String);

/// Block hash type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BlockHash(pub String);

/// Gas amount type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gas(pub u64);

/// Balance amount type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance(pub u64);

/// Block number type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockNumber(pub u64);

/// Timestamp type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(pub u64);

fn strip_hex_prefix(value: &str) -> &str {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value)
}

fn normalize_hex(value: &str) -> Result<String, &'static str> {
    let payload = strip_hex_prefix(value);
    if payload.is_empty() || !payload.len().is_multiple_of(2) {
        return Err("Invalid hex format");
    }
    if hex::decode(payload).is_err() {
        return Err("Invalid hex format");
    }
    Ok(format!("0x{payload}"))
}

impl Address {
    /// Create new address
    pub fn new(address: String) -> Result<Self, &'static str> {
        let payload = strip_hex_prefix(&address);
        if payload.len() != 40 {
            return Err("Invalid address format");
        }
        if hex::decode(payload).is_err() {
            return Err("Invalid address format");
        }
        Ok(Address(format!("0x{payload}")))
    }

    /// Get address as string
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get address bytes (20 bytes)
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(strip_hex_prefix(&self.0)).unwrap_or_default()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Address(format!("0x{}", hex::encode(bytes)))
    }

    /// Check if address is zero
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == "0x0000000000000000000000000000000000000000"
    }
}

impl TransactionHash {
    /// Create new transaction hash
    pub fn new(hash: String) -> Result<Self, &'static str> {
        Ok(TransactionHash(normalize_hex(&hash)?))
    }

    /// Get hash as string
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get hash bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(strip_hex_prefix(&self.0)).unwrap_or_default()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        TransactionHash(format!("0x{}", hex::encode(bytes)))
    }
}

impl BlockHash {
    /// Create new block hash
    pub fn new(hash: String) -> Result<Self, &'static str> {
        Ok(BlockHash(normalize_hex(&hash)?))
    }

    /// Get hash as string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get hash bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(strip_hex_prefix(&self.0)).unwrap_or_default()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Self {
        BlockHash(format!("0x{}", hex::encode(bytes)))
    }
}

impl Gas {
    /// Create new gas amount
    pub fn new(amount: u64) -> Self {
        Gas(amount)
    }

    /// Get gas amount
    pub fn amount(&self) -> u64 {
        self.0
    }

    /// Add gas amounts
    pub fn saturating_add(self, other: Gas) -> Gas {
        Gas(self.0.saturating_add(other.0))
    }

    /// Subtract gas amounts
    pub fn saturating_sub(self, other: Gas) -> Gas {
        Gas(self.0.saturating_sub(other.0))
    }

    /// Check if sufficient gas
    #[must_use]
    pub fn sufficient_for(&self, required: Gas) -> bool {
        self.0 >= required.0
    }
}

impl Balance {
    /// Create new balance
    pub fn new(amount: u64) -> Self {
        Balance(amount)
    }

    /// Get balance amount
    pub fn amount(&self) -> u64 {
        self.0
    }

    /// Add balances
    pub fn saturating_add(self, other: Balance) -> Balance {
        Balance(self.0.saturating_add(other.0))
    }

    /// Subtract balances
    pub fn checked_sub(self, other: Balance) -> Result<Balance, &'static str> {
        if self.0 >= other.0 {
            Ok(Balance(self.0 - other.0))
        } else {
            Err("Insufficient balance")
        }
    }

    /// Check if sufficient balance
    #[must_use]
    pub fn sufficient_for(&self, required: Balance) -> bool {
        self.0 >= required.0
    }

    /// Is zero balance
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl BlockNumber {
    /// Create new block number
    pub fn new(number: u64) -> Self {
        BlockNumber(number)
    }

    /// Get block number
    pub fn number(&self) -> u64 {
        self.0
    }

    /// Get next block number
    pub fn next(self) -> BlockNumber {
        BlockNumber(self.0 + 1)
    }

    /// Get previous block number
    pub fn prev(self) -> Option<BlockNumber> {
        if self.0 > 0 {
            Some(BlockNumber(self.0 - 1))
        } else {
            None
        }
    }
}

impl Timestamp {
    /// Create new timestamp
    pub fn new(timestamp: u64) -> Self {
        Timestamp(timestamp)
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        self.0
    }

    /// Get current timestamp
    pub fn now() -> Self {
        Timestamp(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        )
    }

    /// Add duration in seconds
    pub fn add_seconds(self, seconds: u64) -> Timestamp {
        Timestamp(self.0 + seconds)
    }

    /// Subtract duration in seconds
    pub fn sub_seconds(self, seconds: u64) -> Timestamp {
        Timestamp(self.0.saturating_sub(seconds))
    }
}
