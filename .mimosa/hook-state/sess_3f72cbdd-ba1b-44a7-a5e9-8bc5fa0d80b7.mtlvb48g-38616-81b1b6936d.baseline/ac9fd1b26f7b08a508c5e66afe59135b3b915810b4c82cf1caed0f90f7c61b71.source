use super::*;

use solang_parser::pt::{Import, ImportPath, SourceUnitPart};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Bound the `start`-directory upward climb so degenerate paths
/// (`/a/b/c/...` with hundreds of components) can't make us stat the
/// filesystem forever. Real-world Solidity projects sit a few
/// directories deep. Shared by `extend_with_auto_node_modules` and
/// `extend_with_auto_remappings` so they cannot drift apart.
const MAX_CLIMB: usize = 16;

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSoliditySources {
    pub(crate) files: Vec<(PathBuf, String)>,
    pub(crate) combined_source: String,
}

/// A single Foundry-style import remapping: any import whose path starts with
/// `prefix` is rewritten to start with `replacement` instead. Prefix matching
/// is done on the raw string (so a trailing slash is significant — `foo/=bar/`
/// rewrites `foo/baz.sol` to `bar/baz.sol`; `foo=bar` rewrites `foo` and
/// `foobar/x.sol` alike). Order matters: the first matching remapping wins,
/// which mirrors solc / foundry semantics.
#[derive(Debug, Clone)]
pub(crate) struct ImportRemapping {
    pub(crate) prefix: String,
    pub(crate) replacement: PathBuf,
}

#[allow(dead_code)] // used by the test suite under #[cfg(test)]; kept as a
                    // thin wrapper so callers without remappings have a stable, ergonomic name.
pub(crate) fn resolve_solidity_sources_with_imports(
    entry_file: &Path,
    include_paths: &[PathBuf],
) -> Result<ResolvedSoliditySources, String> {
    resolve_solidity_sources_with_options(entry_file, include_paths, &[])
}

/// Resolve a Solidity entry file's `import` directives from disk (Foundry-style
/// remappings, auto-discovered `node_modules`, and `remappings.txt`) and return
/// the single combined source string ready to hand to `compile_contracts`.
///
/// `compile_contracts` itself takes a source STRING and performs no filesystem
/// import resolution, so tools that compile a FILE — e.g. the `neo-test` runner
/// or any editor/CI integration — need this to support multi-file projects
/// (`import "./Token.sol"`, `import "@openzeppelin/..."`, a shared test base).
///
/// `include_paths` are extra roots to resolve non-relative imports against
/// (in addition to the entry file's directory and auto-discovered node_modules).
pub fn resolve_source_with_imports(
    entry_file: &Path,
    include_paths: &[PathBuf],
) -> Result<String, String> {
    resolve_solidity_sources_with_options(entry_file, include_paths, &[]).map(|r| r.combined_source)
}

pub(crate) fn resolve_solidity_sources_with_options(
    entry_file: &Path,
    include_paths: &[PathBuf],
    remappings: &[ImportRemapping],
) -> Result<ResolvedSoliditySources, String> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut visiting: HashSet<PathBuf> = HashSet::new();
    let mut ordered: Vec<(PathBuf, String)> = Vec::new();
    let mut stack: VecDeque<PathBuf> = VecDeque::new();

    // Expand the user-supplied include paths with auto-discovered `node_modules`
    // directories. The walk starts from each include path and from the entry
    // file's directory, climbing parents up to the filesystem root, so that
    // `import "@openzeppelin/contracts/.../X.sol"` resolves against any
    // `node_modules/` co-located with the project — mirroring how solc /
    // hardhat / foundry locate npm-style imports.
    let mut effective_include_paths: Vec<PathBuf> = include_paths.to_vec();
    extend_with_auto_node_modules(entry_file, &mut effective_include_paths);
    for inc in include_paths {
        extend_with_auto_node_modules(inc, &mut effective_include_paths);
    }

    // Auto-load remappings.txt files from ancestor directories of the entry
    // file. Foundry stores its remappings in `<project>/remappings.txt` (and
    // sometimes one per nested package via `lib/<pkg>/remappings.txt`). The
    // user-supplied remappings take precedence: we append discovered entries
    // after the explicit ones so the latter win on prefix collisions.
    //
    // To resolve auto-discovered remappings whose `path` is relative (the
    // common case), each entry is rebased against the directory that owns
    // the `remappings.txt` — so `@uniswap/v4-core/=lib/v4-core/` in
    // `<project>/remappings.txt` resolves to `<project>/lib/v4-core/`.
    let mut effective_remappings: Vec<ImportRemapping> = remappings.to_vec();
    extend_with_auto_remappings(entry_file, &mut effective_remappings);
    // Also scan each include path for nested packages with foundry-style
    // `lib/` layouts. The entry-file walk only reaches the project tree;
    // explicit `--include-path /path/to/node_modules` should ALSO contribute
    // its package-internal remappings so e.g. `permit2/=lib/permit2/`
    // (vendored inside `@uniswap/v4-periphery`) is auto-registered.
    for inc in include_paths {
        extend_with_auto_remappings(inc, &mut effective_remappings);
    }

    fn extract_imports(source: &str, file: &Path) -> Result<Vec<String>, String> {
        fn offset_to_line_column(source: &str, offset: usize) -> (usize, usize) {
            let mut line = 1usize;
            let mut column = 1usize;
            let mut current = 0usize;

            for ch in source.chars() {
                if current >= offset {
                    break;
                }

                if ch == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }

                current += ch.len_utf8();
            }

            (line, column)
        }

        // Guarded parse: runs on a worker thread with a large bounded stack
        // so deeply nested input becomes a diagnostic, not a stack-overflow
        // abort. Keep ALL solang_parser::parse calls behind this helper.
        let (unit, _comments) = neo_devpack_solidity::frontend::parse_solidity_guarded(source)
            .map_err(|diags| {
                let summary = diags
                    .iter()
                    .map(|diag| {
                        if let solang_parser::pt::Loc::File(_, start, _) = diag.loc {
                            let (line, column) = offset_to_line_column(source, start);
                            format!("{}:{}: {}", line, column, diag.message)
                        } else {
                            diag.message.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "failed to parse '{}' while resolving imports:\n{}",
                    file.display(),
                    summary
                )
            })?;

        let mut imports = Vec::new();
        for part in unit.0.iter() {
            let SourceUnitPart::ImportDirective(import) = part else {
                continue;
            };

            match import {
                Import::Plain(path, _) => {
                    imports.push(extract_import_path_string(path, file)?);
                }
                Import::Rename(path, _renames, _) => {
                    imports.push(extract_import_path_string(path, file)?);
                }
                Import::GlobalSymbol(path, _, _) => {
                    imports.push(extract_import_path_string(path, file)?);
                }
            }
        }

        Ok(imports)
    }

    fn extract_import_path_string(path: &ImportPath, file: &Path) -> Result<String, String> {
        match path {
            ImportPath::Filename(lit) => Ok(lit.string.clone()),
            ImportPath::Path(_) => Err(format!(
                "unsupported import path kind in '{}': path imports are not supported",
                file.display()
            )),
        }
    }

    fn resolve_import_file(
        import_path: &str,
        from_file: &Path,
        include_paths: &[PathBuf],
        remappings: &[ImportRemapping],
    ) -> Result<PathBuf, String> {
        // `import_aliases` is the shared module-level helper (see the bottom of
        // this file) so the CLI and standard-JSON resolvers never drift.

        // Apply remappings first. Foundry's rule: the first matching remapping
        // (in declaration order) wins, longest-prefix first when multiple
        // share a common start — but we rely on the caller to order the list.
        // The result is a "pre-resolved" absolute or relative path that we
        // then run through the normal include-path + alias search.
        let mut working_import = import_path.to_string();
        let mut matched_remapping: Option<&ImportRemapping> = None;
        for remap in remappings {
            if working_import.starts_with(&remap.prefix) {
                // Foundry's semantics: a remapping is a pure string-prefix
                // substitution. The suffix after `prefix` is appended
                // verbatim. We CANNOT use `Path::join` here because
                // `Path::join` of an absolute path replaces the base — a
                // remapping suffix that happens to begin with `/` (very
                // common: `@openzeppelin/contracts@4.8.3=…` strips the
                // prefix with no trailing slash, so the suffix starts with
                // `/token/...`) would silently truncate the replacement.
                let suffix = &working_import[remap.prefix.len()..];
                let replacement_str = remap.replacement.to_string_lossy();
                // Join the replacement and suffix as strings, ensuring
                // exactly one `/` between them.
                let stitched = if replacement_str.ends_with('/') || suffix.starts_with('/') {
                    format!("{replacement_str}{suffix}")
                } else if replacement_str.is_empty() {
                    suffix.to_string()
                } else {
                    format!("{replacement_str}/{suffix}")
                };
                working_import = stitched;
                matched_remapping = Some(remap);
                break;
            }
        }

        let mut candidates: Vec<PathBuf> = Vec::new();
        let from_dir = from_file.parent().unwrap_or_else(|| Path::new("."));

        // If a remapping fired, also try the remapped path *as-is* before the
        // normal search — remappings should usually point directly at the
        // resolved file. The remapped path may already be absolute.
        if matched_remapping.is_some() {
            let remapped = Path::new(&working_import);
            if remapped.is_absolute() {
                candidates.push(remapped.to_path_buf());
            } else {
                candidates.push(remapped.to_path_buf());
                candidates.push(from_dir.join(remapped));
                for include_dir in include_paths {
                    candidates.push(include_dir.join(remapped));
                }
            }
        }

        // Also walk the ORIGINAL import path's aliases (version-pin, etc.)
        // so that a remapping that points at a non-existent location can fall
        // back to alias-based resolution. Concrete repro: Chainlink ships a
        // remappings.txt with `@openzeppelin/contracts@4.8.3=node_modules/
        // @openzeppelin/contracts-4.8.3`, which is correct for a foundry
        // project but resolves to a non-existent path when chainlink itself is
        // installed under `node_modules/@chainlink/contracts/` and the OZ
        // packages are hoisted up to the parent `node_modules/`. Falling
        // through to `@openzeppelin/contracts-4.8.3/...` (a top-level
        // include-path candidate) recovers the hoisted layout.
        let mut alias_sources: Vec<String> = import_aliases(&working_import).into_iter().collect();
        if matched_remapping.is_some() {
            for alias in import_aliases(import_path) {
                if !alias_sources.contains(&alias) {
                    alias_sources.push(alias);
                }
            }
        }
        for candidate_import in alias_sources {
            let import = Path::new(&candidate_import);
            if import.is_absolute() {
                candidates.push(import.to_path_buf());
                continue;
            }

            let is_relative_import =
                candidate_import.starts_with("./") || candidate_import.starts_with("../");

            if is_relative_import {
                // Standard relative-import resolution: resolve against the
                // importing file's own directory.
                candidates.push(from_dir.join(import));

                // Cross-package relative-import fallback (the "virtual path"
                // resolution). When a vendored OpenZeppelin file at
                // `vendor/@openzeppelin/contracts/token/ERC20/ERC20.sol` does
                // `import "./IERC20.sol"`, the relative path resolves against
                // the file's *physical* location — but if that file isn't
                // vendored, we should also try the equivalent relative path
                // against alternate copies of the package layout that the user
                // exposed via `--include-path` (e.g. an installed
                // `node_modules/@openzeppelin/contracts/...`).
                //
                // We compute the importing file's *virtual* directory by
                // finding the longest suffix of its path that lives under
                // some include path, then we ask the resolver to try the
                // import relative to that same virtual directory under every
                // other include path.
                for inc in include_paths {
                    let Some(virtual_dir) = virtual_dir_under(from_dir, inc) else {
                        continue;
                    };
                    for alt in include_paths {
                        candidates.push(alt.join(&virtual_dir).join(import));
                    }
                }
            } else {
                // Bare imports (`@scope/package/...` or `lib/file.sol`):
                // search each include path directly. Auto-discovered
                // `node_modules` directories are already part of
                // `include_paths`.
                for include_dir in include_paths {
                    candidates.push(include_dir.join(import));
                }
                candidates.push(from_dir.join(import));
                candidates.push(import.to_path_buf());
            }
        }

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate.canonicalize().unwrap_or(candidate));
            }
        }

        Err(format!(
            "failed to resolve import '{import_path}' from '{}'",
            from_file.display()
        ))
    }

    fn visit_file(
        file: &Path,
        include_paths: &[PathBuf],
        remappings: &[ImportRemapping],
        visited: &mut HashSet<PathBuf>,
        visiting: &mut HashSet<PathBuf>,
        ordered: &mut Vec<(PathBuf, String)>,
        stack: &mut VecDeque<PathBuf>,
    ) -> Result<(), String> {
        let canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());

        if visited.contains(&canonical) {
            return Ok(());
        }

        if !visiting.insert(canonical.clone()) {
            // Solidity allows cyclic import graphs as long as symbols resolve.
            // Skip this back-edge and let the already-in-progress unit finish.
            return Ok(());
        }

        stack.push_back(canonical.clone());
        let content = fs::read_to_string(&canonical)
            .map_err(|err| format!("failed to read '{}': {err}", canonical.display()))?;

        let imports = extract_imports(&content, &canonical)?;
        for import in imports {
            let resolved = resolve_import_file(&import, &canonical, include_paths, remappings)?;
            visit_file(
                &resolved,
                include_paths,
                remappings,
                visited,
                visiting,
                ordered,
                stack,
            )?;
        }

        stack.pop_back();
        visiting.remove(&canonical);
        visited.insert(canonical.clone());
        ordered.push((canonical, content));
        Ok(())
    }

    visit_file(
        entry_file,
        &effective_include_paths,
        &effective_remappings,
        &mut visited,
        &mut visiting,
        &mut ordered,
        &mut stack,
    )?;

    let mut combined = String::new();
    for (idx, (path, content)) in ordered.iter().enumerate() {
        if idx > 0 {
            combined.push_str("\n\n");
        }
        combined.push_str(&format!("// --- {}\n", path.display()));
        combined.push_str(content);
    }

    Ok(ResolvedSoliditySources {
        files: ordered,
        combined_source: combined,
    })
}

/// Compute the directory of `from_dir` *as it appears under* `root` — i.e. the
/// relative path that `from_dir` would have when re-rooted at `root`. Returns
/// `None` if `from_dir` is not a descendant of `root`. Used for cross-package
/// relative import resolution: a relative `./Foo.sol` from a vendored copy of
/// `@openzeppelin/contracts/utils/Bar.sol` can be re-rooted under an installed
/// `node_modules/@openzeppelin/contracts/utils/Foo.sol`.
pub(crate) fn virtual_dir_under(from_dir: &Path, root: &Path) -> Option<PathBuf> {
    let canonical_from = from_dir.canonicalize().ok()?;
    let canonical_root = root.canonicalize().ok()?;
    canonical_from
        .strip_prefix(&canonical_root)
        .ok()
        .map(std::path::Path::to_path_buf)
}

/// Walk up from `start` looking for `node_modules/` directories. Each one
/// found is appended to `out` if not already present. The walk stops at the
/// filesystem root or after a small bounded number of steps so we never scan
/// the whole machine. This mirrors solc / hardhat / foundry's
/// "search-up-for-node_modules" behaviour for npm-style imports.
pub(crate) fn extend_with_auto_node_modules(start: &Path, out: &mut Vec<PathBuf>) {
    let initial = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start.to_path_buf())
    };

    let Some(mut cursor) = initial.and_then(|p| p.canonicalize().ok()) else {
        return;
    };

    for _ in 0..MAX_CLIMB {
        let candidate = cursor.join("node_modules");
        if candidate.is_dir() && !out.contains(&candidate) {
            out.push(candidate);
        }
        match cursor.parent() {
            Some(parent) => cursor = parent.to_path_buf(),
            None => break,
        }
    }
}

/// Walk up from `start` looking for `remappings.txt` files; parse each one
/// and rebase its entries against the directory that owns the file (so
/// relative `path` values resolve against the project root, not the cwd).
/// Appends discovered remappings to `out` in walk order (closest-first).
/// Bounded the same way `extend_with_auto_node_modules` is.
///
/// Also scans `node_modules/<pkg>/remappings.txt` for foundry-style
/// package-internal remappings. Uniswap V4 periphery ships its
/// `permit2/=lib/v4-core/lib/permit2/...` mapping inside the published
/// package, so `import "permit2/src/..."` from a vendored v4-periphery
/// contract only resolves once that file is loaded.
pub(crate) fn extend_with_auto_remappings(start: &Path, out: &mut Vec<ImportRemapping>) {
    let initial = if start.is_file() {
        start.parent().map(Path::to_path_buf)
    } else {
        Some(start.to_path_buf())
    };
    let Some(start_dir) = initial.and_then(|p| p.canonicalize().ok()) else {
        return;
    };
    let mut seen: HashSet<PathBuf> = HashSet::new();

    fn try_load(dir: &Path, out: &mut Vec<ImportRemapping>, seen: &mut HashSet<PathBuf>) {
        let candidate = dir.join("remappings.txt");
        if candidate.is_file() && seen.insert(candidate.clone()) {
            if let Ok(loaded) = load_remappings_file(&candidate) {
                let base_dir = dir.to_path_buf();
                for mut remap in loaded {
                    if remap.replacement.is_relative() {
                        remap.replacement = base_dir.join(&remap.replacement);
                    }
                    // Don't shadow user-supplied remappings (which are
                    // already in `out`); auto-discovery is a fallback.
                    if !out.iter().any(|existing| existing.prefix == remap.prefix) {
                        out.push(remap);
                    }
                }
            }
        }
    }

    // Phase 1: classic ancestor walk.
    let mut cursor = start_dir.clone();
    for _ in 0..MAX_CLIMB {
        try_load(&cursor, out, &mut seen);
        match cursor.parent() {
            Some(parent) => cursor = parent.to_path_buf(),
            None => break,
        }
    }

    // Phase 2: scan every `node_modules` directory reachable from `start`
    // for packages that ship their own foundry-style `remappings.txt`.
    let mut cursor = start_dir;
    for _ in 0..MAX_CLIMB {
        let node_modules = cursor.join("node_modules");
        if node_modules.is_dir() {
            scan_node_modules_for_remappings(&node_modules, out, &mut seen);
        }
        match cursor.parent() {
            Some(parent) => cursor = parent.to_path_buf(),
            None => break,
        }
    }
}

/// For each top-level package in `node_modules/` (including scoped
/// `@scope/pkg`), check for a package-internal `remappings.txt` and load it.
/// Also auto-register `lib/<dep>/=lib/<dep>/src/` remappings for any package
/// that vendors its dependencies under `lib/` (foundry layout, used by every
/// Uniswap V4 / forge-installed contract). Bounded by the on-disk fanout —
/// most packages don't ship one, so this is effectively one `metadata()` per
/// published package.
pub(crate) fn scan_node_modules_for_remappings(
    node_modules: &Path,
    out: &mut Vec<ImportRemapping>,
    seen: &mut HashSet<PathBuf>,
) {
    fn auto_register_foundry_lib(pkg_dir: &Path, out: &mut Vec<ImportRemapping>) {
        let lib_dir = pkg_dir.join("lib");
        let Ok(lib_entries) = std::fs::read_dir(&lib_dir) else {
            return;
        };
        for lib_entry in lib_entries.flatten() {
            let lib_path = lib_entry.path();
            if !lib_path.is_dir() {
                continue;
            }
            let Some(dep_name) = lib_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            // Foundry convention: dependencies live at `lib/<name>/` and
            // imports include the `src/` prefix explicitly. Mapping
            // `<name>/=lib/<name>/` lets `import "permit2/src/..."` resolve
            // to `lib/permit2/src/...`. We do NOT append `src/` to the
            // replacement — that's what real foundry users do via
            // `<name>/src/=lib/<name>/src/`, but the bare-prefix form is the
            // dominant convention.
            let prefix = format!("{dep_name}/");
            if !out.iter().any(|existing| existing.prefix == prefix) {
                out.push(ImportRemapping {
                    prefix,
                    replacement: lib_path.clone(),
                });
            }
        }
    }
    fn try_load(dir: &Path, out: &mut Vec<ImportRemapping>, seen: &mut HashSet<PathBuf>) {
        let candidate = dir.join("remappings.txt");
        if candidate.is_file() && seen.insert(candidate.clone()) {
            if let Ok(loaded) = load_remappings_file(&candidate) {
                let base_dir = dir.to_path_buf();
                for mut remap in loaded {
                    if remap.replacement.is_relative() {
                        remap.replacement = base_dir.join(&remap.replacement);
                    }
                    if !out.iter().any(|existing| existing.prefix == remap.prefix) {
                        out.push(remap);
                    }
                }
            }
        }
    }

    let Ok(entries) = std::fs::read_dir(node_modules) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('@') {
            // Scoped packages: scan one level deeper.
            let Ok(scoped) = std::fs::read_dir(&path) else {
                continue;
            };
            for scoped_entry in scoped.flatten() {
                let scoped_path = scoped_entry.path();
                if scoped_path.is_dir() {
                    try_load(&scoped_path, out, seen);
                    auto_register_foundry_lib(&scoped_path, out);
                }
            }
        } else if path.is_dir() {
            try_load(&path, out, seen);
            auto_register_foundry_lib(&path, out);
        }
    }
}

/// Return every alternate path we should try for a Foundry inline-version-pin
/// import. Given `@openzeppelin/contracts@4.8.3/IERC20.sol`, returns:
///   1. `@openzeppelin/contracts-4.8.3/IERC20.sol` — npm's
///      `@openzeppelin/contracts-X.Y.Z` aliased-package convention.
///   2. `@openzeppelin/contracts/IERC20.sol` — the unpinned form (whichever
///      version is installed).
///
/// Returns an empty `Vec` when the path doesn't carry an inline pin.
pub(crate) fn version_pin_aliases(import_path: &str) -> Vec<String> {
    let head_and_tail = import_path.splitn(2, '/');
    let mut iter = head_and_tail;
    let Some(head) = iter.next() else {
        return Vec::new();
    };
    let Some(tail) = iter.next() else {
        return Vec::new();
    };
    let (scope_prefix, name_and_rest) = if head.starts_with('@') {
        (Some(head.to_string()), tail.to_string())
    } else {
        (None, import_path.to_string())
    };
    let Some((pkg_name, rest)) = name_and_rest.split_once('/') else {
        return Vec::new();
    };
    let Some((pkg, version)) = pkg_name.split_once('@') else {
        return Vec::new();
    };
    let looks_like_version = version
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || (c == 'v' && version.len() > 1));
    if !looks_like_version {
        return Vec::new();
    }
    let mut out = Vec::new();
    // (a) Dash-separated form: `@scope/pkg-version/rest` — matches npm
    // aliased packages like `@openzeppelin/contracts-4.8.3` that ship the
    // full package tree at a parallel directory.
    let dashed = match &scope_prefix {
        Some(scope) => format!("{scope}/{pkg}-{version}/{rest}"),
        None => format!("{pkg}-{version}/{rest}"),
    };
    out.push(dashed);
    // (b) Unpinned form: drop the version chunk entirely. Use this as the
    // last-resort fallback when the version-pinned directory isn't present.
    let unpinned = match scope_prefix {
        Some(scope) => format!("{scope}/{pkg}/{rest}"),
        None => format!("{pkg}/{rest}"),
    };
    out.push(unpinned);
    out
}

/// Every alternate import path we should try for `import_path`, in order.
///
/// This is the single source of truth for import-alias generation and MUST be
/// used by both the on-disk CLI resolver (`resolve_import_file`) and the
/// standard-JSON in-memory resolver
/// (`standard_json_process::imports::resolve_import_key`). Previously those two
/// carried private copies of this logic that had silently drifted: only the CLI
/// copy applied the Foundry `version_pin_aliases` expansion, so an identical
/// Foundry version-pinned project (`@scope/pkg@1.2.3/path/File.sol`) would
/// resolve via the CLI but fail import resolution via `--standard-json`.
/// Sharing this function prevents that class of divergence.
pub(crate) fn import_aliases(import_path: &str) -> Vec<String> {
    let mut aliases = vec![import_path.to_string()];

    // Foundry-style prefix aliases (the package was published to npm
    // under `@openzeppelin/...` but is sometimes vendored or imported
    // under its bare package name).
    if let Some(rest) = import_path.strip_prefix("openzeppelin-contracts/contracts/") {
        aliases.push(format!("@openzeppelin/contracts/{rest}"));
    } else if let Some(rest) =
        import_path.strip_prefix("openzeppelin-contracts-upgradeable/contracts/")
    {
        aliases.push(format!("@openzeppelin/contracts-upgradeable/{rest}"));
    } else if let Some(rest) = import_path.strip_prefix("openzeppelin-contracts/") {
        aliases.push(format!("@openzeppelin/contracts/{rest}"));
    } else if let Some(rest) = import_path.strip_prefix("openzeppelin-contracts-upgradeable/") {
        aliases.push(format!("@openzeppelin/contracts-upgradeable/{rest}"));
    }

    // Foundry pinned-version syntax: `@scope/pkg@x.y.z/path/to/File.sol`
    // is shorthand for "use exactly this version" — equivalent to a
    // remapping but inlined into the import. When the version-pinned
    // directory isn't available, fall back to (a) the dash-separated form
    // `@scope/pkg-x.y.z/...` and (b) the unpinned form `@scope/pkg/...`.
    for alias in version_pin_aliases(import_path) {
        if alias != import_path {
            aliases.push(alias);
        }
    }

    aliases
}

/// Parse a Foundry-style `prefix=path` remapping string. Returns `Err` if the
/// input doesn't contain an `=` separator or the prefix is empty. The
/// replacement path may be relative (resolved against the current working
/// directory when used) or absolute.
pub(crate) fn parse_remapping(spec: &str) -> Result<ImportRemapping, String> {
    let (prefix, replacement) = spec
        .split_once('=')
        .ok_or_else(|| format!("invalid remapping '{spec}': expected `prefix=path`"))?;
    if prefix.is_empty() {
        return Err(format!("invalid remapping '{spec}': prefix is empty"));
    }
    Ok(ImportRemapping {
        prefix: prefix.to_string(),
        replacement: PathBuf::from(replacement),
    })
}

/// Parse a Foundry-style remappings file (one `prefix=path` per line). Blank
/// lines and lines whose first non-whitespace character is `#` are ignored.
/// Errors out the first time a line is malformed so the user gets a precise
/// line number in the diagnostic.
pub(crate) fn load_remappings_file(path: &Path) -> Result<Vec<ImportRemapping>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("failed to read remappings file '{}': {err}", path.display()))?;
    let mut out = Vec::new();
    for (line_no, raw) in contents.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match parse_remapping(trimmed) {
            Ok(remap) => out.push(remap),
            Err(err) => {
                return Err(format!("{}:{}: {err}", path.display(), line_no + 1));
            }
        }
    }
    Ok(out)
}
