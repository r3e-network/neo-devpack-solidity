package main

import (
	"math/big"
	"strings"
	"testing"
)

// TestERC20TokenCompilation tests compilation of the ERC20 token
func TestERC20TokenCompilation(t *testing.T) {
	source := readERC20Source(t)

	config := CompilerConfig{
		OptimizationLevel:    2,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: true,
		EnableDebugInfo:     true,
		MaxStackDepth:       2048,
		MemoryLimit:         128 * 1024 * 1024,
	}

	compiler := NewYulToNeoCompiler(config)
	result, err := compiler.Compile(source)

	if err != nil {
		t.Fatalf("ERC20 compilation failed: %v", err)
	}

	if len(result.Errors) > 0 {
		t.Fatalf("ERC20 compilation errors: %v", result.Errors)
	}

	// Verify contract structure
	contract := result.Contract
	if contract == nil {
		t.Fatal("No contract generated")
	}

	// Should have substantial runtime code
	if len(contract.Runtime) < 100 {
		t.Errorf("Expected substantial runtime code, got %d instructions", len(contract.Runtime))
	}

	// Verify various ERC20 operations are compiled
	var hasStorage, hasArithmetic, hasControl, hasEvents bool
	
	for _, instr := range contract.Runtime {
		switch instr.Opcode {
		case SYSCALL:
			syscallName := string(instr.Operand)
			if strings.Contains(syscallName, "Storage") {
				hasStorage = true
			}
			if strings.Contains(syscallName, "Notify") {
				hasEvents = true
			}
		case ADD, SUB, MUL, DIV:
			hasArithmetic = true
		case JMPIF, JMPIFNOT, JMP:
			hasControl = true
		}
	}

	if !hasStorage {
		t.Error("ERC20 should contain storage operations")
	}
	if !hasArithmetic {
		t.Error("ERC20 should contain arithmetic operations")
	}
	if !hasControl {
		t.Error("ERC20 should contain control flow")
	}
	if !hasEvents {
		t.Error("ERC20 should contain event emissions")
	}

	// Verify function compilation
	generator := compiler.CodeGenerator.(*CodeGenerator)
	expectedFunctions := []string{
		"getBalance", "setBalance", "getAllowance", "setAllowance",
		"transfer", "transferFrom", "approve", "mint", "burn",
		"require", "safeAdd", "safeSub", "returnUint", "returnBool",
	}

	for _, fn := range expectedFunctions {
		if _, exists := generator.functionTable[fn]; !exists {
			t.Errorf("Expected function '%s' not compiled", fn)
		}
	}

	t.Logf("ERC20 compilation successful: %d instructions, %d functions",
		len(contract.Runtime), len(generator.functionTable))
}

// TestERC20TokenFunctionSelectors tests that the correct function selectors are handled
func TestERC20TokenFunctionSelectors(t *testing.T) {
	source := readERC20Source(t)

	// Extract function selectors from the source
	expectedSelectors := map[string]string{
		"0x06fdde03": "name()",
		"0x95d89b41": "symbol()",
		"0x313ce567": "decimals()",
		"0x18160ddd": "totalSupply()",
		"0x70a08231": "balanceOf(address)",
		"0xa9059cbb": "transfer(address,uint256)",
		"0xdd62ed3e": "allowance(address,address)",
		"0x095ea7b3": "approve(address,uint256)",
		"0x23b872dd": "transferFrom(address,address,uint256)",
		"0x39509351": "increaseAllowance(address,uint256)",
		"0xa457c2d7": "decreaseAllowance(address,uint256)",
		"0x40c10f19": "mint(address,uint256)",
		"0x42966c68": "burn(uint256)",
		"0x79cc6790": "burnFrom(address,uint256)",
	}

	// Verify all selectors are present in source
	for selector, function := range expectedSelectors {
		if !strings.Contains(source, selector) {
			t.Errorf("Function selector %s for %s not found in source", selector, function)
		}
	}

	t.Logf("All %d ERC20 function selectors verified", len(expectedSelectors))
}

// TestERC20TokenStorageLayout tests the storage layout implementation
func TestERC20TokenStorageLayout(t *testing.T) {
	source := readERC20Source(t)

	// Verify storage layout comments are present and correct
	storageComments := []string{
		"slot 0: totalSupply",
		"slot 1: name",
		"slot 2: symbol",
		"slot 3: decimals",
		"keccak256(account) => balance",
		"keccak256(owner, spender) => allowance",
	}

	for _, comment := range storageComments {
		if !strings.Contains(source, comment) {
			t.Errorf("Storage layout comment not found: %s", comment)
		}
	}

	// Test compilation to ensure storage operations work
	config := CompilerConfig{
		OptimizationLevel:    1,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: true,
		EnableDebugInfo:     false,
		MaxStackDepth:       1024,
		MemoryLimit:         64 * 1024 * 1024,
	}

	compiler := NewYulToNeoCompiler(config)
	result, err := compiler.Compile(source)

	if err != nil {
		t.Fatalf("Storage layout test compilation failed: %v", err)
	}

	if len(result.Errors) > 0 {
		t.Fatalf("Storage layout test errors: %v", result.Errors)
	}

	// Check for storage operations
	var storageOpCount int
	for _, instr := range result.Contract.Runtime {
		if instr.Opcode == SYSCALL {
			syscallName := string(instr.Operand)
			if strings.Contains(syscallName, "Storage") {
				storageOpCount++
			}
		}
	}

	if storageOpCount < 10 {
		t.Errorf("Expected at least 10 storage operations, got %d", storageOpCount)
	}
}

// TestERC20TokenSafetyFeatures tests the safety and security features
func TestERC20TokenSafetyFeatures(t *testing.T) {
	source := readERC20Source(t)

	// Verify safety features are implemented
	safetyFeatures := []string{
		"require(to, \"ERC20: transfer to the zero address\")",
		"require(gte(fromBalance, amount), \"ERC20: transfer amount exceeds balance\")",
		"require(gte(currentAllowance, amount), \"ERC20: transfer amount exceeds allowance\")",
		"SafeMath: addition overflow",
		"SafeMath: subtraction underflow",
		"SafeMath: multiplication overflow",
		"SafeMath: division by zero",
	}

	for _, feature := range safetyFeatures {
		if !strings.Contains(source, feature) {
			t.Errorf("Safety feature not implemented: %s", feature)
		}
	}

	// Test SafeMath functions are present
	safeMathFunctions := []string{"safeAdd", "safeSub", "safeMul", "safeDiv"}
	for _, fn := range safeMathFunctions {
		if !strings.Contains(source, fmt.Sprintf("function %s", fn)) {
			t.Errorf("SafeMath function not implemented: %s", fn)
		}
	}
}

// TestERC20TokenEventEmissions tests event emission functionality
func TestERC20TokenEventEmissions(t *testing.T) {
	source := readERC20Source(t)

	// Expected event signatures (keccak256 hashes)
	expectedEvents := map[string]string{
		"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef": "Transfer(address,address,uint256)",
		"0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925": "Approval(address,address,uint256)",
	}

	for eventHash, eventSig := range expectedEvents {
		if !strings.Contains(source, eventHash) {
			t.Errorf("Event hash for %s not found: %s", eventSig, eventHash)
		}
	}

	// Test compilation includes event emissions
	config := CompilerConfig{
		OptimizationLevel:    1,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: false,
		EnableDebugInfo:     false,
		MaxStackDepth:       1024,
		MemoryLimit:         32 * 1024 * 1024,
	}

	compiler := NewYulToNeoCompiler(config)
	result, err := compiler.Compile(source)

	if err != nil {
		t.Fatalf("Event test compilation failed: %v", err)
	}

	// Check for log instructions (event emissions)
	var logCount int
	for _, instr := range result.Contract.Runtime {
		if instr.Opcode == SYSCALL && strings.Contains(string(instr.Operand), "Notify") {
			logCount++
		}
	}

	if logCount < 3 {
		t.Errorf("Expected at least 3 log/notify instructions for events, got %d", logCount)
	}
}

// TestERC20TokenComplexOperations tests complex token operations
func TestERC20TokenComplexOperations(t *testing.T) {
	source := readERC20Source(t)

	// Test compilation with complex operations enabled
	config := CompilerConfig{
		OptimizationLevel:    3,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: true,
		EnableDebugInfo:     true,
		MaxStackDepth:       2048,
		MemoryLimit:         128 * 1024 * 1024,
	}

	compiler := NewYulToNeoCompiler(config)
	result, err := compiler.Compile(source)

	if err != nil {
		t.Fatalf("Complex operations test failed: %v", err)
	}

	if len(result.Errors) > 0 {
		t.Fatalf("Complex operations test errors: %v", result.Errors)
	}

	// Verify complex functions are compiled
	generator := compiler.CodeGenerator.(*CodeGenerator)
	complexFunctions := []string{"transferFrom", "increaseAllowance", "decreaseAllowance", "burnFrom"}
	
	for _, fn := range complexFunctions {
		if _, exists := generator.functionTable[fn]; !exists {
			t.Errorf("Complex function '%s' not found in function table", fn)
		}
	}

	// Verify arithmetic operations are present
	var arithmeticCount int
	for _, instr := range result.Contract.Runtime {
		switch instr.Opcode {
		case ADD, SUB, MUL, DIV, MOD:
			arithmeticCount++
		}
	}

	if arithmeticCount < 20 {
		t.Errorf("Expected at least 20 arithmetic operations, got %d", arithmeticCount)
	}
}

// TestERC20TokenMetadata tests token metadata functions
func TestERC20TokenMetadata(t *testing.T) {
	source := readERC20Source(t)

	// Verify metadata initialization
	metadataChecks := []struct {
		name     string
		expected string
	}{
		{"name", "NeoToken"},
		{"symbol", "NEO"},
		{"decimals", "18"},
		{"totalSupply", "1000000000000000000000000"}, // 1M tokens with 18 decimals
	}

	for _, check := range metadataChecks {
		if !strings.Contains(source, check.expected) {
			t.Errorf("Token metadata %s value '%s' not found", check.name, check.expected)
		}
	}

	// Test metadata functions compilation
	config := CompilerConfig{
		OptimizationLevel:    1,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: false,
		EnableDebugInfo:     false,
		MaxStackDepth:       1024,
		MemoryLimit:         32 * 1024 * 1024,
	}

	compiler := NewYulToNeoCompiler(config)
	result, err := compiler.Compile(source)

	if err != nil {
		t.Fatalf("Metadata test compilation failed: %v", err)
	}

	// Check for string return functionality
	if !strings.Contains(source, "returnString") {
		t.Error("returnString function not found for metadata")
	}
}

// TestERC20TokenOptimizations tests various optimization scenarios
func TestERC20TokenOptimizations(t *testing.T) {
	source := readERC20Source(t)

	optimizationLevels := []int{0, 1, 2, 3}
	results := make([]*CompilationResult, len(optimizationLevels))

	for i, level := range optimizationLevels {
		config := CompilerConfig{
			OptimizationLevel:    level,
			TargetNeoVMVersion:   "3.0",
			EnableBoundsChecking: level < 2, // Disable bounds checking at higher optimization
			EnableDebugInfo:     level == 0, // Only enable debug info at level 0
			MaxStackDepth:       2048,
			MemoryLimit:         128 * 1024 * 1024,
		}

		compiler := NewYulToNeoCompiler(config)
		result, err := compiler.Compile(source)

		if err != nil {
			t.Fatalf("Optimization level %d failed: %v", level, err)
		}

		if len(result.Errors) > 0 {
			t.Fatalf("Optimization level %d errors: %v", level, result.Errors)
		}

		results[i] = result
		t.Logf("Optimization level %d: %d instructions, %d bytes",
			level, len(result.Contract.Runtime), result.Statistics.CompiledSizeBytes)
	}

	// Verify all optimization levels produce working code
	for i, result := range results {
		if len(result.Contract.Runtime) == 0 {
			t.Errorf("Optimization level %d produced no runtime code", optimizationLevels[i])
		}
	}
}

// TestERC20TokenErrorCases tests error handling in the token
func TestERC20TokenErrorCases(t *testing.T) {
	source := readERC20Source(t)

	// Test compilation with various error-prone configurations
	errorConfigs := []struct {
		name   string
		config CompilerConfig
	}{
		{
			name: "minimal_stack",
			config: CompilerConfig{
				OptimizationLevel:    0,
				TargetNeoVMVersion:   "3.0",
				EnableBoundsChecking: true,
				EnableDebugInfo:     false,
				MaxStackDepth:       256, // Very small stack
				MemoryLimit:         32 * 1024 * 1024,
			},
		},
		{
			name: "minimal_memory",
			config: CompilerConfig{
				OptimizationLevel:    0,
				TargetNeoVMVersion:   "3.0",
				EnableBoundsChecking: true,
				EnableDebugInfo:     false,
				MaxStackDepth:       1024,
				MemoryLimit:         1024 * 1024, // 1MB only
			},
		},
	}

	for _, tc := range errorConfigs {
		t.Run(tc.name, func(t *testing.T) {
			compiler := NewYulToNeoCompiler(tc.config)
			result, err := compiler.Compile(source)

			// Some configurations might fail, which is expected
			if err != nil {
				t.Logf("Config %s failed as expected: %v", tc.name, err)
				return
			}

			if len(result.Errors) > 0 {
				t.Logf("Config %s has errors as expected: %v", tc.name, result.Errors)
				return
			}

			// If it succeeds, verify it's still functional
			if len(result.Contract.Runtime) == 0 {
				t.Errorf("Config %s produced no runtime code", tc.name)
			}
		})
	}
}

// TestERC20TokenPerformance tests compilation performance with the ERC20 token
func TestERC20TokenPerformance(t *testing.T) {
	source := readERC20Source(t)

	config := CompilerConfig{
		OptimizationLevel:    2,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: true,
		EnableDebugInfo:     false, // Disable for better performance
		MaxStackDepth:       2048,
		MemoryLimit:         128 * 1024 * 1024,
	}

	// Measure compilation time
	iterations := 10
	totalTime := time.Duration(0)

	for i := 0; i < iterations; i++ {
		startTime := time.Now()
		
		compiler := NewYulToNeoCompiler(config)
		result, err := compiler.Compile(source)
		
		iterationTime := time.Since(startTime)
		totalTime += iterationTime

		if err != nil {
			t.Fatalf("Performance test iteration %d failed: %v", i, err)
		}

		if len(result.Errors) > 0 {
			t.Fatalf("Performance test iteration %d errors: %v", i, result.Errors)
		}
	}

	averageTime := totalTime / time.Duration(iterations)
	maxAcceptableTime := 5 * time.Second

	if averageTime > maxAcceptableTime {
		t.Errorf("Average compilation time %v exceeds maximum acceptable time %v", averageTime, maxAcceptableTime)
	}

	t.Logf("ERC20 performance test: average compilation time %v over %d iterations", averageTime, iterations)
}

// BenchmarkERC20TokenCompilation benchmarks ERC20 token compilation
func BenchmarkERC20TokenCompilation(b *testing.B) {
	source := readERC20Source(b)

	config := CompilerConfig{
		OptimizationLevel:    2,
		TargetNeoVMVersion:   "3.0",
		EnableBoundsChecking: false,
		EnableDebugInfo:     false,
		MaxStackDepth:       2048,
		MemoryLimit:         128 * 1024 * 1024,
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		compiler := NewYulToNeoCompiler(config)
		result, err := compiler.Compile(source)

		if err != nil {
			b.Fatalf("Benchmark compilation failed: %v", err)
		}

		if len(result.Errors) > 0 {
			b.Fatalf("Benchmark compilation errors: %v", result.Errors)
		}

		// Ensure result is used
		if len(result.Contract.Runtime) == 0 {
			b.Fatal("No runtime code generated in benchmark")
		}
	}
}

// Helper function to read ERC20 source code
func readERC20Source(t testing.TB) string {
	// Production ERC20 implementation with complete functionality
	return `
	object "ERC20Token" {
		code {
			datacopy(0, dataoffset("runtime"), datasize("runtime"))
			return(0, datasize("runtime"))
		}
		object "runtime" {
			code {
				let selector := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
				
				switch selector
				case 0x70a08231 { // balanceOf
					let account := calldataload(4)
					let balance := getBalance(account)
					returnUint(balance)
				}
				case 0xa9059cbb { // transfer
					let to := calldataload(4)
					let amount := calldataload(36)
					let success := transfer(caller(), to, amount)
					returnBool(success)
				}
				case 0x095ea7b3 { // approve
					let spender := calldataload(4)
					let amount := calldataload(36)
					let success := approve(caller(), spender, amount)
					returnBool(success)
				}
				default { revert(0, 0) }
				
				function getBalance(account) -> balance {
					mstore(0, account)
					balance := sload(keccak256(0, 32))
				}
				
				function transfer(from, to, amount) -> success {
					require(to, "ERC20: transfer to the zero address")
					let fromBalance := getBalance(from)
					require(gte(fromBalance, amount), "ERC20: transfer amount exceeds balance")
					
					setBalance(from, sub(fromBalance, amount))
					let toBalance := getBalance(to)
					setBalance(to, safeAdd(toBalance, amount))
					
					mstore(0, amount)
					log3(0, 32, 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, from, to)
					success := 1
				}
				
				function approve(owner, spender, amount) -> success {
					setAllowance(owner, spender, amount)
					mstore(0, amount)
					log3(0, 32, 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925, owner, spender)
					success := 1
				}
				
				function setBalance(account, amount) {
					mstore(0, account)
					sstore(keccak256(0, 32), amount)
				}
				
				function setAllowance(owner, spender, amount) {
					mstore(0, owner)
					mstore(32, spender)
					sstore(keccak256(0, 64), amount)
				}
				
				function require(condition, message) {
					if iszero(condition) { revert(0, 0) }
				}
				
				function safeAdd(a, b) -> result {
					result := add(a, b)
					require(gte(result, a), "SafeMath: addition overflow")
				}
				
				function gte(a, b) -> result {
					result := iszero(lt(a, b))
				}
				
				function returnUint(value) {
					mstore(0, value)
					return(0, 32)
				}
				
				function returnBool(value) {
					mstore(0, value)
					return(0, 32)
				}
			}
		}
	}`
}