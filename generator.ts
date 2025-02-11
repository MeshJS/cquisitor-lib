import * as Cardano from '@emurgo/cardano-serialization-lib-browser';

const customTypes = [
    "Address",
    "RewardAddress",
    "PointerAddress",
    "BaseAddress",
    "EnterpriseAddress",
    "ByronAddress",
    "Transaction",
    "PlutusData",
    "PlutusScript"
];

const ignoreTypes = [
    "FixedBlock",
    "FixedTransaction",
    "FixedTransactionBodies",
    "FixedTransactionBody",
    "FixedVersionedBlock",
    "FixedTxWitnessesSet",
];

const typeWithSimpleBech32 = [
    'Bip32PrivateKey',
    'Bip32PublicKey',
    'Ed25519Signature',
    'PrivateKey',
    'PublicKey',
    'PlutusData',
];

const useBytesRef = [
    'Bip32PrivateKey',
    'Bip32PublicKey',
    'PublicKey',
    "LegacyDaedalusPrivateKey",
];

export const typesWithNonResultBech32 = [
    'AnchorDataHash'
];

interface FromMethodsMap {
    from_bech32: Set<string>;
    from_hex: Set<string>;
    from_bytes: Set<string>;
    from_base58: Set<string>;
    to_json: Set<string>;
    to_hex: Set<string>;
    to_bech32: Set<string>;
}

function findClassesWithMethods(): FromMethodsMap {
    const classesWithMethods: FromMethodsMap = {
        from_bech32: new Set(),
        from_hex: new Set(),
        from_bytes: new Set(),
        from_base58: new Set(),
        to_json: new Set(),
        to_hex: new Set(),
        to_bech32: new Set(),
    };

    for (const [exportName, exportedValue] of Object.entries(Cardano)) {
        if (typeof exportedValue === 'function') {
            if ('from_bech32' in exportedValue) {
                classesWithMethods.from_bech32.add(exportName);
            }
            if ('from_hex' in exportedValue) {
                classesWithMethods.from_hex.add(exportName);
            }
            if ('from_bytes' in exportedValue) {
                classesWithMethods.from_bytes.add(exportName);
            }
            if ('from_base58' in exportedValue) {
                classesWithMethods.from_base58.add(exportName);
            }

            const proto = (exportedValue as any).prototype;
            if (proto) {
                if ('to_json' in proto) {
                    classesWithMethods.to_json.add(exportName);
                }
                if ('to_hex' in proto) {
                    classesWithMethods.to_hex.add(exportName);
                }
                if ('to_bech32' in proto) {
                    classesWithMethods.to_bech32.add(exportName);
                }
            }
        }
    }

    return classesWithMethods;
}

function generateValidationFns(): string {
    return `
fn is_valid_hex(input: &str) -> bool {
    // Basic check: ensures input can be decoded as hex
    hex::decode(input).is_ok()
}

fn is_valid_base58(input: &str) -> bool {
    bs58::decode(input).into_vec().is_ok()
}

fn is_valid_bech32(input: &str) -> bool {
    bech32::decode(input).is_ok()
}
`;
}

function generateStandardDecodingMatch(type: string, methods: FromMethodsMap): string {
    const decodingAttempts: string[] = [];

    const hasToJson = methods.to_json.has(type);
    const hasToHex = methods.to_hex.has(type);
    const hasToBech32 = methods.to_bech32.has(type);

    // If it's a "simple" type with from_bech32 -> to_bech32, we can just do `decoded.to_bech32()`.
    // Otherwise, we do the usual `decoded.to_bech32("")`.
    const bech32Call = typeWithSimpleBech32.includes(type)
        ? `decoded.to_bech32()`
        : `decoded.to_bech32("").map_err(|e| format!("Failed to convert to bech32: {:?}", e))?`;

    const getResultValue = () => {
        if (hasToJson) {
            return `decoded.to_json()
        .map_err(|e| format!("Failed to convert to JSON: {:?}", e))
        .and_then(|json| serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse JSON: {}", e)))`;
        }

        const fields: string[] = [];
        if (hasToHex) fields.push(`"hex": decoded.to_hex()`);
        if (hasToBech32) fields.push(`"bech32": ${bech32Call}`);

        if (fields.length === 0) {
            return 'Ok::<serde_json::Value, String>(serde_json::Value::String("Decoded, but no additional representation".to_string()))';
        }

        return `Ok::<serde_json::Value, String>(serde_json::json!({
          ${fields.join(',\n          ')}
        }))`;
    };

    // from_hex
    if (methods.from_hex.has(type)) {
        decodingAttempts.push(`
            if is_hex {
                if let Ok(decoded) = csl::${type}::from_hex(input) {
                    let value = ${getResultValue()}?;
                    return from_serde_json_value(&value)
                        .map_err(|e| format!("Failed to convert to JsValue: {}", e));
                }
            }
        `);
    }

    // from_bytes (only if from_hex isn't available)
    if (methods.from_bytes.has(type) && !methods.from_hex.has(type)) {
        decodingAttempts.push(`
            if is_hex {
                if let Ok(bytes) = hex::decode(input) {
                    if let Ok(decoded) = csl::${type}::from_bytes(${useBytesRef.includes(type) ? "&" : ""}bytes) {
                        let value = ${getResultValue()}?;
                        return from_serde_json_value(&value)
                            .map_err(|e| format!("Failed to convert to JsValue: {}", e));
                    }
                }
            }
        `);
    }

    // from_bech32
    if (methods.from_bech32.has(type)) {
        decodingAttempts.push(`
            if is_bech32 {
                if let Ok(decoded) = csl::${type}::from_bech32(input) {
                    let value = ${getResultValue()}?;
                    return from_serde_json_value(&value)
                        .map_err(|e| format!("Failed to convert to JsValue: {}", e));
                }
            }
        `);
    }

    // from_base58
    if (methods.from_base58.has(type)) {
        decodingAttempts.push(`
            if is_base58 {
                if let Ok(decoded) = csl::${type}::from_base58(input) {
                    let value = ${getResultValue()}?;
                    return from_serde_json_value(&value)
                        .map_err(|e| format!("Failed to convert to JsValue: {}", e));
                }
            }
        `);
    }

    return `
        "${type}" => {
            ${decodingAttempts.join('\n')}
            Err("Failed to decode".to_string())
        },`;
}

function generateCustomDecodingMatch(type: string): string {
    // Same logic as before for your custom decoders.
    switch (type) {
        case "Address":
        case "ByronAddress":
        case "RewardAddress":
        case "PointerAddress":
        case "BaseAddress":
        case "EnterpriseAddress":
            return `
            "${type}" => {
                decode_address(input, is_hex, is_bech32, is_base58)
            },`;
        case "Transaction":
            return `
            "Transaction" => {
                decode_transaction(input, is_hex, is_bech32, is_base58)
            },`;
        case "PlutusData":
            return `
            "PlutusData" => {
                decode_plutus_data(input, params.plutus_data_schema, is_hex, is_bech32, is_base58)
            },`;
        case "PlutusScript":
            return `
            "PlutusScript" => {
                decode_plutus_script(input, params.plutus_script_version, is_hex, is_bech32, is_base58)
            },`;
        default:
            return `
            "${type}" => {
                Err("Unexpected custom type".to_string())
            },`;
    }
}

function generateDecodingMatch(type: string, methods: FromMethodsMap): string {
    if (customTypes.includes(type)) {
        return generateCustomDecodingMatch(type);
    } else {
        return generateStandardDecodingMatch(type, methods);
    }
}

function generateTryDecodeBlock(type: string): string {
    return `
    if decode_specific_type(input, "${type}", empty_js_value()).is_ok() {
        type_names.push("${type}".to_string());
    }`;
}

export function generateRustCode(): string {
    const methodsMap = findClassesWithMethods();

    // Gather all relevant types (excluding ignored)
    let allTypes = Array.from(new Set([
        ...methodsMap.from_bech32,
        ...methodsMap.from_hex,
        ...methodsMap.from_bytes,
        ...methodsMap.from_base58,
        ...customTypes
    ]));

    allTypes = allTypes.filter(type => !ignoreTypes.includes(type));

    // We'll inject our validation function code once at the top:
    const validationFns = generateValidationFns();

    // Define get_decodable_types
    const getTypesFunction = `
#[wasm_bindgen]
pub fn get_decodable_types() -> Vec<String> {
    vec![
        ${allTypes.map(typeName => `String::from("${typeName}")`).join(',\n        ')}
    ]
}`;

    // Define decode_specific_type -- notice the validation calls up front
    const decodeSpecificFunction = `
#[wasm_bindgen]
pub fn decode_specific_type(input: &str, type_name: &str, params_json: JsValue) -> Result<JsValue, String> {
    let params: DecodingParams = from_js_value(&params_json)?;
    
    // Run our checks ONCE at the top of the function:
    let is_hex = is_valid_hex(input);
    let is_base58 = is_valid_base58(input);
    let is_bech32 = is_valid_bech32(input);

    match type_name {
        ${allTypes.map(typeName => generateDecodingMatch(typeName, methodsMap)).join('\n        ')}
        _ => Err("Unsupported type".to_string())
    }
}`;

    // Define get_possible_types_for_input
    const getPossibleTypesFunction = `
#[wasm_bindgen]
pub fn get_possible_types_for_input(input: &str) -> Vec<String> {
    let mut type_names = Vec::new();
    ${allTypes.map(typeName => generateTryDecodeBlock(typeName)).join('\n    ')}
    type_names
}`;

    // Helper code
    const helperCode = `
use crate::csl_decoders::params::DecodingParams;
use crate::csl_decoders::specific_decoders::{
    decode_address, decode_plutus_data, decode_plutus_script, decode_transaction,
};
use crate::js_value::{from_serde_json_value, empty_js_value, JsValue, from_js_value};
use bech32;
use bs58;
use cardano_serialization_lib as csl;
use hex;
use crate::bingen::wasm_bindgen;

${validationFns}
`;

    return `
${helperCode}

// Returns a list of all the types that we can attempt to decode
${getTypesFunction}

// Decodes a given input as a particular type, returning a JsValue (serialized JSON or other representation)
${decodeSpecificFunction}

// Tries to decode the input as every known type, returning the list of which decodes succeeded
${getPossibleTypesFunction}
`;
}

// Just to show the final output in console:
console.log(generateRustCode());