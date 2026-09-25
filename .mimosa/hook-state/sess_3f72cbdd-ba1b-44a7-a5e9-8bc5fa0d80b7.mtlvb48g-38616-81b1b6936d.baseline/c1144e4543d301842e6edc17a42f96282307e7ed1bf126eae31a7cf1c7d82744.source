using System.Collections.Concurrent;
using System.Numerics;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime;
using Neo.Sol.Runtime.Crypto;
using NeoFrameworkRuntime = Neo.SmartContract.Framework.Services.Runtime;
using NeoFrameworkStorage = Neo.SmartContract.Framework.Services.Storage;

namespace Neo.Sol.Runtime.Registry;

/// <summary>
/// Registry for managing contract addresses and cross-contract interactions
/// Provides address resolution, contract metadata, and deployment tracking
/// </summary>
public sealed class AddressRegistry
{
    private readonly StorageContext _context;
    private readonly ConcurrentDictionary<UInt160, ContractInfo> _cache = new();
    
    // Storage prefixes for different registry data
    private const byte CONTRACT_INFO_PREFIX = 0x01;
    private const byte ADDRESS_MAPPING_PREFIX = 0x02;
    private const byte INTERFACE_REGISTRY_PREFIX = 0x03;
    private const byte ENS_REGISTRY_PREFIX = 0x04;
    
    public AddressRegistry(StorageContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    private static void TryLog(string message)
    {
        try
        {
            NeoFrameworkRuntime.Log(message);
        }
        catch
        {
            // Ignore logging failures outside a Neo VM host.
        }
    }
    
    /// <summary>
    /// Register a new contract in the registry
    /// </summary>
    /// <param name="address">Contract address</param>
    /// <param name="info">Contract information</param>
    public void RegisterContract(UInt160 address, ContractInfo info)
    {
        ValidateAddress(address);
        
        var key = CreateStorageKey(CONTRACT_INFO_PREFIX, address);
        var serializedInfo = SerializeContractInfo(info);
        
        NeoFrameworkStorage.Put(_context, key, serializedInfo);
        _cache[address] = info;
        
        // Emit registration event
        TryLog($"ContractRegistered:{info.Name}:{info.Version}");
    }
    
    /// <summary>
    /// Get contract information by address
    /// </summary>
    /// <param name="address">Contract address</param>
    /// <returns>Contract information or null if not found</returns>
    public ContractInfo? GetContractInfo(UInt160 address)
    {
        // Check cache first
        if (_cache.TryGetValue(address, out var cachedInfo))
            return cachedInfo;
            
        var key = CreateStorageKey(CONTRACT_INFO_PREFIX, address);
        var data = NeoFrameworkStorage.Get(_context, key);
        
        if (data == null) return null;
        
        var info = DeserializeContractInfo((byte[])data);
        _cache[address] = info;
        return info;
    }
    
    /// <summary>
    /// Check if contract is registered and active
    /// </summary>
    /// <param name="address">Contract address</param>
    /// <returns>True if contract is registered and active</returns>
    public bool IsContractRegistered(UInt160 address)
    {
        var info = GetContractInfo(address);
        return info != null && info.IsActive;
    }
    
    /// <summary>
    /// Register an interface implementation
    /// </summary>
    /// <param name="contractAddress">Contract address</param>
    /// <param name="interfaceId">Interface identifier (EIP-165)</param>
    /// <param name="isSupported">Whether interface is supported</param>
    public void RegisterInterface(UInt160 contractAddress, byte[] interfaceId, bool isSupported)
    {
        ValidateAddress(contractAddress);
        ValidateInterfaceId(interfaceId);
        
        var key = CreateInterfaceKey(contractAddress, interfaceId);
        
        if (isSupported)
        {
            NeoFrameworkStorage.Put(_context, key, new byte[] { 1 });
        }
        else
        {
            NeoFrameworkStorage.Delete(_context, key);
        }
        
        TryLog($"InterfaceRegistered:{contractAddress}");
    }
    
    /// <summary>
    /// Check if contract supports interface (EIP-165)
    /// </summary>
    /// <param name="contractAddress">Contract address</param>
    /// <param name="interfaceId">Interface identifier</param>
    /// <returns>True if interface is supported</returns>
    public bool SupportsInterface(UInt160 contractAddress, byte[] interfaceId)
    {
        var key = CreateInterfaceKey(contractAddress, interfaceId);
        var data = NeoFrameworkStorage.Get(_context, key);
        return data != null && data.Length > 0 && data[0] == 1;
    }
    
    /// <summary>
    /// Register name-to-address mapping (ENS-style)
    /// </summary>
    /// <param name="name">Domain name</param>
    /// <param name="address">Associated address</param>
    /// <param name="owner">Owner of the name</param>
    public void RegisterName(string name, UInt160 address, UInt160 owner)
    {
        ValidateName(name);
        ValidateAddress(address);
        ValidateAddress(owner);
        
        // Check ownership or registration permission
        if (!CanRegisterName(name, owner))
        {
            throw new UnauthorizedAccessException($"Not authorized to register name: {name}");
        }
        
        var nameHash = CalculateNameHash(name);
        var key = CreateStorageKey(ENS_REGISTRY_PREFIX, nameHash);
        
        var record = new NameRecord
        {
            Name = name,
            Address = address,
            Owner = owner,
            RegisteredAt = NeoFrameworkRuntime.Time,
            ExpiresAt = NeoFrameworkRuntime.Time + 365UL * 24 * 60 * 60 * 1000, // 1 year
            IsActive = true
        };
        
        NeoFrameworkStorage.Put(_context, key, SerializeNameRecord(record));
        
        // Create reverse mapping
        var reverseKey = CreateStorageKey(ADDRESS_MAPPING_PREFIX, address);
        NeoFrameworkStorage.Put(_context, reverseKey, System.Text.Encoding.UTF8.GetBytes(name));
        
        TryLog($"NameRegistered:{name}");
    }
    
    /// <summary>
    /// Resolve name to address
    /// </summary>
    /// <param name="name">Domain name</param>
    /// <returns>Associated address or UInt160.Zero if not found</returns>
    public UInt160 ResolveName(string name)
    {
        var nameHash = CalculateNameHash(name);
        var key = CreateStorageKey(ENS_REGISTRY_PREFIX, nameHash);
        var data = NeoFrameworkStorage.Get(_context, key);
        
        if (data == null) return UInt160.Zero;
        
        var record = DeserializeNameRecord((byte[])data);
        
        // Check if record is still valid
        if (!record.IsActive || NeoFrameworkRuntime.Time > record.ExpiresAt)
            return UInt160.Zero;
            
        return record.Address;
    }
    
    /// <summary>
    /// Get name associated with address (reverse lookup)
    /// </summary>
    /// <param name="address">Address</param>
    /// <returns>Associated name or empty string if not found</returns>
    public string GetAddressName(UInt160 address)
    {
        var key = CreateStorageKey(ADDRESS_MAPPING_PREFIX, address);
        var data = NeoFrameworkStorage.Get(_context, key);
        
        return data != null ? System.Text.Encoding.UTF8.GetString((byte[])data) : "";
    }
    
    /// <summary>
    /// Update contract status
    /// </summary>
    /// <param name="address">Contract address</param>
    /// <param name="isActive">New active status</param>
    /// <param name="updater">Address performing the update</param>
    public void UpdateContractStatus(UInt160 address, bool isActive, UInt160 updater)
    {
        var info = GetContractInfo(address);
        if (info == null)
            throw new ArgumentException("Contract not registered");
            
        // Check permission to update
        if (!CanUpdateContract(address, updater))
            throw new UnauthorizedAccessException("Not authorized to update contract");
            
        info.IsActive = isActive;
        info.UpdatedAt = NeoFrameworkRuntime.Time;
        
        RegisterContract(address, info);
        
        TryLog($"ContractStatusUpdated:{address}:{isActive}");
    }
    
    /// <summary>
    /// Get contracts by interface
    /// </summary>
    /// <param name="interfaceId">Interface identifier</param>
    /// <returns>List of contract addresses supporting the interface</returns>
    public UInt160[] GetContractsByInterface(byte[] interfaceId)
    {
        ValidateInterfaceId(interfaceId);
        
        var contracts = new System.Collections.Generic.List<UInt160>();
        
        try
        {
            // Create interface registry key prefix for efficient searching
            // Key format: [PREFIX][INTERFACE_ID][CONTRACT_ADDRESS]
            var prefixKey = new byte[1 + interfaceId.Length];
            prefixKey[0] = INTERFACE_REGISTRY_PREFIX;
            Array.Copy(interfaceId, 0, prefixKey, 1, interfaceId.Length);
            
            // Use Neo storage iterator to find all contracts supporting this interface
            var iterator = NeoFrameworkStorage.Find(_context, prefixKey, FindOptions.None);
            
            while (iterator.Next())
            {
                var entry = (object[])iterator.Value;
                var key = (byte[])entry[0];
                var value = (byte[])entry[1];
                
                // Verify this is a valid interface registration
                if (key.Length == 1 + interfaceId.Length + 20 && 
                    value.Length > 0 && value[0] == 1)
                {
                    // Extract contract address from the key
                    var addressBytes = new byte[20];
                    Array.Copy(key, 1 + interfaceId.Length, addressBytes, 0, 20);
                    var contractAddress = new UInt160(addressBytes);
                    
                    // Verify contract is still active
                    var contractInfo = GetContractInfo(contractAddress);
                    if (contractInfo != null && contractInfo.IsActive)
                    {
                        contracts.Add(contractAddress);
                    }
                }
            }
            
            // Sort addresses for deterministic results
            contracts.Sort((a, b) => string.Compare(a.ToString(), b.ToString(), StringComparison.Ordinal));
            
            return contracts.ToArray();
        }
        catch (Exception ex)
        {
            TryLog($"Error getting contracts by interface {Convert.ToHexString(interfaceId)}: {ex.Message}");
            return Array.Empty<UInt160>();
        }
    }
    
    /// <summary>
    /// Batch register multiple contracts
    /// </summary>
    /// <param name="registrations">Contract registrations</param>
    public void BatchRegisterContracts(IEnumerable<ContractRegistration> registrations)
    {
        var registrationList = registrations.ToList();
        
        if (registrationList.Count == 0)
            return;
            
        try
        {
            foreach (var registration in registrationList)
            {
                RegisterContract(registration.Address, registration.Info);
            }
            
            TryLog($"BatchContractsRegistered:{registrationList.Count}:{NeoFrameworkRuntime.Time}");
        }
        catch (Exception ex)
        {
            TryLog($"Error in batch registration: {ex.Message}");
            throw;
        }
    }
    
    /// <summary>
    /// Get registry statistics
    /// </summary>
    /// <returns>Registry statistics</returns>
    public RegistryStats GetStats()
    {
        return new RegistryStats
        {
            TotalContracts = CountRegisteredContracts(),
            ActiveContracts = CountActiveContracts(),
            RegisteredNames = CountRegisteredNames(),
            CacheSize = (uint)_cache.Count
        };
    }
    
    // Helper methods
    
    private byte[] CreateStorageKey(byte prefix, UInt160 address)
    {
        var key = new byte[21];
        key[0] = prefix;
        Array.Copy(NeoTypeConversions.ToByteArray(address), 0, key, 1, 20);
        return key;
    }
    
    private byte[] CreateStorageKey(byte prefix, byte[] hash)
    {
        var key = new byte[1 + hash.Length];
        key[0] = prefix;
        Array.Copy(hash, 0, key, 1, hash.Length);
        return key;
    }
    
    private byte[] CreateInterfaceKey(UInt160 contractAddress, byte[] interfaceId)
    {
        var addressBytes = NeoTypeConversions.ToByteArray(contractAddress);
        var key = new byte[1 + interfaceId.Length + addressBytes.Length];
        key[0] = INTERFACE_REGISTRY_PREFIX;
        Array.Copy(interfaceId, 0, key, 1, interfaceId.Length);
        Array.Copy(addressBytes, 0, key, 1 + interfaceId.Length, addressBytes.Length);
        return key;
    }
    
    private byte[] CalculateNameHash(string name)
    {
        // ENS-style name hashing (simplified)
        return CryptoLib.Keccak256(System.Text.Encoding.UTF8.GetBytes(name.ToLowerInvariant()));
    }
    
    private bool CanRegisterName(string name, UInt160 owner)
    {
        // Check if name is already registered
        var existingAddress = ResolveName(name);
        if (existingAddress != UInt160.Zero)
        {
            // Name is already registered, check if caller is the owner
            var nameHash = CalculateNameHash(name);
            var key = CreateStorageKey(ENS_REGISTRY_PREFIX, nameHash);
            var data = NeoFrameworkStorage.Get(_context, key);
            
            if (data != null)
            {
                var record = DeserializeNameRecord((byte[])data);
                return record.Owner.Equals(owner);
            }
        }
        
        return true; // Name is available
    }
    
    private bool CanUpdateContract(UInt160 address, UInt160 updater)
    {
        var info = GetContractInfo(address);
        if (info == null) return false;
        
        // Check if updater is the owner or has admin rights
        return info.Owner.Equals(updater) || info.Admins.Contains(updater);
    }
    
    private static void ValidateAddress(UInt160 address)
    {
        if (address.Equals(UInt160.Zero))
            throw new ArgumentException("Invalid address: zero address");
    }
    
    private static void ValidateName(string name)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Name cannot be null or empty");
        if (name.Length > 255)
            throw new ArgumentException("Name too long");
    }
    
    private static void ValidateInterfaceId(byte[] interfaceId)
    {
        if (interfaceId == null || interfaceId.Length != 4)
            throw new ArgumentException("Interface ID must be 4 bytes");
    }
    
    private uint CountRegisteredContracts()
    {
        try
        {
            uint count = 0;
            
            // Create storage key for contract registry
            var contractPrefix = new byte[] { CONTRACT_INFO_PREFIX };
            
            // Use Neo storage iterator to count registered contracts
            var iterator = NeoFrameworkStorage.Find(_context, contractPrefix, FindOptions.None);
            
            while (iterator.Next())
            {
                var value = (byte[])((object[])iterator.Value)[1];
                
                if (value.Length > 0)
                {
                    try
                    {
                        var contractInfo = DeserializeContractInfo(value);
                        count++;
                    }
                    catch
                    {
                        // Skip invalid records
                        continue;
                    }
                }
            }
            
            return count;
        }
        catch (Exception ex)
        {
            TryLog($"Error counting registered contracts: {ex.Message}");
            // Fallback to cache count
            return (uint)_cache.Count;
        }
    }
    
    private uint CountActiveContracts()
    {
        try
        {
            uint count = 0;
            
            // Create storage key for contract registry
            var contractPrefix = new byte[] { CONTRACT_INFO_PREFIX };
            
            // Use Neo storage iterator to count active contracts
            var iterator = NeoFrameworkStorage.Find(_context, contractPrefix, FindOptions.None);
            
            while (iterator.Next())
            {
                var value = (byte[])((object[])iterator.Value)[1];
                
                if (value.Length > 0)
                {
                    try
                    {
                        var contractInfo = DeserializeContractInfo(value);
                        if (contractInfo.IsActive)
                        {
                            count++;
                        }
                    }
                    catch
                    {
                        // Skip invalid records
                        continue;
                    }
                }
            }
            
            return count;
        }
        catch (Exception ex)
        {
            TryLog($"Error counting active contracts: {ex.Message}");
            // Fallback to cache count
            return (uint)_cache.Values.Count(c => c.IsActive);
        }
    }
    
    private uint CountRegisteredNames()
    {
        try
        {
            uint count = 0;
            
            // Create storage key for name registry
            var namePrefix = new byte[] { ENS_REGISTRY_PREFIX };
            
            // Use Neo storage iterator to count active name registrations
            var iterator = NeoFrameworkStorage.Find(_context, namePrefix, FindOptions.None);
            
            while (iterator.Next())
            {
                var value = (byte[])((object[])iterator.Value)[1];
                
                if (value.Length > 0)
                {
                    try
                    {
                        var record = DeserializeNameRecord(value);
                        
                        // Only count active, non-expired names
                        if (record.IsActive && NeoFrameworkRuntime.Time <= record.ExpiresAt)
                        {
                            count++;
                        }
                    }
                    catch
                    {
                        // Skip invalid records
                        continue;
                    }
                }
            }
            
            return count;
        }
        catch (Exception ex)
        {
            TryLog($"Error counting registered names: {ex.Message}");
            return 0;
        }
    }
    
    // Serialization methods (simplified)
    
    private byte[] SerializeContractInfo(ContractInfo info)
    {
        // This would use a proper serialization format like JSON or protobuf
        // For now, using a simple format
        var data = System.Text.Json.JsonSerializer.SerializeToUtf8Bytes(info);
        return data;
    }
    
    private ContractInfo DeserializeContractInfo(byte[] data)
    {
        var info = System.Text.Json.JsonSerializer.Deserialize<ContractInfo>(data);
        return info ?? new ContractInfo();
    }
    
    private byte[] SerializeNameRecord(NameRecord record)
    {
        var data = System.Text.Json.JsonSerializer.SerializeToUtf8Bytes(record);
        return data;
    }
    
    private NameRecord DeserializeNameRecord(byte[] data)
    {
        var record = System.Text.Json.JsonSerializer.Deserialize<NameRecord>(data);
        return record ?? new NameRecord();
    }
}

/// <summary>
/// Information about a registered contract
/// </summary>
public sealed class ContractInfo
{
    public string Name { get; set; } = "";
    public string Version { get; set; } = "";
    public string Description { get; set; } = "";
    public UInt160 Owner { get; set; } = UInt160.Zero;
    public UInt160[] Admins { get; set; } = Array.Empty<UInt160>();
    public string[] Tags { get; set; } = Array.Empty<string>();
    public bool IsActive { get; set; } = true;
    public ulong CreatedAt { get; set; }
    public ulong UpdatedAt { get; set; }
    public Dictionary<string, string> Metadata { get; set; } = new();
}

/// <summary>
/// Name registration record
/// </summary>
public sealed class NameRecord
{
    public string Name { get; set; } = "";
    public UInt160 Address { get; set; } = UInt160.Zero;
    public UInt160 Owner { get; set; } = UInt160.Zero;
    public ulong RegisteredAt { get; set; }
    public ulong ExpiresAt { get; set; }
    public bool IsActive { get; set; } = true;
}

/// <summary>
/// Contract registration data
/// </summary>
public sealed record ContractRegistration(UInt160 Address, ContractInfo Info);

/// <summary>
/// Registry statistics
/// </summary>
public sealed record RegistryStats
{
    public uint TotalContracts { get; init; }
    public uint ActiveContracts { get; init; }
    public uint RegisteredNames { get; init; }
    public uint CacheSize { get; init; }
}

/// <summary>
/// Standard interface identifiers (EIP-165)
/// </summary>
public static class StandardInterfaces
{
    public static readonly byte[] ERC165 = new byte[] { 0x01, 0xff, 0xc9, 0xa7 };
    public static readonly byte[] ERC20 = new byte[] { 0x36, 0x37, 0x2b, 0x07 };
    public static readonly byte[] ERC721 = new byte[] { 0x80, 0xac, 0x58, 0xcd };
    public static readonly byte[] ERC1155 = new byte[] { 0xd9, 0xb6, 0x7a, 0x26 };
    public static readonly byte[] ERC2981 = new byte[] { 0x2a, 0x55, 0x20, 0x5a }; // Royalties
}
