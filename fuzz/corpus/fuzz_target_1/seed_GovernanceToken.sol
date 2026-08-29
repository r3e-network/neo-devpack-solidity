// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title GovernanceToken
 * @dev Complete governance token with voting, delegation, and proposal system for Neo blockchain
 * Features: ERC20 with votes, delegation, timelock, multi-signature governance
 */
contract GovernanceToken {
    // ERC20 State
    string public name;
    string public symbol;
    uint8 public constant decimals = 18;
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    // Governance State
    mapping(address => address) public delegates;
    mapping(address => Checkpoint[]) public checkpoints;
    mapping(address => uint256) public numCheckpoints;
    mapping(bytes32 => Proposal) public proposals;
    mapping(bytes32 => mapping(address => Receipt)) public receipts;
    
    // Admin State
    address public admin;
    address public pendingAdmin;
    address public timelock;
    address public guardian;
    
    // Voting Configuration
    uint256 public votingDelay = 1 days;          // Delay before voting starts
    uint256 public votingPeriod = 3 days;        // Voting duration
    uint256 public proposalThreshold = 100000e18; // Minimum tokens to propose
    uint256 public quorumVotes = 400000e18;      // Minimum votes for quorum
    
    // Governance State
    uint256 public proposalCount;
    bool public paused;
    uint256 public constant MAX_VOTING_DELAY = 2 weeks;
    uint256 public constant MAX_VOTING_PERIOD = 2 weeks;
    uint256 public constant MIN_VOTING_PERIOD = 1 days;
    uint256 public constant MIN_VOTING_DELAY = 1 hours;
    
    // Structs
    struct Checkpoint {
        uint32 fromBlock;
        uint256 votes;
    }
    
    struct Proposal {
        uint256 id;
        address proposer;
        uint256 eta;
        address[] targets;
        uint256[] values;
        string[] signatures;
        bytes[] calldatas;
        uint256 startBlock;
        uint256 endBlock;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        bool canceled;
        bool executed;
        mapping(address => Receipt) receipts;
        string description;
        bytes32 descriptionHash;
    }
    
    struct Receipt {
        bool hasVoted;
        uint8 support; // 0=against, 1=for, 2=abstain
        uint256 votes;
        string reason;
    }
    
    enum ProposalState {
        Pending,
        Active,
        Canceled,
        Defeated,
        Succeeded,
        Queued,
        Expired,
        Executed
    }
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);
    event DelegateChanged(address indexed delegator, address indexed fromDelegate, address indexed toDelegate);
    event DelegateVotesChanged(address indexed delegate, uint256 previousBalance, uint256 newBalance);
    event ProposalCreated(
        uint256 id,
        address proposer,
        address[] targets,
        uint256[] values,
        string[] signatures,
        bytes[] calldatas,
        uint256 startBlock,
        uint256 endBlock,
        string description
    );
    event VoteCast(address indexed voter, uint256 proposalId, uint8 support, uint256 weight, string reason);
    event ProposalCanceled(uint256 id);
    event ProposalQueued(uint256 id, uint256 eta);
    event ProposalExecuted(uint256 id);
    event NewAdmin(address oldAdmin, address newAdmin);
    event NewPendingAdmin(address oldPendingAdmin, address newPendingAdmin);
    event NewGuardian(address oldGuardian, address newGuardian);
    event VotingDelaySet(uint256 oldVotingDelay, uint256 newVotingDelay);
    event VotingPeriodSet(uint256 oldVotingPeriod, uint256 newVotingPeriod);
    event ProposalThresholdSet(uint256 oldProposalThreshold, uint256 newProposalThreshold);
    event QuorumVotesSet(uint256 oldQuorumVotes, uint256 newQuorumVotes);
    event Paused(address account);
    event Unpaused(address account);
    
    // Custom errors
    error OnlyAdmin();
    error OnlyGuardian();
    error OnlyTimelock();
    error ContractPaused();
    error InvalidProposal();
    error ProposalNotActive();
    error AlreadyVoted();
    error ProposalNotSucceeded();
    error ProposalAlreadyExecuted();
    error ExecutionReverted();
    error InsufficientVotes();
    error InvalidVotingDelay();
    error InvalidVotingPeriod();
    error InvalidAddress();
    error InvalidProposalThreshold();
    error InvalidQuorumVotes();
    
    // Modifiers
    modifier onlyAdmin() {
        if (msg.sender != admin) revert OnlyAdmin();
        _;
    }
    
    modifier onlyGuardian() {
        if (msg.sender != guardian) revert OnlyGuardian();
        _;
    }
    
    modifier onlyTimelock() {
        if (msg.sender != timelock) revert OnlyTimelock();
        _;
    }
    
    modifier whenNotPaused() {
        if (paused) revert ContractPaused();
        _;
    }
    
    modifier validAddress(address addr) {
        if (addr == address(0)) revert InvalidAddress();
        _;
    }
    
    /**
     * @dev Constructor sets up the governance token
     * @param _name Token name
     * @param _symbol Token symbol
     * @param _initialSupply Initial token supply
     * @param _admin Initial admin address
     * @param _timelock Timelock contract address
     * @param _guardian Guardian address for emergency functions
     */
    constructor(
        string memory _name,
        string memory _symbol,
        uint256 _initialSupply,
        address _admin,
        address _timelock,
        address _guardian
    ) 
        validAddress(_admin)
        validAddress(_timelock)
        validAddress(_guardian)
    {
        name = _name;
        symbol = _symbol;
        admin = _admin;
        timelock = _timelock;
        guardian = _guardian;
        
        totalSupply = _initialSupply;
        balanceOf[_admin] = _initialSupply;
        
        emit Transfer(address(0), _admin, _initialSupply);
        emit NewAdmin(address(0), _admin);
    }
    
    // ERC20 Functions
    
    /**
     * @dev Transfer tokens
     */
    function transfer(address to, uint256 amount) public whenNotPaused returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }
    
    /**
     * @dev Transfer tokens on behalf of another address
     */
    function transferFrom(address from, address to, uint256 amount) public whenNotPaused returns (bool) {
        uint256 currentAllowance = allowance[from][msg.sender];
        require(currentAllowance >= amount, "ERC20: transfer amount exceeds allowance");
        
        _transfer(from, to, amount);
        _approve(from, msg.sender, currentAllowance - amount);
        
        return true;
    }
    
    /**
     * @dev Approve spender to transfer tokens
     */
    function approve(address spender, uint256 amount) public whenNotPaused returns (bool) {
        _approve(msg.sender, spender, amount);
        return true;
    }
    
    /**
     * @dev Internal transfer function with vote tracking
     */
    function _transfer(address from, address to, uint256 amount) internal {
        require(from != address(0), "ERC20: transfer from the zero address");
        require(to != address(0), "ERC20: transfer to the zero address");
        
        uint256 fromBalance = balanceOf[from];
        require(fromBalance >= amount, "ERC20: transfer amount exceeds balance");
        
        balanceOf[from] = fromBalance - amount;
        balanceOf[to] += amount;
        
        emit Transfer(from, to, amount);
        
        _moveDelegates(delegates[from], delegates[to], amount);
    }
    
    /**
     * @dev Internal approve function
     */
    function _approve(address owner, address spender, uint256 amount) internal {
        require(owner != address(0), "ERC20: approve from the zero address");
        require(spender != address(0), "ERC20: approve to the zero address");
        
        allowance[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }
    
    // Delegation Functions
    
    /**
     * @dev Delegate votes to another address
     * @param delegatee Address to delegate votes to
     */
    function delegate(address delegatee) public whenNotPaused {
        _delegate(msg.sender, delegatee);
    }
    
    /**
     * @dev Delegate votes by signature
     */
    function delegateBySig(
        address delegatee,
        uint256 nonce,
        uint256 expiry,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) public whenNotPaused {
        bytes32 domainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                block.chainid,
                address(this)
            )
        );
        
        bytes32 structHash = keccak256(
            abi.encode(
                keccak256("Delegation(address delegatee,uint256 nonce,uint256 expiry)"),
                delegatee,
                nonce,
                expiry
            )
        );
        
        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", domainSeparator, structHash));
        address signatory = ecrecover(digest, v, r, s);
        
        require(signatory != address(0), "Invalid signature");
        require(nonce == numCheckpoints[signatory], "Invalid nonce");
        require(block.timestamp <= expiry, "Signature expired");
        
        _delegate(signatory, delegatee);
    }
    
    /**
     * @dev Get current votes for an account
     */
    function getCurrentVotes(address account) public view returns (uint256) {
        uint256 nCheckpoints = numCheckpoints[account];
        return nCheckpoints > 0 ? checkpoints[account][nCheckpoints - 1].votes : 0;
    }
    
    /**
     * @dev Get prior votes for an account at a specific block
     */
    function getPriorVotes(address account, uint256 blockNumber) public view returns (uint256) {
        require(blockNumber < block.number, "Not yet determined");
        
        uint256 nCheckpoints = numCheckpoints[account];
        if (nCheckpoints == 0) {
            return 0;
        }
        
        // First check most recent balance
        if (checkpoints[account][nCheckpoints - 1].fromBlock <= blockNumber) {
            return checkpoints[account][nCheckpoints - 1].votes;
        }
        
        // Next check implicit zero balance
        if (checkpoints[account][0].fromBlock > blockNumber) {
            return 0;
        }
        
        uint256 lower = 0;
        uint256 upper = nCheckpoints - 1;
        while (upper > lower) {
            uint256 center = upper - (upper - lower) / 2; // ceil, avoiding overflow
            Checkpoint memory cp = checkpoints[account][center];
            if (cp.fromBlock == blockNumber) {
                return cp.votes;
            } else if (cp.fromBlock < blockNumber) {
                lower = center;
            } else {
                upper = center - 1;
            }
        }
        return checkpoints[account][lower].votes;
    }
    
    /**
     * @dev Internal delegate function
     */
    function _delegate(address delegator, address delegatee) internal {
        address currentDelegate = delegates[delegator];
        uint256 delegatorBalance = balanceOf[delegator];
        delegates[delegator] = delegatee;
        
        emit DelegateChanged(delegator, currentDelegate, delegatee);
        
        _moveDelegates(currentDelegate, delegatee, delegatorBalance);
    }
    
    /**
     * @dev Move delegate votes from one address to another
     */
    function _moveDelegates(address srcRep, address dstRep, uint256 amount) internal {
        if (srcRep != dstRep && amount > 0) {
            if (srcRep != address(0)) {
                uint256 srcRepNum = numCheckpoints[srcRep];
                uint256 srcRepOld = srcRepNum > 0 ? checkpoints[srcRep][srcRepNum - 1].votes : 0;
                uint256 srcRepNew = srcRepOld - amount;
                _writeCheckpoint(srcRep, srcRepNum, srcRepOld, srcRepNew);
            }
            
            if (dstRep != address(0)) {
                uint256 dstRepNum = numCheckpoints[dstRep];
                uint256 dstRepOld = dstRepNum > 0 ? checkpoints[dstRep][dstRepNum - 1].votes : 0;
                uint256 dstRepNew = dstRepOld + amount;
                _writeCheckpoint(dstRep, dstRepNum, dstRepOld, dstRepNew);
            }
        }
    }
    
    /**
     * @dev Write checkpoint for vote tracking
     */
    function _writeCheckpoint(
        address delegatee,
        uint256 nCheckpoints,
        uint256 oldVotes,
        uint256 newVotes
    ) internal {
        uint32 blockNumber = safe32(block.number, "Block number exceeds 32 bits");
        
        if (nCheckpoints > 0 && checkpoints[delegatee][nCheckpoints - 1].fromBlock == blockNumber) {
            checkpoints[delegatee][nCheckpoints - 1].votes = newVotes;
        } else {
            checkpoints[delegatee][nCheckpoints] = Checkpoint(blockNumber, newVotes);
            numCheckpoints[delegatee] = nCheckpoints + 1;
        }
        
        emit DelegateVotesChanged(delegatee, oldVotes, newVotes);
    }
    
    // Governance Functions
    
    /**
     * @dev Create a new proposal
     */
    function propose(
        address[] memory targets,
        uint256[] memory values,
        string[] memory signatures,
        bytes[] memory calldatas,
        string memory description
    ) public whenNotPaused returns (uint256) {
        require(
            getPriorVotes(msg.sender, block.number - 1) >= proposalThreshold,
            "Proposer votes below proposal threshold"
        );
        require(
            targets.length == values.length &&
            targets.length == signatures.length &&
            targets.length == calldatas.length,
            "Proposal function information arity mismatch"
        );
        require(targets.length != 0, "Must provide actions");
        require(targets.length <= 10, "Too many actions");
        
        uint256 latestProposalId = latestProposalIds[msg.sender];
        if (latestProposalId != 0) {
            ProposalState proposersLatestProposalState = state(latestProposalId);
            require(
                proposersLatestProposalState != ProposalState.Active,
                "One live proposal per proposer, found an already active proposal"
            );
            require(
                proposersLatestProposalState != ProposalState.Pending,
                "One live proposal per proposer, found an already pending proposal"
            );
        }
        
        uint256 startBlock = block.number + votingDelay;
        uint256 endBlock = startBlock + votingPeriod;
        
        proposalCount++;
        bytes32 proposalHash = keccak256(abi.encode(
            proposalCount,
            targets,
            values,
            signatures,
            calldatas,
            keccak256(bytes(description))
        ));
        
        Proposal storage newProposal = proposals[proposalHash];
        newProposal.id = proposalCount;
        newProposal.proposer = msg.sender;
        newProposal.eta = 0;
        newProposal.targets = targets;
        newProposal.values = values;
        newProposal.signatures = signatures;
        newProposal.calldatas = calldatas;
        newProposal.startBlock = startBlock;
        newProposal.endBlock = endBlock;
        newProposal.forVotes = 0;
        newProposal.againstVotes = 0;
        newProposal.abstainVotes = 0;
        newProposal.canceled = false;
        newProposal.executed = false;
        newProposal.description = description;
        newProposal.descriptionHash = keccak256(bytes(description));
        
        latestProposalIds[msg.sender] = proposalCount;
        
        emit ProposalCreated(
            newProposal.id,
            msg.sender,
            targets,
            values,
            signatures,
            calldatas,
            startBlock,
            endBlock,
            description
        );
        
        return newProposal.id;
    }
    
    mapping(address => uint256) public latestProposalIds;
    
    /**
     * @dev Queue a succeeded proposal
     */
    function queue(uint256 proposalId) external {
        require(
            state(proposalId) == ProposalState.Succeeded,
            "Proposal can only be queued if it is succeeded"
        );
        
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage proposal = proposals[proposalHash];
        uint256 eta = block.timestamp + ITimelock(timelock).delay();
        
        for (uint256 i = 0; i < proposal.targets.length; i++) {
            queueOrRevertInternal(
                proposal.targets[i],
                proposal.values[i],
                proposal.signatures[i],
                proposal.calldatas[i],
                eta
            );
        }
        
        proposal.eta = eta;
        emit ProposalQueued(proposalId, eta);
    }
    
    /**
     * @dev Execute a queued proposal
     */
    function execute(uint256 proposalId) external payable {
        require(
            state(proposalId) == ProposalState.Queued,
            "Proposal can only be executed if it is queued"
        );
        
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage proposal = proposals[proposalHash];
        proposal.executed = true;
        
        for (uint256 i = 0; i < proposal.targets.length; i++) {
            // Neo N3 has no EVM-style attached value; enforce explicit token transfers instead.
            require(proposal.values[i] == 0, "Neo N3: value transfers unsupported");
            ITimelock(timelock).executeTransaction(
                proposal.targets[i],
                proposal.values[i],
                proposal.signatures[i],
                proposal.calldatas[i],
                proposal.eta
            );
        }
        
        emit ProposalExecuted(proposalId);
    }
    
    /**
     * @dev Cancel a proposal
     */
    function cancel(uint256 proposalId) external {
        require(state(proposalId) != ProposalState.Executed, "Cannot cancel executed proposal");
        
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage proposal = proposals[proposalHash];
        
        require(
            msg.sender == guardian ||
            getPriorVotes(proposal.proposer, block.number - 1) < proposalThreshold,
            "Proposer above threshold"
        );
        
        proposal.canceled = true;
        
        for (uint256 i = 0; i < proposal.targets.length; i++) {
            ITimelock(timelock).cancelTransaction(
                proposal.targets[i],
                proposal.values[i],
                proposal.signatures[i],
                proposal.calldatas[i],
                proposal.eta
            );
        }
        
        emit ProposalCanceled(proposalId);
    }
    
    /**
     * @dev Cast a vote for a proposal
     */
    function castVote(uint256 proposalId, uint8 support) external whenNotPaused returns (uint256) {
        return _castVote(msg.sender, proposalId, support, "");
    }
    
    /**
     * @dev Cast a vote for a proposal with reason
     */
    function castVoteWithReason(
        uint256 proposalId,
        uint8 support,
        string calldata reason
    ) external whenNotPaused returns (uint256) {
        return _castVote(msg.sender, proposalId, support, reason);
    }
    
    /**
     * @dev Cast a vote by signature
     */
    function castVoteBySig(
        uint256 proposalId,
        uint8 support,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external whenNotPaused returns (uint256) {
        bytes32 domainSeparator = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                block.chainid,
                address(this)
            )
        );
        
        bytes32 structHash = keccak256(abi.encode(
            keccak256("Ballot(uint256 proposalId,uint8 support)"),
            proposalId,
            support
        ));
        
        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", domainSeparator, structHash));
        address signatory = ecrecover(digest, v, r, s);
        require(signatory != address(0), "Invalid signature");
        
        return _castVote(signatory, proposalId, support, "");
    }
    
    /**
     * @dev Internal vote casting function
     */
    function _castVote(
        address voter,
        uint256 proposalId,
        uint8 support,
        string memory reason
    ) internal returns (uint256) {
        require(state(proposalId) == ProposalState.Active, "Voting is closed");
        require(support <= 2, "Invalid vote type");
        
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage proposal = proposals[proposalHash];
        Receipt storage receipt = proposal.receipts[voter];
        
        require(!receipt.hasVoted, "Voter already voted");
        
        uint256 votes = getPriorVotes(voter, proposal.startBlock);
        
        if (support == 0) {
            proposal.againstVotes += votes;
        } else if (support == 1) {
            proposal.forVotes += votes;
        } else {
            proposal.abstainVotes += votes;
        }
        
        receipt.hasVoted = true;
        receipt.support = support;
        receipt.votes = votes;
        receipt.reason = reason;
        
        emit VoteCast(voter, proposalId, support, votes, reason);
        
        return votes;
    }
    
    /**
     * @dev Get the state of a proposal
     */
    function state(uint256 proposalId) public view returns (ProposalState) {
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage proposal = proposals[proposalHash];
        
        require(proposal.id != 0, "Invalid proposal id");
        
        if (proposal.canceled) {
            return ProposalState.Canceled;
        } else if (block.number <= proposal.startBlock) {
            return ProposalState.Pending;
        } else if (block.number <= proposal.endBlock) {
            return ProposalState.Active;
        } else if (proposal.forVotes <= proposal.againstVotes || proposal.forVotes < quorumVotes) {
            return ProposalState.Defeated;
        } else if (proposal.eta == 0) {
            return ProposalState.Succeeded;
        } else if (proposal.executed) {
            return ProposalState.Executed;
        } else if (block.timestamp >= proposal.eta + ITimelock(timelock).GRACE_PERIOD()) {
            return ProposalState.Expired;
        } else {
            return ProposalState.Queued;
        }
    }
    
    /**
     * @dev Get actions for a proposal
     */
    function getActions(uint256 proposalId)
        external
        view
        returns (
            address[] memory targets,
            uint256[] memory values,
            string[] memory signatures,
            bytes[] memory calldatas
        )
    {
        bytes32 proposalHash = getProposalHash(proposalId);
        Proposal storage p = proposals[proposalHash];
        return (p.targets, p.values, p.signatures, p.calldatas);
    }
    
    /**
     * @dev Get receipt for a voter and proposal
     */
    function getReceipt(uint256 proposalId, address voter)
        external
        view
        returns (Receipt memory)
    {
        bytes32 proposalHash = getProposalHash(proposalId);
        return proposals[proposalHash].receipts[voter];
    }
    
    /**
     * @dev Get proposal hash
     */
    function getProposalHash(uint256 proposalId) public view returns (bytes32) {
        // Generate deterministic hash from proposal ID and contract address
        return keccak256(abi.encode(address(this), proposalId, "PROPOSAL"));
    }
    
    // Admin Functions
    
    /**
     * @dev Set pending admin
     */
    function _setPendingAdmin(address newPendingAdmin) external onlyAdmin {
        address oldPendingAdmin = pendingAdmin;
        pendingAdmin = newPendingAdmin;
        emit NewPendingAdmin(oldPendingAdmin, newPendingAdmin);
    }
    
    /**
     * @dev Accept admin role
     */
    function _acceptAdmin() external {
        require(msg.sender == pendingAdmin && pendingAdmin != address(0), "Unauthorized");
        
        address oldAdmin = admin;
        admin = pendingAdmin;
        pendingAdmin = address(0);
        
        emit NewAdmin(oldAdmin, admin);
        emit NewPendingAdmin(pendingAdmin, address(0));
    }
    
    /**
     * @dev Set guardian
     */
    function _setGuardian(address newGuardian) external onlyAdmin validAddress(newGuardian) {
        address oldGuardian = guardian;
        guardian = newGuardian;
        emit NewGuardian(oldGuardian, newGuardian);
    }
    
    /**
     * @dev Set voting delay
     */
    function _setVotingDelay(uint256 newVotingDelay) external onlyAdmin {
        require(
            newVotingDelay >= MIN_VOTING_DELAY && newVotingDelay <= MAX_VOTING_DELAY,
            "Invalid voting delay"
        );
        uint256 oldVotingDelay = votingDelay;
        votingDelay = newVotingDelay;
        emit VotingDelaySet(oldVotingDelay, votingDelay);
    }
    
    /**
     * @dev Set voting period
     */
    function _setVotingPeriod(uint256 newVotingPeriod) external onlyAdmin {
        require(
            newVotingPeriod >= MIN_VOTING_PERIOD && newVotingPeriod <= MAX_VOTING_PERIOD,
            "Invalid voting period"
        );
        uint256 oldVotingPeriod = votingPeriod;
        votingPeriod = newVotingPeriod;
        emit VotingPeriodSet(oldVotingPeriod, votingPeriod);
    }
    
    /**
     * @dev Set proposal threshold
     */
    function _setProposalThreshold(uint256 newProposalThreshold) external onlyAdmin {
        require(newProposalThreshold > 0, "Invalid proposal threshold");
        uint256 oldProposalThreshold = proposalThreshold;
        proposalThreshold = newProposalThreshold;
        emit ProposalThresholdSet(oldProposalThreshold, proposalThreshold);
    }
    
    /**
     * @dev Set quorum votes
     */
    function _setQuorumVotes(uint256 newQuorumVotes) external onlyAdmin {
        require(newQuorumVotes > 0, "Invalid quorum votes");
        uint256 oldQuorumVotes = quorumVotes;
        quorumVotes = newQuorumVotes;
        emit QuorumVotesSet(oldQuorumVotes, quorumVotes);
    }
    
    // Emergency Functions
    
    /**
     * @dev Pause contract
     */
    function pause() external onlyGuardian {
        require(!paused, "Already paused");
        paused = true;
        emit Paused(msg.sender);
    }
    
    /**
     * @dev Unpause contract
     */
    function unpause() external onlyAdmin {
        require(paused, "Not paused");
        paused = false;
        emit Unpaused(msg.sender);
    }
    
    // Utility Functions
    
    function queueOrRevertInternal(
        address target,
        uint256 value,
        string memory signature,
        bytes memory data,
        uint256 eta
    ) internal {
        require(
            ITimelock(timelock).queuedTransactions(
                keccak256(abi.encode(target, value, signature, data, eta))
            ),
            "Proposal action already queued at eta"
        );
        ITimelock(timelock).queueTransaction(target, value, signature, data, eta);
    }
    
    function safe32(uint256 n, string memory errorMessage) internal pure returns (uint32) {
        require(n < 2**32, errorMessage);
        return uint32(n);
    }
    
    function getChainId() internal view returns (uint256) {
        return block.chainid;
    }
    
    // Batch Functions
    
    /**
     * @dev Batch delegate to multiple addresses
     */
    function batchDelegate(address[] memory delegatees, address[] memory delegators) external whenNotPaused {
        require(delegatees.length == delegators.length, "Array length mismatch");
        require(delegatees.length > 0, "Empty arrays");
        require(delegatees.length <= 50, "Too many delegations");
        
        for (uint256 i = 0; i < delegatees.length; i++) {
            require(
                msg.sender == delegators[i] || allowance[delegators[i]][msg.sender] > 0,
                "Not authorized"
            );
            _delegate(delegators[i], delegatees[i]);
        }
    }
    
    /**
     * @dev Get multiple vote counts
     */
    function getMultipleCurrentVotes(address[] memory accounts) 
        external 
        view 
        returns (uint256[] memory votes) 
    {
        votes = new uint256[](accounts.length);
        for (uint256 i = 0; i < accounts.length; i++) {
            votes[i] = getCurrentVotes(accounts[i]);
        }
    }
    
    /**
     * @dev Get multiple proposal states
     */
    function getMultipleProposalStates(uint256[] memory proposalIds)
        external
        view
        returns (ProposalState[] memory states)
    {
        states = new ProposalState[](proposalIds.length);
        for (uint256 i = 0; i < proposalIds.length; i++) {
            states[i] = state(proposalIds[i]);
        }
    }
}

// Required interface for Timelock
interface ITimelock {
    function delay() external view returns (uint256);
    function GRACE_PERIOD() external view returns (uint256);
    function queuedTransactions(bytes32 hash) external view returns (bool);
    function queueTransaction(
        address target,
        uint256 value,
        string calldata signature,
        bytes calldata data,
        uint256 eta
    ) external returns (bytes32);
    function cancelTransaction(
        address target,
        uint256 value,
        string calldata signature,
        bytes calldata data,
        uint256 eta
    ) external;
    function executeTransaction(
        address target,
        uint256 value,
        string calldata signature,
        bytes calldata data,
        uint256 eta
    ) external payable returns (bytes memory);
}
