use super::*;

impl ExecutionContext {
    /// Compute the Keccak-256 hash over a memory slice
    pub fn keccak_memory_slice(
        &mut self,
        offset: usize,
        length: usize,
    ) -> Result<[u8; 32], RuntimeError> {
        use sha3::Digest as KeccakDigest;

        let slice = self.read_memory(offset, length)?;
        let mut hasher = Keccak256::new();
        KeccakDigest::update(&mut hasher, slice);
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Ok(out)
    }
}
