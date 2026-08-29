// SPDX-License-Identifier: MIT
// A minimal on-chain Governor, modeled on the OpenZeppelin Governor lifecycle.
// Reference: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.9.3/contracts/governance/Governor.sol
// Self-contained, dependency-free condensation: propose -> vote (For/Against/Abstain)
// over a voting window -> tally against a quorum. Voting weight comes from a simple
// registered-weight table instead of an external ERC20Votes token, so the file needs
// no imports while preserving the propose/castVote/state lifecycle.
pragma solidity ^0.8.0;

contract SimpleGovernor {
    enum ProposalState {
        Pending,
        Active,
        Defeated,
        Succeeded,
        Executed
    }

    enum VoteType {
        Against,
        For,
        Abstain
    }

    struct ProposalCore {
        address proposer;
        uint256 voteStart;
        uint256 voteEnd;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        bool executed;
    }

    string public name;
    uint256 public votingDelay;
    uint256 public votingPeriod;
    uint256 public quorumVotes;

    address public admin;
    mapping(address => uint256) public votingWeight;

    // proposalId => core
    mapping(uint256 => ProposalCore) private _proposals;
    // proposalId => voter => voted
    mapping(uint256 => mapping(address => bool)) public hasVoted;

    event ProposalCreated(uint256 indexed proposalId, address indexed proposer, uint256 voteStart, uint256 voteEnd);
    event VoteCast(address indexed voter, uint256 indexed proposalId, uint8 support, uint256 weight);
    event ProposalExecuted(uint256 indexed proposalId);

    constructor(
        string memory _name,
        uint256 _votingDelay,
        uint256 _votingPeriod,
        uint256 _quorumVotes
    ) {
        name = _name;
        votingDelay = _votingDelay;
        votingPeriod = _votingPeriod;
        quorumVotes = _quorumVotes;
        admin = msg.sender;
    }

    /// @dev Simple weight registry standing in for an ERC20Votes checkpoint.
    function setVotingWeight(address account, uint256 weight) external {
        require(msg.sender == admin, "Governor: not admin");
        votingWeight[account] = weight;
    }

    function hashProposal(
        address target,
        uint256 value,
        bytes memory data,
        bytes32 descriptionHash
    ) public pure returns (uint256) {
        return uint256(keccak256(abi.encode(target, value, data, descriptionHash)));
    }

    function propose(
        address target,
        uint256 value,
        bytes memory data,
        bytes32 descriptionHash
    ) external returns (uint256) {
        require(votingWeight[msg.sender] > 0, "Governor: proposer has no weight");
        uint256 proposalId = hashProposal(target, value, data, descriptionHash);

        ProposalCore storage proposal = _proposals[proposalId];
        require(proposal.voteStart == 0, "Governor: proposal already exists");

        uint256 start = block.timestamp + votingDelay;
        uint256 end = start + votingPeriod;
        proposal.proposer = msg.sender;
        proposal.voteStart = start;
        proposal.voteEnd = end;

        emit ProposalCreated(proposalId, msg.sender, start, end);
        return proposalId;
    }

    function castVote(uint256 proposalId, uint8 support) external returns (uint256) {
        ProposalCore storage proposal = _proposals[proposalId];
        require(proposal.voteStart != 0, "Governor: unknown proposal");
        require(block.timestamp >= proposal.voteStart, "Governor: voting not started");
        require(block.timestamp <= proposal.voteEnd, "Governor: voting closed");
        require(!hasVoted[proposalId][msg.sender], "Governor: already voted");

        uint256 weight = votingWeight[msg.sender];
        require(weight > 0, "Governor: no voting weight");

        hasVoted[proposalId][msg.sender] = true;

        if (support == uint8(VoteType.Against)) {
            proposal.againstVotes += weight;
        } else if (support == uint8(VoteType.For)) {
            proposal.forVotes += weight;
        } else if (support == uint8(VoteType.Abstain)) {
            proposal.abstainVotes += weight;
        } else {
            revert("Governor: invalid vote type");
        }

        emit VoteCast(msg.sender, proposalId, support, weight);
        return weight;
    }

    function state(uint256 proposalId) public view returns (ProposalState) {
        ProposalCore storage proposal = _proposals[proposalId];
        require(proposal.voteStart != 0, "Governor: unknown proposal");

        if (proposal.executed) {
            return ProposalState.Executed;
        }
        if (block.timestamp < proposal.voteStart) {
            return ProposalState.Pending;
        }
        if (block.timestamp <= proposal.voteEnd) {
            return ProposalState.Active;
        }
        // Voting has ended: tally.
        uint256 totalCast = proposal.forVotes + proposal.abstainVotes;
        if (totalCast < quorumVotes || proposal.forVotes <= proposal.againstVotes) {
            return ProposalState.Defeated;
        }
        return ProposalState.Succeeded;
    }

    function execute(uint256 proposalId) external {
        require(state(proposalId) == ProposalState.Succeeded, "Governor: proposal not successful");
        _proposals[proposalId].executed = true;
        emit ProposalExecuted(proposalId);
    }

    function proposalDeadline(uint256 proposalId) external view returns (uint256) {
        return _proposals[proposalId].voteEnd;
    }

    function proposalSnapshot(uint256 proposalId) external view returns (uint256) {
        return _proposals[proposalId].voteStart;
    }
}
