use std::fmt::Display;
use schemars::JsonSchema;
use std::convert::TryFrom;
use serde::{Serialize, Deserialize};
use std::fmt;

pub use crate::validators::value::{Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GovernanceActionType {
    ParameterChangeAction,
    HardForkInitiationAction,
    TreasuryWithdrawalsAction,
    NoConfidenceAction,
    UpdateCommitteeAction,
    NewConstitutionAction,
    InfoAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum NetworkType {
    Mainnet,
    Preview,
    Preprod,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeeDecomposition {
    pub tx_size_fee: u64,
    pub reference_scripts_fee: u64,
    pub execution_units_fee: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LocalCredential {
    KeyHash(Vec<u8>),
    ScriptHash(Vec<u8>),
}

impl Display for LocalCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalCredential::KeyHash(key_hash) => write!(f, "KeyHash({})", hex::encode(key_hash)),
            LocalCredential::ScriptHash(script_hash) => write!(f, "ScriptHash({})", hex::encode(script_hash)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Voter {
    ConstitutionalCommitteeHotScriptHash(Vec<u8>),
    ConstitutionalCommitteeHotKeyHash(Vec<u8>),
    DRepScriptHash(Vec<u8>),
    DRepKeyHash(Vec<u8>),
    StakingPoolKeyHash(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceActionId {
    pub tx_hash: Vec<u8>,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolVersion {
    pub major: u64,
    pub minor: u64,
}