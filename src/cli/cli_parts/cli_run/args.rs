use super::*;
use clap::{Arg, ArgAction, Command};

pub(crate) fn build_matches() -> clap::ArgMatches {
    build_cli_command().get_matches()
}

pub(crate) fn build_cli_command() -> Command {
    Command::new("neo-solc")
        .version(env!("CARGO_PKG_VERSION"))
        .author(COMPILER_EMAIL)
        .about("Compiles Solidity to Neo N3 smart contracts (.nef + .manifest.json)")
        .arg(
            Arg::new("source")
                .help("Input Solidity file(s)")
                .required_unless_present("standard-json")
                .num_args(1..)
                .index(1),
        )
        .arg(
            Arg::new("standard-json")
                .long("standard-json")
                .help("Use Solidity standard JSON input/output mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("standard-json-input")
                .long("input")
                .value_name("FILE")
                .help("Path to standard JSON input file (defaults to stdin when omitted)")
                .num_args(1)
                .requires("standard-json"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output prefix or directory (normal mode), or JSON output file (standard-json mode; defaults to stdout)")
                .num_args(1),
        )
        .arg(
            Arg::new("optimize")
                .short('O')
                .long("optimize")
                .value_name("LEVEL")
                .help("Optimization level (0-3)")
                .num_args(1)
                .default_value("2"),
        )
        .arg(
            Arg::new("callt")
                .long("callt")
                .help("Emit CALLT + method tokens for native contract calls (Neo N3 optimization)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("analyze")
                .long("analyze")
                .help(
                    "Analyze EVM-to-Neo migration issues and emit a JSON upgrade report instead of writing NEF/manifest artifacts",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("deny-wildcard-permissions")
                .long("deny-wildcard-permissions")
                .help(
                    "Fail compilation if the manifest would require full wildcard permissions (contract='*', methods='*')",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("deny-wildcard-contracts")
                .long("deny-wildcard-contracts")
                .help("Fail compilation if the manifest would require wildcard contract permissions (contract='*')")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("deny-wildcard-methods")
                .long("deny-wildcard-methods")
                .help("Fail compilation if the manifest would require wildcard method permissions (methods='*')")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("manifest-permissions")
                .long("manifest-permissions")
                .value_name("FILE")
                .help("Path to a JSON file containing manifest permissions (array or {\"permissions\": [...]})")
                .num_args(1),
        )
        .arg(
            Arg::new("manifest-permissions-mode")
                .long("manifest-permissions-mode")
                .value_name("MODE")
                .help("How to apply --manifest-permissions: 'merge' (default) or 'replace-wildcards'")
                .num_args(1)
                .value_parser(["merge", "replace-wildcards"])
                .default_value("merge")
                .requires("manifest-permissions"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format")
                .num_args(1)
                .value_parser(["nef", "manifest", "complete", "assembly", "json"])
                .default_value("complete"),
        )
        .arg(
            Arg::new("contract")
                .long("contract")
                .value_name("NAME")
                .help("Only emit outputs for the specified contract name (repeatable)")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("nef-source")
                .long("nef-source")
                .value_name("STRING")
                .help("Value to embed in the NEF 'source' field (defaults to canonical input path)")
                .num_args(1),
        )
        .arg(
            Arg::new("deployer")
                .long("deployer")
                .value_name("HASH160")
                .help("Compute the predicted deployed contract hash for the given sender UInt160 (0x-prefixed big-endian hex)")
                .num_args(1),
        )
        .arg(
            Arg::new("include-path")
                .short('I')
                .long("include-path")
                .value_name("DIR")
                .help("Additional import search path (repeatable)")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("remap")
                .long("remap")
                .value_name("PREFIX=PATH")
                .help(
                    "Import remapping (Foundry/solc style; repeatable). Examples: `--remap @openzeppelin/=node_modules/@openzeppelin/` or `--remap forge-std/=lib/forge-std/src/`",
                )
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("remappings-file")
                .long("remappings")
                .value_name("FILE")
                .help(
                    "Read remappings from a file (one `prefix=path` per line; `#` starts a comment). Equivalent to Foundry's remappings.txt.",
                )
                .num_args(1),
        )
        .arg(
            Arg::new("json-errors")
                .long("json-errors")
                .help("Emit compiler errors as JSON lines on stderr")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("json-warnings")
                .long("json-warnings")
                .help("Emit compiler warnings as JSON lines on stderr")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("Wno")
                .long("Wno")
                .value_name("CODE")
                .help("Suppress warnings with the given code prefix (e.g. --Wno W101)")
                .num_args(1)
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("Werror")
                .long("Werror")
                .value_name("CODE")
                .help("Promote warnings with the given code prefix to errors (e.g. --Werror W101)")
                .num_args(1)
                .action(ArgAction::Append),
        )
}
