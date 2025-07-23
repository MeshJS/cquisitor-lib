/**
* @param {string} tx_hex
* @returns {string}
*/
export function get_necessary_data_list_js(tx_hex: string): string;
/**
* @param {string} tx_hex
* @param {ValidationInputContext} validation_context
* @returns {string}
*/
export function validate_transaction_js(tx_hex: string, validation_context: string): string;

/**
 * @returns {(string)[]}
 */
export function get_decodable_types(): (string)[];
/**
 * @param {string} input
 * @param {string} type_name
 * @param {any} params_json
 * @returns {any}
 */
export function decode_specific_type(input: string, type_name: string, params_json: DecodingParams): any;
/**
 * @param {string} input
 * @returns {(string)[]}
 */
export function get_possible_types_for_input(input: string): (string)[];
/**
 * @param {string} cbor_hex
 * @returns {any}
 */
export function cbor_to_json(cbor_hex: string): CborValue;

export function check_block_or_tx_signatures(hex_str: string): CheckSignaturesResult;

/**
 * @param {string} tx_hex
 * @returns {(string)[]}
 */
export function get_utxo_list_from_tx(tx_hex: string): string[];

/**
 * @param {string} tx_hex
 * @param {UTxO[]} utxo_json
 * @param {CostModels} cost_models_json
 * @returns {ExecuteTxScriptsResult}
 */
export function execute_tx_scripts(tx_hex: string, utxo_json: UTxO[], cost_models_json: CostModels): ExecuteTxScriptsResult;
/**
 * @param {string} hex
 * @returns {ProgramJson}
 */
export function decode_plutus_program_uplc_json(hex: string): ProgramJson;
/**
 * @param {string} hex
 * @returns {string}
 */
export function decode_plutus_program_pretty_uplc(hex: string): string;

export interface CborPosition {
    offset: number;
    length: number;
}

export type CborSimpleType =
    | "Null"
    | "Bool"
    | "U8"
    | "U16"
    | "U32"
    | "U64"
    | "I8"
    | "I16"
    | "I32"
    | "I64"
    | "Int"
    | "F16"
    | "F32"
    | "F64"
    | "Bytes"
    | "String"
    | "Simple"
    | "Undefined"
    | "Break";

export interface CborSimple {
    position_info: CborPosition;
    struct_position_info?: CborPosition;
    value: any;
}

export interface CborArray {
    type: "Array";
    position_info: CborPosition;
    struct_position_info: CborPosition;
    items: number | "Indefinite";
    values: CborValue[]; // nested
}

export interface CborMap {
    type: "Map";
    position_info: CborPosition;
    struct_position_info: CborPosition;
    items: number | "Indefinite";
    values: {
        key: CborValue;
        value: CborValue;
    }[];
}

export interface CborTag {
    type: "Tag";
    position_info: CborPosition;
    struct_position_info: CborPosition;
    tag: string;
    value: CborValue;
}

export interface CborIndefiniteString {
    type: "IndefiniteLengthString";
    position_info: CborPosition;
    struct_position_info: CborPosition;
    chunks: CborValue[];
}

export interface CborIndefiniteBytes {
    type: "IndefiniteLengthBytes";
    position_info: CborPosition;
    struct_position_info: CborPosition;
    chunks: CborValue[];
}

export type CborValue =
    | CborSimple
    | CborArray
    | CborMap
    | CborTag
    | CborIndefiniteString
    | CborIndefiniteBytes;

export interface DecodingParams {
    plutus_script_version?: number;
    plutus_data_schema?: PlutusDataSchema;
}

export type PlutusDataSchema = "BasicConversions" | "DetailedSchema";

export interface CheckSignaturesResult {
    /** Indicates whether the transaction or block is valid. */
    valid: boolean;
    /** The transaction hash as a hexadecimal string (if available). */
    tx_hash?: string;
    /** An array of invalid Catalyst witness signatures (hex strings). */
    invalidCatalystWitnesses: string[];
    /** An array of invalid VKey witness signatures (hex strings). */
    invalidVkeyWitnesses: string[];
}


// The execution units object contains two numeric fields.
export type ExUnits = {
    steps: number;
    mem: number;
};

// The redeemer tag is one of the following literal strings.
export type RedeemerTag = "Spend" | "Mint" | "Cert" | "Reward" | "Propose" | "Vote";

// A successful redeemer evaluation contains the original execution units,
// the calculated execution units, and additional redeemer info.
export interface RedeemerSuccess {
    original_ex_units: ExUnits;
    calculated_ex_units: ExUnits;
    redeemer_index: number;
    redeemer_tag: RedeemerTag;
}

// A failed redeemer evaluation contains the original execution units,
// an error message, and additional redeemer info.
export interface RedeemerError {
    original_ex_units: ExUnits;
    error: string;
    redeemer_index: number;
    redeemer_tag: RedeemerTag;
}

// The result from executing the transaction scripts is an array of redeemer results.
// Each result can be either a success or an error.
export type RedeemerResult = RedeemerSuccess | RedeemerError;

// Type for the `execute_tx_scripts` response after JSON-parsing.
export type ExecuteTxScriptsResult = RedeemerResult[];


// The overall JSON produced by `to_json_program`:
export interface ProgramJson {
    program: {
        version: string;
        term: Term;
    };
}

// A UPLC term can be one of several forms.
export type Term =
    | VarTerm
    | DelayTerm
    | LambdaTerm
    | ApplyTerm
    | ConstantTerm
    | ForceTerm
    | ErrorTerm
    | BuiltinTerm
    | ConstrTerm
    | CaseTerm;

export interface VarTerm {
    var: string;
}

export interface DelayTerm {
    delay: Term;
}

export interface LambdaTerm {
    lambda: {
        parameter_name: string;
        body: Term;
    };
}

export interface ApplyTerm {
    apply: {
        function: Term;
        argument: Term;
    };
}

export interface ConstantTerm {
    constant: Constant;
}

export interface ForceTerm {
    force: Term;
}

export interface ErrorTerm {
    error: "error";
}

export interface BuiltinTerm {
    builtin: string;
}

export interface ConstrTerm {
    constr: {
        tag: number;
        fields: Term[];
    };
}

export interface CaseTerm {
    case: {
        constr: Term;
        branches: Term[];
    };
}

// The UPLC constant is one of several union types.
export type Constant =
    | IntegerConstant
    | ByteStringConstant
    | StringConstant
    | UnitConstant
    | BoolConstant
    | ListConstant
    | PairConstant
    | DataConstant
    | Bls12_381G1ElementConstant
    | Bls12_381G2ElementConstant;

export interface IntegerConstant {
    integer: string; // represented as a string
}

export interface ByteStringConstant {
    bytestring: string; // hex-encoded string
}

export interface StringConstant {
    string: string;
}

export interface UnitConstant {
    unit: "()";
}

export interface BoolConstant {
    bool: boolean;
}

export interface ListConstant {
    list: {
        type: Type;
        items: Constant[];
    };
}

export interface PairConstant {
    pair: {
        type_left: Type;
        type_right: Type;
        left: Constant;
        right: Constant;
    };
}

export interface DataConstant {
    data: PlutusData;
}

export interface Bls12_381G1ElementConstant {
    bls12_381_G1_element: {
        x: number;
        y: number;
        z: number;
    };
}

export interface Bls12_381G2ElementConstant {
    bls12_381_G2_element: BlstP2;
}

// The UPLC type is represented either as a string literal or an object.
export type Type =
    | "bool"
    | "integer"
    | "string"
    | "bytestring"
    | "unit"
    | "data"
    | "bls12_381_G1_element"
    | "bls12_381_G2_element"
    | "bls12_381_mlresult"
    | ListType
    | PairType;

export interface ListType {
    list: Type;
}

export interface PairType {
    pair: {
        left: Type;
        right: Type;
    };
}

// The JSON representation for a blst_p2 element: each coordinate is an array of numbers.
export interface BlstP2 {
    x: number[];
    y: number[];
    z: number[];
}

// Plutus data is also a tagged union.
export type PlutusData =
    | ConstrData
    | MapData
    | BigIntData
    | BoundedBytesData
    | ArrayData;

export interface ConstrData {
    constr: {
        tag: number;
        any_constructor: boolean;
        fields: PlutusData[];
    };
}

export interface MapData {
    map: Array<{
        key: PlutusData;
        value: PlutusData;
    }>;
}

export interface BigIntData {
    integer: string; // big integers are represented as strings
}

export interface BoundedBytesData {
    bytestring: string; // hex-encoded
}

export interface ArrayData {
    list: PlutusData[];
}

export interface Asset {
    unit: string;
    quantity: string;
}

export interface TxInput {
    outputIndex: number;
    txHash: string;
}

export interface TxOutput {
    address: string;
    amount: Asset[];
    dataHash?: string;
    plutusData?: string;
    scriptRef?: string;
    scriptHash?: string;
}

export interface UTxO {
    input: TxInput;
    output: TxOutput;
}

export interface CostModels {
    plutusV1?: number[];
    plutusV2?: number[];
    plutusV3?: number[];
}


///AUTOGENERATED
export interface NecessaryInputData {
    accounts: string[];
    committeeMembers: LocalCredential[];
    dReps: string[];
    govActions: GovernanceActionId[];
    lastEnactedGovAction: GovernanceActionType[];
    pools: string[];
    utxos: TxInput[];
}

/**
 * Phase 1 validation errors
 */
export type Phase1Error =
    | (
    | "GenesisKeyDelegationCertificateIsNotSupported"
    | "MoveInstantaneousRewardsCertificateIsNotSupported"
    )
    | {
    BadInputsUTxO: {
        invalid_input: TxInput;
    };
}
    | {
    OutsideValidityIntervalUTxO: {
        current_slot: bigint;
        interval_end: bigint;
        interval_start: bigint;
    };
}
    | {
    MaxTxSizeUTxO: {
        actual_size: bigint;
        max_size: bigint;
    };
}
    | "InputSetEmptyUTxO"
    | {
    FeeTooSmallUTxO: {
        actual_fee: bigint;
        fee_decomposition: FeeDecomposition;
        min_fee: bigint;
    };
}
    | {
    ValueNotConservedUTxO: {
        difference: Value;
        input_sum: Value;
        output_sum: Value;
    };
}
    | {
    WrongNetwork: {
        wrong_addresses: string[];
    };
}
    | {
    WrongNetworkWithdrawal: {
        wrong_addresses: string[];
    };
}
    | {
    WrongNetworkInTxBody: {
        actual_network: number;
        expected_network: number;
    };
}
    | {
    OutputTooSmallUTxO: {
        min_amount: number;
        output_amount: number;
    };
}
    | {
    CollateralReturnTooSmall: {
        min_amount: number;
        output_amount: number;
    };
}
    | {
    OutputBootAddrAttrsTooBig: {
        actual_size: bigint;
        max_size: bigint;
        output: unknown;
    };
}
    | {
    OutputTooBigUTxO: {
        actual_size: bigint;
        max_size: bigint;
    };
}
    | {
    InsufficientCollateral: {
        required_collateral: number;
        total_collateral: number;
    };
}
    | {
    ExUnitsTooBigUTxO: {
        actual_memory_units: bigint;
        actual_steps_units: bigint;
        max_memory_units: bigint;
        max_steps_units: bigint;
    };
}
    | "CalculatedCollateralContainsNonAdaAssets"
    | {
    CollateralInputContainsNonAdaAssets: {
        collateral_input: string;
    };
}
    | {
    CollateralIsLockedByScript: {
        invalid_collateral: string;
    };
}
    | {
    TooManyCollateralInputs: {
        actual_count: number;
        max_count: number;
    };
}
    | "NoCollateralInputs"
    | {
    IncorrectTotalCollateralField: {
        actual_sum: number;
        declared_total: number;
    };
}
    | {
    InvalidSignature: {
        invalid_signature: string;
    };
}
    | {
    ExtraneousSignature: {
        extraneous_signature: string;
    };
}
    | {
    NativeScriptIsUnsuccessful: {
        native_script_hash: string;
    };
}
    | {
    PlutusScriptIsUnsuccessful: {
        plutus_script_hash: string;
    };
}
    | {
    MissingVKeyWitnesses: {
        missing_key_hash: string;
    };
}
    | {
    MissingScriptWitnesses: {
        missing_script_hash: string;
    };
}
    | {
    MissingRedeemer: {
        index: number;
        tag: string;
    };
}
    | "MissingTxBodyMetadataHash"
    | "MissingTxMetadata"
    | {
    ConflictingMetadataHash: {
        actual_hash: string;
        expected_hash: string;
    };
}
    | {
    InvalidMetadata: {
        message: string;
    };
}
    | {
    ExtraneousScriptWitnesses: {
        extraneous_script: string;
    };
}
    | {
    StakeAlreadyRegistered: {
        reward_address: string;
    };
}
    | {
    StakeNotRegistered: {
        reward_address: string;
    };
}
    | {
    StakeNonZeroAccountBalance: {
        remaining_balance: bigint;
        reward_address: string;
    };
}
    | {
    RewardAccountNotExisting: {
        reward_address: string;
    };
}
    | {
    WrongRequestedWithdrawalAmount: {
        expected_amount: number;
        requested_amount: bigint;
        reward_address: string;
    };
}
    | {
    StakePoolNotRegistered: {
        pool_id: string;
    };
}
    | {
    WrongRetirementEpoch: {
        current_epoch: bigint;
        max_epoch: bigint;
        min_epoch: bigint;
        specified_epoch: bigint;
    };
}
    | {
    StakePoolCostTooLow: {
        min_cost: bigint;
        specified_cost: bigint;
    };
}
    | {
    InsufficientFundsForMir: {
        available_amount: bigint;
        requested_amount: bigint;
    };
}
    | {
    InvalidCommitteeVote: {
        message: string;
        voter: unknown;
    };
}
    | {
    DRepIncorrectDeposit: {
        cert_index: number;
        required_deposit: number;
        supplied_deposit: number;
    };
}
    | {
    DRepDeregistrationWrongRefund: {
        cert_index: number;
        required_refund: number;
        supplied_refund: number;
    };
}
    | {
    StakeRegistrationWrongDeposit: {
        cert_index: number;
        required_deposit: number;
        supplied_deposit: number;
    };
}
    | {
    StakeDeregistrationWrongRefund: {
        cert_index: number;
        required_refund: number;
        supplied_refund: number;
    };
}
    | {
    PoolRegistrationWrongDeposit: {
        cert_index: number;
        required_deposit: number;
        supplied_deposit: number;
    };
}
    | {
    CommitteeHasPreviouslyResigned: {
        committee_credential: LocalCredential;
    };
}
    | {
    TreasuryValueMismatch: {
        actual_value: bigint;
        declared_value: bigint;
    };
}
    | {
    RefScriptsSizeTooBig: {
        actual_size: bigint;
        max_size: bigint;
    };
}
    | {
    WithdrawalNotAllowedBecauseNotDelegatedToDRep: {
        reward_address: string;
    };
}
    | {
    CommitteeIsUnknown: {
        /**
         * The committee key hash
         */
        committee_key_hash:
            | {
            keyHash: number[];
        }
            | {
            scriptHash: number[];
        };
    };
}
    | {
    GovActionsDoNotExist: {
        /**
         * The list of invalid governance action IDs
         */
        invalid_action_ids: GovernanceActionId[];
    };
}
    | {
    MalformedProposal: {
        gov_action: GovernanceActionId;
    };
}
    | {
    ProposalProcedureNetworkIdMismatch: {
        /**
         * The expected network ID
         */
        expected_network: number;
        /**
         * The reward account
         */
        reward_account: string;
    };
}
    | {
    TreasuryWithdrawalsNetworkIdMismatch: {
        /**
         * The expected network ID
         */
        expected_network: number;
        /**
         * The set of mismatched reward accounts
         */
        mismatched_account: string;
    };
}
    | {
    VotingProposalIncorrectDeposit: {
        proposal_index: number;
        /**
         * The required deposit amount
         */
        required_deposit: number;
        /**
         * The supplied deposit amount
         */
        supplied_deposit: number;
    };
}
    | {
    DisallowedVoters: {
        /**
         * List of disallowed voter and action ID pairs
         */
        disallowed_pairs: [unknown, unknown][];
    };
}
    | {
    ConflictingCommitteeUpdate: {
        /**
         * The set of conflicting credentials
         */
        conflicting_credentials:
            | {
            keyHash: number[];
        }
            | {
            scriptHash: number[];
        };
    };
}
    | {
    ExpirationEpochTooSmall: {
        /**
         * Map of credentials to their invalid expiration epochs
         */
        invalid_expirations: {
            [k: string]: number;
        };
    };
}
    | {
    InvalidPrevGovActionId: {
        /**
         * The invalid proposal
         */
        proposal: {
            [k: string]: unknown;
        };
    };
}
    | {
    VotingOnExpiredGovAction: {
        expired_gov_action: GovernanceActionId;
    };
}
    | {
    ProposalCantFollow: {
        /**
         * The expected protocol version
         */
        expected_versions: ProtocolVersion[];
        /**
         * Previous governance action ID
         */
        prev_gov_action_id?: GovernanceActionId | null;
        supplied_version: ProtocolVersion;
    };
}
    | {
    InvalidConstitutionPolicyHash: {
        /**
         * The expected policy hash
         */
        expected_hash?: string | null;
        /**
         * The supplied policy hash
         */
        supplied_hash?: string | null;
    };
}
    | {
    VoterDoNotExist: {
        /**
         * List of non-existent voters
         */
        missing_voter: {
            [k: string]: unknown;
        };
    };
}
    | {
    ZeroTreasuryWithdrawals: {
        gov_action: GovernanceActionId;
    };
}
    | {
    ProposalReturnAccountDoesNotExist: {
        /**
         * The invalid return account
         */
        return_account: string;
    };
}
    | {
    TreasuryWithdrawalReturnAccountsDoNotExist: {
        /**
         * List of non-existent return accounts
         */
        missing_account: string;
    };
}
    | {
    AuxiliaryDataHashMismatch: {
        /**
         * The actual auxiliary data hash
         */
        actual_hash?: string | null;
        /**
         * The expected auxiliary data hash
         */
        expected_hash: string;
    };
}
    | "AuxiliaryDataHashMissing"
    | "AuxiliaryDataHashPresentButNotExpected"
    | {
    UnknownError: {
        message: string;
    };
}
    | {
    MissingDatum: {
        datum_hash: string;
    };
}
    | {
    ExtraneousDatumWitnesses: {
        datum_hash: string;
    };
}
    | {
    ScriptDataHashMismatch: {
        /**
         * The expected script data hash
         */
        expected_hash?: string | null;
        /**
         * The actual script data hash
         */
        provided_hash?: string | null;
    };
}
    | {
    ReferenceInputOverlapsWithInput: {
        input: TxInput;
    };
};

export type Phase1Warning =
    | ("InputsAreNotSorted" | "CollateralIsUnnecessary" | "TotalCollateralIsNotDeclared")
    | {
    FeeIsBiggerThanMinFee: {
        actual_fee: bigint;
        fee_decomposition: FeeDecomposition;
        min_fee: bigint;
    };
}
    | {
    InputUsesRewardAddress: {
        invalid_input: string;
    };
}
    | {
    CollateralInputUsesRewardAddress: {
        invalid_collateral: string;
    };
}
    | {
    CannotCheckStakeDeregistrationRefund: {
        cert_index: number;
    };
}
    | {
    CannotCheckDRepDeregistrationRefund: {
        cert_index: number;
    };
}
    | {
    PoolAlreadyRegistered: {
        pool_id: string;
    };
}
    | {
    DRepAlreadyRegistered: {
        drep_id: string;
    };
}
    | {
    CommitteeAlreadyAuthorized: {
        committee_key: string;
    };
}
    | {
    DRepNotRegistered: {
        cert_index: number;
    };
}
    | {
    DuplicateRegistrationInTx: {
        cert_index: number;
        entity_id: string;
        entity_type: string;
    };
}
    | {
    DuplicateCommitteeColdResignationInTx: {
        cert_index: number;
        committee_credential: LocalCredential;
    };
}
    | {
    DuplicateCommitteeHotRegistrationInTx: {
        cert_index: number;
        committee_credential: LocalCredential;
    };
};

export interface ValidationResult {
    errors: ValidationError[];
    warnings: ValidationWarning[];
}
export interface ValidationError {
    error: Phase1Error;
    error_message: string;
    hint?: string | null;
    locations: string[];
}
/**
 * The invalid input UTxO
 */

export interface FeeDecomposition {
    executionUnitsFee: bigint;
    referenceScriptsFee: bigint;
    txSizeFee: bigint;
}
export interface Value {
    assets: MultiAsset;
    coins: number;
}
export interface MultiAsset {
    assets: ValidatorAsset[];
}
export interface ValidatorAsset {
    asset_name: string;
    policy_id: string;
    quantity: number;
}

/**
 * The invalid governance action
 */

/**
 * The expired governance action
 */

export interface ProtocolVersion {
    major: bigint;
    minor: bigint;
}
/**
 * The supplied protocol version
 */

/**
 * The governance action with zero withdrawals
 */

export interface ValidationWarning {
    hint?: string | null;
    locations: string[];
    warning: Phase1Warning;
}

export type LocalCredential =
    | {
    keyHash: number[];
}
    | {
    scriptHash: number[];
};
export type GovernanceActionType =
    | "parameterChangeAction"
    | "hardForkInitiationAction"
    | "treasuryWithdrawalsAction"
    | "noConfidenceAction"
    | "updateCommitteeAction"
    | "newConstitutionAction"
    | "infoAction";
export type NetworkType = "mainnet" | "testnet";

export interface ValidationInputContext {
    accountContexts: AccountInputContext[];
    committeeContext: CommitteeInputContext;
    drepContexts: DrepInputContext[];
    govActionContexts: GovActionInputContext[];
    lastEnactedGovAction: GovActionInputContext[];
    networkType: NetworkType;
    poolContexts: PoolInputContext[];
    protocolParameters: ProtocolParameters;
    slot: bigint;
    treasuryValue: bigint;
    utxoSet: UtxoInputContext[];
}
export interface AccountInputContext {
    balance?: number | null;
    bech32Address: string;
    delegatedToDrep?: string | null;
    delegatedToPool?: string | null;
    isRegistered: boolean;
    payedDeposit?: number | null;
}
export interface CommitteeInputContext {
    activeCommitteeMembers: LocalCredential[];
    potentialCommitteeMembers: LocalCredential[];
    resignedCommitteeMembers: LocalCredential[];
}
export interface DrepInputContext {
    bech32Drep: string;
    isRegistered: boolean;
    payedDeposit?: number | null;
}
export interface GovActionInputContext {
    actionId: GovernanceActionId;
    actionType: GovernanceActionType;
    isActive: boolean;
}
export interface GovernanceActionId {
    index: number;
    txHash: number[];
}
export interface PoolInputContext {
    isRegistered: boolean;
    poolId: string;
    retirementEpoch?: number | null;
}
export interface ProtocolParameters {
    /**
     * Cost per UTxO byte in lovelace
     */
    adaPerUtxoByte: bigint;
    /**
     * Percentage of transaction fee required as collateral
     */
    collateralPercentage: number;
    costModels: CostModels;
    /**
     * Deposit amount required for registering as a DRep
     */
    drepDeposit: bigint;
    executionPrices: ExUnitPrices;
    /**
     * Deposit amount required for submitting a governance action
     */
    governanceActionDeposit: bigint;
    /**
     * Maximum block body size in bytes
     */
    maxBlockBodySize: number;
    maxBlockExecutionUnits: ExUnits;
    /**
     * Maximum block header size in bytes
     */
    maxBlockHeaderSize: number;
    /**
     * Maximum number of collateral inputs
     */
    maxCollateralInputs: number;
    /**
     * Maximum number of epochs that can be used for pool retirement ahead
     */
    maxEpochForPoolRetirement: number;
    /**
     * Maximum transaction size in bytes
     */
    maxTransactionSize: number;
    maxTxExecutionUnits: ExUnits;
    /**
     * Maximum size of a Value in bytes
     */
    maxValueSize: number;
    /**
     * Linear factor for the minimum fee calculation formula
     */
    minFeeCoefficientA: bigint;
    /**
     * Constant factor for the minimum fee calculation formula
     */
    minFeeConstantB: bigint;
    /**
     * Minimum pool cost in lovelace
     */
    minPoolCost: bigint;
    /**
     * Protocol version (major, minor)
     *
     * @minItems 2
     * @maxItems 2
     */
    protocolVersion: [unknown, unknown];
    referenceScriptCostPerByte: SubCoin;
    stakeKeyDeposit: bigint;
    stakePoolDeposit: bigint;
}

export interface ExUnitPrices {
    memPrice: SubCoin;
    stepPrice: SubCoin;
}
export interface SubCoin {
    denominator: bigint;
    numerator: bigint;
}

export interface UtxoInputContext {
    isSpent: boolean;
    utxo: UTxO;
}