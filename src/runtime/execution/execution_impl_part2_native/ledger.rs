use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_ledger(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "currentindex" => {
                let height = *self.block_height.get_or_insert(self.default_block_height);
                StackItem::UnsignedInteger(height)
            }
            "currenthash" => StackItem::byte_array(self.ledger_current_hash.to_vec()),
            "getblock" => {
                let index = Self::extract_first_int(&params);
                let height = *self.block_height.get_or_insert(self.default_block_height);
                if index > height {
                    return StackItem::Null;
                }
                // Check cached blocks first
                if let Some(block) = self.ledger_blocks.get(&index) {
                    return Self::ledger_block_to_stack_item(block);
                }
                // Generate synthetic block
                let block = Self::generate_synthetic_block(index);
                let result = Self::ledger_block_to_stack_item(&block);
                self.ledger_blocks.insert(index, block);
                result
            }
            "gettransaction" => {
                let hash_bytes = Self::extract_first_bytes(&params);
                if hash_bytes.len() == 32 {
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&hash_bytes);
                    if let Some(tx) = self.ledger_transactions.get(&hash) {
                        return Self::ledger_tx_to_stack_item(tx);
                    }
                }
                StackItem::Null
            }
            "gettransactionheight" => {
                let hash_bytes = Self::extract_first_bytes(&params);
                if hash_bytes.len() == 32 {
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&hash_bytes);
                    if self.ledger_transactions.contains_key(&hash) {
                        let height = *self.block_height.get_or_insert(self.default_block_height);
                        return StackItem::UnsignedInteger(height);
                    }
                }
                StackItem::Integer(-1)
            }
            "gettransactionfromblock" => {
                // params: [block_index_or_hash, tx_index]
                // Cockatrice: returns a transaction from a block by index.
                // Generate a deterministic synthetic tx for any valid block.
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let block_index = borrowed.first().map(Self::extract_first_int).unwrap_or(0);
                    let tx_index = borrowed.get(1).map(Self::extract_first_int).unwrap_or(0);

                    let height = *self.block_height.get_or_insert(self.default_block_height);
                    if block_index > height {
                        return StackItem::Null;
                    }

                    // Build a deterministic synthetic tx hash from (block_index, tx_index)
                    let mut seed = Vec::new();
                    seed.extend_from_slice(&block_index.to_le_bytes());
                    seed.extend_from_slice(&tx_index.to_le_bytes());
                    let hash_bytes = Sha256::digest(&seed);
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&hash_bytes);

                    let tx = LedgerTransaction {
                        hash,
                        version: 0,
                        nonce: tx_index as u32,
                        sender: vec![0u8; 20],
                        system_fee: 0,
                        network_fee: 0,
                        valid_until_block: (block_index + 2102400) as u32,
                        script: vec![],
                    };
                    self.ledger_transactions.insert(hash, tx.clone());
                    return Self::ledger_tx_to_stack_item(&tx);
                }
                StackItem::Null
            }
            "gettransactionsigners" => {
                // Return empty array — embedded runtime doesn't model signers
                StackItem::array(Vec::new())
            }
            "gettransactionvmstate" => {
                // Return HALT (1) as default VM state
                StackItem::UnsignedInteger(1)
            }
            _ => StackItem::Null,
        }
    }

    /// Generate a deterministic synthetic block from an index using SHA-256.
    fn generate_synthetic_block(index: u64) -> LedgerBlock {
        let seed = index.to_le_bytes();
        let hash_bytes = Sha256::digest(seed);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_bytes);

        let prev_seed = index.wrapping_sub(1).to_le_bytes();
        let prev_bytes = Sha256::digest(prev_seed);
        let mut prev_hash = [0u8; 32];
        prev_hash.copy_from_slice(&prev_bytes);

        let merkle_seed = [&seed[..], b"merkle"].concat();
        let merkle_bytes = Sha256::digest(&merkle_seed);
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&merkle_bytes);

        LedgerBlock {
            hash,
            version: 0,
            prev_hash,
            merkle_root,
            timestamp: 1_600_000_000_000 + index * 15_000,
            nonce: index,
            index,
            primary_index: 0,
            next_consensus: vec![0u8; 20],
            transaction_count: 0,
        }
    }

    /// Convert a LedgerBlock to a StackItem array matching Neo N3 interop.
    fn ledger_block_to_stack_item(block: &LedgerBlock) -> StackItem {
        StackItem::array(vec![
            StackItem::byte_array(block.hash.to_vec()),
            StackItem::UnsignedInteger(block.version as u64),
            StackItem::byte_array(block.prev_hash.to_vec()),
            StackItem::byte_array(block.merkle_root.to_vec()),
            StackItem::UnsignedInteger(block.timestamp),
            StackItem::UnsignedInteger(block.nonce),
            StackItem::UnsignedInteger(block.index),
            StackItem::UnsignedInteger(block.primary_index as u64),
            StackItem::byte_array(block.next_consensus.clone()),
            StackItem::UnsignedInteger(block.transaction_count as u64),
        ])
    }

    /// Convert a LedgerTransaction to a StackItem array.
    fn ledger_tx_to_stack_item(tx: &LedgerTransaction) -> StackItem {
        StackItem::array(vec![
            StackItem::byte_array(tx.hash.to_vec()),
            StackItem::UnsignedInteger(tx.version as u64),
            StackItem::UnsignedInteger(tx.nonce as u64),
            StackItem::byte_array(tx.sender.clone()),
            StackItem::UnsignedInteger(tx.system_fee),
            StackItem::UnsignedInteger(tx.network_fee),
            StackItem::UnsignedInteger(tx.valid_until_block as u64),
            StackItem::byte_array(tx.script.clone()),
        ])
    }
}
