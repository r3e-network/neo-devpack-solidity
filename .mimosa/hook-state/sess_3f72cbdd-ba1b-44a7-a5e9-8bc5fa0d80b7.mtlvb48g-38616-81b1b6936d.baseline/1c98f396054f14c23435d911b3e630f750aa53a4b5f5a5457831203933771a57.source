use super::*;

impl VMBridge {
    pub(crate) fn extract_return_data(
        &self,
        context: &execution::ExecutionContext,
    ) -> Result<Vec<u8>, RuntimeError> {
        Ok(context.return_data().to_vec())
    }
}
