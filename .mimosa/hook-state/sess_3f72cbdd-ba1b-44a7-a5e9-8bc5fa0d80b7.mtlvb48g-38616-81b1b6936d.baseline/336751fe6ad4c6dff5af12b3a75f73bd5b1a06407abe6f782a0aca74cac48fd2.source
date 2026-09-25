use super::*;

/// Stack budget for the compiler driver thread spawned by [`run`]. Keep in
/// sync with `PARSE_THREAD_STACK_BYTES` in
/// src/frontend/frontend_guarded_parse.rs so every stage shares one ceiling.
pub(crate) const DRIVER_THREAD_STACK_BYTES: usize = 256 * 1024 * 1024;

pub fn run() {
    // The Solidity AST is deeply recursive. Parsing is guarded by
    // `frontend::parse_solidity_guarded` (own worker thread, large stack)
    // and the IR lowering walks are `stacker`-guarded, but other passes
    // still recurse over the full tree on the calling thread's stack —
    // e.g. the derive(Clone) walk in `analyse_all_sources` and the
    // recursive `Drop` of the parse tree at scope exit. Run the whole
    // driver on a worker thread with a large bounded stack so a deeply
    // nested (but otherwise valid) source can't abort the process with a
    // main-thread stack overflow.
    let worker = std::thread::Builder::new()
        .name("neo-solc-driver".to_string())
        .stack_size(DRIVER_THREAD_STACK_BYTES)
        .spawn(run_driver);

    match worker {
        Ok(handle) => {
            if let Err(panic) = handle.join() {
                std::panic::resume_unwind(panic);
            }
        }
        // Spawning a thread only fails under extreme resource exhaustion;
        // fall back to running on the current thread (pre-guard behavior)
        // rather than refusing to compile.
        Err(_) => run_driver(),
    }
}

pub(crate) fn run_driver() {
    let matches = build_matches();

    if try_run_standard_json(&matches) {
        return;
    }

    run_single_file(&matches);
}
