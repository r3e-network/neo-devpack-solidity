using System.Numerics;
using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime;

namespace Neo.Sol.Examples;

/// <summary>
/// ERC20-compatible token implementation using Neo.Sol.Runtime
/// Demonstrates Solidity-style development patterns on Neo
/// </summary>
[DisplayName("ERC20Token")]
[ManifestExtra("Description", "ERC20-compatible token using EVM runtime")]
[ManifestExtra("Author", "Neo Project")]
[ContractPermission("*", "*")]
public class ERC20Token : SmartContract
{
    private static readonly EvmRuntime runtime = Evm.CreateRuntime();
    
    // Storage slots (following Solidity layout)
    private static readonly BigInteger TOTAL_SUPPLY_SLOT = 0;
    private static readonly BigInteger BALANCES_SLOT = 1;        // mapping(address => uint256)
    private static readonly BigInteger ALLOWANCES_SLOT = 2;     // mapping(address => mapping(address => uint256))
    
    // Token metadata
    private const string NAME = "Neo DevPack for Solidity Token";
    private const string SYMBOL = "NST";
    private const byte DECIMALS = 18;
    
    /// <summary>
    /// Contract initialization
    /// </summary>
    /// <param name="owner">Initial token owner</param>
    /// <param name="initialSupply">Initial token supply</param>
    [DisplayName("_deploy")]
    public static void Deploy(object data, bool update)
    {
        if (update) return;
        
        var deployData = (object[])data;
        var owner = (UInt160)deployData[0];
        var initialSupply = (BigInteger)deployData[1];
        
        // Validate parameters
        runtime.Require(owner.IsValid && !owner.IsZero, "Invalid owner address");
        runtime.Require(initialSupply > 0, "Initial supply must be positive");
        
        // Set total supply
        runtime.Storage.Store(TOTAL_SUPPLY_SLOT, initialSupply);
        
        // Mint initial supply to owner
        var balanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, owner.ToArray());
        runtime.Storage.Store(balanceSlot, initialSupply);
        
        // Register contract
        var contractInfo = new ContractInfo
        {
            Name = NAME,
            Version = "1.0.0",
            Description = "ERC20-compatible token",
            Owner = owner,
            IsActive = true,
            CreatedAt = runtime.Now
        };
        runtime.Registry.RegisterContract(runtime.ContractAddress, contractInfo);
        
        // Emit Transfer event (from zero address)
        StandardEvents.EmitTransfer(runtime.Events, UInt160.Zero, owner, initialSupply);
    }
    
    /// <summary>
    /// Get token name
    /// </summary>
    [DisplayName("name")]
    [Safe]
    public static string Name() => NAME;
    
    /// <summary>
    /// Get token symbol
    /// </summary>
    [DisplayName("symbol")]
    [Safe]
    public static string Symbol() => SYMBOL;
    
    /// <summary>
    /// Get token decimals
    /// </summary>
    [DisplayName("decimals")]
    [Safe]
    public static byte Decimals() => DECIMALS;
    
    /// <summary>
    /// Get total token supply
    /// </summary>
    [DisplayName("totalSupply")]
    [Safe]
    public static BigInteger TotalSupply()
    {
        return runtime.Storage.LoadBigInteger(TOTAL_SUPPLY_SLOT);
    }
    
    /// <summary>
    /// Get balance of account
    /// </summary>
    /// <param name="account">Account address</param>
    /// <returns>Token balance</returns>
    [DisplayName("balanceOf")]
    [Safe]
    public static BigInteger BalanceOf(UInt160 account)
    {
        runtime.Require(account.IsValid, "Invalid account address");
        
        var balanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, account.ToArray());
        return runtime.Storage.LoadBigInteger(balanceSlot);
    }
    
    /// <summary>
    /// Transfer tokens to recipient
    /// </summary>
    /// <param name="to">Recipient address</param>
    /// <param name="amount">Amount to transfer</param>
    /// <returns>True if successful</returns>
    [DisplayName("transfer")]
    public static bool Transfer(UInt160 to, BigInteger amount)
    {
        var from = runtime.Msg.Sender;
        return TransferInternal(from, to, amount);
    }
    
    /// <summary>
    /// Transfer tokens from one account to another (with allowance)
    /// </summary>
    /// <param name="from">Source address</param>
    /// <param name="to">Recipient address</param>
    /// <param name="amount">Amount to transfer</param>
    /// <returns>True if successful</returns>
    [DisplayName("transferFrom")]
    public static bool TransferFrom(UInt160 from, UInt160 to, BigInteger amount)
    {
        var spender = runtime.Msg.Sender;
        
        // Check allowance
        var allowance = AllowanceInternal(from, spender);
        runtime.Require(allowance >= amount, "Transfer amount exceeds allowance");
        
        // Update allowance
        var newAllowance = allowance - amount;
        SetAllowanceInternal(from, spender, newAllowance);
        
        // Emit Approval event for allowance update
        StandardEvents.EmitApproval(runtime.Events, from, spender, newAllowance);
        
        // Perform transfer
        return TransferInternal(from, to, amount);
    }
    
    /// <summary>
    /// Approve spender to spend tokens
    /// </summary>
    /// <param name="spender">Spender address</param>
    /// <param name="amount">Amount to approve</param>
    /// <returns>True if successful</returns>
    [DisplayName("approve")]
    public static bool Approve(UInt160 spender, BigInteger amount)
    {
        var owner = runtime.Msg.Sender;
        
        runtime.Require(owner.IsValid, "Invalid owner address");
        runtime.Require(spender.IsValid, "Invalid spender address");
        runtime.Require(amount >= 0, "Approval amount cannot be negative");
        
        SetAllowanceInternal(owner, spender, amount);
        
        // Emit Approval event
        StandardEvents.EmitApproval(runtime.Events, owner, spender, amount);
        
        return true;
    }
    
    /// <summary>
    /// Get allowance amount
    /// </summary>
    /// <param name="owner">Owner address</param>
    /// <param name="spender">Spender address</param>
    /// <returns>Allowance amount</returns>
    [DisplayName("allowance")]
    [Safe]
    public static BigInteger Allowance(UInt160 owner, UInt160 spender)
    {
        return AllowanceInternal(owner, spender);
    }
    
    /// <summary>
    /// Increase allowance (safer than direct approve)
    /// </summary>
    /// <param name="spender">Spender address</param>
    /// <param name="addedValue">Amount to add to allowance</param>
    /// <returns>True if successful</returns>
    [DisplayName("increaseAllowance")]
    public static bool IncreaseAllowance(UInt160 spender, BigInteger addedValue)
    {
        var owner = runtime.Msg.Sender;
        var currentAllowance = AllowanceInternal(owner, spender);
        var newAllowance = currentAllowance + addedValue;
        
        return Approve(spender, newAllowance);
    }
    
    /// <summary>
    /// Decrease allowance (safer than direct approve)
    /// </summary>
    /// <param name="spender">Spender address</param>
    /// <param name="subtractedValue">Amount to subtract from allowance</param>
    /// <returns>True if successful</returns>
    [DisplayName("decreaseAllowance")]
    public static bool DecreaseAllowance(UInt160 spender, BigInteger subtractedValue)
    {
        var owner = runtime.Msg.Sender;
        var currentAllowance = AllowanceInternal(owner, spender);
        
        runtime.Require(currentAllowance >= subtractedValue, "Decreased allowance below zero");
        
        var newAllowance = currentAllowance - subtractedValue;
        return Approve(spender, newAllowance);
    }
    
    /// <summary>
    /// Mint new tokens (owner only)
    /// </summary>
    /// <param name="to">Recipient address</param>
    /// <param name="amount">Amount to mint</param>
    /// <returns>True if successful</returns>
    [DisplayName("mint")]
    public static bool Mint(UInt160 to, BigInteger amount)
    {
        var contractInfo = runtime.Registry.GetContractInfo(runtime.ContractAddress);
        runtime.Require(contractInfo != null && contractInfo.Owner.Equals(runtime.Msg.Sender), 
                       "Only owner can mint tokens");
        
        runtime.Require(to.IsValid && !to.IsZero, "Invalid recipient address");
        runtime.Require(amount > 0, "Mint amount must be positive");
        
        // Update total supply
        var totalSupply = TotalSupply();
        var newTotalSupply = totalSupply + amount;
        runtime.Storage.Store(TOTAL_SUPPLY_SLOT, newTotalSupply);
        
        // Update recipient balance
        var balanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, to.ToArray());
        var currentBalance = runtime.Storage.LoadBigInteger(balanceSlot);
        var newBalance = currentBalance + amount;
        runtime.Storage.Store(balanceSlot, newBalance);
        
        // Emit Transfer event (from zero address)
        StandardEvents.EmitTransfer(runtime.Events, UInt160.Zero, to, amount);
        
        return true;
    }
    
    /// <summary>
    /// Burn tokens from caller's balance
    /// </summary>
    /// <param name="amount">Amount to burn</param>
    /// <returns>True if successful</returns>
    [DisplayName("burn")]
    public static bool Burn(BigInteger amount)
    {
        var from = runtime.Msg.Sender;
        
        runtime.Require(amount > 0, "Burn amount must be positive");
        
        // Check balance
        var balanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, from.ToArray());
        var currentBalance = runtime.Storage.LoadBigInteger(balanceSlot);
        runtime.Require(currentBalance >= amount, "Burn amount exceeds balance");
        
        // Update balance
        var newBalance = currentBalance - amount;
        runtime.Storage.Store(balanceSlot, newBalance);
        
        // Update total supply
        var totalSupply = TotalSupply();
        var newTotalSupply = totalSupply - amount;
        runtime.Storage.Store(TOTAL_SUPPLY_SLOT, newTotalSupply);
        
        // Emit Transfer event (to zero address)
        StandardEvents.EmitTransfer(runtime.Events, from, UInt160.Zero, amount);
        
        return true;
    }
    
    // Internal helper methods
    
    private static bool TransferInternal(UInt160 from, UInt160 to, BigInteger amount)
    {
        runtime.Require(from.IsValid, "Invalid from address");
        runtime.Require(to.IsValid && !to.IsZero, "Invalid to address");
        runtime.Require(amount > 0, "Transfer amount must be positive");
        
        // Get balance slots
        var fromBalanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, from.ToArray());
        var toBalanceSlot = StorageManager.CalculateMappingElementSlot(BALANCES_SLOT, to.ToArray());
        
        // Check from balance
        var fromBalance = runtime.Storage.LoadBigInteger(fromBalanceSlot);
        runtime.Require(fromBalance >= amount, "Transfer amount exceeds balance");
        
        // Update balances
        var newFromBalance = fromBalance - amount;
        runtime.Storage.Store(fromBalanceSlot, newFromBalance);
        
        var toBalance = runtime.Storage.LoadBigInteger(toBalanceSlot);
        var newToBalance = toBalance + amount;
        runtime.Storage.Store(toBalanceSlot, newToBalance);
        
        // Emit Transfer event
        StandardEvents.EmitTransfer(runtime.Events, from, to, amount);
        
        return true;
    }
    
    private static BigInteger AllowanceInternal(UInt160 owner, UInt160 spender)
    {
        runtime.Require(owner.IsValid, "Invalid owner address");
        runtime.Require(spender.IsValid, "Invalid spender address");
        
        // Calculate nested mapping slot: allowances[owner][spender]
        var ownerSlot = StorageManager.CalculateMappingElementSlot(ALLOWANCES_SLOT, owner.ToArray());
        var allowanceSlot = StorageManager.CalculateMappingElementSlot(ownerSlot, spender.ToArray());
        
        return runtime.Storage.LoadBigInteger(allowanceSlot);
    }
    
    private static void SetAllowanceInternal(UInt160 owner, UInt160 spender, BigInteger amount)
    {
        // Calculate nested mapping slot: allowances[owner][spender]
        var ownerSlot = StorageManager.CalculateMappingElementSlot(ALLOWANCES_SLOT, owner.ToArray());
        var allowanceSlot = StorageManager.CalculateMappingElementSlot(ownerSlot, spender.ToArray());
        
        runtime.Storage.Store(allowanceSlot, amount);
    }
    
    /// <summary>
    /// Get contract runtime statistics (for debugging)
    /// </summary>
    [DisplayName("getStats")]
    [Safe]
    public static object GetStats()
    {
        var stats = runtime.GetStats();
        return new object[]
        {
            stats.MemoryStats.TotalSize,
            stats.StorageStats.CachedSlots,
            stats.RegistryStats.TotalContracts,
            stats.GasUsed
        };
    }
}