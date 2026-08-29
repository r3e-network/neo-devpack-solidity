use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_notary(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "verify" => StackItem::Boolean(true),

            "balanceof" => {
                let account = Self::extract_first_bytes(&params);
                let balance = self
                    .notary_deposits
                    .get(&account)
                    .map(|d| d.amount)
                    .unwrap_or(0);
                StackItem::UnsignedInteger(balance)
            }

            "expirationof" => {
                let account = Self::extract_first_bytes(&params);
                let till = self
                    .notary_deposits
                    .get(&account)
                    .map(|d| d.till as u64)
                    .unwrap_or(0);
                StackItem::UnsignedInteger(till)
            }

            "lockdeposituntil" => {
                let account = Self::extract_first_bytes(&params);
                let till = if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    Self::stack_item_to_int(borrowed.get(1).cloned().unwrap_or(StackItem::Null))
                        as u32
                } else {
                    0
                };
                if let Some(deposit) = self.notary_deposits.get_mut(&account) {
                    if till > deposit.till {
                        deposit.till = till;
                        return StackItem::Boolean(true);
                    }
                }
                StackItem::Boolean(false)
            }

            "withdraw" => {
                let account = Self::extract_first_bytes(&params);
                if self.notary_deposits.remove(&account).is_some() {
                    StackItem::Boolean(true)
                } else {
                    StackItem::Boolean(false)
                }
            }

            "getmaxnotvalidbeforedelta" => {
                StackItem::UnsignedInteger(self.notary_max_not_valid_before_delta as u64)
            }

            "setmaxnotvalidbeforedelta" => {
                self.notary_max_not_valid_before_delta = Self::extract_first_int(&params) as u32;
                StackItem::Null
            }

            "onnep17payment" => {
                // Track deposit from NEP-17 payment
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let from = Self::stack_item_to_bytes(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    );
                    let amount = Self::stack_item_to_int(
                        borrowed.get(1).cloned().unwrap_or(StackItem::Null),
                    ) as u64;
                    let till = Self::stack_item_to_int(
                        borrowed.get(2).cloned().unwrap_or(StackItem::Null),
                    ) as u32;

                    let deposit =
                        self.notary_deposits
                            .entry(from.clone())
                            .or_insert_with(|| NotaryDeposit {
                                account: from,
                                amount: 0,
                                till: 0,
                            });
                    deposit.amount += amount;
                    if till > deposit.till {
                        deposit.till = till;
                    }
                }
                StackItem::Null
            }

            _ => StackItem::Null,
        }
    }
}
