use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_gas(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "symbol" => StackItem::byte_array(b"GAS".to_vec()),
            "decimals" => StackItem::UnsignedInteger(8),
            "totalsupply" => StackItem::UnsignedInteger(self.gas_total_supply),
            "balanceof" => {
                if let StackItem::Array(args) = params {
                    if let Some(StackItem::ByteArray { data: acc, .. }) = args.borrow().first() {
                        let bal = *self.gas_balances.get(acc.borrow().as_slice()).unwrap_or(&0);
                        StackItem::UnsignedInteger(bal)
                    } else {
                        StackItem::UnsignedInteger(0)
                    }
                } else {
                    StackItem::UnsignedInteger(0)
                }
            }
            "transfer" => self.handle_native_transfer(false, params),
            _ => StackItem::Null,
        }
    }
}
