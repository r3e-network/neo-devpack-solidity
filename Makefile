# Neo DevPack for Solidity - Professional Build System
# Author: Jimmy <jimmy@r3e.network>
# Repository: https://github.com/r3e-network/neo-devpack-solidity

.PHONY: all build clean test test-all test-all-full test-fuzz-gate test-deploy-smoke test-deploy-callt-smoke test-deploy-constructor-smoke test-deploy-update-smoke test-deploy-permissions-smoke test-deploy-encoding-smoke test-deploy-abidecode-smoke test-deploy-typestrict-smoke test-deploy-abortmsg-smoke test-deploy-trycatch-smoke test-deploy-bytesn-smoke test-deploy-arith-smoke test-deploy-lowlevel-call-smoke test-deploy-lowlevel-call-failure-smoke test-deploy-external-call-smoke test-deploy-view-readonly-call-smoke test-deploy-compound-assignment-smoke test-deploy-struct-array-element-smoke test-deploy-struct-param-smoke test-deploy-nested-struct-smoke test-deploy-delete-smoke test-deploy-new-showcases-smoke test-deploy-evm-compat-smoke test-deploy-smoke-full test-deploy-wgas-smoke test-deploy-flashloan-smoke test-deploy-amm-smoke test-deploy-vesting-smoke test-deploy-lending-smoke test-deploy-dao-smoke test-deploy-famous-all docs docs-api docs-site install format lint release help install-deps tooling-install tooling-build tooling-test tooling-lint runtime-build runtime-test coverage coverage-ci check-coverage
.PHONY: test-compile-strict production-gate test-fuzz-start test-fuzz-status test-fuzz-stop test-fuzz-cargo-all test-fuzz-differential test-fuzz-coverage

all: build

build:
	@echo "🔨 Building Neo DevPack for Solidity..."
	cargo build --release
	@echo "✅ Build complete"

test:
	@echo "🧪 Running tests..."
	cargo test --workspace
	@echo "✅ Tests passed"

test-all: test tooling-test tooling-lint
	@echo "✅ Core test suites complete"

test-all-full: test tooling-test runtime-test
	@echo "✅ Full test suites complete"

test-fuzz-gate:
	@echo "🧪 Running fuzz/compiler verification gate..."
	cargo test --test fuzz_tests

test-fuzz-continuous:
	@echo "🧪 Running continuous fuzz suite with deep case counts..."
	@./scripts/run_fuzz_suite.sh deep

test-fuzz-cargo-all:
	@echo "🧪 Running every cargo-fuzz target (60s each)..."
	@for t in $$(cargo +nightly fuzz list); do \
		echo "== $$t =="; \
		cargo +nightly fuzz run $$t -- -max_total_time=60 || exit 1; \
	done
	@echo "✅ All cargo-fuzz targets clean"

test-fuzz-differential:
	@echo "🧪 Running differential proptests (compiler vs reference crates)..."
	cargo test --test fuzz_tests differential

test-fuzz-coverage:
	@echo "🧪 Generating per-test coverage for the fuzz suite..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov not installed; run: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	cargo llvm-cov --test fuzz_tests --html
	@echo "✅ HTML report written to target/llvm-cov/html/index.html"

test-fuzz-start:
	@echo "🧪 Starting background fuzz run..."
	bash scripts/fuzz_start.sh

test-fuzz-status:
	@echo "🧪 Checking background fuzz run status..."
	bash scripts/fuzz_status.sh

test-fuzz-stop:
	@echo "🧪 Stopping background fuzz run..."
	bash scripts/fuzz_stop.sh

test-fuzz-ci:
	@echo "🧪 Running CI fuzz suite..."
	@./scripts/run_fuzz_suite.sh ci
	cargo test --test e2e_compilation_tests -- --test-threads=4
	bash examples/test_strict_compatibility_sweep.sh
	@echo "✅ Fuzz/compiler verification gate passed"

test-compile-strict:
	@echo "🧭 Running strict compatibility compilation sweep..."
	bash examples/test_strict_compatibility_sweep.sh

production-gate:
	@echo "🛡️  Running production readiness gate..."
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	node --check scripts/github_contracts_pipeline.js
	cargo build --release
	cargo test --workspace --all-features
	$(MAKE) tooling-test
	$(MAKE) tooling-lint
	$(MAKE) runtime-test
	cargo test --test e2e_compilation_tests -- --test-threads=4
	STRICT_SWEEP_FAIL_ON_UNEXPECTED_WARNINGS=1 $(MAKE) test-compile-strict
	$(MAKE) test-deploy-smoke-full
	@echo "✅ Production readiness gate passed"

test-deploy-smoke:
	@echo "🚀 Running Neo-Express deploy smoke test..."
	bash examples/test_neoxp_deploy.sh

test-deploy-callt-smoke:
	@echo "🪙 Running Neo-Express CALLT smoke test..."
	bash examples/test_neoxp_callt_smoke.sh

test-deploy-constructor-smoke:
	@echo "🏗️  Running Neo-Express constructor deploy smoke test..."
	bash examples/test_neoxp_constructor_smoke.sh

test-deploy-update-smoke:
	@echo "🔁 Running Neo-Express contract update smoke test..."
	bash examples/test_neoxp_update_smoke.sh

test-deploy-permissions-smoke:
	@echo "🔐 Running Neo-Express manifest permissions smoke test..."
	bash examples/test_neoxp_permissions_smoke.sh

test-deploy-encoding-smoke:
	@echo "🧩 Running Neo-Express abi.encode/abi.decode smoke test..."
	bash examples/test_neoxp_encoding_smoke.sh

test-deploy-abidecode-smoke:
	@echo "🔁 Running Neo-Express abi.decode type-fidelity smoke test..."
	bash examples/test_neoxp_abidecode_smoke.sh

test-deploy-typestrict-smoke:
	@echo "🧮 Running Neo-Express NeoVM type-strictness smoke test..."
	bash examples/test_neoxp_typestrict_smoke.sh

test-deploy-abortmsg-smoke:
	@echo "💥 Running Neo-Express revert reason smoke test..."
	bash examples/test_neoxp_abortmsg_smoke.sh

test-deploy-trycatch-smoke:
	@echo "🪤 Running Neo-Express try/catch (Panic/Error match + frame balance) smoke test..."
	bash examples/test_neoxp_trycatch_smoke.sh

test-deploy-bytesn-smoke:
	@echo "🔤 Running Neo-Express bytesN (encoding + return canonicalization) smoke test..."
	bash examples/test_neoxp_bytesn_smoke.sh

test-deploy-arith-smoke:
	@echo "🧮 Running Neo-Express wide-integer arithmetic smoke test..."
	bash examples/test_neoxp_arith_smoke.sh

test-deploy-lowlevel-call-smoke:
	@echo "📞 Running Neo-Express low-level call smoke test..."
	bash examples/test_neoxp_lowlevel_call_smoke.sh

test-deploy-lowlevel-call-failure-smoke:
	@echo "📞 Running Neo-Express low-level call failure smoke test..."
	bash examples/test_neoxp_lowlevel_call_failure_smoke.sh

test-deploy-external-call-smoke:
	@echo "🔗 Running Neo-Express external member-call smoke test..."
	bash examples/test_neoxp_external_member_call_smoke.sh

test-deploy-view-readonly-call-smoke:
	@echo "🔒 Running Neo-Express view ReadOnly external-call smoke test..."
	bash examples/test_neoxp_view_readonly_call_smoke.sh

test-deploy-compound-assignment-smoke:
	@echo "🧮 Running Neo-Express compound assignment smoke test..."
	bash examples/test_neoxp_compound_assignment_smoke.sh

test-deploy-struct-array-element-smoke:
	@echo "📦 Running Neo-Express struct-array element smoke test..."
	bash examples/test_neoxp_struct_array_element_smoke.sh

test-deploy-struct-param-smoke:
	@echo "🧱 Running Neo-Express struct-parameter smoke test..."
	bash examples/test_neoxp_struct_param_smoke.sh

test-deploy-nested-struct-smoke:
	@echo "🧬 Running Neo-Express nested struct storage smoke test..."
	bash examples/test_neoxp_nested_struct_storage_smoke.sh

test-deploy-delete-smoke:
	@echo "🗑️  Running Neo-Express delete smoke test..."
	bash examples/test_neoxp_delete_smoke.sh

test-deploy-new-showcases-smoke:
	@echo "🧪 Running Neo-Express new showcase smoke suite..."
	bash examples/test_neoxp_new_showcases_smoke.sh

test-deploy-evm-compat-smoke:
	@echo "🌉 Running EVM compatibility adapter smoke test..."
	bash examples/test_neoxp_evm_compat_smoke.sh

test-deploy-smoke-full: test-deploy-smoke test-deploy-callt-smoke test-deploy-constructor-smoke test-deploy-update-smoke test-deploy-permissions-smoke test-deploy-encoding-smoke test-deploy-abidecode-smoke test-deploy-typestrict-smoke test-deploy-abortmsg-smoke test-deploy-trycatch-smoke test-deploy-bytesn-smoke test-deploy-arith-smoke test-deploy-lowlevel-call-smoke test-deploy-lowlevel-call-failure-smoke test-deploy-external-call-smoke test-deploy-view-readonly-call-smoke test-deploy-compound-assignment-smoke test-deploy-struct-array-element-smoke test-deploy-struct-param-smoke test-deploy-nested-struct-smoke test-deploy-delete-smoke test-deploy-new-showcases-smoke test-deploy-evm-compat-smoke
	@echo "✅ Neo-Express smoke tests complete"

# ========== Famous DeFi/Web3 contract smoke tests ==========

test-deploy-wgas-smoke:
	@echo "💰 Running WGAS (Wrapped GAS) smoke test..."
	bash examples/test_neoxp_wgas_smoke.sh

test-deploy-flashloan-smoke:
	@echo "⚡ Running FlashLoan smoke test..."
	bash examples/test_neoxp_flashloan_smoke.sh

test-deploy-amm-smoke:
	@echo "🔄 Running SimpleAMM smoke test..."
	bash examples/test_neoxp_amm_smoke.sh

test-deploy-vesting-smoke:
	@echo "⏳ Running TokenVesting smoke test..."
	bash examples/test_neoxp_vesting_smoke.sh

test-deploy-lending-smoke:
	@echo "🏦 Running SimpleLending smoke test..."
	bash examples/test_neoxp_lending_smoke.sh

test-deploy-dao-smoke:
	@echo "🗳️  Running SimpleDAO smoke test..."
	bash examples/test_neoxp_dao_smoke.sh

test-deploy-famous-all: test-deploy-wgas-smoke test-deploy-flashloan-smoke test-deploy-amm-smoke test-deploy-vesting-smoke test-deploy-lending-smoke test-deploy-dao-smoke
	@echo "✅ Famous contract smoke tests complete"

install-deps: tooling-install
	@echo "📦 Fetching Rust dependencies..."
	cargo fetch
	@echo "✅ Dependencies installed"

tooling-install:
	@echo "📦 Installing tooling dependencies..."
	npm --prefix tooling install
	@echo "✅ Tooling dependencies installed"

tooling-build: tooling-install
	@echo "🔨 Building tooling packages..."
	npm --prefix tooling run build
	@echo "✅ Tooling build complete"

tooling-test: tooling-install
	@echo "🧪 Running tooling tests..."
	npm --prefix tooling test
	@echo "✅ Tooling tests passed"

tooling-lint: tooling-install
	@echo "🔍 Linting tooling packages..."
	npm --prefix tooling run lint
	@echo "✅ Tooling lint passed"

runtime-build:
	@echo "🔨 Building C# runtime (optional)..."
	@if command -v dotnet >/dev/null 2>&1; then \
		dotnet build src/Neo.Sol.Runtime/Neo.Sol.Runtime.csproj --configuration Release; \
	else \
		echo "⚠️  dotnet not found; skipping runtime build"; \
	fi

runtime-test:
	@echo "🧪 Running C# runtime tests (optional)..."
	@if command -v dotnet >/dev/null 2>&1; then \
		dotnet test tests/Neo.Sol.Runtime.Tests/Neo.Sol.Runtime.Tests.csproj --configuration Release; \
	else \
		echo "⚠️  dotnet not found; skipping runtime tests"; \
	fi

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"

format:
	@echo "🎨 Formatting code..."
	cargo fmt
	@echo "✅ Code formatted"

lint:
	@echo "🔍 Linting code..."
	cargo clippy -- -D warnings
	@echo "✅ Linting passed"

coverage:
	@echo "📊 Running code coverage..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --out Html --output-dir coverage; \
		echo "✅ Coverage report generated in coverage/"; \
	else \
		echo "⚠️  cargo-tarpaulin not installed. Install with:"; \
		echo "   cargo install cargo-tarpaulin"; \
	fi

coverage-ci:
	@echo "📊 Running code coverage for CI..."
	cargo tarpaulin --out Lcov --output-dir coverage --ignore-panics

check-coverage:
	@echo "📊 Checking minimum coverage threshold..."
	@if command -v cargo-tarpaulin >/dev/null 2>&1; then \
		cargo tarpaulin --out Text --threshold 70; \
	else \
		echo "⚠️  cargo-tarpaulin not installed. Install with:"; \
		echo "   cargo install cargo-tarpaulin"; \
	fi

install: build
	@echo "📦 Installing neo-solc..."
	cargo install --path .
	@echo "✅ neo-solc installed successfully"

docs:
	@echo "📚 Building documentation..."
	cargo doc --no-deps
	@echo "✅ Documentation built"

docs-api:
	@echo "📚 Building API documentation payload..."
	cargo doc --no-deps --document-private-items
	rm -rf docs/public/api/rust
	mkdir -p docs/public/api/rust
	cp -r target/doc/* docs/public/api/rust/ || true
	bash scripts/generate_ts_docs.sh
	bash scripts/generate_cs_docs.sh
	@echo "✅ API documentation payload built"

docs-site: docs-api
	@echo "🌐 Building VitePress documentation site..."
	npm install
	npm run docs:build
	@echo "✅ Documentation site built at docs/.vitepress/dist"

release: clean build test
	@echo "🚀 Creating release..."
	cargo package
	@echo "✅ Release ready"

help:
	@echo "Neo DevPack for Solidity - Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build the compiler"
	@echo "  test            - Run all tests"
	@echo "  production-gate - Run release readiness gates"
	@echo "  clean           - Clean build artifacts"
	@echo "  format          - Format code"
	@echo "  lint            - Lint code"
	@echo "  coverage        - Generate HTML coverage report"
	@echo "  coverage-ci     - Generate LCOV coverage for CI"
	@echo "  check-coverage  - Check minimum coverage threshold (70%)"
	@echo "  docs            - Build Rust API docs only (cargo doc)"
	@echo "  docs-api        - Build Rust/TypeScript/C# API docs payload in docs/public/api"
	@echo "  docs-site       - Build full VitePress docs site"
	@echo "  install         - Install neo-solc binary"
	@echo "  release         - Create release package"
	@echo "  help            - Show this help message"
	@echo ""
	@echo "Repository: https://github.com/r3e-network/neo-devpack-solidity"
	@echo ""
	@echo "Testing targets:"
	@echo "  test-deploy-smoke      - Basic deployment test"
	@echo "  test-deploy-new-showcases-smoke - New strict-safe showcase smoke suite"
	@echo "  test-deploy-evm-compat-smoke - EVM compatibility adapter smoke test"
	@echo "  test-deploy-smoke-full - All deployment smoke tests"
	@echo "  test-compile-strict - Strict compile sweep (devpack + examples/new)"
	@echo "  production-gate - Full production readiness verification"
	@echo ""
	@echo "Tooling targets:"
	@echo "  tooling-install  - Install Node.js tooling"
	@echo "  tooling-build    - Build tooling packages"
	@echo "  tooling-test     - Run tooling tests"
	@echo "  tooling-lint     - Lint tooling code"
