use super::*;

impl ExecutionContext {
    pub(crate) fn handle_crypto_syscall(&mut self, name: &str) -> Result<bool, RuntimeError> {
        match name {
            "System.Crypto.CheckSig" => {
                let sig_item = self.pop_stack()?;
                let pub_item = self.pop_stack()?;
                let pubkey = Self::stack_item_to_bytes(pub_item);
                let sig = Self::stack_item_to_bytes(sig_item);
                // No host-injected signing hash means CheckSig cannot
                // meaningfully verify (there is no script container to derive
                // a real transaction digest). Default to reject (`false`) — the
                // honest answer without a verifiable hash.
                let ok = match self.get_current_message_hash() {
                    Some(h) => Self::verify_secp256r1_with_message(&h, &pubkey, &sig),
                    None => false,
                };
                self.push_stack(StackItem::Boolean(ok))?;
                Ok(true)
            }
            "System.Crypto.CheckMultisig" => {
                let sigs_item = self.pop_stack()?;
                let pubs_item = self.pop_stack()?;
                // No host-injected signing hash means CheckMultisig cannot
                // meaningfully verify (there is no script container to derive
                // a real transaction digest). Default to reject (`false`) — the
                // honest answer without a verifiable hash. Args are already
                // popped, so the stack stays balanced.
                let msg_hash = match self.get_current_message_hash() {
                    Some(h) => h,
                    None => {
                        self.push_stack(StackItem::Boolean(false))?;
                        return Ok(true);
                    }
                };

                // Extract individual items from arrays
                let pub_items = match pubs_item {
                    StackItem::Array(items) => {
                        let items = items.borrow();
                        items.clone()
                    }
                    _ => {
                        // Fallback: treat as single pubkey
                        vec![pubs_item]
                    }
                };
                let sig_items = match sigs_item {
                    StackItem::Array(items) => {
                        let items = items.borrow();
                        items.clone()
                    }
                    _ => {
                        vec![sigs_item]
                    }
                };

                // M-of-N verification: each sig must match a pubkey in order
                let mut pub_idx = 0;
                let mut all_valid = !sig_items.is_empty() && !pub_items.is_empty();

                for sig_item in &sig_items {
                    let sig = Self::stack_item_to_bytes(sig_item.clone());
                    let mut found = false;
                    while pub_idx < pub_items.len() {
                        let pubkey = Self::stack_item_to_bytes(pub_items[pub_idx].clone());
                        pub_idx += 1;
                        if Self::verify_secp256r1_with_message(&msg_hash, &pubkey, &sig) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        all_valid = false;
                        break;
                    }
                }

                self.push_stack(StackItem::Boolean(all_valid))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
