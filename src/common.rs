use cardano_serialization_lib as csl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub unit: String,
    pub quantity: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TxInput {
    pub output_index: u32,
    pub tx_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TxOutput {
    pub address: String,
    pub amount: Vec<Asset>,
    pub data_hash: Option<String>,
    pub plutus_data: Option<String>,
    pub script_ref: Option<String>,
    pub script_hash: Option<String>,
}

impl TxOutput {
    pub fn find_ada_asset(&self) -> Option<&Asset> {
        self.amount
            .iter()
            .find(|asset| asset.unit == "lovelace" || asset.unit == "")
    }

    pub fn has_non_ada_assets(&self) -> bool {
        self.amount
            .iter()
            .any(|asset| asset.unit != "lovelace" && asset.unit != "")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UTxO {
    pub input: TxInput,
    pub output: TxOutput,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CostModels {
    pub plutus_v1: Option<Vec<i64>>,
    pub plutus_v2: Option<Vec<i64>>,
    pub plutus_v3: Option<Vec<i64>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExUnitPrices {
    pub mem_price: SubCoin,
    pub step_price: SubCoin,
}

impl ExUnitPrices {
    pub fn to_csl(&self) -> csl::ExUnitPrices {
        csl::ExUnitPrices::new(&self.mem_price.to_csl(), &self.step_price.to_csl())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubCoin {
    pub numerator: u64,
    pub denominator: u64,
}

impl SubCoin {
    pub fn to_csl(&self) -> csl::UnitInterval {
        csl::UnitInterval::new(
            &csl::BigNum::from(self.numerator),
            &csl::BigNum::from(self.denominator),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExUnits {
    pub mem: u64,
    pub steps: u64,
}
