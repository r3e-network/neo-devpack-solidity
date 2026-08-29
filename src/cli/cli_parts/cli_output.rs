use super::*;

pub(crate) fn contract_output_prefix(
    base: &str,
    contract_name: &str,
    index: usize,
    total: usize,
) -> String {
    let base_path = std::path::Path::new(base);
    let base_is_dir = base.ends_with('/') || base.ends_with('\\') || base_path.is_dir();

    let sanitized = standard_json::sanitize_contract_name(contract_name).unwrap_or_else(|| {
        if total <= 1 {
            "contract".to_string()
        } else {
            format!("contract{index}")
        }
    });

    if base_is_dir {
        return base_path.join(sanitized).to_string_lossy().to_string();
    }

    if total <= 1 {
        return base.to_string();
    }

    let (stem, ext) = split_extension(base);
    if ext.is_empty() {
        format!("{stem}-{sanitized}")
    } else {
        format!("{stem}-{sanitized}{ext}")
    }
}

fn split_extension(path: &str) -> (String, String) {
    use std::path::Path;

    let p = Path::new(path);
    let Some(file_name) = p.file_name().and_then(|name| name.to_str()) else {
        return (path.to_string(), String::new());
    };

    let Some((file_stem, ext)) = file_name.rsplit_once('.') else {
        return (path.to_string(), String::new());
    };

    // Treat dotfiles like `.env` as having no extension.
    if file_stem.is_empty() {
        return (path.to_string(), String::new());
    }

    let stem_path = match p.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        Some(parent) => parent.join(file_stem).to_string_lossy().to_string(),
        None => file_stem.to_string(),
    };

    (stem_path, format!(".{ext}"))
}

pub(crate) fn ensure_output_dir(path: &str) -> Result<(), String> {
    let Some(parent) = std::path::Path::new(path).parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent).map_err(|err| {
        format!(
            "Failed to create output directory '{}': {err}",
            parent.display()
        )
    })
}

pub(crate) fn write_nef_file(
    path: &str,
    script: &[u8],
    tokens: &[neo_devpack_solidity::neo::MethodToken],
    source: &str,
    json_warnings: bool,
) -> Result<u32, String> {
    // Canonicalize paths when they look like local files; leave URLs/overrides intact.
    let resolved_source = if source.contains("://") {
        source.to_string()
    } else {
        std::path::Path::new(source)
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| source.to_string())
    };

    let (clamped, truncated) = clamp_nef_source_with_flag(&resolved_source);
    if truncated {
        let msg = format!("NEF source exceeds {NEF_SOURCE_MAX_BYTES} bytes and was truncated");
        emit_warning(&msg, None, json_warnings, Some("NEF_SOURCE_TRUNCATED"));
    }

    let nef = build_nef_with_tokens(script, COMPILER_ID, clamped.as_ref(), tokens)?;
    let checksum_offset = nef.len().saturating_sub(4);
    let checksum = if nef.len() >= 4 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&nef[checksum_offset..checksum_offset + 4]);
        u32::from_le_bytes(buf)
    } else {
        0
    };
    ensure_output_dir(path)?;
    fs::write(path, nef).map_err(|err| format!("Failed to write NEF file '{path}': {err}"))?;
    Ok(checksum)
}

pub(crate) fn write_manifest_file(path: &str, manifest: &serde_json::Value) -> Result<(), String> {
    let manifest_str = serde_json::to_string_pretty(manifest)
        .map_err(|err| format!("Manifest serialization failed: {err}"))?;
    ensure_output_dir(path)?;
    fs::write(path, manifest_str)
        .map_err(|err| format!("Failed to write manifest file '{path}': {err}"))?;
    Ok(())
}

pub(crate) fn write_json_file(
    path: &str,
    script: &[u8],
    tokens: &[neo_devpack_solidity::neo::MethodToken],
    manifest: &serde_json::Value,
    metadata: &ContractMetadata,
    source: &str,
    json_warnings: bool,
) -> Result<(), String> {
    // Canonicalize paths when they look like local files; leave URLs/overrides intact.
    let resolved_source = if source.contains("://") {
        source.to_string()
    } else {
        std::path::Path::new(source)
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| source.to_string())
    };

    let (clamped, truncated) = clamp_nef_source_with_flag(&resolved_source);
    if truncated {
        let msg = format!("NEF source exceeds {NEF_SOURCE_MAX_BYTES} bytes and was truncated");
        emit_warning(&msg, None, json_warnings, Some("NEF_SOURCE_TRUNCATED"));
    }

    let tokens_json: Vec<_> = tokens
        .iter()
        .map(|token| {
            let hash_be = token.hash.iter().rev().copied().collect::<Vec<_>>();
            json!({
                "hash": format!("0x{}", hex::encode(hash_be)),
                "method": token.method,
                "parametersCount": token.parameters_count,
                "hasReturnValue": token.has_return_value,
                "callFlags": token.call_flags,
            })
        })
        .collect();
    let neo_method_map = standard_json::build_neo_method_map(metadata);

    let nef_bytes = build_nef_with_tokens(script, COMPILER_ID, clamped.as_ref(), tokens)?;
    let checksum = if nef_bytes.len() >= 4 {
        hex::encode(&nef_bytes[nef_bytes.len() - 4..])
    } else {
        "00000000".to_string()
    };
    let nef_image = hex::encode(nef_bytes);

    let json_output = json!({
        "contract": metadata.name,
        "compiler": COMPILER_ID,
        "author": COMPILER_EMAIL,
        "nef": {
            "magic": "NEF3",
            "compiler": COMPILER_ID,
            "version": compiler_version_string_4(),
            "source": clamped.as_ref(),
            "tokens": tokens_json,
            "script": hex::encode(script),
            "image": nef_image,
            "checksum": checksum,
        },
        "manifest": manifest,
        "methodMap": neo_method_map,
    });

    let json_str = serde_json::to_string_pretty(&json_output)
        .map_err(|err| format!("Failed to serialise JSON output: {err}"))?;
    ensure_output_dir(path)?;
    fs::write(path, json_str)
        .map_err(|err| format!("Failed to write JSON file '{path}': {err}"))?;
    Ok(())
}

pub(crate) fn write_assembly_file(path: &str, script: &[u8]) -> Result<(), String> {
    let assembly = codegen::disassemble_neovm_bytecode(script);
    ensure_output_dir(path)?;
    fs::write(path, assembly)
        .map_err(|err| format!("Failed to write assembly file '{path}': {err}"))?;
    Ok(())
}
