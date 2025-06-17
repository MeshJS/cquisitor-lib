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
export function validate_transaction_js(tx_hex: string, validation_context: ValidationInputContext): string;

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

export interface GovernanceActionId {
    index: number;
    txHash: number[];
  }
  
export type GovernanceActionType = "parameterChangeAction" | "hardForkInitiationAction" | "treasuryWithdrawalsAction" | "noConfidenceAction" | "updateCommitteeAction" | "newConstitutionAction" | "infoAction";
  
export type LocalCredential = {
    keyHash: number[];
  } | {
    scriptHash: number[];
  };
  
export interface TxInput {
    outputIndex: number;
    txHash: string;
  }
  
export interface FeeDecomposition {
    executionUnitsFee: number;
    referenceScriptsFee: number;
    txSizeFee: number;
  }
  
export interface MultiAsset {
    assets: ValidatorAsset[];
  }
  
export type Phase1Error = "GenesisKeyDelegationCertificateIsNotSupported" | "MoveInstantaneousRewardsCertificateIsNotSupported" | {
    BadInputsUTxO: {
    invalid_input: TxInput;
  };
  } | {
    OutsideValidityIntervalUTxO: {
    current_slot: number;
    interval_end: number;
    interval_start: number;
  };
  } | {
    MaxTxSizeUTxO: {
    actual_size: number;
    max_size: number;
  };
  } | string | {
    FeeTooSmallUTxO: {
    actual_fee: number;
    fee_decomposition: FeeDecomposition;
    min_fee: number;
  };
  } | {
    ValueNotConservedUTxO: {
    difference: Value;
    input_sum: Value;
    output_sum: Value;
  };
  } | {
    WrongNetwork: {
    wrong_addresses: string[];
  };
  } | {
    WrongNetworkWithdrawal: {
    wrong_addresses: string[];
  };
  } | {
    WrongNetworkInTxBody: {
    actual_network: number;
    expected_network: number;
  };
  } | {
    OutputTooSmallUTxO: {
    min_amount: number;
    output_amount: number;
  };
  } | {
    CollateralReturnTooSmall: {
    min_amount: number;
    output_amount: number;
  };
  } | {
    OutputBootAddrAttrsTooBig: {
    actual_size: number;
    max_size: number;
    output: any;
  };
  } | {
    OutputTooBigUTxO: {
    max_size: number;
    oversized_outputs: string[];
  };
  } | {
    InsufficientCollateral: {
    required_collateral: number;
    total_collateral: number;
  };
  } | {
    ScriptsNotPaidUTxO: {
    missing_witness: string[];
  };
  } | {
    ExUnitsTooBigUTxO: {
    actual_units: number;
    max_units: number;
  };
  } | string | {
    CollateralInputContainsNonAdaAssets: {
    collateral_input: string;
  };
  } | {
    CollateralIsLockedByScript: {
    invalid_collateral: string;
  };
  } | {
    OutsideForecast: {
    current_slot: number;
    max_forecast_slot: number;
  };
  } | {
    TooManyCollateralInputs: {
    actual_count: number;
    max_count: number;
  };
  } | string | {
    IncorrectTotalCollateralField: {
    actual_sum: number;
exportd_total: number;
  };
  } | {
    InvalidSignature: {
    invalid_signature: string;
  };
  } | {
    ExtraneousSignature: {
    extraneous_signature: string;
  };
  } | {
    NativeScriptIsUnsuccessful: {
    native_script_hash: string;
  };
  } | {
    PlutusScriptIsUnsuccessful: {
    plutus_script_hash: string;
  };
  } | {
    MissingVKeyWitnesses: {
    missing_key_hash: string;
  };
  } | {
    MissingScriptWitnesses: {
    missing_script_hash: string;
  };
  } | {
    MissingRedeemer: {
    index: number;
    tag: string;
  };
  } | string | string | {
    ConflictingMetadataHash: {
    actual_hash: string;
    expected_hash: string;
  };
  } | {
    InvalidMetadata: {
    message: string;
  };
  } | {
    ExtraneousScriptWitnesses: {
    extraneous_scripts: string[];
  };
  } | {
    StakeAlreadyRegistered: {
    reward_address: string;
  };
  } | {
    StakeNotRegistered: {
    reward_address: string;
  };
  } | {
    StakeNonZeroAccountBalance: {
    remaining_balance: number;
    reward_address: string;
  };
  } | {
    RewardAccountNotExisting: {
    reward_address: string;
  };
  } | {
    WrongRequestedWithdrawalAmount: {
    expected_amount: number;
    requested_amount: number;
    reward_address: string;
  };
  } | {
    StakePoolNotRegistered: {
    pool_id: string;
  };
  } | {
    WrongRetirementEpoch: {
    current_epoch: number;
    max_epoch: number;
    min_epoch: number;
    specified_epoch: number;
  };
  } | {
    StakePoolCostTooLow: {
    min_cost: number;
    specified_cost: number;
  };
  } | {
    InsufficientFundsForMir: {
    available_amount: number;
    requested_amount: number;
  };
  } | {
    InvalidCommitteeVote: {
    message: string;
    voter: any;
  };
  } | {
    DRepIncorrectDeposit: {
    cert_index: number;
    required_deposit: number;
    supplied_deposit: number;
  };
  } | {
    DRepDeregistrationWrongRefund: {
    cert_index: number;
    required_refund: number;
    supplied_refund: number;
  };
  } | {
    StakeRegistrationWrongDeposit: {
    cert_index: number;
    required_deposit: number;
    supplied_deposit: number;
  };
  } | {
    StakeDeregistrationWrongRefund: {
    cert_index: number;
    required_refund: number;
    supplied_refund: number;
  };
  } | {
    PoolRegistrationWrongDeposit: {
    cert_index: number;
    required_deposit: number;
    supplied_deposit: number;
  };
  } | {
    CommitteeHasPreviouslyResigned: {
    committee_credential: LocalCredential;
  };
  } | {
    TreasuryValueMismatch: {
    actual_value: number;
exportd_value: number;
  };
  } | {
    RefScriptsSizeTooBig: {
    actual_size: number;
    max_size: number;
  };
  } | {
    WdrlNotDelegatedToDRep: {
    stake_credential: LocalCredential;
  };
  } | {
    CommitteeIsUnknown: {
    committee_key_hash: LocalCredential;
  };
  } | {
    GovActionsDoNotExist: {
    invalid_action_ids: GovernanceActionId[];
  };
  } | {
    MalformedProposal: {
    gov_action: GovernanceActionId;
  };
  } | {
    ProposalProcedureNetworkIdMismatch: {
    expected_network: number;
    reward_account: string;
  };
  } | {
    TreasuryWithdrawalsNetworkIdMismatch: {
    expected_network: number;
    mismatched_account: string;
  };
  } | {
    VotingProposalIncorrectDeposit: {
    proposal_index: number;
    required_deposit: number;
    supplied_deposit: number;
  };
  } | {
    DisallowedVoters: {
    disallowed_pairs: any[][];
  };
  } | {
    ConflictingCommitteeUpdate: {
    conflicting_credentials: LocalCredential;
  };
  } | {
    ExpirationEpochTooSmall: {
    invalid_expirations: Record<string, any>;
  };
  } | {
    InvalidPrevGovActionId: {
    proposal: any;
  };
  } | {
    VotingOnExpiredGovAction: {
    expired_gov_action: GovernanceActionId;
  };
  } | {
    ProposalCantFollow: {
    expected_versions: ProtocolVersion[];
    prev_gov_action_id?: GovernanceActionId | null;
    supplied_version: ProtocolVersion;
  };
  } | {
    InvalidConstitutionPolicyHash: {
    expected_hash?: any;
    supplied_hash?: any;
  };
  } | {
    VotersDoNotExist: {
    missing_voter: any;
  };
  } | {
    ZeroTreasuryWithdrawals: {
    gov_action: GovernanceActionId;
  };
  } | {
    ProposalReturnAccountDoesNotExist: {
    return_account: string;
  };
  } | {
    TreasuryWithdrawalReturnAccountsDoNotExist: {
    missing_account: string;
  };
  } | {
    AuxiliaryDataHashMismatch: {
    actual_hash?: any;
    expected_hash: string;
  };
  } | string | string;
  
export type Phase1Warning = "InputsAreNotSorted" | "CollateralIsUnnecessary" | "TotalCollateralIsNotDeclared" | {
    FeeIsBiggerThanMinFee: {
    actual_fee: number;
    fee_decomposition: FeeDecomposition;
    min_fee: number;
  };
  } | {
    InputUsesRewardAddress: {
    invalid_input: string;
  };
  } | {
    CollateralInputUsesRewardAddress: {
    invalid_collateral: string;
  };
  } | {
    CannotCheckStakeDeregistrationRefund: {
    cert_index: number;
  };
  } | {
    CannotCheckDRepDeregistrationRefund: {
    cert_index: number;
  };
  } | {
    PoolAlreadyRegistered: {
    pool_id: string;
  };
  } | {
    DRepAlreadyRegistered: {
    drep_id: string;
  };
  } | {
    CommitteeAlreadyAuthorized: {
    committee_key: string;
  };
  } | {
    DRepNotRegistered: {
    cert_index: number;
  };
  } | {
    DuplicateRegistrationInTx: {
    cert_index: number;
    entity_id: string;
    entity_type: string;
  };
  } | {
    DuplicateCommitteeColdResignationInTx: {
    cert_index: number;
    committee_credential: LocalCredential;
  };
  } | {
    DuplicateCommitteeHotRegistrationInTx: {
    cert_index: number;
    committee_credential: LocalCredential;
  };
  };
  
export interface ProtocolVersion {
    major: number;
    minor: number;
  }
  
export interface ValidationError {
    error: Phase1Error;
    error_message: string;
    hint?: any;
    locations: string[];
  }
  
export interface ValidationWarning {
    hint?: any;
    locations: string[];
    warning: Phase1Warning;
  }
  
export interface ValidatorAsset {
    asset_name: string;
    policy_id: string;
    quantity: number;
  }
  
export interface Value {
    assets: MultiAsset;
    coins: number;
  }
  
export type Voter = {
    constitutionalCommitteeHotScriptHash: number[];
  } | {
    constitutionalCommitteeHotKeyHash: number[];
  } | {
    dRepScriptHash: number[];
  } | {
    dRepKeyHash: number[];
  } | {
    stakingPoolKeyHash: number[];
  };
  
export interface AccountInputContext {
    balance?: any;
    bech32Address: string;
    isRegistered: boolean;
    payedDeposit?: any;
  }
  
export interface Asset {
    quantity: string;
    unit: string;
  }
  
export interface CommitteeInputContext {
    activeCommitteeMembers: LocalCredential[];
    potentialCommitteeMembers: LocalCredential[];
    resignedCommitteeMembers: LocalCredential[];
  }
  
export interface DrepInputContext {
    bech32Drep: string;
    isRegistered: boolean;
    payedDeposit?: any;
  }
  
export interface ExUnitPrices {
    memPrice: SubCoin;
    stepPrice: SubCoin;
  }
  
export interface GovActionInputContext {
    actionId: GovernanceActionId;
    actionType: GovernanceActionType;
    isActive: boolean;
  }
  
export type NetworkType = "mainnet" | "testnet";
  
export interface PoolInputContext {
    isRegistered: boolean;
    poolId: string;
    retirementEpoch?: any;
  }
  
export interface ProtocolParameters {
    adaPerUtxoByte: number;
    collateralPercentage: number;
    costModels: CostModels;
    drepDeposit: number;
    executionPrices: ExUnitPrices;
    governanceActionDeposit: number;
    maxBlockBodySize: number;
    maxBlockExecutionUnits: ExUnits;
    maxBlockHeaderSize: number;
    maxCollateralInputs: number;
    maxEpochForPoolRetirement: number;
    maxTransactionSize: number;
    maxTxExecutionUnits: ExUnits;
    maxValueSize: number;
    minFeeCoefficientA: number;
    minFeeConstantB: number;
    minPoolCost: number;
    protocolVersion: any[];
    referenceScriptCostPerByte: SubCoin;
    stakeKeyDeposit: number;
    stakePoolDeposit: number;
  }
  
export interface SubCoin {
    denominator: number;
    numerator: number;
  }
  
export interface UTxO {
    input: TxInput;
    output: TxOutput;
  }
  
export interface UtxoInputContext {
    isSpent: boolean;
    utxo: UTxO;
  }
  
export interface NecessaryInputData {
    accounts: string[];
    committeeMembers: LocalCredential[];
    dReps: string[];
    govActions: GovernanceActionId[];
    lastEnactedGovAction: GovernanceActionType[];
    pools: string[];
    utxos: TxInput[];
  }
  
export interface ValidationResult {
    errors: ValidationError[];
    warnings: ValidationWarning[];
  }
  
export interface ValidationInputContext {
    accountContexts: AccountInputContext[];
    committeeContext: CommitteeInputContext;
    drepContexts: DrepInputContext[];
    govActionContexts: GovActionInputContext[];
    lastEnactedGovAction: GovActionInputContext[];
    networkType: NetworkType;
    poolContexts: PoolInputContext[];
    protocolParameters: ProtocolParameters;
    slot: number;
    treasuryValue: number;
    utxoSet: UtxoInputContext[];
  }
  