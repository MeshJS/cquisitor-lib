use crate::bingen::wasm_bindgen;
use crate::cbor::cbor_decoder::{get_tokenizer, get_value};
use crate::js_error::JsError;
use crate::js_value::{from_serde_json_value, JsValue};

mod cbor_decoder;

#[wasm_bindgen]
pub fn cbor_to_json(cbor_hex: &str) -> Result<JsValue, JsError> {
    let cbor = hex::decode(cbor_hex)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let tokenizer = get_tokenizer(&cbor);
    let value = get_value(tokenizer)?;
    from_serde_json_value(&value)
        .map_err(|e| JsError::new(&e))
}