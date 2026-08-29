# Famous DeFi/Web3 Contracts — Neo N3 Ports

Six iconic DeFi/Web3 Solidity contracts ported to Neo N3, compiled to NEF+manifest,
and validated with smoke tests on Neo Express.

## Contracts

### 1. WGAS — Wrapped GAS (`WGAS.sol`)

**Inspired by:** WETH9 (Wrapped Ether) — the most deployed contract on Ethereum.

Wraps native GAS into a NEP-17-compatible token. Deposits arrive via `onNEP17Payment`,
minting 1:1 WGAS. Withdrawals burn WGAS and return GAS via `NativeCalls.gasTransfer()`.
Uses the standard NEP-17 `transfer(from, to, amount, data)` signature with `Runtime.checkWitness` authorization.

**Key features:** deposit, withdraw, transfer (NEP-17), approve, transferFrom.

**Current limitation (explicit):**
- `mapping`/dynamic array storage paths currently depend on `keccak256` runtime support in the Neo-Express smoke environment.
- This sample uses a single-holder ledger model (one active WGAS holder at a time) to keep deploy + smoke tests deterministic.
- Full WETH-style multi-holder behavior requires compiler/runtime support for keccak-backed storage key derivation.

### 2. FlashLoan (`FlashLoan.sol`)

**Inspired by:** Aave V2 Flash Loans.

Pool holds GAS from liquidity providers. `flashLoan()` transfers GAS to a borrower
contract, calls its `onFlashLoan()` callback, then verifies repayment + 0.09% fee.

**Key features:** deposit (via onNEP17Payment), withdraw, flashLoan, poolBalance.

### 3. SimpleAMM (`SimpleAMM.sol`)

**Inspired by:** Uniswap V2 constant-product AMM (simplified).

Two-token pool with `x*y=k` invariant. LP shares minted/burned proportionally.
0.3% swap fee. Spot price oracle included. Token pair set via `initialize()` after deploy.

**Key features:** initialize, addLiquidity, removeLiquidity, swap, getPrice.

### 4. TokenVesting (`TokenVesting.sol`)

**Inspired by:** OpenZeppelin VestingWallet / Sablier token streaming.

Linear vesting with cliff period. Beneficiary calls `release()` to claim vested GAS.
Owner can revoke unvested tokens. Deploys with sensible defaults; call `initialize()` to
customize beneficiary, start time, cliff, and duration before first release.

**Key features:** initialize, release, revoke, vestedAmount, releasableAmount.

### 5. SimpleLending (`SimpleLending.sol`)

**Inspired by:** Compound / Aave lending pool (simplified).

Single-asset GAS lending with 150% collateral ratio, per-block interest accrual,
and liquidation of undercollateralized positions.

**Key features:** deposit, withdraw, borrow, repay (via onNEP17Payment), liquidate.

### 6. SimpleDAO (`SimpleDAO.sol`)

**Inspired by:** Compound Governor / MolochDAO.

Stake GAS for voting power. Create proposals, vote, execute after timelock.
20% quorum required. Default voting period: 100 blocks, timelock delay: 10 blocks.
Uses `Syscalls.contractCall()` for Neo-native execution.

**Key features:** stake (via onNEP17Payment), unstake, propose, vote, execute, cancel.

## Neo N3 Adaptations

All contracts respect the neo-solc compiler constraints:

| EVM Pattern                | Neo N3 Replacement          |
| -------------------------- | --------------------------- |
| `{value: ...}`             | `NativeCalls.gasTransfer()` |
| `receive()` / `fallback()` | `onNEP17Payment()` callback |
| Inline assembly            | Library functions           |
| `address.call(bytes)`      | `Syscalls.contractCall()`   |

## Compile

```bash
# Single contract
target/release/neo-solc examples/famous/WGAS.sol -I devpack -O2 -o /tmp/WGAS

# All famous contracts
for f in examples/famous/*.sol; do
  target/release/neo-solc "$f" -I devpack -O2 -o "/tmp/$(basename "$f" .sol)"
done
```

## Test

```bash
# E2E compilation tests
cargo test famous

# Individual smoke test (requires neoxp)
make test-deploy-wgas-smoke

# All famous contract smoke tests
make test-deploy-famous-all
```
