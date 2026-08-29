use super::*;

impl VMBridge {
    pub(crate) fn apply_storage_overlay(
        &self,
        context: &mut execution::ExecutionContext,
        storage: &mut storage::StorageManager,
    ) -> Result<Vec<String>, RuntimeError> {
        if let Some((account, entries)) = context.drain_dirty_storage_overlay() {
            for (key, value) in entries {
                match value {
                    Some(bytes) => storage.set(&account, &key, &bytes)?,
                    None => storage.delete(&account, &key)?,
                }
            }
            Ok(vec![account])
        } else {
            Ok(Vec::new())
        }
    }
}
