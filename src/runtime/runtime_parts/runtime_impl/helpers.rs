use super::*;

impl NeoRuntime {
    pub(crate) fn prepare_function_call(
        &self,
        function_name: &str,
        args: &[types::StackItem],
    ) -> Result<Vec<u8>, RuntimeError> {
        // Complete function call preparation with selector and argument encoding
        let mut call_data = Vec::new();

        // Add function selector (first 4 bytes of keccak256 hash of function signature)
        let selector = self.calculate_function_selector(function_name)?;
        call_data.extend_from_slice(&selector);

        // Encode arguments
        for arg in args {
            call_data.extend_from_slice(&arg.to_bytes());
        }

        Ok(call_data)
    }

    pub(crate) fn calculate_function_selector(
        &self,
        function_name: &str,
    ) -> Result<[u8; 4], RuntimeError> {
        // Calculate keccak256 hash and take first 4 bytes
        use sha3::{Digest, Keccak256};

        let hash = Keccak256::digest(function_name.as_bytes());
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash[..4]);
        Ok(selector)
    }
}
