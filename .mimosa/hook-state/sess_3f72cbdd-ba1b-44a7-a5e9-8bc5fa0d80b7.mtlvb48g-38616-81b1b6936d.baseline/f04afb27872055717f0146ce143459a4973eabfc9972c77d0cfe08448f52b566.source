use super::*;

/// Dispatch table for same-name functions: `(name, arity)` maps to every
/// overload sharing that key, each carrying its parameter ValueTypes and the
/// type-mangled neo_name. See [`LoweringContext::resolve_overload`].
pub(crate) type FunctionOverloadTable = HashMap<(String, usize), Vec<(Vec<ValueType>, String)>>;

/// Compatibility test for same-arity overload resolution: is a call argument
/// of type `arg` an acceptable match for a parameter of type `param`?
///
/// Integers match by SIGNEDNESS only — Solidity rejects same-arity overloads
/// that differ solely by integer width as ambiguous, so width never has to
/// distinguish them, whereas `int` vs `uint` can. Everything else (address,
/// bool, bytesN, string, struct, ...) must match exactly so e.g. `f(uint256)`
/// and `f(address)` are told apart.
pub(crate) fn overload_arg_matches(arg: &ValueType, param: &ValueType) -> bool {
    match (arg, param) {
        (ValueType::Integer { signed: a, .. }, ValueType::Integer { signed: p, .. }) => a == p,
        _ => arg == param,
    }
}

impl<'a> LoweringContext<'a> {
    pub(crate) fn neo_function_name(&self, name: &str, arg_count: usize) -> Option<String> {
        self.function_overloads
            .get(&(name.to_string(), arg_count))
            .and_then(|bucket| bucket.first().map(|(_, neo_name)| neo_name.clone()))
    }

    /// Resolve a same-arity overload by argument type. Solidity allows
    /// `f(uint256)` and `f(address)` to share `(name, arity)`; the frontend
    /// mangles them into distinct neo_names, and this picks the right one by
    /// matching the call's inferred argument types against each overload's
    /// declared parameter types. With a single overload it returns that name
    /// directly (the common, non-overloaded case). Returns `None` when several
    /// overloads exist and none matches confidently, so the caller fails loud
    /// instead of dispatching to the wrong function.
    pub(crate) fn resolve_overload(
        &self,
        name: &str,
        arg_count: usize,
        arg_types: &[Option<ValueType>],
        arg_is_int_literal: &[bool],
    ) -> Option<String> {
        let bucket = self
            .function_overloads
            .get(&(name.to_string(), arg_count))?;
        if bucket.len() == 1 {
            return Some(bucket[0].1.clone());
        }
        let mut best: Option<(usize, &String)> = None;
        for (params, neo_name) in bucket {
            if params.len() != arg_count {
                continue;
            }
            let mut score = 0usize;
            let mut compatible = true;
            for (i, (param, arg)) in params.iter().zip(arg_types.iter()).enumerate() {
                // A positive integer LITERAL is convertible to any integer type
                // (Solidity implicit literal conversion), so it matches any
                // integer param regardless of signedness. Without this,
                // `f(intVar, 3)` — where `3` infers to uint256 — leaves
                // `[int256, uint256]`, which disqualifies BOTH the (uint,uint)
                // and (int,int) overloads (exact-signedness match) and the call
                // traps at runtime with "no compiled body". The non-literal args
                // still disambiguate by exact type.
                if arg_is_int_literal.get(i).copied().unwrap_or(false)
                    && matches!(param, ValueType::Integer { .. })
                {
                    score += 1;
                    continue;
                }
                match arg {
                    Some(arg) if overload_arg_matches(arg, param) => score += 1,
                    Some(_) => {
                        compatible = false;
                        break;
                    }
                    None => {} // unknown arg type — neither matches nor disqualifies
                }
            }
            if compatible && best.is_none_or(|(best_score, _)| score > best_score) {
                best = Some((score, neo_name));
            }
        }
        best.map(|(_, neo_name)| neo_name.clone())
    }

    pub(crate) fn has_using_directives(&self) -> bool {
        !self.using_target_types.is_empty()
    }

    pub(crate) fn using_target_allows_receiver(&self, receiver_type: &ValueType) -> bool {
        if self.using_target_types.is_empty() {
            return false;
        }

        let receiver_sig =
            normalize_solidity_like_type_signature(&value_type_signature(receiver_type));
        self.using_target_types.iter().any(|target| match target {
            None => true,
            Some(target_type) => using_target_matches_signature(target_type, &receiver_sig),
        })
    }

    pub(crate) fn using_function_list_allows_receiver(
        &self,
        function_name: &str,
        receiver_type: Option<&ValueType>,
    ) -> bool {
        let Some(receiver_type) = receiver_type else {
            // Unknown receiver type: preserve compatibility, defer to runtime behavior.
            return true;
        };

        let receiver_sig =
            normalize_solidity_like_type_signature(&value_type_signature(receiver_type));
        let list_scope_applies = self.using_function_list_scope_targets.iter().any(|target| {
            target
                .as_ref()
                .is_none_or(|expected| using_target_matches_signature(expected, &receiver_sig))
        });

        // No function-list directives apply to this receiver type.
        if !list_scope_applies {
            return true;
        }

        let key = function_name.to_ascii_lowercase();
        let Some(targets) = self.using_function_list_targets.get(&key) else {
            // Function-list directives apply, but this function name was not listed.
            return false;
        };

        targets.iter().any(|target| match target {
            None => true,
            Some(target_type) => using_target_matches_signature(target_type, &receiver_sig),
        })
    }

    pub(crate) fn receiver_matches_function_overload(
        &self,
        function_name: &str,
        arg_count: usize,
        receiver_type: &ValueType,
    ) -> bool {
        let key = (function_name.to_string(), arg_count);
        let Some(expected_types) = self.function_first_param_types.get(&key) else {
            // Without type metadata, don't over-constrain existing behavior.
            return true;
        };
        // A library `using` directive resolves the receiver against ANY
        // overload of the named function — Solidity's overload resolution
        // picks the first one whose parameter types are implicitly
        // convertible from the actuals. We mirror that here: accept the call
        // if any registered overload's first parameter accepts the receiver.
        // This fixes Uniswap V4 `int256.toInt128()` after we taught the IR
        // builder to keep BOTH `toInt128(int256)` and `toInt128(uint256)`
        // entries in the same bucket.
        expected_types
            .iter()
            .any(|expected| is_implicitly_convertible(receiver_type, expected))
    }

    /// Returns the ordered parameter names for a function overload, if known.
    pub(crate) fn get_function_param_names(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Option<&[String]> {
        self.function_param_names
            .get(&(name.to_string(), arg_count))
            .map(|v| v.as_slice())
    }

    /// Task #191 — look up the first return type for a function overload.
    /// `None` means no metadata (the function may be external, have no
    /// return value, or the build pipeline didn't register it). Callers
    /// should degrade gracefully (e.g., `infer_type_from_expression` falls
    /// through to the existing type-inference logic).
    pub(crate) fn get_function_return_type(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Option<&ValueType> {
        self.function_return_types
            .get(&(name.to_string(), arg_count))
    }

    /// Returns true if the named function returns void (no return values).
    pub(crate) fn is_void_function(&self, name: &str) -> bool {
        self.void_functions.contains(name)
    }

    /// The declared type of the `index`-th parameter of a call to `name` with
    /// `arg_count` arguments, when it is UNAMBIGUOUS across overloads. Returns
    /// `None` if the function is unknown (e.g. an external method on a contract
    /// not compiled in this unit) or if overloads disagree on that parameter's
    /// type. Used to canonicalize an integer-backed `bytesN` literal argument to
    /// its big-endian ByteString at call sites — a `bytesN` literal otherwise
    /// reaches the callee as a little-endian Integer, so any byte-level
    /// operation on it (indexing, comparison, hashing) is wrong on a real node
    /// (the simulator's lenient typing masks it).
    pub(crate) fn callee_param_type(
        &self,
        name: &str,
        arg_count: usize,
        index: usize,
    ) -> Option<ValueType> {
        let overloads = self
            .function_overloads
            .get(&(name.to_string(), arg_count))?;
        let mut found: Option<&ValueType> = None;
        for (param_types, _neo_name) in overloads {
            let ty = param_types.get(index)?;
            match found {
                None => found = Some(ty),
                Some(existing) if existing == ty => {}
                Some(_) => return None, // overloads disagree — don't guess
            }
        }
        found.cloned()
    }

    /// Returns the renamed super-method name for `super.method()` resolution.
    ///
    /// Task #85 — look up the caller-qualified key first
    /// (`"{current_fn}::{method_name}"`) so that inside a preserved base body
    /// `__super_foo` the nested `super.foo()` resolves to the NEXT-older
    /// `__super2_foo`, not back to itself. Falls back to the unqualified key
    /// for the top-of-chain derived frame and for any legacy call sites.
    pub(crate) fn super_method_name(&self, method_name: &str) -> Option<&str> {
        let qualified = format!("{}::{}", self.function_name, method_name);
        if let Some(target) = self.super_method_map.get(&qualified) {
            return Some(target.as_str());
        }
        self.super_method_map.get(method_name).map(|s| s.as_str())
    }
}
