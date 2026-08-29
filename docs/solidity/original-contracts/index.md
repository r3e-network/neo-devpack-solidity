# Original Famous Solidity Contracts (Per Contract)

This section documents **upstream famous Solidity contracts** (vendored in-repo sources), not simplified demo ports.

- Generated at (UTC): `2026-02-15T08:56:34.043Z`
- Snapshot scope: historical compatibility output; rerun `npm run audit:famous-contracts` and `npm run docs:generate:original-contracts` before treating these per-contract results as current release evidence.
- Contracts in this section: `92`

Each contract has a dedicated page with:

1. Compilation status on NeoVM
1. Primary blocker and required Neo-side capability/refactor
1. Diagnostics summarized from the audit run

## Common Production Hardening

For contracts that compiled in this historical snapshot:

1. Review generated manifest permissions and remove wildcard entries when possible.
1. Run Neo-Express state-changing tests for target workflows, not only read-only calls.
1. Validate semantic differences such as `tx.origin`, payable semantics, callback models, and manifest permissions for the integration context.

## Project Summary

| Project | Contracts | Pass | Fail |
| --- | ---: | ---: | ---: |
| Aave V3 | 13 | 13 | 0 |
| Chainlink | 15 | 15 | 0 |
| OpenZeppelin | 31 | 31 | 0 |
| OpenZeppelin Upgradeable | 9 | 9 | 0 |
| Safe | 8 | 8 | 0 |
| Uniswap V2 Core | 3 | 3 | 0 |
| Uniswap V4 Core | 7 | 7 | 0 |
| Uniswap V4 Periphery | 6 | 6 | 0 |

## Aave V3

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [AaveOracle](/solidity/original-contracts/aave-v3/aaveoracle) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ACLManager](/solidity/original-contracts/aave-v3/aclmanager) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [AToken](/solidity/original-contracts/aave-v3/atoken) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FlashLoanReceiverBase](/solidity/original-contracts/aave-v3/flashloanreceiverbase) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [InitializableImmutableAdminUpgradeabilityProxy](/solidity/original-contracts/aave-v3/initializableimmutableadminupgradeabilityproxy) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [L2Pool](/solidity/original-contracts/aave-v3/l2pool) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Pool](/solidity/original-contracts/aave-v3/pool) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [PoolAddressesProvider](/solidity/original-contracts/aave-v3/pooladdressesprovider) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [PoolAddressesProviderRegistry](/solidity/original-contracts/aave-v3/pooladdressesproviderregistry) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [PoolConfigurator](/solidity/original-contracts/aave-v3/poolconfigurator) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [StableDebtToken](/solidity/original-contracts/aave-v3/stabledebttoken) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [VariableDebtToken](/solidity/original-contracts/aave-v3/variabledebttoken) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [WETH9](/solidity/original-contracts/aave-v3/weth9) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## Chainlink

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [ArbitrumSequencerUptimeFeed](/solidity/original-contracts/chainlink/arbitrumsequenceruptimefeed) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [AutomationCompatible](/solidity/original-contracts/chainlink/automationcompatible) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [AutomationRegistry2_3](/solidity/original-contracts/chainlink/automationregistry2-3) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [DataFeedsCache](/solidity/original-contracts/chainlink/datafeedscache) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FunctionsClient](/solidity/original-contracts/chainlink/functionsclient) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FunctionsClient_v1_3_0](/solidity/original-contracts/chainlink/functionsclient-v1-3-0) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FunctionsCoordinator](/solidity/original-contracts/chainlink/functionscoordinator) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FunctionsCoordinator_v1_3_0](/solidity/original-contracts/chainlink/functionscoordinator-v1-3-0) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [FunctionsRouter](/solidity/original-contracts/chainlink/functionsrouter) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [MockV3Aggregator](/solidity/original-contracts/chainlink/mockv3aggregator) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Operator](/solidity/original-contracts/chainlink/operator) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [OwnerIsCreator](/solidity/original-contracts/chainlink/owneriscreator) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [VRFConsumerBaseV2](/solidity/original-contracts/chainlink/vrfconsumerbasev2) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [VRFCoordinatorV2_5](/solidity/original-contracts/chainlink/vrfcoordinatorv2-5) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ZKSyncFunctionsRouter](/solidity/original-contracts/chainlink/zksyncfunctionsrouter) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## OpenZeppelin

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [AccessControl](/solidity/original-contracts/openzeppelin/accesscontrol) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [AccessManager](/solidity/original-contracts/openzeppelin/accessmanager) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Account](/solidity/original-contracts/openzeppelin/account) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [BeaconProxy](/solidity/original-contracts/openzeppelin/beaconproxy) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [EIP712](/solidity/original-contracts/openzeppelin/eip712) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC1155](/solidity/original-contracts/openzeppelin/erc1155) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC1967Proxy](/solidity/original-contracts/openzeppelin/erc1967proxy) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20](/solidity/original-contracts/openzeppelin/erc20) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20Capped](/solidity/original-contracts/openzeppelin/erc20capped) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20FlashMint](/solidity/original-contracts/openzeppelin/erc20flashmint) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20Permit](/solidity/original-contracts/openzeppelin/erc20permit) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20Votes](/solidity/original-contracts/openzeppelin/erc20votes) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC2771Forwarder](/solidity/original-contracts/openzeppelin/erc2771forwarder) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC2981](/solidity/original-contracts/openzeppelin/erc2981) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC4626](/solidity/original-contracts/openzeppelin/erc4626) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC721](/solidity/original-contracts/openzeppelin/erc721) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC721Enumerable](/solidity/original-contracts/openzeppelin/erc721enumerable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC721URIStorage](/solidity/original-contracts/openzeppelin/erc721uristorage) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC721Votes](/solidity/original-contracts/openzeppelin/erc721votes) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Governor](/solidity/original-contracts/openzeppelin/governor) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorSettings](/solidity/original-contracts/openzeppelin/governorsettings) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorTimelockControl](/solidity/original-contracts/openzeppelin/governortimelockcontrol) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorVotes](/solidity/original-contracts/openzeppelin/governorvotes) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Initializable](/solidity/original-contracts/openzeppelin/initializable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Multicall](/solidity/original-contracts/openzeppelin/multicall) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Ownable](/solidity/original-contracts/openzeppelin/ownable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ProxyAdmin](/solidity/original-contracts/openzeppelin/proxyadmin) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [TimelockController](/solidity/original-contracts/openzeppelin/timelockcontroller) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [TransparentUpgradeableProxy](/solidity/original-contracts/openzeppelin/transparentupgradeableproxy) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [UUPSUpgradeable](/solidity/original-contracts/openzeppelin/uupsupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [VestingWallet](/solidity/original-contracts/openzeppelin/vestingwallet) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## OpenZeppelin Upgradeable

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [ERC20PermitUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/erc20permitupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC20Upgradeable](/solidity/original-contracts/openzeppelin-upgradeable/erc20upgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC721Upgradeable](/solidity/original-contracts/openzeppelin-upgradeable/erc721upgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorTimelockControlUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/governortimelockcontrolupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/governorupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [GovernorVotesUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/governorvotesupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [InitializableUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/initializableupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [OwnableUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/ownableupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [UUPSUpgradeable](/solidity/original-contracts/openzeppelin-upgradeable/uupsupgradeable) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## Safe

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [CompatibilityFallbackHandler](/solidity/original-contracts/safe/compatibilityfallbackhandler) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [MultiSend](/solidity/original-contracts/safe/multisend) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [MultiSendCallOnly](/solidity/original-contracts/safe/multisendcallonly) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Safe](/solidity/original-contracts/safe/safe) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [SafeL2](/solidity/original-contracts/safe/safel2) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [SafeProxy](/solidity/original-contracts/safe/safeproxy) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [SafeProxyFactory](/solidity/original-contracts/safe/safeproxyfactory) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [SignMessageLib](/solidity/original-contracts/safe/signmessagelib) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## Uniswap V2 Core

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [UniswapV2ERC20](/solidity/original-contracts/uniswap-v2-core/uniswapv2erc20) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [UniswapV2Factory](/solidity/original-contracts/uniswap-v2-core/uniswapv2factory) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [UniswapV2Pair](/solidity/original-contracts/uniswap-v2-core/uniswapv2pair) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## Uniswap V4 Core

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [ERC6909](/solidity/original-contracts/uniswap-v4-core/erc6909) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ERC6909Claims](/solidity/original-contracts/uniswap-v4-core/erc6909claims) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Extsload](/solidity/original-contracts/uniswap-v4-core/extsload) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [Exttload](/solidity/original-contracts/uniswap-v4-core/exttload) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [NoDelegateCall](/solidity/original-contracts/uniswap-v4-core/nodelegatecall) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [PoolManager](/solidity/original-contracts/uniswap-v4-core/poolmanager) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [ProtocolFees](/solidity/original-contracts/uniswap-v4-core/protocolfees) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |

## Uniswap V4 Periphery

| Contract | Status | Blocker | Need On Neo |
| --- | --- | --- | --- |
| [PositionDescriptor](/solidity/original-contracts/uniswap-v4-periphery/positiondescriptor) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [PositionManager](/solidity/original-contracts/uniswap-v4-periphery/positionmanager) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [StateView](/solidity/original-contracts/uniswap-v4-periphery/stateview) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [UniswapV4DeployerCompetition](/solidity/original-contracts/uniswap-v4-periphery/uniswapv4deployercompetition) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [V4Quoter](/solidity/original-contracts/uniswap-v4-periphery/v4quoter) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
| [V4Router](/solidity/original-contracts/uniswap-v4-periphery/v4router) | ✅ pass | `none` | 可直接编译为 NeoVM（如需生产使用，仍需做 manifest 权限最小化和业务安全审计） |
