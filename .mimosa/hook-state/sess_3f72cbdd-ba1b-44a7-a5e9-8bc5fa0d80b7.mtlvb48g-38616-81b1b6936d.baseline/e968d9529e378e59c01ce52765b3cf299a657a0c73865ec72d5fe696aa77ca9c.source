export interface ContractVerifier {
  verify(request: VerificationRequest): Promise<VerificationResult>;
  checkStatus(guid: string): Promise<VerificationStatus>;
  getSourceCode(address: string): Promise<SourceCodeResult>;
  getSimilarContracts(address: string): Promise<SimilarContract[]>;
}

export interface VerificationRequest {
  contractAddress: string;
  sourceCode: string | SourceFile[];
  contractName: string;
  compilerVersion: string;
  constructorArguments?: string;
  optimizationUsed: boolean;
  runs: number;
  evmVersion?: string;
  libraries?: { [name: string]: string };
  licenseType?: string;
  proxy?: boolean;
  implementationAddress?: string;
}

export interface SourceFile {
  fileName: string;
  content: string;
}

export interface VerificationResult {
  success: boolean;
  guid?: string;
  status: VerificationStatus;
  message: string;
  sourceCodeId?: string;
  abi?: any[];
  bytecode?: string;
  errors?: VerificationError[];
  warnings?: VerificationWarning[];
  explorerUrl?: string;
}

export interface VerificationStatus {
  guid: string;
  status: 'pending' | 'success' | 'failure' | 'unknown';
  result: string;
  message: string;
  submissionDate: Date;
  completionDate?: Date;
}

export interface VerificationError {
  type:
    | 'compilation'
    | 'bytecode_mismatch'
    | 'constructor_args'
    | 'library_linking'
    | 'submission'
    | 'verification'
    | 'pipeline';
  message: string;
  line?: number;
  column?: number;
  file?: string;
  suggestion?: string;
}

export interface VerificationWarning {
  type: 'optimization' | 'pragma' | 'import' | 'license';
  message: string;
  line?: number;
  column?: number;
  file?: string;
}

export interface SourceCodeResult {
  contractAddress: string;
  contractName: string;
  compilerVersion: string;
  optimizationUsed: boolean;
  runs: number;
  evmVersion: string;
  sourceCode: string | SourceFile[];
  abi: any[];
  constructorArguments: string;
  libraries: { [name: string]: string };
  licenseType: string;
  proxy: boolean;
  implementationAddress?: string;
  swarmSource?: string;
  verificationDate: Date;
}

export interface SimilarContract {
  address: string;
  name: string;
  similarity: number;
  matchedFunctions: string[];
  compilerVersion: string;
  verificationDate: Date;
}

export interface MultiVerifier {
  verifiers: Map<string, ContractVerifier>;
  
  addVerifier(name: string, verifier: ContractVerifier): void;
  removeVerifier(name: string): void;
  verify(request: VerificationRequest, verifiers?: string[]): Promise<MultiVerificationResult>;
  verifyAll(request: VerificationRequest): Promise<MultiVerificationResult>;
}

export interface MultiVerificationResult {
  address: string;
  results: { [verifier: string]: VerificationResult };
  overallSuccess: boolean;
  consensusResult?: VerificationResult;
  discrepancies: VerificationDiscrepancy[];
}

export interface VerificationDiscrepancy {
  type: 'bytecode' | 'abi' | 'metadata' | 'constructor_args';
  verifiers: string[];
  details: string;
  severity: 'low' | 'medium' | 'high';
}

export interface VerificationCache {
  get(address: string): Promise<CachedVerification | null>;
  set(address: string, result: VerificationResult, ttl?: number): Promise<void>;
  invalidate(address: string): Promise<void>;
  clear(): Promise<void>;
  getStats(): Promise<CacheStats>;
}

export interface CachedVerification {
  result: VerificationResult;
  timestamp: Date;
  ttl: number;
  verifier: string;
}

export interface CacheStats {
  hits: number;
  misses: number;
  size: number;
  hitRate: number;
  averageResponseTime: number;
}

export interface BytecodeAnalyzer {
  analyze(bytecode: string): Promise<BytecodeAnalysis>;
  compareWithSource(bytecode: string, sourceCode: string, compiler: CompilerInfo): Promise<ComparisonResult>;
  extractMetadata(bytecode: string): Promise<BytecodeMetadata>;
  decompile(bytecode: string): Promise<DecompiledCode>;
}

export interface BytecodeAnalysis {
  size: number;
  codeHash: string;
  functions: FunctionSignature[];
  events: EventSignature[];
  errors: ErrorSignature[];
  storage: StorageVariable[];
  imports: string[];
  libraries: LibraryReference[];
  metadata: BytecodeMetadata;
  security: SecurityAnalysis;
}

export interface FunctionSignature {
  selector: string;
  signature?: string;
  mutability: 'pure' | 'view' | 'nonpayable' | 'payable';
  visibility: 'public' | 'external';
  gasEstimate?: number;
}

export interface EventSignature {
  topic0: string;
  signature?: string;
  indexed: number;
  anonymous: boolean;
}

export interface ErrorSignature {
  selector: string;
  signature?: string;
}

export interface StorageVariable {
  slot: number;
  offset: number;
  type: string;
  name?: string;
  size: number;
}

export interface LibraryReference {
  name: string;
  address?: string;
  placeholder: string;
}

export interface BytecodeMetadata {
  ipfs?: string;
  bzzr0?: string;
  bzzr1?: string;
  solcVersion?: string;
  experimental?: boolean;
}

export interface SecurityAnalysis {
  warnings: SecurityWarning[];
  vulnerabilities: Vulnerability[];
  riskScore: number;
  recommendations: SecurityRecommendation[];
}

export interface SecurityWarning {
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  location?: string;
}

export interface Vulnerability {
  id: string;
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  impact: string;
  recommendation: string;
  references: string[];
}

export interface SecurityRecommendation {
  type: string;
  description: string;
  implementation: string;
  priority: 'low' | 'medium' | 'high';
}

export interface CompilerInfo {
  version: string;
  optimizationEnabled: boolean;
  runs: number;
  evmVersion: string;
  libraries: { [name: string]: string };
}

export interface ComparisonResult {
  match: boolean;
  similarity: number;
  differences: BytecodeDifference[];
  runtimeMatch: boolean;
  constructorMatch: boolean;
  metadataMatch: boolean;
}

export interface BytecodeDifference {
  type: 'insertion' | 'deletion' | 'modification';
  offset: number;
  length: number;
  expected: string;
  actual: string;
  description: string;
}

export interface DecompiledCode {
  success: boolean;
  functions: DecompiledFunction[];
  storage: StorageVariable[];
  events: EventSignature[];
  modifiers: DecompiledModifier[];
  confidence: number;
  warnings: string[];
}

export interface DecompiledFunction {
  selector: string;
  name?: string;
  signature?: string;
  mutability: string;
  visibility: string;
  code: string;
  confidence: number;
}

export interface DecompiledModifier {
  name: string;
  code: string;
  confidence: number;
}

export interface VerificationPipeline {
  steps: VerificationStep[];
  
  addStep(step: VerificationStep): void;
  removeStep(name: string): void;
  execute(request: VerificationRequest): Promise<VerificationResult>;
}

export interface VerificationStep {
  name: string;
  description: string;
  execute(request: VerificationRequest, context: VerificationContext): Promise<StepResult>;
  rollback?(context: VerificationContext): Promise<void>;
}

export interface VerificationContext {
  request: VerificationRequest;
  compilationResult?: any;
  bytecodeAnalysis?: BytecodeAnalysis;
  metadata: { [key: string]: any };
}

export interface StepResult {
  success: boolean;
  data?: any;
  errors?: string[];
  warnings?: string[];
}
