use super::*;

impl ExecutionContext {
    pub(crate) fn handle_native_transfer(&mut self, is_neo: bool, params: StackItem) -> StackItem {
        if let StackItem::Array(args) = params {
            let args = args.borrow();
            if args.len() < 3 {
                return StackItem::Boolean(false);
            }
            let from = match args.first() {
                Some(StackItem::ByteArray { data: acc, .. }) => acc.borrow().clone(),
                _ => Vec::new(),
            };
            let to = match args.get(1) {
                Some(StackItem::ByteArray { data: acc, .. }) => acc.borrow().clone(),
                _ => Vec::new(),
            };
            let amount = match args.get(2) {
                Some(StackItem::UnsignedInteger(u)) => *u,
                Some(StackItem::Integer(i)) if *i >= 0 => *i as u64,
                _ => 0,
            };
            if amount == 0 {
                return StackItem::Boolean(false);
            }

            // Read balances
            let mut from_bal = if is_neo {
                *self.neo_balances.get(&from).unwrap_or(&0)
            } else {
                *self.gas_balances.get(&from).unwrap_or(&0)
            };
            let mut to_bal = if is_neo {
                *self.neo_balances.get(&to).unwrap_or(&0)
            } else {
                *self.gas_balances.get(&to).unwrap_or(&0)
            };

            if !from.is_empty() && from_bal < amount {
                return StackItem::Boolean(false);
            }
            if !from.is_empty() {
                from_bal = from_bal.saturating_sub(amount);
            }
            to_bal = to_bal.saturating_add(amount);

            if is_neo {
                if !from.is_empty() {
                    self.neo_balances.insert(from.clone(), from_bal);
                }
                self.neo_balances.insert(to.clone(), to_bal);
            } else {
                if !from.is_empty() {
                    self.gas_balances.insert(from.clone(), from_bal);
                }
                self.gas_balances.insert(to.clone(), to_bal);
            }

            StackItem::Boolean(true)
        } else {
            StackItem::Boolean(false)
        }
    }
}
