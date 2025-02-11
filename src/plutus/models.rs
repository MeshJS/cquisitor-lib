use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    unit: String,
    quantity: String,
}

impl Asset {
    pub fn unit_to_tuple(unit: &str) -> (String, String) {
        let unit = if unit == "lovelace" {
            "".to_string()
        } else {
            unit.to_string()
        };
        let policy = unit.chars().take(56).collect();
        let name = unit.chars().skip(56).collect();
        (policy, name)
    }
    pub fn new(unit: String, quantity: String) -> Self {
        Asset { unit, quantity }
    }
    pub fn new_from_str(unit: &str, quantity: &str) -> Self {
        Asset {
            unit: unit.to_string(),
            quantity: quantity.to_string(),
        }
    }
    pub fn unit(&self) -> String {
        self.unit.clone()
    }
    pub fn policy(&self) -> String {
        self.unit.chars().take(56).collect()
    }
    pub fn name(&self) -> String {
        self.unit.chars().skip(56).collect()
    }
    pub fn quantity(&self) -> String {
        self.quantity.clone()
    }
    pub fn quantity_i128(&self) -> i128 {
        self.quantity.parse().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInput {
    pub output_index: u32,
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    pub address: String,
    pub amount: Vec<Asset>,
    pub data_hash: Option<String>,
    pub plutus_data: Option<String>,
    pub script_ref: Option<String>,
    pub script_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UTxO {
    pub input: TxInput,
    pub output: TxOutput,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CostModels {
    pub(crate) plutus_v1: Option<Vec<i64>>,
    pub(crate) plutus_v2:  Option<Vec<i64>>,
    pub(crate) plutus_v3: Option<Vec<i64>>,
}