use super::*;
use crate::opcode::OpCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallPatchKind {
    /// CALL_L — fixup writes a 4-byte signed offset relative to the CALL_L
    /// opcode position (opcode byte preceding `position`).
    CallRelative,
    /// PUSHINT32 — fixup writes a 4-byte signed absolute offset of the target
    /// function. Used by `PushFunctionOffset` for internal function-pointer
    /// values consumed later by `CALLA`.
    AbsoluteOffset,
}

#[derive(Clone, Debug)]
pub(crate) struct CallPatch {
    pub(crate) position: usize,
    pub(crate) target: String,
    pub(crate) kind: CallPatchKind,
}

#[derive(Clone, Debug)]
pub(crate) struct BytecodeBuildOutput {
    pub script: Vec<u8>,
    pub tokens: Vec<crate::neo::MethodToken>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct MethodTokenKey {
    pub(crate) hash: [u8; 20],
    pub(crate) method: String,
    pub(crate) parameters_count: u16,
    pub(crate) has_return_value: bool,
    pub(crate) call_flags: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct MethodTokenPatch {
    pub(crate) position: usize,
    pub(crate) token: MethodTokenKey,
}

/// Neo N3 uses CallFlags bitmask values.
pub(crate) const CALLFLAGS_ALL: u8 = 0x0F;
/// Read-only call flags (ReadStates | AllowCall).
pub(crate) const CALLFLAGS_READ_ONLY: u8 = 0x05;

/// Native contract script hashes as they must be pushed onto the NeoVM stack
/// (UInt160 little-endian byte order).
const NATIVE_NEO_HASH_LE: [u8; 20] =
    *b"\xf5\x63\xea\x40\xbc\x28\x3d\x4d\x0e\x05\xc4\x8e\xa3\x05\xb3\xf2\xa0\x73\x40\xef";
const NATIVE_GAS_HASH_LE: [u8; 20] =
    *b"\xcf\x76\xe2\x8b\xd0\x06\x2c\x4a\x47\x8e\xe3\x55\x61\x01\x13\x19\xf3\xcf\xa4\xd2";
const NATIVE_CONTRACT_MANAGEMENT_HASH_LE: [u8; 20] =
    *b"\xfd\xa3\xfa\x43\x46\xea\x53\x2a\x25\x8f\xc4\x97\xdd\xad\xdb\x64\x37\xc9\xfd\xff";
const NATIVE_POLICY_HASH_LE: [u8; 20] =
    *b"\x7b\xc6\x81\xc0\xa1\xf7\x1d\x54\x34\x57\xb6\x8b\xba\x8d\x5f\x9f\xdd\x4e\x5e\xcc";
const NATIVE_ORACLE_HASH_LE: [u8; 20] =
    *b"\x58\x87\x17\x11\x7e\x0a\xa8\x10\x72\xaf\xab\x71\xd2\xdd\x89\xfe\x7c\x4b\x92\xfe";
const NATIVE_ROLE_MANAGEMENT_HASH_LE: [u8; 20] =
    *b"\xe2\x95\xe3\x91\x54\x4c\x17\x8a\xd9\x4f\x03\xec\x4d\xcd\xff\x78\x53\x4e\xcf\x49";
const NATIVE_NOTARY_HASH_LE: [u8; 20] =
    *b"\x3b\xec\x35\x31\x11\x9b\xba\xd7\x6d\xd0\x44\x92\x0b\x0d\xe6\xc3\x19\x4f\xe1\xc1";
const NATIVE_TREASURY_HASH_LE: [u8; 20] =
    *b"\xc1\x3a\x56\xc9\x83\x53\xa7\xea\x6a\x32\x4d\x9a\x83\x5d\x1b\x5b\xf2\x26\x63\x15";
const NATIVE_LEDGER_HASH_LE: [u8; 20] =
    *b"\xbe\xf2\x04\x31\x40\x36\x2a\x77\xc1\x50\x99\xc7\xe6\x4c\x12\xf7\x00\xb6\x65\xda";
const NATIVE_CRYPTOLIB_HASH_LE: [u8; 20] =
    *b"\x1b\xf5\x75\xab\x11\x89\x68\x84\x13\x61\x0a\x35\xa1\x28\x86\xcd\xe0\xb6\x6c\x72";
const NATIVE_STDLIB_HASH_LE: [u8; 20] =
    *b"\xc0\xef\x39\xce\xe0\xe4\xe9\x25\xc6\xc2\xa0\x6a\x79\xe1\x44\x0d\xd8\x6f\xce\xac";

pub(crate) fn native_contract_hash(contract: ir::NativeContract) -> [u8; 20] {
    match contract {
        ir::NativeContract::Neo => NATIVE_NEO_HASH_LE,
        ir::NativeContract::Gas => NATIVE_GAS_HASH_LE,
        ir::NativeContract::ContractManagement => NATIVE_CONTRACT_MANAGEMENT_HASH_LE,
        ir::NativeContract::Policy => NATIVE_POLICY_HASH_LE,
        ir::NativeContract::Oracle => NATIVE_ORACLE_HASH_LE,
        ir::NativeContract::RoleManagement => NATIVE_ROLE_MANAGEMENT_HASH_LE,
        ir::NativeContract::Notary => NATIVE_NOTARY_HASH_LE,
        ir::NativeContract::Treasury => NATIVE_TREASURY_HASH_LE,
        ir::NativeContract::Ledger => NATIVE_LEDGER_HASH_LE,
        ir::NativeContract::CryptoLib => NATIVE_CRYPTOLIB_HASH_LE,
        ir::NativeContract::StdLib => NATIVE_STDLIB_HASH_LE,
    }
}

pub(crate) fn generate_contract_bytecode(
    metadata: &mut ContractMetadata,
    ir_module: &ir::Module,
    verbose: bool,
    optimizer_level: u8,
    use_callt: bool,
) -> Result<BytecodeBuildOutput, String> {
    let function_map: HashMap<_, _> = ir_module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect();

    if verbose {
        println!("  • Using optimizer level {}", optimizer_level.min(3));
    }

    let mut bytecode = Vec::new();
    let mut call_fixups: Vec<CallPatch> = Vec::new();
    let mut token_fixups: Vec<MethodTokenPatch> = Vec::new();

    // Sort methods so that public/external methods are emitted before internal/private ones.
    // This ensures the first public method (typically `run`) starts at offset 0, which is
    // where the runtime begins execution. Internal helper functions are placed after.
    metadata.methods.sort_by_key(|m| match m.visibility {
        VisibilityKind::Public | VisibilityKind::External => 0,
        VisibilityKind::Internal | VisibilityKind::Private => 1,
    });

    for method in metadata.methods.iter_mut() {
        // Constructors without bodies are not emitted as standalone NeoVM methods.
        // They are only relevant when called from an injected `_deploy`.
        if matches!(method.kind, FunctionKind::Constructor) && method.body.is_none() {
            continue;
        }

        let method_name = method.neo_name.clone();
        let ir_function = function_map
            .get(method_name.as_str())
            .copied()
            .ok_or_else(|| {
                format!("internal compiler error: missing IR for method '{method_name}'")
            })?;

        // NeoVM's `INITSLOT` encodes the local-slot count as a single byte, and
        // LDLOC/STLOC index the same 0..=255 range. A function with more than 255
        // locals would silently truncate (wrong slot count + wrong indices),
        // producing miscompiled bytecode — fail loudly instead.
        if ir_function.local_count > u8::MAX as u16 {
            return Err(format!(
                "function '{method_name}' requires {} local slots, exceeding NeoVM's 255-slot limit; \
                 reduce local variables (e.g. split the function or reuse temporaries)",
                ir_function.local_count
            ));
        }

        // INITSLOT's `args` operand and LDARG/STARG indices share the same
        // 0..=255 byte range, so >255 parameters would silently truncate the
        // operand (as local_count above) and miscompile — fail loudly instead.
        if method.parameters.len() > u8::MAX as usize {
            return Err(format!(
                "function '{method_name}' has {} parameters, exceeding NeoVM's 255-argument limit",
                method.parameters.len()
            ));
        }

        let instruction_count: usize = ir_function
            .basic_blocks
            .iter()
            .map(|block| block.instructions.len())
            .sum();

        let offset = bytecode.len() as u32;
        method.offset = offset;

        if verbose {
            println!(
                "  • Emitting method '{method_name}' at offset {offset} ({instruction_count} IR instruction(s))"
            );
        }

        let (function_bytes, patches, token_patches) =
            emit_ir_function(ir_function, ir_module, method, use_callt)?;
        let base_position = bytecode.len();
        bytecode.extend_from_slice(&function_bytes);

        for patch in patches {
            call_fixups.push(CallPatch {
                position: base_position + patch.position,
                target: patch.target,
                kind: patch.kind,
            });
        }

        for patch in token_patches {
            token_fixups.push(MethodTokenPatch {
                position: base_position + patch.position,
                token: patch.token,
            });
        }
    }

    let offset_map: HashMap<String, u32> = metadata
        .methods
        .iter()
        .map(|method| (method.neo_name.clone(), method.offset))
        .collect();

    for fixup in call_fixups {
        if let Some(target_offset) = offset_map.get(&fixup.target) {
            let bytes = match fixup.kind {
                CallPatchKind::CallRelative => {
                    // CALL_L uses a 4-byte signed offset from the beginning of the CALL_L opcode.
                    let opcode_pos = fixup.position.saturating_sub(1) as i32;
                    let relative = (*target_offset as i32).checked_sub(opcode_pos).unwrap_or(0);
                    relative.to_le_bytes()
                }
                CallPatchKind::AbsoluteOffset => {
                    // PUSHA operand is a SIGNED offset relative to the PUSHA opcode
                    // (the byte preceding `position`), matching real NeoVM's
                    // `Pointer{ instruction_pointer + operand }`. The resulting
                    // Pointer is consumed by `CALLA`. Task #186.
                    let opcode_pos = fixup.position.saturating_sub(1) as i32;
                    let relative = (*target_offset as i32).checked_sub(opcode_pos).unwrap_or(0);
                    relative.to_le_bytes()
                }
            };
            if fixup.position + 4 <= bytecode.len() {
                bytecode[fixup.position..fixup.position + 4].copy_from_slice(&bytes);
            }
        } else {
            // M-BC1 fix — an unresolved call target previously fell back to
            // `eprintln!` + leaving the operand as zero bytes, which emits a
            // CALL_L with offset 0 (= infinite loop / call-to-self on-chain).
            // Surface it as a hard error instead.
            return Err(format!(
                "bytecode emission: unresolved call target '{}' (method offset                  unavailable — the callee may be external and unlinked, or the                  IR call graph is inconsistent)",
                fixup.target
            ));
        }
    }

    if bytecode.is_empty() {
        bytecode.push(OpCode::RET.byte());
    }

    let tokens = if use_callt {
        apply_method_tokens(bytecode.as_mut_slice(), &token_fixups)?
    } else {
        Vec::new()
    };

    Ok(BytecodeBuildOutput {
        script: bytecode,
        tokens,
    })
}

fn apply_method_tokens(
    bytecode: &mut [u8],
    patches: &[MethodTokenPatch],
) -> Result<Vec<crate::neo::MethodToken>, String> {
    use std::collections::{BTreeMap, BTreeSet};

    if patches.is_empty() {
        return Ok(Vec::new());
    }

    let unique: BTreeSet<MethodTokenKey> = patches.iter().map(|p| p.token.clone()).collect();
    if unique.len() > crate::neo::MAX_METHOD_TOKENS {
        return Err(format!(
            "CALLT requires <= {} method token(s); got {}",
            crate::neo::MAX_METHOD_TOKENS,
            unique.len()
        ));
    }

    let mut token_map: BTreeMap<MethodTokenKey, u16> = BTreeMap::new();
    let mut tokens: Vec<crate::neo::MethodToken> = Vec::with_capacity(unique.len());

    for (index, key) in unique.into_iter().enumerate() {
        // M-BC1 fix — the u16 token index must not silently alias to #511 when
        // the MAX_METHOD_TOKENS cap is exceeded; surface it as a hard error so
        // a too-large token table fails loudly instead of emitting wrong CALLT
        // operands. (The current cap is 512, which fits in u16, but the cap is
        // a spec constant that could change.)
        let token_index = u16::try_from(index).map_err(|_| {
            format!(
                "bytecode emission: method-token index {index} exceeds u16 range                  (MAX_METHOD_TOKENS cap exceeded)"
            )
        })?;
        token_map.insert(key.clone(), token_index);
        tokens.push(crate::neo::MethodToken::new(
            key.hash,
            &key.method,
            key.parameters_count,
            key.has_return_value,
            key.call_flags,
        ));
    }

    for patch in patches {
        // M-BC1 fix — a patch whose token isn't in the map, or whose position
        // is out of range, previously `continue`d silently and left the CALLT
        // operand as 0x0000 (token #0 = wrong native call at runtime). Surface
        // both as hard errors.
        let Some(index) = token_map.get(&patch.token) else {
            return Err(format!(
                "bytecode emission: CALLT patch references an unregistered method                  token (hash={:?}, method='{}')",
                patch.token.hash, patch.token.method
            ));
        };
        let start = patch.position;
        let end = start + 2;
        if end > bytecode.len() {
            return Err(format!(
                "bytecode emission: CALLT patch position {start} out of range                  (bytecode len {})",
                bytecode.len()
            ));
        }
        bytecode[start..end].copy_from_slice(&index.to_le_bytes());
    }

    Ok(tokens)
}
