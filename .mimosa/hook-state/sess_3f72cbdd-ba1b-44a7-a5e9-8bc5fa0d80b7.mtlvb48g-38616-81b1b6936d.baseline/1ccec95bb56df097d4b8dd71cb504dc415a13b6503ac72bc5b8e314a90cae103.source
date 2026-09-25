use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_role_management(
        &mut self,
        method: &str,
        params: StackItem,
    ) -> StackItem {
        match method {
            "designateasrole" => {
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    let role = Self::stack_item_to_int(
                        borrowed.first().cloned().unwrap_or(StackItem::Null),
                    ) as u8;
                    let keys = if let Some(StackItem::Array(key_arr)) = borrowed.get(1) {
                        key_arr
                            .borrow()
                            .iter()
                            .map(|k| Self::stack_item_to_bytes(k.clone()))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    self.role_designations.insert(role, keys);
                }
                StackItem::Null
            }
            "getdesignatedbyrole" => {
                let role = Self::extract_first_int(&params) as u8;
                let keys = self
                    .role_designations
                    .get(&role)
                    .cloned()
                    .unwrap_or_default();
                let items: Vec<StackItem> = keys.into_iter().map(StackItem::byte_array).collect();
                StackItem::array(items)
            }
            _ => StackItem::Null,
        }
    }
}
