// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SimpleDAO — On-chain Governance
 * @notice Neo N3 adaptation of Compound Governor / MolochDAO.
 *
 * Members stake GAS for voting power. Proposals require quorum (20%)
 * and pass after a voting period + timelock before execution.
 *
 * Compiler constraints respected:
 *   - No {value: ...} — uses NativeCalls.gasTransfer()
 *   - No receive()/fallback() — uses onNEP17Payment()
 *   - Uses Syscalls.contractCall() for Neo-native cross-contract execution
 *   - Import devpack via -I devpack
 */
contract SimpleDAO {
    address private constant GAS_TOKEN = NativeCalls.GAS_CONTRACT;

    /// @notice Quorum: 20% of total staked (scaled by 100).
    uint256 public constant QUORUM_PERCENT = 20;

    /// @notice Voting period in blocks.
    uint256 public votingPeriod;

    /// @notice Timelock delay in blocks after voting ends.
    uint256 public timelockDelay;

    enum ProposalState { Pending, Active, Succeeded, Defeated, Executed, Cancelled }

    struct Proposal {
        uint256 id;
        address proposer;
        string description;
        address target;
        string methodName;
        bytes args;
        uint256 startBlock;
        uint256 endBlock;
        uint256 executeAfter;
        uint256 forVotes;
        uint256 againstVotes;
        bool executed;
        bool cancelled;
    }

    mapping(address => uint256) public stakes;
    uint256 public totalStaked;

    uint256 public proposalCount;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public hasVoted;

    event Staked(address indexed member, uint256 amount);
    event Unstaked(address indexed member, uint256 amount);
    event ProposalCreated(uint256 indexed proposalId, address indexed proposer, string description);
    event VoteCast(uint256 indexed proposalId, address indexed voter, bool support, uint256 weight);
    event ProposalExecuted(uint256 indexed proposalId);
    event ProposalCancelled(uint256 indexed proposalId);

    /// @notice Default voting period (100 blocks) and timelock delay (10 blocks).
    constructor() {
        votingPeriod = 100;
        timelockDelay = 10;
    }

    /// @notice NEP-17 callback — receives GAS stakes.
    function onNEP17Payment(address from, uint256 amount, Any calldata /*data*/) external {
        address caller = Syscalls.getCallingScriptHash();
        require(caller == GAS_TOKEN, "DAO: only GAS accepted");
        require(amount > 0, "DAO: zero stake");

        stakes[from] += amount;
        totalStaked += amount;

        emit Staked(from, amount);
    }

    /// @notice Unstake GAS and withdraw.
    function unstake(uint256 amount) external {
        require(stakes[msg.sender] >= amount, "DAO: insufficient stake");

        stakes[msg.sender] -= amount;
        totalStaked -= amount;

        bool ok = NativeCalls.gasTransfer(address(this), msg.sender, amount, "");
        require(ok, "DAO: transfer failed");

        emit Unstaked(msg.sender, amount);
    }

    /// @notice Create a new proposal.
    /// @param description Human-readable description.
    /// @param target Contract to call on execution.
    /// @param methodName Method name to invoke (e.g. "setValue").
    /// @param args ABI-encoded arguments for the method.
    function propose(
        string memory description,
        address target,
        string memory methodName,
        bytes memory args
    ) external returns (uint256 proposalId) {
        require(stakes[msg.sender] > 0, "DAO: must be staker");
        require(target != address(0), "DAO: zero target");

        proposalId = proposalCount;
        proposalCount += 1;

        Proposal storage p = proposals[proposalId];
        p.id = proposalId;
        p.proposer = msg.sender;
        p.description = description;
        p.target = target;
        p.methodName = methodName;
        p.args = args;
        p.startBlock = block.number;
        p.endBlock = block.number + votingPeriod;
        p.executeAfter = block.number + votingPeriod + timelockDelay;

        emit ProposalCreated(proposalId, msg.sender, description);
    }

    /// @notice Cast a vote on a proposal.
    function vote(uint256 proposalId, bool support) external {
        require(proposalId < proposalCount, "DAO: invalid proposal");
        Proposal storage p = proposals[proposalId];

        require(block.number >= p.startBlock, "DAO: voting not started");
        require(block.number <= p.endBlock, "DAO: voting ended");
        require(!hasVoted[proposalId][msg.sender], "DAO: already voted");
        require(stakes[msg.sender] > 0, "DAO: no voting power");

        uint256 weight = stakes[msg.sender];
        hasVoted[proposalId][msg.sender] = true;

        if (support) {
            p.forVotes += weight;
        } else {
            p.againstVotes += weight;
        }

        emit VoteCast(proposalId, msg.sender, support, weight);
    }

    /// @notice Execute a succeeded proposal after timelock.
    function execute(uint256 proposalId) external {
        require(proposalId < proposalCount, "DAO: invalid proposal");
        Proposal storage p = proposals[proposalId];

        require(!p.executed, "DAO: already executed");
        require(!p.cancelled, "DAO: cancelled");
        require(block.number > p.endBlock, "DAO: voting not ended");
        require(block.number >= p.executeAfter, "DAO: timelock active");

        // Check quorum: forVotes >= 20% of totalStaked.
        uint256 quorum = (totalStaked * QUORUM_PERCENT) / 100;
        require(p.forVotes >= quorum, "DAO: quorum not reached");
        require(p.forVotes > p.againstVotes, "DAO: proposal defeated");

        p.executed = true;

        // Neo-native cross-contract call via Syscalls.
        Syscalls.contractCall(p.target, p.methodName, p.args);

        emit ProposalExecuted(proposalId);
    }

    /// @notice Cancel a proposal (only proposer).
    function cancel(uint256 proposalId) external {
        require(proposalId < proposalCount, "DAO: invalid proposal");
        Proposal storage p = proposals[proposalId];

        require(msg.sender == p.proposer, "DAO: not proposer");
        require(!p.executed, "DAO: already executed");

        p.cancelled = true;
        emit ProposalCancelled(proposalId);
    }

    /// @notice Get the current state of a proposal.
    function getProposalState(uint256 proposalId) external view returns (uint256) {
        require(proposalId < proposalCount, "DAO: invalid proposal");
        Proposal storage p = proposals[proposalId];

        if (p.cancelled) return uint256(ProposalState.Cancelled);
        if (p.executed) return uint256(ProposalState.Executed);
        if (block.number <= p.endBlock) return uint256(ProposalState.Active);

        uint256 quorum = (totalStaked * QUORUM_PERCENT) / 100;
        if (p.forVotes >= quorum && p.forVotes > p.againstVotes) {
            return uint256(ProposalState.Succeeded);
        }
        return uint256(ProposalState.Defeated);
    }
}
