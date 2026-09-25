using System.Numerics;
using FluentAssertions;
using Neo.SmartContract.Framework.Services;
using Neo.Sol.Runtime.Registry;
using NUnit.Framework;

namespace Neo.Sol.Runtime.Tests.Registry;

[TestFixture]
public class AddressRegistryTests
{
    private AddressRegistry _registry = null!;
    private StorageContext _context = null!;

    [SetUp]
    public void SetUp()
    {
        _context = new StorageContext();
        _registry = new AddressRegistry(_context);
    }

    #region Constructor Tests

    [Test]
    public void Constructor_ShouldAcceptValidStorageContext()
    {
        var registry = new AddressRegistry(new StorageContext());
        registry.Should().NotBeNull();
    }

    [Test]
    public void Constructor_ShouldThrowOnNullContext()
    {
        Action act = () => new AddressRegistry(null!);
        act.Should().Throw<ArgumentNullException>()
            .WithParameterName("context");
    }

    #endregion

    #region RegisterContract / GetContractInfo Tests

    [Test]
    public void RegisterContract_ShouldStoreContractInfo()
    {
        var address = CreateTestAddress(1);
        var info = CreateTestContractInfo("TestContract", "1.0.0");

        _registry.RegisterContract(address, info);

        var retrieved = _registry.GetContractInfo(address);
        retrieved.Should().NotBeNull();
        retrieved!.Name.Should().Be("TestContract");
        retrieved.Version.Should().Be("1.0.0");
    }

    [Test]
    public void RegisterContract_ShouldCacheContractInfo()
    {
        var address = CreateTestAddress(1);
        var info = CreateTestContractInfo("CachedContract", "2.0.0");

        _registry.RegisterContract(address, info);

        // First retrieval
        var first = _registry.GetContractInfo(address);
        // Second retrieval should come from cache
        var second = _registry.GetContractInfo(address);

        first.Should().BeSameAs(second);
    }

    [Test]
    public void RegisterContract_ShouldThrowOnZeroAddress()
    {
        var info = CreateTestContractInfo("InvalidContract", "1.0.0");

        Action act = () => _registry.RegisterContract(UInt160.Zero, info);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*zero address*");
    }

    [Test]
    public void GetContractInfo_ShouldReturnNullForUnregisteredContract()
    {
        var address = CreateTestAddress(99);

        var result = _registry.GetContractInfo(address);

        result.Should().BeNull();
    }

    [Test]
    public void IsContractRegistered_ShouldReturnTrueForActiveContract()
    {
        var address = CreateTestAddress(2);
        var info = CreateTestContractInfo("ActiveContract", "1.0.0");
        info.IsActive = true;

        _registry.RegisterContract(address, info);

        _registry.IsContractRegistered(address).Should().BeTrue();
    }

    [Test]
    public void IsContractRegistered_ShouldReturnFalseForInactiveContract()
    {
        var address = CreateTestAddress(3);
        var info = CreateTestContractInfo("InactiveContract", "1.0.0");
        info.IsActive = false;

        _registry.RegisterContract(address, info);

        _registry.IsContractRegistered(address).Should().BeFalse();
    }

    #endregion

    #region RegisterInterface / SupportsInterface Tests (EIP-165)

    [Test]
    public void RegisterInterface_ShouldRegisterSupportedInterface()
    {
        var address = CreateTestAddress(4);
        var interfaceId = StandardInterfaces.ERC20;

        _registry.RegisterInterface(address, interfaceId, true);

        _registry.SupportsInterface(address, interfaceId).Should().BeTrue();
    }

    [Test]
    public void RegisterInterface_ShouldRemoveUnsupportedInterface()
    {
        var address = CreateTestAddress(5);
        var interfaceId = StandardInterfaces.ERC721;

        // Register first
        _registry.RegisterInterface(address, interfaceId, true);
        _registry.SupportsInterface(address, interfaceId).Should().BeTrue();

        // Unregister
        _registry.RegisterInterface(address, interfaceId, false);
        _registry.SupportsInterface(address, interfaceId).Should().BeFalse();
    }

    [Test]
    public void RegisterInterface_ShouldThrowOnZeroAddress()
    {
        var interfaceId = StandardInterfaces.ERC165;

        Action act = () => _registry.RegisterInterface(UInt160.Zero, interfaceId, true);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*zero address*");
    }

    [Test]
    public void RegisterInterface_ShouldThrowOnInvalidInterfaceId()
    {
        var address = CreateTestAddress(6);

        Action act = () => _registry.RegisterInterface(address, new byte[] { 0x01, 0x02 }, true);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*4 bytes*");
    }

    [Test]
    public void SupportsInterface_ShouldReturnFalseForUnregisteredInterface()
    {
        var address = CreateTestAddress(7);
        var interfaceId = StandardInterfaces.ERC1155;

        _registry.SupportsInterface(address, interfaceId).Should().BeFalse();
    }

    #endregion

    #region RegisterName / ResolveName Tests (ENS-style)

    [Test]
    public void RegisterName_ShouldRegisterNameToAddress()
    {
        var address = CreateTestAddress(8);
        var owner = CreateTestAddress(9);
        var name = "test.neo";

        _registry.RegisterName(name, address, owner);

        var resolved = _registry.ResolveName(name);
        resolved.Should().Be(address);
    }

    [Test]
    public void RegisterName_ShouldCreateReverseMapping()
    {
        var address = CreateTestAddress(10);
        var owner = CreateTestAddress(11);
        var name = "reverse.neo";

        _registry.RegisterName(name, address, owner);

        var retrievedName = _registry.GetAddressName(address);
        retrievedName.Should().Be(name);
    }

    [Test]
    public void RegisterName_ShouldThrowOnNullOrEmptyName()
    {
        var address = CreateTestAddress(12);
        var owner = CreateTestAddress(13);

        Action actNull = () => _registry.RegisterName(null!, address, owner);
        Action actEmpty = () => _registry.RegisterName("", address, owner);
        Action actWhitespace = () => _registry.RegisterName("   ", address, owner);

        actNull.Should().Throw<ArgumentException>();
        actEmpty.Should().Throw<ArgumentException>();
        actWhitespace.Should().Throw<ArgumentException>();
    }

    [Test]
    public void RegisterName_ShouldThrowOnTooLongName()
    {
        var address = CreateTestAddress(14);
        var owner = CreateTestAddress(15);
        var longName = new string('a', 256);

        Action act = () => _registry.RegisterName(longName, address, owner);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*too long*");
    }

    [Test]
    public void RegisterName_ShouldThrowOnZeroAddress()
    {
        var owner = CreateTestAddress(16);

        Action act = () => _registry.RegisterName("test.neo", UInt160.Zero, owner);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*zero address*");
    }

    [Test]
    public void RegisterName_ShouldThrowOnDuplicateNameFromDifferentOwner()
    {
        var address1 = CreateTestAddress(17);
        var owner1 = CreateTestAddress(18);
        var address2 = CreateTestAddress(19);
        var owner2 = CreateTestAddress(20);
        var name = "duplicate.neo";

        // First registration succeeds
        _registry.RegisterName(name, address1, owner1);

        // Second registration from different owner should fail
        Action act = () => _registry.RegisterName(name, address2, owner2);

        act.Should().Throw<UnauthorizedAccessException>()
            .WithMessage("*Not authorized*");
    }

    [Test]
    public void ResolveName_ShouldReturnZeroForUnregisteredName()
    {
        var resolved = _registry.ResolveName("nonexistent.neo");

        resolved.Should().Be(UInt160.Zero);
    }

    [Test]
    public void GetAddressName_ShouldReturnEmptyForUnmappedAddress()
    {
        var address = CreateTestAddress(21);

        var name = _registry.GetAddressName(address);

        name.Should().BeEmpty();
    }

    #endregion

    #region UpdateContractStatus Tests

    [Test]
    public void UpdateContractStatus_ShouldUpdateActiveStatus()
    {
        var address = CreateTestAddress(22);
        var owner = CreateTestAddress(23);
        var info = CreateTestContractInfo("StatusTest", "1.0.0");
        info.Owner = owner;
        info.IsActive = true;

        _registry.RegisterContract(address, info);

        _registry.UpdateContractStatus(address, false, owner);

        var updated = _registry.GetContractInfo(address);
        updated!.IsActive.Should().BeFalse();
    }

    [Test]
    public void UpdateContractStatus_ShouldThrowForUnregisteredContract()
    {
        var address = CreateTestAddress(24);
        var updater = CreateTestAddress(25);

        Action act = () => _registry.UpdateContractStatus(address, false, updater);

        act.Should().Throw<ArgumentException>()
            .WithMessage("*not registered*");
    }

    [Test]
    public void UpdateContractStatus_ShouldThrowForUnauthorizedUpdater()
    {
        var address = CreateTestAddress(26);
        var owner = CreateTestAddress(27);
        var unauthorized = CreateTestAddress(28);
        var info = CreateTestContractInfo("UnauthorizedTest", "1.0.0");
        info.Owner = owner;

        _registry.RegisterContract(address, info);

        Action act = () => _registry.UpdateContractStatus(address, false, unauthorized);

        act.Should().Throw<UnauthorizedAccessException>()
            .WithMessage("*Not authorized*");
    }

    [Test]
    public void UpdateContractStatus_ShouldAllowAdminToUpdate()
    {
        var address = CreateTestAddress(29);
        var owner = CreateTestAddress(30);
        var admin = CreateTestAddress(31);
        var info = CreateTestContractInfo("AdminTest", "1.0.0");
        info.Owner = owner;
        info.Admins = new[] { admin };
        info.IsActive = true;

        _registry.RegisterContract(address, info);

        _registry.UpdateContractStatus(address, false, admin);

        var updated = _registry.GetContractInfo(address);
        updated!.IsActive.Should().BeFalse();
    }

    #endregion

    #region GetContractsByInterface Tests

    [Test]
    public void GetContractsByInterface_ShouldReturnMatchingContracts()
    {
        var address1 = CreateTestAddress(32);
        var address2 = CreateTestAddress(33);
        var interfaceId = StandardInterfaces.ERC20;

        // Register contracts with interface
        var info1 = CreateTestContractInfo("ERC20Token1", "1.0.0");
        var info2 = CreateTestContractInfo("ERC20Token2", "1.0.0");
        _registry.RegisterContract(address1, info1);
        _registry.RegisterContract(address2, info2);
        _registry.RegisterInterface(address1, interfaceId, true);
        _registry.RegisterInterface(address2, interfaceId, true);

        var contracts = _registry.GetContractsByInterface(interfaceId);

        contracts.Should().HaveCount(2);
        contracts.Should().Contain(address1);
        contracts.Should().Contain(address2);
    }

    [Test]
    public void GetContractsByInterface_ShouldExcludeInactiveContracts()
    {
        var address1 = CreateTestAddress(34);
        var address2 = CreateTestAddress(35);
        var interfaceId = StandardInterfaces.ERC721;

        // Register active and inactive contracts
        var info1 = CreateTestContractInfo("ActiveNFT", "1.0.0");
        info1.IsActive = true;
        var info2 = CreateTestContractInfo("InactiveNFT", "1.0.0");
        info2.IsActive = false;

        _registry.RegisterContract(address1, info1);
        _registry.RegisterContract(address2, info2);
        _registry.RegisterInterface(address1, interfaceId, true);
        _registry.RegisterInterface(address2, interfaceId, true);

        var contracts = _registry.GetContractsByInterface(interfaceId);

        contracts.Should().HaveCount(1);
        contracts.Should().Contain(address1);
        contracts.Should().NotContain(address2);
    }

    [Test]
    public void GetContractsByInterface_ShouldReturnEmptyForNoMatches()
    {
        var interfaceId = StandardInterfaces.ERC1155;

        var contracts = _registry.GetContractsByInterface(interfaceId);

        contracts.Should().BeEmpty();
    }

    [Test]
    public void GetContractsByInterface_ShouldThrowOnInvalidInterfaceId()
    {
        Action act = () => _registry.GetContractsByInterface(new byte[] { 0x01 });

        act.Should().Throw<ArgumentException>()
            .WithMessage("*4 bytes*");
    }

    #endregion

    #region BatchRegisterContracts Tests

    [Test]
    public void BatchRegisterContracts_ShouldRegisterMultipleContracts()
    {
        var registrations = new[]
        {
            new ContractRegistration(CreateTestAddress(36), CreateTestContractInfo("Batch1", "1.0.0")),
            new ContractRegistration(CreateTestAddress(37), CreateTestContractInfo("Batch2", "1.0.0")),
            new ContractRegistration(CreateTestAddress(38), CreateTestContractInfo("Batch3", "1.0.0"))
        };

        _registry.BatchRegisterContracts(registrations);

        foreach (var reg in registrations)
        {
            var info = _registry.GetContractInfo(reg.Address);
            info.Should().NotBeNull();
            info!.Name.Should().Be(reg.Info.Name);
        }
    }

    [Test]
    public void BatchRegisterContracts_ShouldHandleEmptyList()
    {
        var emptyList = Array.Empty<ContractRegistration>();

        Action act = () => _registry.BatchRegisterContracts(emptyList);

        act.Should().NotThrow();
    }

    [Test]
    public void BatchRegisterContracts_ShouldThrowOnInvalidAddress()
    {
        var registrations = new[]
        {
            new ContractRegistration(CreateTestAddress(39), CreateTestContractInfo("Valid", "1.0.0")),
            new ContractRegistration(UInt160.Zero, CreateTestContractInfo("Invalid", "1.0.0"))
        };

        Action act = () => _registry.BatchRegisterContracts(registrations);

        act.Should().Throw<ArgumentException>();
    }

    #endregion

    #region GetStats Tests

    [Test]
    public void GetStats_ShouldReturnInitialStats()
    {
        var stats = _registry.GetStats();

        stats.Should().NotBeNull();
        stats.TotalContracts.Should().Be(0);
        stats.ActiveContracts.Should().Be(0);
        stats.RegisteredNames.Should().Be(0);
        stats.CacheSize.Should().Be(0);
    }

    [Test]
    public void GetStats_ShouldReflectRegistrations()
    {
        var address1 = CreateTestAddress(40);
        var address2 = CreateTestAddress(41);
        var info1 = CreateTestContractInfo("Stats1", "1.0.0");
        var info2 = CreateTestContractInfo("Stats2", "1.0.0");

        _registry.RegisterContract(address1, info1);
        _registry.RegisterContract(address2, info2);

        var stats = _registry.GetStats();

        stats.CacheSize.Should().Be(2);
    }

    #endregion

    #region Helper Methods

    private static UInt160 CreateTestAddress(int seed)
    {
        var bytes = new byte[20];
        bytes[0] = (byte)(seed & 0xFF);
        bytes[1] = (byte)((seed >> 8) & 0xFF);
        return new UInt160(bytes);
    }

    private static ContractInfo CreateTestContractInfo(string name, string version)
    {
        return new ContractInfo
        {
            Name = name,
            Version = version,
            Description = $"Test contract {name}",
            Owner = CreateTestAddress(1000),
            Admins = Array.Empty<UInt160>(),
            Tags = Array.Empty<string>(),
            IsActive = true,
            CreatedAt = 0,
            UpdatedAt = 0,
            Metadata = new Dictionary<string, string>()
        };
    }

    #endregion
}
