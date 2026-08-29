// ERC20 Token Implementation in Yul for Neo Blockchain
// Implements the standard ERC20 interface with full functionality

object "ERC20Token" {
    code {
        // Deploy code - sets up initial storage and returns runtime code
        datacopy(0, dataoffset("runtime"), datasize("runtime"))
        return(0, datasize("runtime"))
    }
    
    object "runtime" {
        code {
            // Storage layout:
            // slot 0: totalSupply
            // slot 1: name (as bytes32)
            // slot 2: symbol (as bytes32) 
            // slot 3: decimals
            // keccak256(account) => balance
            // keccak256(owner, spender) => allowance
            
            // Constructor initialization (only runs on deployment)
            if iszero(extcodesize(address())) {
                // Set token metadata
                sstore(0, 1000000000000000000000000) // 1M tokens with 18 decimals
                sstore(1, 0x4E656F546F6B656E0000000000000000000000000000000000000000000000) // "NeoToken"
                sstore(2, 0x4E454F0000000000000000000000000000000000000000000000000000000000) // "NEO"
                sstore(3, 18) // decimals
                
                // Mint initial supply to deployer
                let deployer := caller()
                let balanceSlot := keccak256(0, 32)
                mstore(0, deployer)
                mstore(32, balanceSlot)
                sstore(keccak256(0, 64), sload(0))
                
                // Emit Transfer(0, deployer, totalSupply)
                mstore(0, sload(0))
                log3(0, 32, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, 0, deployer)
            }
            
            // Function selector from calldata
            let selector := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
            
            switch selector
            
            // name() returns (string)
            case 0x06fdde03 {
                returnString(sload(1))
            }
            
            // symbol() returns (string)  
            case 0x95d89b41 {
                returnString(sload(2))
            }
            
            // decimals() returns (uint8)
            case 0x313ce567 {
                returnUint(sload(3))
            }
            
            // totalSupply() returns (uint256)
            case 0x18160ddd {
                returnUint(sload(0))
            }
            
            // balanceOf(address) returns (uint256)
            case 0x70a08231 {
                let account := calldataload(4)
                returnUint(getBalance(account))
            }
            
            // transfer(address,uint256) returns (bool)
            case 0xa9059cbb {
                let to := calldataload(4)
                let amount := calldataload(36)
                let success := transfer(caller(), to, amount)
                returnBool(success)
            }
            
            // allowance(address,address) returns (uint256)
            case 0xdd62ed3e {
                let owner := calldataload(4)
                let spender := calldataload(36)
                returnUint(getAllowance(owner, spender))
            }
            
            // approve(address,uint256) returns (bool)
            case 0x095ea7b3 {
                let spender := calldataload(4)
                let amount := calldataload(36)
                let success := approve(caller(), spender, amount)
                returnBool(success)
            }
            
            // transferFrom(address,address,uint256) returns (bool)
            case 0x23b872dd {
                let from := calldataload(4)
                let to := calldataload(36)
                let amount := calldataload(68)
                let success := transferFrom(caller(), from, to, amount)
                returnBool(success)
            }
            
            // increaseAllowance(address,uint256) returns (bool)
            case 0x39509351 {
                let spender := calldataload(4)
                let addedValue := calldataload(36)
                let currentAllowance := getAllowance(caller(), spender)
                let newAllowance := safeAdd(currentAllowance, addedValue)
                let success := approve(caller(), spender, newAllowance)
                returnBool(success)
            }
            
            // decreaseAllowance(address,uint256) returns (bool)
            case 0xa457c2d7 {
                let spender := calldataload(4)
                let subtractedValue := calldataload(36)
                let currentAllowance := getAllowance(caller(), spender)
                require(gte(currentAllowance, subtractedValue), "ERC20: decreased allowance below zero")
                let newAllowance := sub(currentAllowance, subtractedValue)
                let success := approve(caller(), spender, newAllowance)
                returnBool(success)
            }
            
            // mint(address,uint256) - only owner function
            case 0x40c10f19 {
                requireOwner()
                let to := calldataload(4)
                let amount := calldataload(36)
                mint(to, amount)
                returnBool(1)
            }
            
            // burn(uint256)
            case 0x42966c68 {
                let amount := calldataload(4)
                burn(caller(), amount)
                returnBool(1)
            }
            
            // burnFrom(address,uint256)
            case 0x79cc6790 {
                let from := calldataload(4)
                let amount := calldataload(36)
                let currentAllowance := getAllowance(from, caller())
                require(gte(currentAllowance, amount), "ERC20: burn amount exceeds allowance")
                approve(from, caller(), sub(currentAllowance, amount))
                burn(from, amount)
                returnBool(1)
            }
            
            default {
                revert(0, 0)
            }
            
            // ===== HELPER FUNCTIONS =====
            
            function getBalance(account) -> balance {
                mstore(0, account)
                balance := sload(keccak256(0, 32))
            }
            
            function setBalance(account, amount) {
                mstore(0, account)
                sstore(keccak256(0, 32), amount)
            }
            
            function getAllowance(owner, spender) -> allowance {
                mstore(0, owner)
                mstore(32, spender)
                allowance := sload(keccak256(0, 64))
            }
            
            function setAllowance(owner, spender, amount) {
                mstore(0, owner)
                mstore(32, spender)
                sstore(keccak256(0, 64), amount)
            }
            
            function transfer(from, to, amount) -> success {
                // Check for zero address
                require(to, "ERC20: transfer to the zero address")
                
                // Get balances
                let fromBalance := getBalance(from)
                require(gte(fromBalance, amount), "ERC20: transfer amount exceeds balance")
                
                // Update balances
                setBalance(from, sub(fromBalance, amount))
                let toBalance := getBalance(to)
                setBalance(to, safeAdd(toBalance, amount))
                
                // Emit Transfer event
                mstore(0, amount)
                log3(0, 32, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, from, to)
                
                success := 1
            }
            
            function transferFrom(spender, from, to, amount) -> success {
                // Check allowance
                let currentAllowance := getAllowance(from, spender)
                require(gte(currentAllowance, amount), "ERC20: transfer amount exceeds allowance")
                
                // Update allowance
                setAllowance(from, spender, sub(currentAllowance, amount))
                
                // Perform transfer
                success := transfer(from, to, amount)
            }
            
            function approve(owner, spender, amount) -> success {
                require(owner, "ERC20: approve from the zero address")
                require(spender, "ERC20: approve to the zero address")
                
                setAllowance(owner, spender, amount)
                
                // Emit Approval event
                mstore(0, amount)
                log3(0, 32, 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925, owner, spender)
                
                success := 1
            }
            
            function mint(to, amount) -> success {
                require(to, "ERC20: mint to the zero address")
                
                // Increase total supply
                let totalSupply := sload(0)
                let newTotalSupply := safeAdd(totalSupply, amount)
                sstore(0, newTotalSupply)
                
                // Increase recipient balance
                let toBalance := getBalance(to)
                setBalance(to, safeAdd(toBalance, amount))
                
                // Emit Transfer event from zero address
                mstore(0, amount)
                log3(0, 32, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, 0, to)
                
                success := 1
            }
            
            function burn(from, amount) -> success {
                require(from, "ERC20: burn from the zero address")
                
                let fromBalance := getBalance(from)
                require(gte(fromBalance, amount), "ERC20: burn amount exceeds balance")
                
                // Decrease balance
                setBalance(from, sub(fromBalance, amount))
                
                // Decrease total supply
                let totalSupply := sload(0)
                sstore(0, sub(totalSupply, amount))
                
                // Emit Transfer event to zero address
                mstore(0, amount)
                log3(0, 32, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, from, 0)
                
                success := 1
            }
            
            // ===== UTILITY FUNCTIONS =====
            
            function require(condition, message) {
                if iszero(condition) {
                    revert(0, 0)
                }
            }
            
            function requireOwner() {
                // In this example, the deployer is the owner
                // In production, you might want to store the owner address
                require(eq(caller(), address()), "ERC20: caller is not the owner")
            }
            
            function safeAdd(a, b) -> result {
                result := add(a, b)
                require(gte(result, a), "SafeMath: addition overflow")
            }
            
            function safeSub(a, b) -> result {
                require(gte(a, b), "SafeMath: subtraction underflow")
                result := sub(a, b)
            }
            
            function safeMul(a, b) -> result {
                if iszero(a) {
                    result := 0
                    leave
                }
                result := mul(a, b)
                require(eq(div(result, a), b), "SafeMath: multiplication overflow")
            }
            
            function safeDiv(a, b) -> result {
                require(b, "SafeMath: division by zero")
                result := div(a, b)
            }
            
            function gte(a, b) -> result {
                result := iszero(lt(a, b))
            }
            
            function lte(a, b) -> result {
                result := iszero(gt(a, b))
            }
            
            // ===== RETURN FUNCTIONS =====
            
            function returnUint(value) {
                mstore(0, value)
                return(0, 32)
            }
            
            function returnBool(value) {
                mstore(0, value)
                return(0, 32)
            }
            
            function returnString(value) {
                // Convert bytes32 to string and return
                mstore(0, 32)  // offset
                mstore(32, 32) // length  
                mstore(64, value)
                return(0, 96)
            }
        }
    }
}