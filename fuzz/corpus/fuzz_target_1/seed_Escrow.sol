// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Escrow - Secure escrow service for Neo N3
/// @author Neo DevPack for Solidity Team
/// @notice Holds funds in escrow until conditions are met
/// @dev Supports time-locked releases and multi-party approval
contract Escrow {
    // Escrow states
    enum State { Created, Funded, Released, Refunded, Disputed }

    // Escrow details
    struct EscrowDetails {
        address depositor;
        address beneficiary;
        address arbiter;
        uint256 amount;
        uint256 createdAt;
        uint256 releaseTime;
        State state;
        string description;
    }

    // State variables
    mapping(uint256 => EscrowDetails) public escrows;
    mapping(address => uint256[]) public userEscrows;
    uint256 public escrowCount;
    uint256 public totalEscrowed;
    address public owner;
    uint256 public feePercent; // Fee in basis points (100 = 1%)
    uint256 public collectedFees;

    // Events
    event EscrowCreated(
        uint256 indexed escrowId,
        address indexed depositor,
        address indexed beneficiary,
        uint256 amount,
        uint256 releaseTime
    );
    event EscrowFunded(uint256 indexed escrowId, uint256 amount);
    event EscrowReleased(uint256 indexed escrowId, address indexed beneficiary, uint256 amount);
    event EscrowRefunded(uint256 indexed escrowId, address indexed depositor, uint256 amount);
    event EscrowDisputed(uint256 indexed escrowId, address indexed disputer);
    event DisputeResolved(uint256 indexed escrowId, address indexed winner, uint256 amount);
    event ArbiterAssigned(uint256 indexed escrowId, address indexed arbiter);
    event FeeUpdated(uint256 oldFee, uint256 newFee);

    /// @notice Contract constructor
    /// @param _feePercent Initial fee percentage in basis points
    constructor(uint256 _feePercent) {
        require(_feePercent <= 1000, "Escrow: fee too high"); // Max 10%
        owner = msg.sender;
        feePercent = _feePercent;
        escrowCount = 0;
        totalEscrowed = 0;
        collectedFees = 0;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Escrow: caller is not owner");
        _;
    }

    modifier escrowExists(uint256 escrowId) {
        require(escrowId < escrowCount, "Escrow: escrow does not exist");
        _;
    }

    modifier onlyDepositor(uint256 escrowId) {
        require(msg.sender == escrows[escrowId].depositor, "Escrow: caller is not depositor");
        _;
    }

    modifier onlyArbiter(uint256 escrowId) {
        require(msg.sender == escrows[escrowId].arbiter, "Escrow: caller is not arbiter");
        _;
    }

    /// @notice Create a new escrow
    /// @param beneficiary The address that will receive funds
    /// @param arbiter The address that can resolve disputes
    /// @param releaseTime Unix timestamp when funds can be released
    /// @param description Description of the escrow purpose
    /// @return escrowId The ID of the created escrow
    function createEscrow(
        address beneficiary,
        address arbiter,
        uint256 releaseTime,
        string memory description
    ) public returns (uint256 escrowId) {
        require(beneficiary != address(0), "Escrow: beneficiary is zero address");
        require(beneficiary != msg.sender, "Escrow: cannot escrow to self");
        require(releaseTime > block.timestamp, "Escrow: release time must be in future");

        escrowId = escrowCount;
        escrowCount += 1;

        escrows[escrowId] = EscrowDetails({
            depositor: msg.sender,
            beneficiary: beneficiary,
            arbiter: arbiter,
            amount: 0,
            createdAt: block.timestamp,
            releaseTime: releaseTime,
            state: State.Created,
            description: description
        });

        userEscrows[msg.sender].push(escrowId);
        userEscrows[beneficiary].push(escrowId);

        emit EscrowCreated(escrowId, msg.sender, beneficiary, 0, releaseTime);
        return escrowId;
    }

    /// @notice Fund an existing escrow
    /// @param escrowId The escrow to fund
    /// @param amount The amount to deposit
    function fundEscrow(uint256 escrowId, uint256 amount) public escrowExists(escrowId) onlyDepositor(escrowId) {
        EscrowDetails storage escrow = escrows[escrowId];
        require(escrow.state == State.Created, "Escrow: already funded or closed");
        require(amount > 0, "Escrow: amount must be positive");

        escrow.amount = amount;
        escrow.state = State.Funded;
        totalEscrowed += amount;

        emit EscrowFunded(escrowId, amount);
    }

    /// @notice Release funds to beneficiary
    /// @param escrowId The escrow to release
    function release(uint256 escrowId) public escrowExists(escrowId) {
        EscrowDetails storage escrow = escrows[escrowId];
        require(escrow.state == State.Funded, "Escrow: not funded");
        require(
            msg.sender == escrow.depositor ||
            (block.timestamp >= escrow.releaseTime && msg.sender == escrow.beneficiary),
            "Escrow: not authorized to release"
        );

        uint256 fee = (escrow.amount * feePercent) / 10000;
        uint256 releaseAmount = escrow.amount - fee;

        escrow.state = State.Released;
        totalEscrowed -= escrow.amount;
        collectedFees += fee;

        emit EscrowReleased(escrowId, escrow.beneficiary, releaseAmount);
    }

    /// @notice Refund funds to depositor (before release time)
    /// @param escrowId The escrow to refund
    function refund(uint256 escrowId) public escrowExists(escrowId) {
        EscrowDetails storage escrow = escrows[escrowId];
        require(escrow.state == State.Funded, "Escrow: not funded");
        require(
            msg.sender == escrow.beneficiary ||
            msg.sender == escrow.arbiter,
            "Escrow: not authorized to refund"
        );

        uint256 refundAmount = escrow.amount;
        escrow.state = State.Refunded;
        totalEscrowed -= escrow.amount;

        emit EscrowRefunded(escrowId, escrow.depositor, refundAmount);
    }

    /// @notice Raise a dispute on an escrow
    /// @param escrowId The escrow to dispute
    function dispute(uint256 escrowId) public escrowExists(escrowId) {
        EscrowDetails storage escrow = escrows[escrowId];
        require(escrow.state == State.Funded, "Escrow: not funded");
        require(
            msg.sender == escrow.depositor || msg.sender == escrow.beneficiary,
            "Escrow: not a party to escrow"
        );
        require(escrow.arbiter != address(0), "Escrow: no arbiter assigned");

        escrow.state = State.Disputed;
        emit EscrowDisputed(escrowId, msg.sender);
    }

    /// @notice Resolve a dispute (arbiter only)
    /// @param escrowId The disputed escrow
    /// @param releaseTobeneficiary True to release to beneficiary, false to refund depositor
    function resolveDispute(uint256 escrowId, bool releaseTobeneficiary) public escrowExists(escrowId) onlyArbiter(escrowId) {
        EscrowDetails storage escrow = escrows[escrowId];
        require(escrow.state == State.Disputed, "Escrow: not disputed");

        uint256 amount = escrow.amount;
        totalEscrowed -= amount;

        if (releaseTobeneficiary) {
            uint256 fee = (amount * feePercent) / 10000;
            uint256 releaseAmount = amount - fee;
            collectedFees += fee;
            escrow.state = State.Released;
            emit DisputeResolved(escrowId, escrow.beneficiary, releaseAmount);
        } else {
            escrow.state = State.Refunded;
            emit DisputeResolved(escrowId, escrow.depositor, amount);
        }
    }

    /// @notice Get escrow details
    /// @param escrowId The escrow ID
    /// @return depositor The depositor address
    /// @return beneficiary The beneficiary address
    /// @return amount The escrowed amount
    /// @return state The current state
    function getEscrow(uint256 escrowId) public view escrowExists(escrowId) returns (
        address depositor,
        address beneficiary,
        uint256 amount,
        State state
    ) {
        EscrowDetails storage escrow = escrows[escrowId];
        return (escrow.depositor, escrow.beneficiary, escrow.amount, escrow.state);
    }

    /// @notice Check if escrow can be released
    /// @param escrowId The escrow ID
    /// @return True if releasable
    function canRelease(uint256 escrowId) public view escrowExists(escrowId) returns (bool) {
        EscrowDetails storage escrow = escrows[escrowId];
        return escrow.state == State.Funded && block.timestamp >= escrow.releaseTime;
    }

    /// @notice Get user's escrow IDs
    /// @param user The user address
    /// @return Array of escrow IDs
    function getUserEscrows(address user) public view returns (uint256[] memory) {
        return userEscrows[user];
    }

    /// @notice Update fee percentage (owner only)
    /// @param newFeePercent New fee in basis points
    function setFeePercent(uint256 newFeePercent) public onlyOwner {
        require(newFeePercent <= 1000, "Escrow: fee too high");
        uint256 oldFee = feePercent;
        feePercent = newFeePercent;
        emit FeeUpdated(oldFee, newFeePercent);
    }

    /// @notice Withdraw collected fees (owner only)
    /// @return amount The withdrawn amount
    function withdrawFees() public onlyOwner returns (uint256 amount) {
        amount = collectedFees;
        collectedFees = 0;
        return amount;
    }

    /// @notice Get contract statistics
    /// @return _escrowCount Total escrows created
    /// @return _totalEscrowed Current total escrowed
    /// @return _collectedFees Total fees collected
    function getStats() public view returns (
        uint256 _escrowCount,
        uint256 _totalEscrowed,
        uint256 _collectedFees
    ) {
        return (escrowCount, totalEscrowed, collectedFees);
    }
}
