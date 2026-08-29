# Famous Solidity Contracts on NeoVM: Compatibility Audit

> **Update (2026-07-02, v0.26.0/v0.27.0) — read this first.** The `92 / 92` figures below are the **old deps-STRIPPED** vendored corpus (each contract flattened, so they are not the real capability signal). The current, honest end-to-end signal is measured against **~600 real Ethereum contracts with their FULL dependency trees**: roughly **87% end-to-end compilation (522/601)**. Per-corpus breakdown — zero-knowledge **100%**, solmate **100%**, solady **99%**, Uniswap-v3 **93%**, OpenZeppelin **89%**, Aave v3 **85%**. It handles the hardest material in the wild: snarkjs / PLONK / Fflonk ZK verifiers, MakerDAO's assembly `Vat`, Gnosis Safe, and the full Uniswap V4 + Permit2. Categories covered span DeFi, NFT, GameFi, zero-knowledge, and infrastructure/DAO.
>
> For a hermetic, self-contained set you can compile locally, see the **sample corpus** at `third_party/famous-contracts/samples/` — **33 famous contracts** across `defi/`, `nft/`, `gamefi/`, `zkp/`, `infra-dao/`, and `patterns/`, each compiling with no external dependencies. `cargo test --release --test famous_samples_compile` asserts **100%** of the sample set compiles.

- Generated at (UTC): `2026-02-15T08:56:34.043Z`
- Snapshot scope: historical compatibility output; rerun `npm run audit:famous-contracts` before treating these results as current release evidence.
- Contracts shown in this report (upstream, vendored in repo): `92`
- Compile success: `92`
- Compile failed: `0`

> The `92` numbers on the three lines above are the deps-stripped vendored corpus; see the **2026-07-02** note at the top of this file for the real full-dependency ~87% signal.

Primary navigation for original upstream contracts: [Original Famous Contracts (Per Contract)](/solidity/original-contracts/).

## What "Need XXX to Implement" Means

- This report marks each failing contract with the **primary unsupported point** in current `neo-devpack-solidity`.
- The "Need on Neo" column states what is required to make that pattern work:
  1) compiler capability expansion, and/or
  2) Solidity source refactor to Neo-native patterns (`Runtime`, `Syscalls`, `NativeCalls`, `onNEP17Payment`, etc.).

## Top Blockers (Upstream Contracts)

- No blockers in this run.

## Per-Contract Results (Upstream Famous Contracts)

| # | Project | Contract | Result | Primary Unsupported Point | Need on Neo | Source |
|---:|---|---|---|---|---|---|
| 1 | OpenZeppelin | ERC20 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/ERC20.sol` |
| 2 | OpenZeppelin | ERC20Permit | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol` |
| 3 | OpenZeppelin | ERC20Votes | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol` |
| 4 | OpenZeppelin | ERC20Capped | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20Capped.sol` |
| 5 | OpenZeppelin | ERC20FlashMint | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC20FlashMint.sol` |
| 6 | OpenZeppelin | ERC4626 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol` |
| 7 | OpenZeppelin | ERC721 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/ERC721.sol` |
| 8 | OpenZeppelin | ERC721Enumerable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol` |
| 9 | OpenZeppelin | ERC721URIStorage | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol` |
| 10 | OpenZeppelin | ERC721Votes | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC721/extensions/ERC721Votes.sol` |
| 11 | OpenZeppelin | ERC1155 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/ERC1155/ERC1155.sol` |
| 12 | OpenZeppelin | ERC2981 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/token/common/ERC2981.sol` |
| 13 | OpenZeppelin | Ownable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/access/Ownable.sol` |
| 14 | OpenZeppelin | AccessControl | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/access/AccessControl.sol` |
| 15 | OpenZeppelin | AccessManager | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/access/manager/AccessManager.sol` |
| 16 | OpenZeppelin | Governor | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/Governor.sol` |
| 17 | OpenZeppelin | GovernorTimelockControl | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol` |
| 18 | OpenZeppelin | GovernorVotes | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/extensions/GovernorVotes.sol` |
| 19 | OpenZeppelin | GovernorSettings | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/extensions/GovernorSettings.sol` |
| 20 | OpenZeppelin | TimelockController | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/governance/TimelockController.sol` |
| 21 | OpenZeppelin | TransparentUpgradeableProxy | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol` |
| 22 | OpenZeppelin | ProxyAdmin | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol` |
| 23 | OpenZeppelin | ERC1967Proxy | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol` |
| 24 | OpenZeppelin | BeaconProxy | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol` |
| 25 | OpenZeppelin | UUPSUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol` |
| 26 | OpenZeppelin | Initializable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/proxy/utils/Initializable.sol` |
| 27 | OpenZeppelin | VestingWallet | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/finance/VestingWallet.sol` |
| 28 | OpenZeppelin | ERC2771Forwarder | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/metatx/ERC2771Forwarder.sol` |
| 29 | OpenZeppelin | Multicall | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/utils/Multicall.sol` |
| 30 | OpenZeppelin | EIP712 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/utils/cryptography/EIP712.sol` |
| 31 | OpenZeppelin | Account | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts/account/Account.sol` |
| 32 | OpenZeppelin Upgradeable | InitializableUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol` |
| 33 | OpenZeppelin Upgradeable | UUPSUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol` |
| 34 | OpenZeppelin Upgradeable | ERC20Upgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol` |
| 35 | OpenZeppelin Upgradeable | ERC20PermitUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol` |
| 36 | OpenZeppelin Upgradeable | ERC721Upgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol` |
| 37 | OpenZeppelin Upgradeable | OwnableUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol` |
| 38 | OpenZeppelin Upgradeable | GovernorUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/GovernorUpgradeable.sol` |
| 39 | OpenZeppelin Upgradeable | GovernorVotesUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/extensions/GovernorVotesUpgradeable.sol` |
| 40 | OpenZeppelin Upgradeable | GovernorTimelockControlUpgradeable | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@openzeppelin/contracts-upgradeable/governance/extensions/GovernorTimelockControlUpgradeable.sol` |
| 41 | Aave V3 | Pool | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/Pool.sol` |
| 42 | Aave V3 | PoolConfigurator | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/PoolConfigurator.sol` |
| 43 | Aave V3 | L2Pool | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/pool/L2Pool.sol` |
| 44 | Aave V3 | AToken | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/tokenization/AToken.sol` |
| 45 | Aave V3 | VariableDebtToken | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/tokenization/VariableDebtToken.sol` |
| 46 | Aave V3 | StableDebtToken | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/tokenization/StableDebtToken.sol` |
| 47 | Aave V3 | PoolAddressesProvider | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/PoolAddressesProvider.sol` |
| 48 | Aave V3 | PoolAddressesProviderRegistry | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/PoolAddressesProviderRegistry.sol` |
| 49 | Aave V3 | ACLManager | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/configuration/ACLManager.sol` |
| 50 | Aave V3 | AaveOracle | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/misc/AaveOracle.sol` |
| 51 | Aave V3 | FlashLoanReceiverBase | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/flashloan/base/FlashLoanReceiverBase.sol` |
| 52 | Aave V3 | InitializableImmutableAdminUpgradeabilityProxy | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/protocol/libraries/aave-upgradeability/InitializableImmutableAdminUpgradeabilityProxy.sol` |
| 53 | Aave V3 | WETH9 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@aave/core-v3/contracts/dependencies/weth/WETH9.sol` |
| 54 | Safe | Safe | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/Safe.sol` |
| 55 | Safe | SafeL2 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/SafeL2.sol` |
| 56 | Safe | SafeProxy | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/proxies/SafeProxy.sol` |
| 57 | Safe | SafeProxyFactory | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/proxies/SafeProxyFactory.sol` |
| 58 | Safe | CompatibilityFallbackHandler | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/handler/CompatibilityFallbackHandler.sol` |
| 59 | Safe | MultiSend | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/MultiSend.sol` |
| 60 | Safe | MultiSendCallOnly | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/MultiSendCallOnly.sol` |
| 61 | Safe | SignMessageLib | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@safe-global/safe-contracts/contracts/libraries/SignMessageLib.sol` |
| 62 | Uniswap V2 Core | UniswapV2Factory | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Factory.sol` |
| 63 | Uniswap V2 Core | UniswapV2Pair | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2Pair.sol` |
| 64 | Uniswap V2 Core | UniswapV2ERC20 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v2-core/contracts/UniswapV2ERC20.sol` |
| 65 | Uniswap V4 Core | PoolManager | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/PoolManager.sol` |
| 66 | Uniswap V4 Core | ProtocolFees | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/ProtocolFees.sol` |
| 67 | Uniswap V4 Core | ERC6909 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/ERC6909.sol` |
| 68 | Uniswap V4 Core | ERC6909Claims | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/ERC6909Claims.sol` |
| 69 | Uniswap V4 Core | NoDelegateCall | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/NoDelegateCall.sol` |
| 70 | Uniswap V4 Core | Extsload | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/Extsload.sol` |
| 71 | Uniswap V4 Core | Exttload | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-core/src/Exttload.sol` |
| 72 | Uniswap V4 Periphery | V4Router | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/V4Router.sol` |
| 73 | Uniswap V4 Periphery | PositionManager | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/PositionManager.sol` |
| 74 | Uniswap V4 Periphery | PositionDescriptor | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/PositionDescriptor.sol` |
| 75 | Uniswap V4 Periphery | V4Quoter | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/lens/V4Quoter.sol` |
| 76 | Uniswap V4 Periphery | StateView | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/lens/StateView.sol` |
| 77 | Uniswap V4 Periphery | UniswapV4DeployerCompetition | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@uniswap/v4-periphery/src/UniswapV4DeployerCompetition.sol` |
| 78 | Chainlink | FunctionsClient | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsClient.sol` |
| 79 | Chainlink | FunctionsRouter | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsRouter.sol` |
| 80 | Chainlink | FunctionsCoordinator | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsCoordinator.sol` |
| 81 | Chainlink | AutomationCompatible | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/automation/AutomationCompatible.sol` |
| 82 | Chainlink | AutomationRegistry2_3 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/automation/v2_3/AutomationRegistry2_3.sol` |
| 83 | Chainlink | VRFConsumerBaseV2 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/vrf/VRFConsumerBaseV2.sol` |
| 84 | Chainlink | VRFCoordinatorV2_5 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/vrf/dev/VRFCoordinatorV2_5.sol` |
| 85 | Chainlink | Operator | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/operatorforwarder/Operator.sol` |
| 86 | Chainlink | DataFeedsCache | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/data-feeds/DataFeedsCache.sol` |
| 87 | Chainlink | OwnerIsCreator | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/access/OwnerIsCreator.sol` |
| 88 | Chainlink | MockV3Aggregator | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol` |
| 89 | Chainlink | ArbitrumSequencerUptimeFeed | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/l2ep/arbitrum/ArbitrumSequencerUptimeFeed.sol` |
| 90 | Chainlink | FunctionsClient_v1_3_0 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsClient.sol` |
| 91 | Chainlink | FunctionsCoordinator_v1_3_0 | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsCoordinator.sol` |
| 92 | Chainlink | ZKSyncFunctionsRouter | ✅ pass | - | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） | `third_party/famous-contracts/sources/@chainlink/contracts/src/v0.8/functions/v1_3_0_zksync/ZKSyncFunctionsRouter.sol` |

## Notes

- This report prioritizes **upstream famous contracts vendored in this repository** in the main results table.
- A **pass** means the contract compiled through `neo-solc` in this environment.
- A **fail** does not mean the contract is impossible on Neo; it means current source + current compiler need refactor or feature work.
- Use this as a migration backlog: prioritize high-value blockers (`delegatecall`, import cycles, ABI overload collisions, named mapping syntax).
