use super::*;

impl ExecutionContext {
    pub(crate) fn handle_runtime_syscall(&mut self, name: &str) -> Result<bool, RuntimeError> {
        match name {
            "System.Runtime.GetNetwork" => {
                self.push_stack(StackItem::UnsignedInteger(self.network_magic as u64))?;
                Ok(true)
            }
            "System.Runtime.GetTrigger" => {
                // Application trigger = 0x40 in Neo N3's TriggerType enum
                // (OnPersist 0x01, PostPersist 0x02, Verification 0x20,
                // Application 0x40). The embedded runtime models the common
                // deployed-contract case, which executes under Application.
                self.push_stack(StackItem::Integer(0x40))?;
                Ok(true)
            }
            "System.Runtime.Platform" => {
                self.push_stack(StackItem::byte_array(b"NEO".to_vec()))?;
                Ok(true)
            }
            "System.Runtime.GetAddressVersion" => {
                // Neo N3 default address version (ProtocolSettings.AddressVersion).
                // MainNet/TestNet use 53 (0x35).
                self.push_stack(StackItem::UnsignedInteger(53))?;
                Ok(true)
            }
            "System.Runtime.GasLeft" => {
                let remaining = self.gas_limit.saturating_sub(self.gas_used);
                self.push_stack(StackItem::UnsignedInteger(remaining))?;
                Ok(true)
            }
            "System.Runtime.GetInvocationCounter" => {
                self.push_stack(StackItem::UnsignedInteger(self.invocation_counter))?;
                Ok(true)
            }
            "System.Runtime.GetCallingScriptHash" => {
                // Task #123 — honour per-frame `msg.sender` overrides pushed
                // by `handle_contract_call` when entering a self-offsets
                // "virtual contract boundary". Walking the call stack here
                // (rather than mutating `caller_account`) keeps `tx.origin`
                // pinned to `Transaction.Sender` across nested frames while
                // still giving `msg.sender` the direct caller's identity.
                if let Some(override_bytes) = self.active_msg_sender_override() {
                    self.push_stack(StackItem::byte_array(override_bytes))?;
                    return Ok(true);
                }
                // The entry frame's caller is the transaction's entry script,
                // NOT the contract itself. Defaulting `caller_account` to the
                // distinct `entry_script_hash` (rather than the contract's own
                // `default_account_bytes`) is what lets the `msg.sender`
                // lowering's `CallingScriptHash == EntryScriptHash` test fire
                // for a genuine top-level call (so `msg.sender == tx.origin`),
                // while a self-offsets inner hop — whose override is the
                // bundle's executing hash — does NOT match `EntryScriptHash`
                // and therefore resolves `msg.sender` to the direct caller.
                if self.caller_account.is_none() {
                    self.caller_account = Some(self.entry_script_hash());
                    self.storage_account = Some(self.default_account.clone());
                }
                let bytes = self
                    .caller_account
                    .clone()
                    .unwrap_or_else(|| self.entry_script_hash());
                self.push_stack(StackItem::byte_array(bytes))?;
                Ok(true)
            }
            "System.Runtime.GetEntryScriptHash" => {
                // The transaction's entry-script hash — DISTINCT from the
                // executing contract hash (see `entry_script_hash`). Keeping
                // these separate matches Neo N3, where the entry script is the
                // tx invocation script and never a contract's own hash.
                self.push_stack(StackItem::byte_array(self.entry_script_hash()))?;
                Ok(true)
            }
            "System.Runtime.GetExecutingScriptHash" => {
                // `address(this)` — the currently executing contract's hash.
                self.push_stack(StackItem::byte_array(self.default_account_bytes.clone()))?;
                Ok(true)
            }
            "System.Runtime.LoadScript" => {
                // The embedded runtime does not support dynamic script loading.
                // Consume arguments to maintain stack discipline, then return error.
                let _args = self.pop_stack()?;
                let _call_flags = self.pop_stack()?;
                let _script = self.pop_stack()?;
                Err(RuntimeError::ExecutionError {
                    message: "System.Runtime.LoadScript is not supported in the embedded \
                             runtime. Use System.Contract.Call for inter-contract calls."
                        .to_string(),
                })
            }
            "System.Runtime.GetScriptContainer" => {
                // Return a Transaction-like array matching the Neo devpack field order:
                // [Hash, Version, Nonce, Sender, SystemFee, NetworkFee, ValidUntilBlock, Script]
                // Only `Sender` is currently used by the Solidity compiler (for
                // msg.sender / tx.origin). It tracks `caller_account` (honouring
                // any host `override_caller_account`), defaulting to the
                // distinct `entry_script_hash` so a direct entry call keeps
                // `tx.origin == msg.sender` while a nested self-offsets hop
                // (whose `msg.sender` is the bundle's executing hash) diverges.
                let sender = self
                    .caller_account
                    .clone()
                    .unwrap_or_else(|| self.entry_script_hash());
                let tx = StackItem::array(vec![
                    StackItem::byte_array(vec![0u8; 32]),
                    StackItem::UnsignedInteger(0),
                    StackItem::UnsignedInteger(0),
                    StackItem::byte_array(sender),
                    StackItem::Integer(0),
                    StackItem::Integer(0),
                    StackItem::UnsignedInteger(0),
                    StackItem::byte_array(self.input_data.clone()),
                ]);
                self.push_stack(tx)?;
                Ok(true)
            }
            "System.Runtime.GetTime" => {
                let timestamp = *self.timestamp.get_or_insert(self.default_timestamp);
                self.push_stack(StackItem::UnsignedInteger(timestamp))?;
                Ok(true)
            }
            // Task #113 — Solidity `msg.value` host-injection slot. Returns
            // the value the host injected via
            // `NeoRuntime::override_value` / `ExecutionOverrides::value`
            // for the current invocation. Coalesces `None` → 0 so source
            // that reads `msg.value` without a host override observes the
            // Neo-native "no attached value" default (matching the former
            // literal `push 0` lowering).
            "System.Runtime.GetMsgValue" => {
                let value = self.msg_value.unwrap_or(0);
                self.push_stack(StackItem::UnsignedInteger(value))?;
                Ok(true)
            }
            "System.Runtime.GetRandom" => {
                // Initialize seed on first call from block hash + tx context
                if self.random_seed.is_none() {
                    let height = self.block_height.unwrap_or(self.default_block_height);
                    let mut seed_input = height.to_le_bytes().to_vec();
                    seed_input.extend_from_slice(&self.default_account_bytes);
                    let hash = Sha256::digest(&seed_input);
                    let mut seed = [0u8; 32];
                    seed.copy_from_slice(&hash);
                    self.random_seed = Some(seed);
                }
                // Hash seed || counter for each call
                let seed = self
                    .random_seed
                    .ok_or_else(|| RuntimeError::ExecutionError {
                        message: "Random seed not initialized".to_string(),
                    })?;
                let mut input = seed.to_vec();
                input.extend_from_slice(&self.random_counter.to_le_bytes());
                self.random_counter += 1;
                let hash = Sha256::digest(&input);
                self.push_stack(StackItem::byte_array(hash[..].to_vec()))?;
                Ok(true)
            }
            "System.Runtime.Notify" => {
                // S6 fix — Neo N3 faults Notify unless the active CallFlags
                // carry AllowNotify (0b1000). A staticcall-shaped read-only
                // callee (flags 0x05 = ReadStates|AllowCall) must not be able
                // to emit notifications. The gate fires before any stack
                // mutation so a rejecting context leaves the eval stack intact.
                if self.active_call_flags & CALL_FLAG_ALLOW_NOTIFY == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "System.Runtime.Notify requires CallFlag.AllowNotify".to_string(),
                    });
                }
                // Neo N3 signature: `Notify(eventName, stateArray)`.
                //
                // The Solidity frontend uses this same syscall to deliver an
                // EVM-canonical `LogEntry { topics, data }`. The three shapes
                // are distinguished by the `eventName` length:
                //
                //   * **EVM non-anonymous shape** — `eventName` is the
                //     human-readable UTF-8 event name and `stateArray` is an
                //     Array whose first item is the 32-byte
                //     `keccak256("Name(type1,type2,...)")` topic[0]. The
                //     remaining state items are `[topic1, topic2, ..., data]`
                //     — indexed-arg topics first (already 32 bytes each),
                //     then the abi-encoded non-indexed `data` payload as the
                //     final element. This matches Ethereum's log model while
                //     keeping Neo's eventName printable.
                //
                //   * **EVM anonymous shape** — `eventName.is_empty()` AND
                //     `stateArray` is an `Array`. Per the EVM ABI (and the
                //     Solidity handbook §Events), anonymous events suppress
                //     the signature-hash topic0 so they can carry up to 4
                //     indexed topics. Layout is the same as non-anonymous
                //     except NO topic0 is prepended — `topics` is composed
                //     solely of the indexed-arg slots from the stateArray.
                //     NOTE (events-native gap): the Solidity lowering no
                //     longer emits empty event names — Neo nodes >= 3.6
                //     reject notifications whose name is not declared in
                //     the manifest, so anonymous events now Notify under
                //     their DECLARED name with the topic0-less state
                //     layout. Anonymous events with >= 1 indexed arg are
                //     therefore classified by the non-anonymous detector
                //     (their first state item is a 32-byte topic slot),
                //     which yields the correct `topics` either way. The
                //     empty-name path is retained for raw / legacy
                //     `Syscalls.notify("", [...])` payloads.
                //
                //   * **Legacy Neo shape** — any other combination. The
                //     event_name is a short ByteArray (the event's
                //     declaration-time name, e.g. `"Custom"`), and the
                //     stateArray is the full Neo-native payload. We preserve
                //     this path for `Syscalls.notify(name, data)` calls in
                //     devpack code and for any frontend that hasn't adopted
                //     the EVM shape.
                let event_name = self.pop_stack()?;
                let state = self.pop_stack()?;

                let event_name_bytes = Self::stack_item_to_bytes(event_name.clone());
                let state_is_array = matches!(state, StackItem::Array(_));

                // M-RT1 fix — record the notification so GetNotifications can
                // return it. Capture a clone of the state BEFORE the EVM-shape
                // decoding below consumes/moves it. Source script hash = the
                // executing contract's default account (Neo N3 semantics).
                let notification_state = state.clone();
                let notification_src = self.default_account_bytes.clone();
                let notification_name = event_name_bytes.clone();

                // EVM-shape detection (post-fix): the lowering now emits the
                // human-readable event name as the eventName arg (UTF-8 safe)
                // and carries topic[0] (the 32-byte keccak signature) as the
                // FIRST element of the state array — so the wire format
                // doesn't require Neo's RPC layer to UTF-8-decode arbitrary
                // hash bytes (notably 0xDD for `Transfer(...)`).
                //
                // Detection rules:
                //   * Non-anonymous EVM shape: state[0] is a 32-byte
                //     ByteString (the keccak topic), state.len() >= 2.
                //   * Anonymous EVM shape: eventName is empty and state is
                //     an Array. Topic0 is suppressed per EVM ABI; topics
                //     are the indexed args only.
                //   * Legacy Neo shape: anything else.
                let (is_evm_non_anonymous, evm_topic0_in_state) = if state_is_array {
                    if let StackItem::Array(ref items_rc) = state {
                        let first_is_32 = items_rc
                            .borrow()
                            .first()
                            .map(|i| Self::stack_item_to_bytes(i.clone()).len() == 32)
                            .unwrap_or(false);
                        (first_is_32 && !event_name_bytes.is_empty(), first_is_32)
                    } else {
                        (false, false)
                    }
                } else {
                    (false, false)
                };
                let is_evm_anonymous = event_name_bytes.is_empty() && state_is_array;

                if is_evm_non_anonymous || is_evm_anonymous {
                    let StackItem::Array(items_rc) = state else {
                        return Ok(true);
                    };
                    let items = items_rc.borrow().clone();
                    drop(items_rc);

                    // Layout for non-anonymous: state = [topic0, topic1, ..., topicN, data].
                    // Layout for anonymous:     state = [topic1, ..., topicN, data].
                    // Pull off topic0 from the head if present, then split
                    // (indexed_tail, data) where data is the last element.
                    let (head_topic0, body) = if evm_topic0_in_state && !items.is_empty() {
                        let topic0 = Self::stack_item_to_bytes(items[0].clone());
                        (Some(topic0), &items[1..])
                    } else {
                        (None, &items[..])
                    };

                    let (indexed_topics, data_bytes) = if body.is_empty() {
                        (Vec::new(), Vec::new())
                    } else {
                        let (indexed_slice, tail_slice) = body.split_at(body.len() - 1);
                        let topics: Vec<Vec<u8>> = indexed_slice
                            .iter()
                            .map(|item| Self::stack_item_to_bytes(item.clone()))
                            .collect();
                        let data = Self::stack_item_to_bytes(tail_slice[0].clone());
                        (topics, data)
                    };

                    let mut topics = Vec::with_capacity(1 + indexed_topics.len());
                    if let Some(topic0) = head_topic0 {
                        topics.push(topic0);
                    }
                    // Anonymous events: no topic0 prepend.
                    topics.extend(indexed_topics);
                    self.logs.push(LogEntry {
                        address: self.default_account.clone(),
                        topics,
                        data: data_bytes.clone(),
                    });
                    self.return_data = data_bytes;
                } else {
                    // Legacy Neo-native shape.
                    let bytes = Self::stack_item_to_bytes(state);
                    self.logs.push(LogEntry {
                        address: self.default_account.clone(),
                        topics: vec![event_name_bytes],
                        data: bytes.clone(),
                    });
                    self.return_data = bytes;
                }
                // M-RT1 fix — record the notification for GetNotifications.
                self.notifications
                    .push((notification_src, notification_name, notification_state));
                Ok(true)
            }
            "System.Runtime.Log" => {
                // S6 fix — Log shares Notify's AllowNotify gate.
                if self.active_call_flags & CALL_FLAG_ALLOW_NOTIFY == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "System.Runtime.Log requires CallFlag.AllowNotify".to_string(),
                    });
                }
                let msg = Self::stack_item_to_bytes(self.pop_stack()?);
                self.logs.push(LogEntry {
                    address: self.default_account.clone(),
                    topics: Vec::new(),
                    data: msg,
                });
                Ok(true)
            }
            "System.Runtime.CheckWitness" => {
                let witness_item = self.pop_stack()?;
                let caller_bytes = self
                    .caller_account
                    .clone()
                    .unwrap_or_else(|| self.entry_script_hash());

                let check_bytes = |bytes: &[u8]| -> bool {
                    // If witness_signers is populated, check against it
                    if !self.witness_signers.is_empty() {
                        return self.witness_signers.iter().any(|s| s == bytes);
                    }
                    // Fall back to caller (Transaction.Sender / tx.origin, which
                    // defaults to entry_script_hash) OR the executing contract.
                    bytes == caller_bytes || bytes == self.default_account_bytes
                };

                let is_match = match witness_item {
                    StackItem::Array(items) => items.borrow().iter().any(|w| {
                        let bytes = Self::stack_item_to_bytes(w.clone());
                        check_bytes(&bytes)
                    }),
                    other => {
                        let bytes = Self::stack_item_to_bytes(other);
                        check_bytes(&bytes)
                    }
                };
                self.push_stack(StackItem::Boolean(is_match))?;
                Ok(true)
            }
            "System.Runtime.GetNotifications" => {
                // M-RT1 fix — return the ACTUAL notification list (optionally
                // filtered by source script hash), not an empty array. Neo N3
                // returns `Array<Notification>` where each Notification is
                // `[scriptHash, itemName, state]`. Contracts that consume their
                // own emitted notifications (e.g. re-entrancy guards, event-
                // driven state machines) now see real data.
                //
                // Call signature: GetNotifications([hash160]) — the optional
                // filter is a 1-element Array whose sole item is the source
                // script hash (UInt160). An empty/null filter returns all.
                let filter_item = self.pop_stack()?;
                let filter_hash: Option<Vec<u8>> = match &filter_item {
                    StackItem::Array(items) if !items.borrow().is_empty() => {
                        Some(Self::stack_item_to_bytes(
                            items.borrow().first().cloned().unwrap_or(StackItem::Null),
                        ))
                    }
                    StackItem::Null => None,
                    other => Some(Self::stack_item_to_bytes(other.clone())),
                };
                let matching: Vec<StackItem> = self
                    .notifications
                    .iter()
                    .filter(|(src, _, _)| filter_hash.as_ref().map(|h| h == src).unwrap_or(true))
                    .map(|(src, name, state)| {
                        StackItem::array(vec![
                            StackItem::byte_array(src.clone()),
                            StackItem::byte_array(name.clone()),
                            state.clone(),
                        ])
                    })
                    .collect();
                self.push_stack(StackItem::array(matching))?;
                Ok(true)
            }
            "System.Runtime.BurnGas" => {
                // Call signature: BurnGas(datoshi)
                let amount_item = self.pop_stack()?;
                let amount = match amount_item {
                    StackItem::UnsignedInteger(u) => u,
                    StackItem::Integer(i) if i > 0 => i as u64,
                    StackItem::ByteArray { data: bytes, .. } => {
                        let mut buf = [0u8; 8];
                        for (i, b) in bytes.borrow().iter().take(8).enumerate() {
                            buf[i] = *b;
                        }
                        u64::from_le_bytes(buf)
                    }
                    _ => 0,
                };

                if amount == 0 {
                    return Err(RuntimeError::ExecutionError {
                        message: "BurnGas: GAS must be positive".to_string(),
                    });
                }
                self.gas_used = self.gas_used.saturating_add(amount);
                Ok(true)
            }
            "System.Runtime.CurrentSigners" => {
                // The embedded runtime does not model transaction signers. Return an empty array.
                self.push_stack(StackItem::array(Vec::new()))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
