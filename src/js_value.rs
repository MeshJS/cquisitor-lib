#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
use std::fmt::format;

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
use serde::Serialize;

use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
pub use wasm_bindgen::prelude::JsValue;

#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[derive(Debug, Clone)]
pub struct JsValue {
    msg: String,
}

#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
impl JsValue {
    pub fn new(s: &str) -> Self {
        Self { msg: s.to_owned() }
    }

    pub fn as_string(&self) -> Option<String> {
        Some(self.msg.clone())
    }
}

pub fn from_serde_json_value(value: &JsonValue) -> Result<JsValue, String> {
    #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
    {
        let mut serializer = serde_wasm_bindgen::Serializer::json_compatible();
        serializer = serializer.serialize_large_number_types_as_bigints(true);
        value.serialize(&serializer)
            .map_err(|err| format!("Failed to convert JSON to JsValue: {:?}", err))
    }

    #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
    {
        Ok(JsValue::new(&value.to_string()))
    }
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
pub fn from_js_value<T>(js_value: &JsValue) -> Result<T, String>
where
    T: DeserializeOwned,
{
    serde_wasm_bindgen::from_value(js_value.clone()).map_err(|err| {
        format!("Failed to deserialize JsValue to type: {:?}", err).to_string()
    })
}

#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
pub fn from_js_value<T>(js_value: &JsValue) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let json_str = js_value.as_string().unwrap_or_default();
    serde_json::from_str(&json_str).map_err(|err| {
        format!("Failed to deserialize JsValue to type: {:?}", err).to_string()
    })
}

#[allow(unused)]
pub fn empty_js_value() -> JsValue {
    #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
    {
        js_sys::Object::new().into()
    }
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
    {
        JsValue::new("{}")
    }
}