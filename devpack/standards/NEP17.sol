// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NEP-17 Fungible Token Standard
 * @dev Complete implementation of Neo N3 NEP-17 standard for Solidity
 * @author Jimmy <jimmy@r3e.network>
 * 
 * NEP-17 is Neo's enhanced fungible token standard, providing:
 * - Standard ERC-20 compatibility
 * - Neo-specific features (onNEP17Payment callback)
 * - Advanced transfer capabilities
 * - Integration with Neo native tokens
 * - Event system compatible with Neo Runtime.Notify
 */

import "../contracts/FrameworkBase.sol";
import "../contracts/Types.sol";
import "../libraries/Neo.sol";
import "../libraries/Runtime.sol";
import "../libraries/Storage.sol";

/**
 * @title INEP17
 * @dev Interface for NEP-17 fungible token standard
 */
interface INEP17 {
    // Standard NEP-17 functions
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    // NEP-17 `data` is an unconstrained StackItem (Neo ABI type: Any).
    // This devpack uses the Neo DevPack for Solidity `Any` type to accurately reflect the standard.
    function transfer(address from, address to, uint256 amount, Any calldata data) external returns (bool);
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 amount);
}

/**
 * @title INEP17Receiver
 * @dev Interface for contracts that can receive NEP-17 tokens
 */
interface INEP17Receiver {
    function onNEP17Payment(address from, uint256 amount, Any calldata data) external;
}

/**
 * @title NEP17
 * @dev Complete NEP-17 token implementation with Neo N3 integration
 */
contract NEP17 is INEP17, FrameworkBase {
    using Neo for *;
    using Runtime for *;

    // Neo N3 Oracle native contract hash.
    address private constant ORACLE_NATIVE_CONTRACT = 0xfe924b7cfe89ddd271abaf7210a80a7e11178758;
    
    // Token metadata
    string private _name;
    string private _symbol;
    uint8 private _decimals;
    uint256 private _totalSupply;
    
    // Balances and allowances
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    
    // NEP-17 specific features
    bool private _transfersEnabled = true;
    address private _minter;
    uint256 private _maxSupply;
    uint256 private _conditionalTransferNonce;

    // Multi-signature authorization: only owner-approved signers may move the
    // contract's own (escrowed) token pool via multiSigTransfer, and at least
    // `_multisigThreshold` of them must witness the transaction.
    mapping(address => bool) private _multisigSigners;
    uint256 private _multisigThreshold;

    // Events (NEP-17 compatible)
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);
    event Mint(address indexed to, uint256 amount);
    event Burn(address indexed from, uint256 amount);
    
    // NEP-17 specific events
    event TransfersEnabled();
    event TransfersDisabled();
    event MinterChanged(address indexed oldMinter, address indexed newMinter);
    event MaxSupplySet(uint256 maxSupply);

    // Extended events
    event EmergencyPause(address indexed caller, uint256 timestamp);
    event EmergencyUnpause(address indexed caller, uint256 timestamp);
    event TimelockCreated(
        bytes32 indexed timelockId,
        address indexed from,
        address indexed to,
        uint256 amount,
        uint256 releaseTime
    );
    event TimelockClaimed(bytes32 indexed timelockId, address indexed to, uint256 amount);
    event ConditionalTransferCreated(
        bytes32 indexed requestId,
        address indexed from,
        address indexed to,
        uint256 amount
    );
    event ConditionalTransferExecuted(address indexed from, address indexed to, uint256 amount);
    event ConditionalTransferFailed(address indexed from, uint256 amount);
    
    // Custom errors
    error NEP17InsufficientBalance(address account, uint256 balance, uint256 needed);
    error NEP17InvalidReceiver(address receiver);
    error NEP17TransfersDisabled();
    error NEP17ExceedsMaxSupply(uint256 amount, uint256 maxSupply);
    error NEP17InvalidAmount(uint256 amount);
    error NEP17NotMinter(address caller);
    
    // Modifiers
    modifier whenTransfersEnabled() {
        if (!_transfersEnabled) revert NEP17TransfersDisabled();
        _;
    }
    
    modifier onlyMinter() {
        if (msg.sender != _minter) revert NEP17NotMinter(msg.sender);
        _;
    }
    
    modifier validAmount(uint256 amount) {
        if (amount == 0) revert NEP17InvalidAmount(amount);
        _;
    }
    
    modifier validReceiver(address to) {
        if (to == address(0)) revert NEP17InvalidReceiver(to);
        _;
    }
    
    /**
     * @dev Constructor
     */
    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        uint256 initialSupply,
        uint256 maxSupply_
    ) FrameworkBase() {
        require(bytes(name_).length > 0, "NEP17: name cannot be empty");
        require(bytes(symbol_).length > 0, "NEP17: symbol cannot be empty");
        require(decimals_ <= 18, "NEP17: decimals cannot exceed 18");
        
        // A non-zero max supply must accommodate the initial mint. `_mint`
        // (used below) does not enforce the cap — unlike the public `mint()` —
        // so guard the initial supply here to keep the cap invariant.
        require(
            maxSupply_ == 0 || initialSupply <= maxSupply_,
            "NEP17: initial supply exceeds max supply"
        );

        _name = name_;
        _symbol = symbol_;
        _decimals = decimals_;
        _minter = msg.sender;
        _maxSupply = maxSupply_;

        if (initialSupply > 0) {
            _mint(msg.sender, initialSupply);
        }
    }
    
    // ========== View Functions ==========
    
    /**
     * @dev Returns the name of the token
     */
    function name() public view returns (string memory) {
        return _name;
    }
    
    /**
     * @dev Returns the symbol of the token
     */
    function symbol() public view override returns (string memory) {
        return _symbol;
    }
    
    /**
     * @dev Returns the number of decimals
     */
    function decimals() public view override returns (uint8) {
        return _decimals;
    }
    
    /**
     * @dev Returns the total supply
     */
    function totalSupply() public view override returns (uint256) {
        return _totalSupply;
    }
    
    /**
     * @dev Returns the balance of an account
     */
    function balanceOf(address account) public view override returns (uint256) {
        return _balances[account];
    }
    
    /**
     * @dev Returns the allowance
     */
    function allowance(address owner, address spender) public view returns (uint256) {
        return _allowances[owner][spender];
    }
    
    /**
     * @dev Returns if transfers are enabled
     */
    function transfersEnabled() public view returns (bool) {
        return _transfersEnabled;
    }
    
    /**
     * @dev Returns the minter address
     */
    function minter() public view returns (address) {
        return _minter;
    }
    
    /**
     * @dev Returns the maximum supply
     */
    function maxSupply() public view returns (uint256) {
        return _maxSupply;
    }
    
    // ========== Transfer Functions ==========
    
    /**
     * @dev NEP-17 transfer function
     */
    function transfer(
        address from,
        address to,
        uint256 amount,
        Any calldata data
    ) public override whenTransfersEnabled validReceiver(to) returns (bool) {
        // NEP-17 requires a zero-amount transfer to be processed normally
        // (emit Transfer, run the receiver callback, return true) — so
        // `validAmount` is intentionally NOT applied here; it stays on
        // mint/burn where a non-zero amount is a legitimate business rule.
        // Authorization: the owner (acting as the direct sender or via witness)
        // or a spender holding sufficient allowance. Cache the witness result so
        // it is consulted once for both the check and the allowance accounting.
        //
        // M-DEV3 note — this is a HYBRID authorization model that mixes
        // NEP-17 witness semantics with ERC-20 allowance semantics:
        //   1. The owner authorizes either by being the direct caller OR by
        //      passing `Runtime.checkWitness(from)` (Neo N3 witness check).
        //   2. OR any address holding `_allowances[from][spender] >= amount`
        //      may move the tokens WITHOUT a witness check (pure ERC-20 path).
        // This is stricter than ERC-20 (adds the witness option) but allows a
        // non-witnessed, allowance-only path that strict NEP-17 does not. The
        // hybrid is intentional for Solidity-source compatibility (existing
        // approve/transferFrom contracts keep working), but callers relying on
        // a strict NEP-17 "only the owner or a witness holder can move tokens"
        // invariant should be aware of this allowance path.
        bool ownerAuthorized = from == msg.sender || Runtime.checkWitness(from);
        require(
            ownerAuthorized || _allowances[from][msg.sender] >= amount,
            "NEP17: unauthorized transfer"
        );

        _transfer(from, to, amount, data);

        // Consume allowance ONLY when the transfer was authorized via an
        // approval (i.e. the spender is acting on the owner's behalf), never when
        // the owner authorized directly. Decrementing on the owner-authorized
        // path would compute `_allowances[from][msg.sender] - amount`, which
        // underflows (Panic 0x11) and reverts when the allowance is below
        // `amount` — e.g. a witness-authorized transfer with zero allowance.
        if (!ownerAuthorized && _allowances[from][msg.sender] != type(uint256).max) {
            require(
                _allowances[from][msg.sender] >= amount,
                "NEP17: insufficient allowance"
            );
            _approve(from, msg.sender, _allowances[from][msg.sender] - amount);
        }

        return true;
    }
    
    /**
     * @dev Standard ERC-20 transfer
     */
    function transfer(address to, uint256 amount) public returns (bool) {
        return transfer(msg.sender, to, amount, "");
    }
    
    /**
     * @dev Transfer from (ERC-20 compatibility)
     */
    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        return transfer(from, to, amount, "");
    }
    
    /**
     * @dev Approve spender
     */
    function approve(address spender, uint256 amount) public returns (bool) {
        _approve(msg.sender, spender, amount);
        return true;
    }
    
    /**
     * @dev Increase allowance
     */
    function increaseAllowance(address spender, uint256 addedValue) public returns (bool) {
        _approve(msg.sender, spender, _allowances[msg.sender][spender] + addedValue);
        return true;
    }
    
    /**
     * @dev Decrease allowance
     */
    function decreaseAllowance(address spender, uint256 subtractedValue) public returns (bool) {
        uint256 currentAllowance = _allowances[msg.sender][spender];
        require(currentAllowance >= subtractedValue, "NEP17: decreased allowance below zero");
        _approve(msg.sender, spender, currentAllowance - subtractedValue);
        return true;
    }
    
    // ========== Minting and Burning ==========
    
    /**
     * @dev Mint tokens
     */
    function mint(address to, uint256 amount) public onlyMinter validReceiver(to) validAmount(amount) {
        if (_maxSupply > 0 && _totalSupply + amount > _maxSupply) {
            revert NEP17ExceedsMaxSupply(amount, _maxSupply);
        }
        
        _mint(to, amount);
    }
    
    /**
     * @dev Burn tokens
     */
    function burn(uint256 amount) public validAmount(amount) {
        _burn(msg.sender, amount);
    }
    
    /**
     * @dev Burn tokens from account (with allowance)
     */
    function burnFrom(address from, uint256 amount) public validAmount(amount) {
        uint256 currentAllowance = _allowances[from][msg.sender];
        require(currentAllowance >= amount, "NEP17: burn amount exceeds allowance");
        
        _burn(from, amount);
        _approve(from, msg.sender, currentAllowance - amount);
    }
    
    // ========== Admin Functions ==========
    
    /**
     * @dev Owner: authorize/deauthorize a multi-signature signer for the pool.
     */
    function setMultisigSigner(address signer, bool allowed) public onlyOwner {
        require(signer != address(0), "NEP17: invalid signer");
        _multisigSigners[signer] = allowed;
    }

    /**
     * @dev Whether `signer` is in the owner-managed multi-signature allow-list.
     * Derived contracts MUST gate multi-sig operations on this — counting any
     * witness that merely passes `Runtime.checkWitness` lets an attacker supply
     * keys they control and forge a quorum.
     */
    function isMultisigSigner(address signer) public view returns (bool) {
        return _multisigSigners[signer];
    }

    /**
     * @dev Owner: set the minimum number of authorized witnesses required for
     * a multiSigTransfer (must be at least 2).
     */
    function setMultisigThreshold(uint256 threshold) public onlyOwner {
        require(threshold >= 2, "NEP17: threshold must be >= 2");
        _multisigThreshold = threshold;
    }

    /**
     * @dev Enable transfers
     */
    function enableTransfers() public onlyOwner {
        require(!_transfersEnabled, "NEP17: transfers already enabled");
        _transfersEnabled = true;
        emit TransfersEnabled();
    }

    /**
     * @dev Disable transfers
     */
    function disableTransfers() public onlyOwner {
        require(_transfersEnabled, "NEP17: transfers already disabled");
        _transfersEnabled = false;
        emit TransfersDisabled();
    }
    
    /**
     * @dev Change minter
     */
    function changeMinter(address newMinter) public onlyOwner {
        require(newMinter != address(0), "NEP17: new minter is zero address");
        address oldMinter = _minter;
        _minter = newMinter;
        emit MinterChanged(oldMinter, newMinter);
    }
    
    /**
     * @dev Set maximum supply
     */
    function setMaxSupply(uint256 newMaxSupply) public onlyOwner {
        require(newMaxSupply >= _totalSupply, "NEP17: max supply below current supply");
        _maxSupply = newMaxSupply;
        emit MaxSupplySet(newMaxSupply);
    }
    
    // ========== Batch Operations ==========
    
    /**
     * @dev Batch transfer to multiple recipients
     */
    function batchTransfer(
        address[] memory recipients,
        uint256[] memory amounts,
        bytes[] memory data
    ) public whenTransfersEnabled returns (bool) {
        require(recipients.length == amounts.length, "NEP17: array length mismatch");
        require(recipients.length == data.length, "NEP17: array length mismatch");
        require(recipients.length > 0, "NEP17: empty arrays");
        require(recipients.length <= 100, "NEP17: too many recipients");
        
        for (uint256 i = 0; i < recipients.length; i++) {
            transfer(msg.sender, recipients[i], amounts[i], data[i]);
        }
        
        return true;
    }
    
    /**
     * @dev Batch mint to multiple recipients
     */
    function batchMint(address[] memory recipients, uint256[] memory amounts) 
        public 
        onlyMinter 
        returns (bool) 
    {
        require(recipients.length == amounts.length, "NEP17: array length mismatch");
        require(recipients.length > 0, "NEP17: empty arrays");
        require(recipients.length <= 100, "NEP17: too many recipients");
        
        for (uint256 i = 0; i < recipients.length; i++) {
            mint(recipients[i], amounts[i]);
        }
        
        return true;
    }
    
    // ========== Internal Functions ==========
    
    /**
     * @dev Internal transfer function
     */
    function _transfer(address from, address to, uint256 amount, Any memory data) internal {
        uint256 fromBalance = _balances[from];
        if (fromBalance < amount) {
            revert NEP17InsufficientBalance(from, fromBalance, amount);
        }
        
        unchecked {
            _balances[from] = fromBalance - amount;
        }
        _balances[to] += amount;
        
        emit Transfer(from, to, amount);

        // Call onNEP17Payment if the recipient is a *different* contract.
        // Escrowing to `address(this)` (timelock / conditional / staking /
        // scheduled transfers all move tokens to self) is an internal
        // bookkeeping move: this contract does not implement onNEP17Payment,
        // so invoking the callback on itself would fault and revert the whole
        // escrow-in leg. Skipping self also avoids spurious self-reentrancy.
        if (to != address(this) && to.code.length > 0) {
            try INEP17Receiver(to).onNEP17Payment(from, amount, data) {
                // Success
            } catch {
                // Revert if recipient doesn't implement interface correctly
                revert NEP17InvalidReceiver(to);
            }
        }
    }
    
    /**
     * @dev Internal mint function
     */
    function _mint(address to, uint256 amount) internal {
        _totalSupply += amount;
        _balances[to] += amount;
        
        emit Transfer(address(0), to, amount);
        emit Mint(to, amount);
        
        // Call onNEP17Payment if recipient is a contract
        if (to.code.length > 0) {
            try INEP17Receiver(to).onNEP17Payment(address(0), amount, "") {
                // Success
            } catch {
                // Mint can proceed even if recipient doesn't implement interface
            }
        }
    }
    
    /**
     * @dev Internal burn function
     */
    function _burn(address from, uint256 amount) internal {
        uint256 accountBalance = _balances[from];
        if (accountBalance < amount) {
            revert NEP17InsufficientBalance(from, accountBalance, amount);
        }
        
        unchecked {
            _balances[from] = accountBalance - amount;
        }
        // Keep totalSupply decrement checked for defense-in-depth
        _totalSupply -= amount;

        emit Transfer(from, address(0), amount);
        emit Burn(from, amount);
    }
    
    /**
     * @dev Internal approve function
     */
    function _approve(address owner, address spender, uint256 amount) internal {
        require(owner != address(0), "NEP17: approve from zero address");
        require(spender != address(0), "NEP17: approve to zero address");
        
        _allowances[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }
    
    // ========== Neo Integration Functions ==========
    
    // NOTE: getHoldersCount() was removed because Solidity mappings use
    // keccak256-derived storage slots, not prefix-based keys. Storage.find()
    // with a "balance" prefix cannot iterate over mapping entries. If your
    // contract needs holder enumeration, maintain an explicit address[] array
    // or use the Neo-specific Storage.put()/find() API directly instead of
    // the native `mapping` type.
    
    /**
     * @dev Get token info for Neo blockchain
     */
    function getTokenInfo() public view virtual returns (
        string memory tokenName,
        string memory tokenSymbol,
        uint8 tokenDecimals,
        uint256 tokenTotalSupply,
        uint256 tokenMaxSupply,
        address tokenMinter,
        bool tokenTransfersEnabled
    ) {
        return (_name, _symbol, _decimals, _totalSupply, _maxSupply, _minter, _transfersEnabled);
    }
    
    /**
     * @dev Get contract metadata for Neo
     */
    function getContractMetadata() public view virtual returns (
        string memory standard,
        string memory name,
        string memory version,
        string memory author
    ) {
        return (
            "NEP-17",
            "Neo N3 Fungible Token",
            "1.0.0",
            "Jimmy <jimmy@r3e.network>"
        );
    }
    
    // ========== Emergency Functions ==========
    
    /**
     * @dev Emergency pause (disable transfers)
     */
    function emergencyPause() public virtual onlyOwner {
        disableTransfers();
        emit EmergencyPause(msg.sender, block.timestamp);
    }
    
    /**
     * @dev Emergency unpause (enable transfers)
     */
    function emergencyUnpause() public onlyOwner {
        enableTransfers();
        emit EmergencyUnpause(msg.sender, block.timestamp);
    }
    
    // ========== Advanced Features ==========
    
    /**
     * @dev Time-locked transfer
     */
    function transferWithTimelock(
        address to,
        uint256 amount,
        uint256 releaseTime
    ) public whenTransfersEnabled validReceiver(to) validAmount(amount) {
        require(releaseTime > block.timestamp, "NEP17: release time must be in future");
        
        // Store the time-locked transfer
        bytes32 timelockId = keccak256(abi.encode(msg.sender, to, amount, releaseTime, block.timestamp));
        
        // Transfer to this contract temporarily
        _transfer(msg.sender, address(this), amount, "");
        
        // Store timelock info
        Storage.put(
            abi.encode("timelock", timelockId),
            abi.encode(msg.sender, to, amount, releaseTime)
        );
        
        // Emit event
        emit TimelockCreated(timelockId, msg.sender, to, amount, releaseTime);
    }
    
    /**
     * @dev Claim time-locked tokens
     */
    function claimTimelock(bytes32 timelockId) public {
        bytes memory timelockData = Storage.get(abi.encode("timelock", timelockId));
        require(timelockData.length > 0, "NEP17: timelock not found");
        
        (address from, address to, uint256 amount, uint256 releaseTime) = 
            abi.decode(timelockData, (address, address, uint256, uint256));
        
        require(block.timestamp >= releaseTime, "NEP17: timelock not yet expired");
        require(msg.sender == to, "NEP17: only recipient can claim");
        
        // Delete timelock
        Storage.remove(abi.encode("timelock", timelockId));
        
        // Transfer tokens
        _transfer(address(this), to, amount, "");
        
        // Emit event
        emit TimelockClaimed(timelockId, to, amount);
    }
    
    /**
     * @dev Multi-signature transfer
     */
    function multiSigTransfer(
        address to,
        uint256 amount,
        address[] memory signers
    ) public whenTransfersEnabled validReceiver(to) validAmount(amount) {
        require(signers.length >= 2, "NEP17: minimum 2 signers required");
        require(signers.length <= 10, "NEP17: maximum 10 signers allowed");
        // The threshold must be configured by the owner; an unconfigured (zero)
        // threshold disables the pool transfer entirely (fail closed).
        require(_multisigThreshold >= 2, "NEP17: multisig not configured");
        require(signers.length >= _multisigThreshold, "NEP17: below threshold");

        // On-chain authorization is enforced via Neo witness checks for each
        // declared signer. Off-chain signature collection is the client's
        // responsibility — this function does not verify signatures on-chain.
        for (uint256 i = 0; i < signers.length; i++) {
            address signer = signers[i];
            require(signer != address(0), "NEP17: invalid signer");

            // CRITICAL: the signer must be an OWNER-AUTHORIZED pool signer.
            // Without this, any caller could supply two arbitrary accounts they
            // control, witness the tx, and drain the contract's escrowed pool.
            require(_multisigSigners[signer], "NEP17: signer not authorized");

            // Prevent duplicate signers from satisfying quorum multiple times.
            for (uint256 j = 0; j < i; j++) {
                require(signers[j] != signer, "NEP17: duplicate signer");
            }

            require(Runtime.checkWitness(signer), "NEP17: signer witness missing");
        }
        
        // Execute transfer from multisig pool
        _transfer(address(this), to, amount, abi.encode("multisig", signers));
    }
    
    /**
     * @dev Conditional transfer based on oracle data
     */
    function conditionalTransfer(
        address to,
        uint256 amount,
        string memory oracleUrl,
        string memory condition
    ) public whenTransfersEnabled validReceiver(to) validAmount(amount) {
        // Escrow tokens in contract until condition is met
        _transfer(msg.sender, address(this), amount, "");
        
        // Create request id and escrow record.
        // Include a monotonic nonce to avoid collisions for repeated same-params requests.
        uint256 nonce = _conditionalTransferNonce++;
        bytes32 requestId = keccak256(abi.encode(msg.sender, to, amount, condition, block.timestamp, nonce));

        // Store pending transfer by local request id so callbacks can be validated and replay-protected.
        Storage.put(
            abi.encode("conditional_transfer", requestId),
            abi.encode(msg.sender, to, amount)
        );

        // Pass only the local request id through oracle userData; callback must load escrowed state.
        Syscalls.oracleRequest(
            oracleUrl,
            condition,
            "conditionalTransferCallback",
            abi.encode(requestId),
            10000000
        );

        emit ConditionalTransferCreated(requestId, msg.sender, to, amount);
    }
    
    /**
     * @dev Oracle callback for conditional transfers
     */
    // The Neo N3 Oracle native contract invokes the registered callback with the
    // fixed argument order `(string url, bytes userData, int code, bytes result)`.
    // The previous declaration had these in the wrong order, so `userData` (which
    // carries our local request id) was read from the wrong slot and the escrowed
    // tokens could never be released — permanently locking them.
    function conditionalTransferCallback(
        string calldata url,
        bytes calldata userData,
        uint256 code,
        bytes calldata result
    ) external {
        url; // oracle request URL (reserved for diagnostics)
        require(msg.sender == ORACLE_NATIVE_CONTRACT, "NEP17: unauthorized callback");

        bytes32 localRequestId = abi.decode(userData, (bytes32));
        bytes memory pending = Storage.get(abi.encode("conditional_transfer", localRequestId));
        require(pending.length > 0, "NEP17: conditional transfer not found");

        (address from, address to, uint256 amount) = abi.decode(pending, (address, address, uint256));

        // Consume request before state transitions to prevent callback replay.
        Storage.remove(abi.encode("conditional_transfer", localRequestId));

        // The oracle `result` is the filtered response body, NOT necessarily a
        // 32-byte ABI word. `abi.decode(result, (bool))` on a shorter body
        // Panics (0x41) and would revert this callback AFTER the request record
        // was consumed above — permanently stranding the escrow. Short-circuit
        // on `result.length >= 32` so an unparsable/short result falls through
        // to the refund branch instead of reverting.
        if (code == 0 && result.length >= 32 && abi.decode(result, (bool))) {
            _transfer(address(this), to, amount, "");
            emit ConditionalTransferExecuted(from, to, amount);
        } else {
            // Refund escrowed tokens on oracle errors or unmet conditions.
            _transfer(address(this), from, amount, "");
            emit ConditionalTransferFailed(from, amount);
        }
    }
    
    // NOTE: getAllBalances() was removed for the same reason as getHoldersCount().
    // Solidity mappings use keccak256-derived storage slots and cannot be iterated
    // via Storage.find() with a prefix. See the comment above getHoldersCount's
    // former location for alternatives.
    
    /**
     * @dev NEP-17 specific metadata
     */
    function nep17Metadata() public view returns (
        string memory standard,
        bytes memory logo,
        string memory website,
        string memory description
    ) {
        return (
            "NEP-17",
            "", // Logo data (optional)
            "https://r3e.network",
            string(abi.encodePacked("NEP-17 token: ", _name))
        );
    }
}
