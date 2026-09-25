use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_policy(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            // ── Getters ──
            "getfeeperbyte" => StackItem::UnsignedInteger(self.policy_fee_per_byte),
            "getexecfeefactor" => StackItem::UnsignedInteger(self.policy_exec_fee_factor as u64),
            "getexecpicofeefactor" => {
                StackItem::UnsignedInteger(self.policy_exec_fee_factor as u64 * 1000)
            }
            "getstorageprice" => StackItem::UnsignedInteger(self.policy_storage_price),
            "getmillisecondsperblock" => {
                StackItem::UnsignedInteger(self.policy_milliseconds_per_block as u64)
            }
            "getmaxvaliduntilblockincrement" => {
                StackItem::UnsignedInteger(self.policy_max_valid_until_block_increment as u64)
            }
            "getmaxtraceableblocks" => {
                StackItem::UnsignedInteger(self.policy_max_traceable_blocks as u64)
            }
            "getattributefee" => {
                let attr_type = Self::extract_first_int(&params) as u8;
                let fee = self
                    .policy_attribute_fees
                    .get(&attr_type)
                    .copied()
                    .unwrap_or(0);
                StackItem::UnsignedInteger(fee)
            }

            // ── Setters ──
            "setfeeperbyte" => {
                self.policy_fee_per_byte = Self::extract_first_int(&params);
                StackItem::Null
            }
            "setexecfeefactor" => {
                self.policy_exec_fee_factor = Self::extract_first_int(&params) as u32;
                StackItem::Null
            }
            "setstorageprice" => {
                self.policy_storage_price = Self::extract_first_int(&params);
                StackItem::Null
            }
            "setmillisecondsperblock" => {
                self.policy_milliseconds_per_block = Self::extract_first_int(&params) as u32;
                StackItem::Null
            }
            "setmaxvaliduntilblockincrement" => {
                self.policy_max_valid_until_block_increment =
                    Self::extract_first_int(&params) as u32;
                StackItem::Null
            }
            "setmaxtraceableblocks" => {
                self.policy_max_traceable_blocks = Self::extract_first_int(&params) as u32;
                StackItem::Null
            }
            "setattributefee" => {
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let attr_type = Self::stack_item_to_int(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    ) as u8;
                    let fee = Self::stack_item_to_int(
                        borrowed.get(1).cloned().unwrap_or(StackItem::Null),
                    ) as u64;
                    self.policy_attribute_fees.insert(attr_type, fee);
                }
                StackItem::Null
            }

            // ── Account blocking ──
            "blockaccount" => {
                let account = Self::extract_first_bytes(&params);
                if account.is_empty() {
                    StackItem::Boolean(false)
                } else {
                    self.policy_blocked_accounts.insert(account);
                    StackItem::Boolean(true)
                }
            }
            "unblockaccount" => {
                let account = Self::extract_first_bytes(&params);
                StackItem::Boolean(self.policy_blocked_accounts.remove(&account))
            }
            "isblocked" => {
                let account = Self::extract_first_bytes(&params);
                StackItem::Boolean(self.policy_blocked_accounts.contains(&account))
            }
            "getblockedaccounts" => {
                let items: Vec<StackItem> = self
                    .policy_blocked_accounts
                    .iter()
                    .map(|a| StackItem::byte_array(a.clone()))
                    .collect();
                StackItem::array(items)
            }

            // ── Whitelist ──
            "setwhitelistfeecontract" => {
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let hash = Self::stack_item_to_bytes(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    );
                    let fee = Self::stack_item_to_int(
                        borrowed.get(1).cloned().unwrap_or(StackItem::Null),
                    ) as u64;
                    self.policy_whitelisted_fee_contracts
                        .push(WhitelistedFeeContract {
                            contract_hash: hash,
                            max_fee: fee,
                        });
                }
                StackItem::Null
            }
            "removewhitelistfeecontract" => {
                let hash = Self::extract_first_bytes(&params);
                self.policy_whitelisted_fee_contracts
                    .retain(|w| w.contract_hash != hash);
                StackItem::Null
            }
            "getwhitelistfeecontracts" => {
                let items: Vec<StackItem> = self
                    .policy_whitelisted_fee_contracts
                    .iter()
                    .map(|w| {
                        StackItem::array(vec![
                            StackItem::byte_array(w.contract_hash.clone()),
                            StackItem::UnsignedInteger(w.max_fee),
                        ])
                    })
                    .collect();
                StackItem::array(items)
            }

            // ── Recovery ──
            "recoverfund" => {
                // No-op in embedded runtime; returns true to indicate success
                StackItem::Boolean(true)
            }

            _ => StackItem::Null,
        }
    }
}
