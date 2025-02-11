use serde::Deserialize;

#[derive(Deserialize)]
pub struct DecodingParams {
    pub plutus_script_version: Option<i32>,
    pub plutus_data_schema: Option<PlutusDataSchema>,
}

#[derive(Deserialize)]
pub enum PlutusDataSchema {
    BasicConversions,
    DetailedSchema,
}