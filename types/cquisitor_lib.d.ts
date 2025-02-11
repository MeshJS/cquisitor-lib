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