use super::*;

impl ExecutionContext {
    pub(crate) fn invoke_native_neo(&mut self, method: &str, params: StackItem) -> StackItem {
        match method {
            "symbol" => StackItem::byte_array(b"NEO".to_vec()),
            "decimals" => StackItem::UnsignedInteger(0),
            "totalsupply" => StackItem::UnsignedInteger(self.neo_total_supply),
            "getgasperblock" => StackItem::UnsignedInteger(5),
            "getcommittee" => {
                // Return deterministic committee ECPoint public keys (ByteString).
                let member1 = (0u8..33u8).collect::<Vec<_>>();
                let member2 = (33u8..66u8).collect::<Vec<_>>();
                StackItem::array(vec![
                    StackItem::byte_array(member1),
                    StackItem::byte_array(member2),
                ])
            }
            "getnextblockvalidators" => {
                // Return deterministic validator ECPoint public keys (ByteString).
                let validator = (100u8..133u8).collect::<Vec<_>>();
                StackItem::array(vec![StackItem::byte_array(validator)])
            }
            "getcandidates" => {
                // Neo N3: getCandidates() -> Array<Struct(ECPoint publicKey, BigInteger votes)>
                //
                // Represent structs as arrays in the runtime, matching the compiler's
                // array-backed struct lowering.
                let cand1_key = (10u8..43u8).collect::<Vec<_>>();
                let cand2_key = (43u8..76u8).collect::<Vec<_>>();
                StackItem::array(vec![
                    StackItem::array(vec![
                        StackItem::byte_array(cand1_key),
                        StackItem::UnsignedInteger(1_000),
                    ]),
                    StackItem::array(vec![
                        StackItem::byte_array(cand2_key),
                        StackItem::UnsignedInteger(2_000),
                    ]),
                ])
            }
            "getaccountstate" => {
                // Neo N3: getAccountState(account) -> NeoAccountState? (nullable)
                // Shape: [balance, balanceHeight, voteTo (ECPoint bytes or null), lastGasPerVote]
                if let StackItem::Array(args) = params {
                    if let Some(StackItem::ByteArray { data: acc, .. }) = args.borrow().first() {
                        if let Some(balance) =
                            self.neo_balances.get(acc.borrow().as_slice()).copied()
                        {
                            let height = self.block_height.unwrap_or(self.default_block_height);
                            return StackItem::array(vec![
                                StackItem::UnsignedInteger(balance),
                                StackItem::UnsignedInteger(height),
                                // Return `null` for voteTo by default so the compiler's
                                // helper can normalize it to empty bytes.
                                StackItem::Null,
                                StackItem::UnsignedInteger(0),
                            ]);
                        }
                    }
                }
                StackItem::Null
            }
            "vote" | "registercandidate" | "unregistercandidate" | "setgasperblock" => {
                // Keep these as deterministic placeholders for now.
                StackItem::Boolean(true)
            }
            // Neo N3: getAllCandidates() -> Array<ECPoint>
            // Returns only the public keys of all registered candidates
            // (without vote counts, unlike getCandidates).
            "getallcandidates" => {
                let cand1_key = (10u8..43u8).collect::<Vec<_>>();
                let cand2_key = (43u8..76u8).collect::<Vec<_>>();
                StackItem::array(vec![
                    StackItem::byte_array(cand1_key),
                    StackItem::byte_array(cand2_key),
                ])
            }
            // Neo N3: getCandidateVote(pubkey) -> BigInteger
            // Returns the vote count for a specific candidate.
            "getcandidatevote" => {
                if let StackItem::Array(args) = &params {
                    if let Some(StackItem::ByteArray { data: pubkey, .. }) = args.borrow().first() {
                        let pk = pubkey.borrow();
                        // Return deterministic vote count based on the
                        // candidate public key — matches getCandidates
                        // output for the same synthetic keys.
                        let cand1: Vec<u8> = (10u8..43u8).collect();
                        let cand2: Vec<u8> = (43u8..76u8).collect();
                        if pk.as_slice() == cand1 {
                            return StackItem::UnsignedInteger(1_000);
                        }
                        if pk.as_slice() == cand2 {
                            return StackItem::UnsignedInteger(2_000);
                        }
                    }
                }
                StackItem::UnsignedInteger(0)
            }
            // Neo N3: getRegisterPrice() -> BigInteger
            // Default registration price is 1000 GAS (1000 * 10^8 fractions).
            "getregisterprice" => StackItem::Integer(1_000 * 100_000_000),
            // Neo N3: setRegisterPrice(price) — governance operation.
            // No-op in the embedded runtime; return success.
            "setregisterprice" => StackItem::Boolean(true),
            // Neo N3: unclaimedGas(account, endHeight) -> BigInteger
            // Computes GAS that has accrued but not yet been claimed.
            // Formula: balance * (endHeight - balanceHeight) * gasPerBlock / totalSupply
            "unclaimedgas" => {
                if let StackItem::Array(args) = &params {
                    let borrowed = args.borrow();
                    if let (Some(StackItem::ByteArray { data: acc, .. }), Some(end_height_item)) =
                        (borrowed.first(), borrowed.get(1))
                    {
                        let acc_bytes = acc.borrow();
                        let end_height = Self::extract_first_int(end_height_item);
                        let balance = *self.neo_balances.get(acc_bytes.as_slice()).unwrap_or(&0);
                        let balance_height = self.block_height.unwrap_or(self.default_block_height);
                        let gas_per_block = 5u64;
                        let total_supply = self.neo_total_supply.max(1);
                        if balance > 0 && end_height > balance_height {
                            let gas = balance
                                .saturating_mul(end_height - balance_height)
                                .saturating_mul(gas_per_block)
                                / total_supply;
                            return StackItem::UnsignedInteger(gas);
                        }
                    }
                }
                StackItem::UnsignedInteger(0)
            }
            "balanceof" => {
                if let StackItem::Array(args) = params {
                    if let Some(StackItem::ByteArray { data: acc, .. }) = args.borrow().first() {
                        let bal = *self.neo_balances.get(acc.borrow().as_slice()).unwrap_or(&0);
                        StackItem::UnsignedInteger(bal)
                    } else {
                        StackItem::UnsignedInteger(0)
                    }
                } else {
                    StackItem::UnsignedInteger(0)
                }
            }
            "transfer" => self.handle_native_transfer(true, params),
            _ => StackItem::Null,
        }
    }
}
