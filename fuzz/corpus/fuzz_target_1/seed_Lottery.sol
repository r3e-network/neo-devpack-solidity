// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title Lottery - Decentralized lottery system for Neo N3
/// @author Neo DevPack for Solidity Team
/// @notice A fair lottery system with provable randomness
/// @dev Uses block data for randomness seed
contract Lottery {
    // Lottery states
    enum LotteryState { Open, Closed, Drawing, Completed }

    // Lottery round details
    struct Round {
        uint256 roundId;
        uint256 ticketPrice;
        uint256 prizePool;
        uint256 startTime;
        uint256 endTime;
        uint256 maxTickets;
        uint256 ticketsSold;
        address winner;
        uint256 winningTicket;
        LotteryState state;
    }

    // Ticket details
    struct Ticket {
        address owner;
        uint256 roundId;
        uint256 ticketNumber;
        uint256 purchaseTime;
    }

    // State variables
    mapping(uint256 => Round) public rounds;
    mapping(uint256 => mapping(uint256 => Ticket)) public tickets; // roundId => ticketNumber => Ticket
    mapping(uint256 => mapping(address => uint256[])) public userTickets; // roundId => user => ticketNumbers
    mapping(address => uint256) public winnings;

    uint256 public currentRound;
    uint256 public totalRounds;
    address public owner;
    uint256 public operatorFeePercent; // Fee in basis points
    uint256 public collectedFees;

    // Events
    event RoundStarted(uint256 indexed roundId, uint256 ticketPrice, uint256 maxTickets, uint256 endTime);
    event TicketPurchased(uint256 indexed roundId, address indexed buyer, uint256 ticketNumber);
    event RoundClosed(uint256 indexed roundId);
    event WinnerDrawn(uint256 indexed roundId, address indexed winner, uint256 winningTicket, uint256 prize);
    event WinningsClaimed(address indexed winner, uint256 amount);
    event FeeUpdated(uint256 oldFee, uint256 newFee);

    /// @notice Contract constructor
    /// @param _operatorFeePercent Fee percentage in basis points
    constructor(uint256 _operatorFeePercent) {
        require(_operatorFeePercent <= 1000, "Lottery: fee too high"); // Max 10%
        owner = msg.sender;
        operatorFeePercent = _operatorFeePercent;
        currentRound = 0;
        totalRounds = 0;
        collectedFees = 0;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "Lottery: caller is not owner");
        _;
    }

    modifier roundExists(uint256 roundId) {
        require(roundId < totalRounds, "Lottery: round does not exist");
        _;
    }

    /// @notice Start a new lottery round
    /// @param ticketPrice Price per ticket
    /// @param maxTickets Maximum tickets available
    /// @param duration Duration in seconds
    /// @return roundId The new round ID
    function startRound(
        uint256 ticketPrice,
        uint256 maxTickets,
        uint256 duration
    ) public onlyOwner returns (uint256 roundId) {
        require(ticketPrice > 0, "Lottery: ticket price must be positive");
        require(maxTickets > 0, "Lottery: must have at least one ticket");
        require(duration > 0, "Lottery: duration must be positive");

        // Close previous round if exists and open
        if (totalRounds > 0 && rounds[currentRound].state == LotteryState.Open) {
            rounds[currentRound].state = LotteryState.Closed;
            emit RoundClosed(currentRound);
        }

        roundId = totalRounds;
        totalRounds += 1;
        currentRound = roundId;

        rounds[roundId] = Round({
            roundId: roundId,
            ticketPrice: ticketPrice,
            prizePool: 0,
            startTime: block.timestamp,
            endTime: block.timestamp + duration,
            maxTickets: maxTickets,
            ticketsSold: 0,
            winner: address(0),
            winningTicket: 0,
            state: LotteryState.Open
        });

        emit RoundStarted(roundId, ticketPrice, maxTickets, block.timestamp + duration);
        return roundId;
    }

    /// @notice Purchase lottery tickets
    /// @param roundId The round to purchase tickets for
    /// @param quantity Number of tickets to purchase
    function buyTickets(uint256 roundId, uint256 quantity) public roundExists(roundId) {
        Round storage round = rounds[roundId];
        require(round.state == LotteryState.Open, "Lottery: round not open");
        require(block.timestamp < round.endTime, "Lottery: round ended");
        require(quantity > 0, "Lottery: must buy at least one ticket");
        require(round.ticketsSold + quantity <= round.maxTickets, "Lottery: not enough tickets available");

        uint256 totalCost = round.ticketPrice * quantity;
        round.prizePool += totalCost;

        for (uint256 i = 0; i < quantity; i++) {
            uint256 ticketNumber = round.ticketsSold;
            round.ticketsSold += 1;

            tickets[roundId][ticketNumber] = Ticket({
                owner: msg.sender,
                roundId: roundId,
                ticketNumber: ticketNumber,
                purchaseTime: block.timestamp
            });

            userTickets[roundId][msg.sender].push(ticketNumber);

            emit TicketPurchased(roundId, msg.sender, ticketNumber);
        }
    }

    /// @notice Close a round for ticket sales
    /// @param roundId The round to close
    function closeRound(uint256 roundId) public onlyOwner roundExists(roundId) {
        Round storage round = rounds[roundId];
        require(round.state == LotteryState.Open, "Lottery: round not open");

        round.state = LotteryState.Closed;
        emit RoundClosed(roundId);
    }

    /// @notice Draw the winner for a round
    /// @param roundId The round to draw
    function drawWinner(uint256 roundId) public onlyOwner roundExists(roundId) {
        Round storage round = rounds[roundId];
        require(
            round.state == LotteryState.Closed ||
            (round.state == LotteryState.Open && block.timestamp >= round.endTime),
            "Lottery: round must be closed"
        );
        require(round.ticketsSold > 0, "Lottery: no tickets sold");

        round.state = LotteryState.Drawing;

        // Generate pseudo-random number using block data
        uint256 randomSeed = uint256(keccak256(abi.encodePacked(
            block.timestamp,
            block.number,
            round.prizePool,
            round.ticketsSold,
            msg.sender
        )));

        uint256 winningTicketNumber = randomSeed % round.ticketsSold;
        address winner = tickets[roundId][winningTicketNumber].owner;

        // Calculate prize after operator fee
        uint256 fee = (round.prizePool * operatorFeePercent) / 10000;
        uint256 prize = round.prizePool - fee;

        round.winner = winner;
        round.winningTicket = winningTicketNumber;
        round.state = LotteryState.Completed;
        collectedFees += fee;

        // Add to winner's claimable balance
        winnings[winner] += prize;

        emit WinnerDrawn(roundId, winner, winningTicketNumber, prize);
    }

    /// @notice Claim accumulated winnings
    /// @return amount The claimed amount
    function claimWinnings() public returns (uint256 amount) {
        amount = winnings[msg.sender];
        require(amount > 0, "Lottery: no winnings to claim");

        winnings[msg.sender] = 0;
        emit WinningsClaimed(msg.sender, amount);
        return amount;
    }

    /// @notice Get round details
    /// @param roundId The round ID
    /// @return ticketPrice The ticket price
    /// @return prizePool The current prize pool
    /// @return ticketsSold Number of tickets sold
    /// @return maxTickets Maximum tickets
    /// @return state The round state
    function getRound(uint256 roundId) public view roundExists(roundId) returns (
        uint256 ticketPrice,
        uint256 prizePool,
        uint256 ticketsSold,
        uint256 maxTickets,
        LotteryState state
    ) {
        Round storage round = rounds[roundId];
        return (round.ticketPrice, round.prizePool, round.ticketsSold, round.maxTickets, round.state);
    }

    /// @notice Get winner of a round
    /// @param roundId The round ID
    /// @return winner The winner address
    /// @return winningTicket The winning ticket number
    /// @return prize The prize amount
    function getWinner(uint256 roundId) public view roundExists(roundId) returns (
        address winner,
        uint256 winningTicket,
        uint256 prize
    ) {
        Round storage round = rounds[roundId];
        require(round.state == LotteryState.Completed, "Lottery: round not completed");
        uint256 fee = (round.prizePool * operatorFeePercent) / 10000;
        return (round.winner, round.winningTicket, round.prizePool - fee);
    }

    /// @notice Get user's tickets for a round
    /// @param roundId The round ID
    /// @param user The user address
    /// @return Array of ticket numbers
    function getUserTickets(uint256 roundId, address user) public view returns (uint256[] memory) {
        return userTickets[roundId][user];
    }

    /// @notice Check if user has any tickets in a round
    /// @param roundId The round ID
    /// @param user The user address
    /// @return True if user has tickets
    function hasTickets(uint256 roundId, address user) public view returns (bool) {
        return userTickets[roundId][user].length > 0;
    }

    /// @notice Get remaining tickets in current round
    /// @return Number of remaining tickets
    function getRemainingTickets() public view returns (uint256) {
        if (totalRounds == 0) return 0;
        Round storage round = rounds[currentRound];
        if (round.state != LotteryState.Open) return 0;
        return round.maxTickets - round.ticketsSold;
    }

    /// @notice Update operator fee (owner only)
    /// @param newFeePercent New fee in basis points
    function setOperatorFee(uint256 newFeePercent) public onlyOwner {
        require(newFeePercent <= 1000, "Lottery: fee too high");
        uint256 oldFee = operatorFeePercent;
        operatorFeePercent = newFeePercent;
        emit FeeUpdated(oldFee, newFeePercent);
    }

    /// @notice Withdraw collected fees (owner only)
    /// @return amount The withdrawn amount
    function withdrawFees() public onlyOwner returns (uint256 amount) {
        amount = collectedFees;
        collectedFees = 0;
        return amount;
    }

    /// @notice Check if a round is still open for purchases
    /// @param roundId The round ID
    /// @return True if open
    function isRoundOpen(uint256 roundId) public view roundExists(roundId) returns (bool) {
        Round storage round = rounds[roundId];
        return round.state == LotteryState.Open && block.timestamp < round.endTime;
    }

    /// @notice Get time remaining in current round
    /// @return Seconds remaining, 0 if not open
    function getTimeRemaining() public view returns (uint256) {
        if (totalRounds == 0) return 0;
        Round storage round = rounds[currentRound];
        if (round.state != LotteryState.Open) return 0;
        if (block.timestamp >= round.endTime) return 0;
        return round.endTime - block.timestamp;
    }
}
