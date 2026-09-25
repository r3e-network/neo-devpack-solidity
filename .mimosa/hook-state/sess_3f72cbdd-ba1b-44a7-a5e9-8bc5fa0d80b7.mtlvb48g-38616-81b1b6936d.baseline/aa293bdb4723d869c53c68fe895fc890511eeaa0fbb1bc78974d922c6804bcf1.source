use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_treasury(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "verify" => StackItem::Boolean(true),

            "onnep17payment" => {
                // Track NEP-17 token deposits
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let from = Self::stack_item_to_bytes(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    );
                    let amount = Self::stack_item_to_int(
                        borrowed.get(1).cloned().unwrap_or(StackItem::Null),
                    ) as u64;
                    let entry = self.treasury_nep17_balances.entry(from).or_insert(0);
                    *entry += amount;
                }
                StackItem::Null
            }

            "onnep11payment" => {
                // Track NEP-11 token deposits
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let from = Self::stack_item_to_bytes(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    );
                    let token_id = Self::stack_item_to_bytes(
                        borrowed.get(2).cloned().unwrap_or(StackItem::Null),
                    );
                    let tokens = self.treasury_nep11_tokens.entry(from).or_default();
                    tokens.push(token_id);
                }
                StackItem::Null
            }

            _ => StackItem::Null,
        }
    }
}
