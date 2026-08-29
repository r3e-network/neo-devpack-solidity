use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_oracle(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "request" => {
                // Extract parameters: url, filter, callback_contract, callback_method, user_data, gas_for_response
                let (url, filter, cb_contract, cb_method, user_data, gas_for_response) =
                    if let StackItem::Array(args) = &params {
                        let borrowed = args.borrow();
                        let url = Self::stack_item_to_bytes(
                            borrowed.first().cloned().unwrap_or(StackItem::Null),
                        );
                        let filter = Self::stack_item_to_bytes(
                            borrowed.get(1).cloned().unwrap_or(StackItem::Null),
                        );
                        let cb_contract = Self::stack_item_to_bytes(
                            borrowed.get(2).cloned().unwrap_or(StackItem::Null),
                        );
                        let cb_method = Self::stack_item_to_bytes(
                            borrowed.get(3).cloned().unwrap_or(StackItem::Null),
                        );
                        let user_data = Self::stack_item_to_bytes(
                            borrowed.get(4).cloned().unwrap_or(StackItem::Null),
                        );
                        let gas = Self::extract_first_int(
                            &borrowed
                                .get(5)
                                .cloned()
                                .unwrap_or(StackItem::UnsignedInteger(0)),
                        );
                        (url, filter, cb_contract, cb_method, user_data, gas)
                    } else {
                        (
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            0,
                        )
                    };

                let id = self.oracle_next_request_id;
                self.oracle_next_request_id += 1;

                let request = OracleRequest {
                    id,
                    original_tx_hash: vec![0u8; 32],
                    url: String::from_utf8_lossy(&url).to_string(),
                    filter: String::from_utf8_lossy(&filter).to_string(),
                    callback_contract: cb_contract,
                    callback_method: String::from_utf8_lossy(&cb_method).to_string(),
                    user_data,
                    gas_for_response,
                };
                self.oracle_requests.insert(id, request);

                // Return 4-byte request id (matching original behavior)
                let id_bytes = (id as u32).to_le_bytes();
                StackItem::byte_array(id_bytes.to_vec())
            }
            "getprice" => StackItem::UnsignedInteger(self.oracle_price),
            "setprice" => {
                self.oracle_price = Self::extract_first_int(&params);
                StackItem::Null
            }
            "finish" => {
                // No-op in embedded runtime
                StackItem::Null
            }
            "verify" => StackItem::Boolean(true),
            // NOTE: The on-chain Oracle contract exposes exactly five ABI
            // methods (request, getPrice, setPrice, finish, verify). Former
            // `getoraclenodes` / `getrequests` / `getrequest` handlers modelled
            // methods that DO NOT exist on any Neo N3 node — they made tests
            // pass for Solidity wrappers that would FAULT on-chain with
            // "Method not found", so they were removed together with the
            // devpack wrappers. The in-memory `oracle_requests` registry is
            // still maintained by `request` for callback bookkeeping.
            _ => StackItem::Null,
        }
    }
}
