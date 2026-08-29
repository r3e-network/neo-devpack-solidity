# Famous Contracts Runtime Smoke Report

_Generated_: UNIX epoch 1782533424 (`cargo test --features neoxp-diff --test famous_contracts_runtime_smoke -- --ignored --nocapture`)

- Compiler: `neo-solc` (release)
- Backend: Neo-Express (real-node deploy + invoke)
- Source root: `third_party/famous-contracts/sources/`

## Summary

| Metric | Count |
| --- | --- |
| Total contracts | 92 |
| Compile pass | 7 |
| Deploy pass (compile pass + HALT deploy) | 5 |
| Smoke HALT (full pass — compile + deploy + smoke method) | 1 |

## Per-contract results

Legend: ✓ = passed, ✗ = failed, `—` = not attempted (earlier stage failed).

| Contract | Pragma | Compile | Deploy | Smoke method | State | Stack[0] |
| --- | --- | --- | --- | --- | --- | --- |
| `@chainlink/contracts/src/v0.8/shared/access/OwnerIsCreator.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/shared/mocks/MockV3Aggregator.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/data-feeds/DataFeedsCache.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/automation/v2_3/AutomationRegistry2_3.sol` | `pragma solidity 0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/automation/AutomationCompatible.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/vrf/dev/VRFCoordinatorV2_5.sol` | `pragma solidity 0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/vrf/VRFConsumerBaseV2.sol` | `pragma solidity ^0.8.4;` | ✓ | ✗ | `` |  | _deploy error_: neoxp ["contract", "deploy", "-i", "/tmp/.tmphGioZl/chain.neo-express", "/tmp/neo-solc-famous-smoke-1782533369685190606-6/VRFConsumerBaseV2.nef", "node1", "-j"] exited Some(1) |
| `@chainlink/contracts/src/v0.8/functions/v1_3_0_zksync/ZKSyncFunctionsRouter.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsClient.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsCoordinator.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/functions/v1_0_0/FunctionsRouter.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsClient.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/functions/v1_3_0/FunctionsCoordinator.sol` | `pragma solidity ^0.8.19;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/l2ep/arbitrum/ArbitrumSequencerUptimeFeed.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@chainlink/contracts/src/v0.8/operatorforwarder/Operator.sol` | `pragma solidity 0.8.19;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/V4Router.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/PositionManager.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/UniswapV4DeployerCompetition.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/lens/V4Quoter.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/lens/StateView.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-periphery/src/PositionDescriptor.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/ERC6909Claims.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/NoDelegateCall.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/Exttload.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/PoolManager.sol` | `pragma solidity 0.8.26;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/Extsload.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/ERC6909.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v4-core/src/ProtocolFees.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@uniswap/v2-core/contracts/UniswapV2Factory.sol` | `pragma solidity =0.5.16;` | ✗ | — | `` |  |  |
| `@uniswap/v2-core/contracts/UniswapV2ERC20.sol` | `pragma solidity =0.5.16;` | ✗ | — | `` |  |  |
| `@uniswap/v2-core/contracts/UniswapV2Pair.sol` | `pragma solidity =0.5.16;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/account/Account.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/finance/VestingWallet.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/utils/Multicall.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/utils/cryptography/EIP712.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/governance/TimelockController.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/governance/extensions/GovernorVotes.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/governance/extensions/GovernorSettings.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/governance/Governor.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/access/Ownable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/access/manager/AccessManager.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/access/AccessControl.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/ERC20.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/extensions/ERC20FlashMint.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC20/extensions/ERC20Capped.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC1155/ERC1155.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/common/ERC2981.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC721/extensions/ERC721Votes.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/token/ERC721/ERC721.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/proxy/utils/Initializable.sol` | `pragma solidity ^0.8.20;` | ✓ | ✓ | `—` |  |  |
| `@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/proxy/transparent/ProxyAdmin.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/proxy/beacon/BeaconProxy.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts/metatx/ERC2771Forwarder.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/governance/GovernorUpgradeable.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/governance/extensions/GovernorTimelockControlUpgradeable.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/governance/extensions/GovernorVotesUpgradeable.sol` | `pragma solidity ^0.8.24;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/token/ERC20/extensions/ERC20PermitUpgradeable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/token/ERC721/ERC721Upgradeable.sol` | `pragma solidity ^0.8.20;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol` | `pragma solidity ^0.8.22;` | ✗ | — | `` |  |  |
| `@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol` | `pragma solidity ^0.8.20;` | ✓ | ✓ | `—` |  |  |
| `@safe-global/safe-contracts/contracts/SafeL2.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✗ | — | `` |  |  |
| `@safe-global/safe-contracts/contracts/proxies/SafeProxyFactory.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✗ | — | `` |  |  |
| `@safe-global/safe-contracts/contracts/proxies/SafeProxy.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✓ | ✗ | `` |  | _deploy error_: neoxp ["contract", "deploy", "-i", "/tmp/.tmp4nvc0p/chain.neo-express", "/tmp/neo-solc-famous-smoke-1782533397220728472-73/SafeProxy.nef", "node1", "-j"] exited Some(1) |
| `@safe-global/safe-contracts/contracts/libraries/MultiSend.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✓ | ✓ | `—` |  |  |
| `@safe-global/safe-contracts/contracts/libraries/SignMessageLib.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✗ | — | `` |  |  |
| `@safe-global/safe-contracts/contracts/libraries/MultiSendCallOnly.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✓ | ✓ | `—` |  |  |
| `@safe-global/safe-contracts/contracts/handler/CompatibilityFallbackHandler.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✗ | — | `` |  |  |
| `@safe-global/safe-contracts/contracts/Safe.sol` | `pragma solidity >=0.7.0 <0.9.0;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/pool/L2Pool.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/pool/Pool.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/pool/PoolConfigurator.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/tokenization/AToken.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/tokenization/StableDebtToken.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/tokenization/VariableDebtToken.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/libraries/aave-upgradeability/InitializableImmutableAdminUpgradeabilityProxy.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/configuration/PoolAddressesProviderRegistry.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/configuration/ACLManager.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/protocol/configuration/PoolAddressesProvider.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/dependencies/weth/WETH9.sol` | `pragma solidity ^0.8.10;` | ✓ | ✓ | `name` | HALT | {"type":"ByteString","value":"V3JhcHBlZCBFdGhlcg=="} |
| `@aave/core-v3/contracts/misc/AaveOracle.sol` | `pragma solidity ^0.8.10;` | ✗ | — | `` |  |  |
| `@aave/core-v3/contracts/flashloan/base/FlashLoanReceiverBase.sol` | `pragma solidity ^0.8.0;` | ✗ | — | `` |  |  |
