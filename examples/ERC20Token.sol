// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ERC20Token
 * @dev NEP-17 compliant fungible token for Neo N3 blockchain.
 *
 * Compiler constraints respected:
 *   - NEP-17 transfer(from, to, amount, data) with Runtime.checkWitness(from)
 *   - No {value: ...} — uses NativeCalls where needed
 *   - No receive()/fallback() — uses onNEP17Payment() callback
 *   - Parameterless constructor (Neo deploy constraint)
 *   - Import devpack via -I devpack
 *
 * Features: transfers, allowances, minting, burning, pausable, ownership
 */
contract ERC20Token {
    // State variables
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    
    uint256 private _totalSupply;
    uint8 private _decimals;
    string private _name;
    string private _symbol;
    address private _owner;
    bool private _paused;
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Mint(address indexed to, uint256 value);
    event Burn(address indexed from, uint256 value);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event Pause();
    event Unpause();
    
    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == _owner, "ERC20: caller is not the owner");
        _;
    }
    
    modifier whenNotPaused() {
        require(!_paused, "ERC20: token transfer while paused");
        _;
    }
    
    modifier validAddress(address account) {
        require(account != address(0), "ERC20: invalid address");
        _;
    }
    
    bool private _initialized;

    /// @dev Parameterless constructor — Neo N3 deploy constraint.
    /// Sets sensible defaults; call initialize() to customize.
    constructor() {
        _name = "MyToken";
        _symbol = "MTK";
        _decimals = 8;
        _owner = msg.sender;
        _paused = false;
        emit OwnershipTransferred(address(0), msg.sender);
    }

    /// @notice Initialize token metadata. Only owner, only once.
    function initialize(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        uint256 initialSupply
    ) external onlyOwner {
        require(!_initialized, "ERC20: already initialized");
        require(bytes(name_).length > 0, "ERC20: name cannot be empty");
        require(bytes(symbol_).length > 0, "ERC20: symbol cannot be empty");
        require(decimals_ <= 18, "ERC20: decimals cannot exceed 18");

        _name = name_;
        _symbol = symbol_;
        _decimals = decimals_;
        _initialized = true;

        if (initialSupply > 0) {
            _mint(msg.sender, initialSupply);
        }
    }
    
    /**
     * @dev Returns the name of the token
     */
    function name() public view returns (string memory) {
        return _name;
    }
    
    /**
     * @dev Returns the symbol of the token
     */
    function symbol() public view returns (string memory) {
        return _symbol;
    }
    
    /**
     * @dev Returns the number of decimals used for token amounts
     */
    function decimals() public view returns (uint8) {
        return _decimals;
    }
    
    /**
     * @dev Returns the total supply of tokens
     */
    function totalSupply() public view returns (uint256) {
        return _totalSupply;
    }
    
    /**
     * @dev Returns the balance of the specified account
     * @param account The address to query the balance of
     */
    function balanceOf(address account) public view returns (uint256) {
        return _balances[account];
    }
    
    /**
     * @dev Returns the remaining number of tokens that spender can spend on behalf of owner
     * @param owner The address that owns the funds
     * @param spender The address that will spend the funds
     */
    function allowance(address owner, address spender) public view returns (uint256) {
        return _allowances[owner][spender];
    }
    
    /**
     * @dev Returns the address of the current owner
     */
    function owner() public view returns (address) {
        return _owner;
    }
    
    /**
     * @dev Returns true if the contract is paused
     */
    function paused() public view returns (bool) {
        return _paused;
    }
    
    /**
     * @dev NEP-17 standard transfer (4-parameter signature).
     * Authorization via Runtime.checkWitness(from) instead of msg.sender.
     * @param from The address to transfer from (must pass witness check)
     * @param to The address to transfer to
     * @param amount The amount to transfer
     * @param data Arbitrary data passed to onNEP17Payment callback
     */
    function transfer(address from, address to, uint256 amount, Any calldata data)
        public
        whenNotPaused
        validAddress(to)
        returns (bool)
    {
        require(Runtime.checkWitness(from), "ERC20: unauthorized");
        _transfer(from, to, amount);
        return true;
    }
    
    /**
     * @dev Approves spender to spend tokens on behalf of caller
     * @param spender The address which will spend the funds
     * @param amount The amount of tokens to approve
     */
    function approve(address spender, uint256 amount) 
        public 
        whenNotPaused 
        validAddress(spender) 
        returns (bool) 
    {
        _approve(msg.sender, spender, amount);
        return true;
    }
    
    /**
     * @dev Transfers tokens from sender to recipient using allowance mechanism
     * @param from The address to transfer from
     * @param to The address to transfer to
     * @param amount The amount to transfer
     */
    function transferFrom(address from, address to, uint256 amount) 
        public 
        whenNotPaused 
        validAddress(from) 
        validAddress(to) 
        returns (bool) 
    {
        uint256 currentAllowance = _allowances[from][msg.sender];
        require(currentAllowance >= amount, "ERC20: transfer amount exceeds allowance");
        
        _transfer(from, to, amount);
        _approve(from, msg.sender, currentAllowance - amount);
        
        return true;
    }
    
    /**
     * @dev Atomically increases the allowance granted to spender
     * @param spender The address which will spend the funds
     * @param addedValue The amount to increase the allowance by
     */
    function increaseAllowance(address spender, uint256 addedValue) 
        public 
        whenNotPaused 
        validAddress(spender) 
        returns (bool) 
    {
        uint256 currentAllowance = _allowances[msg.sender][spender];
        _approve(msg.sender, spender, currentAllowance + addedValue);
        return true;
    }
    
    /**
     * @dev Atomically decreases the allowance granted to spender
     * @param spender The address which will spend the funds
     * @param subtractedValue The amount to decrease the allowance by
     */
    function decreaseAllowance(address spender, uint256 subtractedValue) 
        public 
        whenNotPaused 
        validAddress(spender) 
        returns (bool) 
    {
        uint256 currentAllowance = _allowances[msg.sender][spender];
        require(currentAllowance >= subtractedValue, "ERC20: decreased allowance below zero");
        _approve(msg.sender, spender, currentAllowance - subtractedValue);
        return true;
    }
    
    /**
     * @dev Creates tokens and assigns them to account
     * @param to The address to mint tokens to
     * @param amount The amount of tokens to mint
     */
    function mint(address to, uint256 amount) 
        public 
        onlyOwner 
        validAddress(to) 
    {
        require(amount > 0, "ERC20: mint amount must be greater than 0");
        _mint(to, amount);
    }
    
    /**
     * @dev Destroys tokens from caller's account
     * @param amount The amount of tokens to burn
     */
    function burn(uint256 amount) public whenNotPaused {
        require(amount > 0, "ERC20: burn amount must be greater than 0");
        _burn(msg.sender, amount);
    }
    
    /**
     * @dev Destroys tokens from account using allowance mechanism
     * @param from The address to burn tokens from
     * @param amount The amount of tokens to burn
     */
    function burnFrom(address from, uint256 amount) 
        public 
        whenNotPaused 
        validAddress(from) 
    {
        require(amount > 0, "ERC20: burn amount must be greater than 0");
        
        uint256 currentAllowance = _allowances[from][msg.sender];
        require(currentAllowance >= amount, "ERC20: burn amount exceeds allowance");
        
        _burn(from, amount);
        _approve(from, msg.sender, currentAllowance - amount);
    }
    
    /**
     * @dev Transfers ownership of the contract to a new account
     * @param newOwner The address of the new owner
     */
    function transferOwnership(address newOwner) 
        public 
        onlyOwner 
        validAddress(newOwner) 
    {
        emit OwnershipTransferred(_owner, newOwner);
        _owner = newOwner;
    }
    
    /**
     * @dev Renounces ownership of the contract
     */
    function renounceOwnership() public onlyOwner {
        emit OwnershipTransferred(_owner, address(0));
        _owner = address(0);
    }
    
    /**
     * @dev Pauses all token transfers
     */
    function pause() public onlyOwner {
        require(!_paused, "ERC20: already paused");
        _paused = true;
        emit Pause();
    }
    
    /**
     * @dev Unpauses all token transfers
     */
    function unpause() public onlyOwner {
        require(_paused, "ERC20: not paused");
        _paused = false;
        emit Unpause();
    }
    
    /**
     * @dev Internal transfer function
     * @param from The address to transfer from
     * @param to The address to transfer to  
     * @param amount The amount to transfer
     */
    function _transfer(address from, address to, uint256 amount) internal {
        require(from != address(0), "ERC20: transfer from the zero address");
        require(to != address(0), "ERC20: transfer to the zero address");
        require(amount > 0, "ERC20: transfer amount must be greater than 0");
        
        uint256 fromBalance = _balances[from];
        require(fromBalance >= amount, "ERC20: transfer amount exceeds balance");
        
        unchecked {
            _balances[from] = fromBalance - amount;
        }
        _balances[to] += amount;
        
        emit Transfer(from, to, amount);
    }
    
    /**
     * @dev Internal mint function
     * @param to The address to mint tokens to
     * @param amount The amount to mint
     */
    function _mint(address to, uint256 amount) internal {
        require(to != address(0), "ERC20: mint to the zero address");
        require(amount > 0, "ERC20: mint amount must be greater than 0");
        
        // Check for overflow
        require(_totalSupply + amount >= _totalSupply, "ERC20: total supply overflow");
        
        _totalSupply += amount;
        _balances[to] += amount;
        
        emit Transfer(address(0), to, amount);
        emit Mint(to, amount);
    }
    
    /**
     * @dev Internal burn function
     * @param from The address to burn tokens from
     * @param amount The amount to burn
     */
    function _burn(address from, uint256 amount) internal {
        require(from != address(0), "ERC20: burn from the zero address");
        require(amount > 0, "ERC20: burn amount must be greater than 0");
        
        uint256 accountBalance = _balances[from];
        require(accountBalance >= amount, "ERC20: burn amount exceeds balance");
        
        unchecked {
            _balances[from] = accountBalance - amount;
            _totalSupply -= amount;
        }
        
        emit Transfer(from, address(0), amount);
        emit Burn(from, amount);
    }
    
    /**
     * @dev Internal approve function
     * @param owner The address that owns the funds
     * @param spender The address that will spend the funds
     * @param amount The amount to approve
     */
    function _approve(address owner, address spender, uint256 amount) internal {
        require(owner != address(0), "ERC20: approve from the zero address");
        require(spender != address(0), "ERC20: approve to the zero address");
        
        _allowances[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }
    
    /**
     * @dev Batch transfer to multiple recipients
     * @param recipients Array of recipient addresses
     * @param amounts Array of amounts to transfer
     */
    function batchTransfer(address[] memory recipients, uint256[] memory amounts) 
        public 
        whenNotPaused 
        returns (bool) 
    {
        require(recipients.length == amounts.length, "ERC20: arrays length mismatch");
        require(recipients.length > 0, "ERC20: empty arrays");
        require(recipients.length <= 100, "ERC20: too many recipients");
        
        for (uint256 i = 0; i < recipients.length; i++) {
            require(recipients[i] != address(0), "ERC20: transfer to zero address");
            require(amounts[i] > 0, "ERC20: transfer amount must be greater than 0");
            _transfer(msg.sender, recipients[i], amounts[i]);
        }
        
        return true;
    }
    
    /// @notice NEP-17 callback — receives token payments.
    function onNEP17Payment(address from, uint256 amount, bytes memory /*data*/) external {
        // Accept any NEP-17 token payment; override for custom logic.
    }

    /**
     * @dev Emergency function to recover accidentally sent tokens.
     * Uses abi.encodeWithSignature for Neo-compatible cross-contract call.
     * @param token The address of the token contract to recover from
     * @param amount The amount to recover
     */
    function emergencyTokenRecovery(address token, uint256 amount)
        public
        onlyOwner
        validAddress(token)
    {
        require(token != address(this), "ERC20: cannot recover own tokens");
        require(amount > 0, "ERC20: recovery amount must be greater than 0");

        (bool success, bytes memory data) = token.call(
            abi.encodeWithSignature("transfer(address,address,uint256,bytes)", address(this), _owner, amount, "")
        );

        require(success && (data.length == 0 || abi.decode(data, (bool))),
                "ERC20: token recovery failed");
    }
}