use super::*;

/// Stack budget for the dedicated parsing thread used by
/// [`parse_solidity_guarded`].
///
/// `solang_parser`'s recursive-descent grammar recurses once per nesting
/// level of the input, so a hostile (or generated) source such as
/// `return 1 +1 +1 ...` a few thousand terms deep overflows the default
/// ~8 MiB main-thread stack and aborts the whole process (SIGABRT) before
/// any error can be reported. 256 MiB raises the depth ceiling by ~32x —
/// far beyond anything a legitimate contract reaches — while staying a
/// hard bound (the memory is only reserved, not committed, until used).
const PARSE_THREAD_STACK_BYTES: usize = 256 * 1024 * 1024;

/// Run `solang_parser::parse(source, 0)` on a worker thread with a large
/// bounded stack, joining failures back as ordinary parse diagnostics.
///
/// The compiler's own recursive AST walks are already `stacker`-guarded
/// (see `src/ir/expressions/dispatch/entry.rs`), but the parser entry point
/// runs *before* those guards and `solang_parser` recurses on its own
/// internal stack. Every call site that parses Solidity source MUST go
/// through this helper — CLI single-file (`extract_imports`), standard-json
/// import resolution, and `parse_source` — so the nesting-depth ceiling
/// stays consistent across entry points.
///
/// A parser panic (rather than a returned `Err`) is converted into a normal
/// error diagnostic instead of unwinding into — or aborting — the caller.
pub fn parse_solidity_guarded(
    source: &str,
) -> Result<(solang_parser::pt::SourceUnit, Vec<Comment>), Vec<Diagnostic>> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("solang-parse".to_string())
            .stack_size(PARSE_THREAD_STACK_BYTES)
            .spawn_scoped(scope, || parse(source, 0));

        match worker {
            Ok(handle) => handle.join().unwrap_or_else(|panic| {
                let detail = panic
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown parser panic".to_string());
                Err(vec![Diagnostic::parser_error(
                    Loc::File(0, 0, 0),
                    format!("internal parser error: {detail}"),
                )])
            }),
            // Spawning a thread only fails under extreme resource
            // exhaustion; fall back to parsing on the current thread
            // (the pre-helper behavior) rather than refusing to compile.
            Err(_) => parse(source, 0),
        }
    })
}
