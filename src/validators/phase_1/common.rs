use std::{collections::{HashMap, HashSet}, str};
use serde::{Serialize, Deserialize, ser::{SerializeStruct, Serializer}, de::{self, Deserializer, Visitor}};
use std::fmt;

use cardano_serialization_lib as csl;
use cardano_serialization_lib::{
    Credential,
    Vkeywitness,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Value {
    pub assets: Vec<Asset>,
    pub coins: i128,
}

pub struct ValidationContext {
    pub utxo_set: Vec<csl::TransactionUnspentOutput>,
    pub protocol_parameters: csl::ProtocolParamUpdate,
}

pub struct CollateralContext {
    pub inputs: Vec<csl::TransactionUnspentOutput>,
    pub outputs: Vec<csl::TransactionOutput>,
    pub total_collateral: Value,
    pub actual_collateral: Value,
    pub estimated_minimal_collateral: i128,
}

pub struct AuxiliaryDataContext {
    pub auxiliary_data: Option<csl::AuxiliaryData>,
    pub expected_auxiliary_data_hash: Option<csl::AuxiliaryDataHash>,
    pub actual_auxiliary_data_hash: Option<csl::AuxiliaryDataHash>,
}

pub struct ScriptDataContext {
    pub cost_models: csl::Costmdls,
    pub used_plutus_script_versions: csl::Languages,
    pub transaction_witness_set: Vec<csl::TransactionWitnessSet>,
    pub expected_script_data_hash: Option<csl::ScriptDataHash>,
    pub actual_script_data_hash: Option<csl::ScriptDataHash>,
}

pub struct DepositContext {
    pub total_deposit: i128,
    pub total_refund: i128,
}

pub struct RegistrationContext {
    pub registered_accounts: HashMap<csl::RewardAddress, i128>,
    pub registered_pools: HashSet<csl::Ed25519KeyHash>,
    pub registered_dreps: HashSet<csl::DRep>,
    pub registered_consitution_commettee: HashMap<Credential, u32>,
    pub registered_consitution_committee_hot_credentials: HashSet<Credential>,
    pub registered_voting_proposals: HashSet<csl::GovernanceActionId>,
}

pub struct InputContext {
    pub inputs: csl::TransactionUnspentOutput,
    pub outputs: csl::TransactionOutput,
    pub deposit_context: DepositContext,
    pub mint: csl::Mint,
    pub withdrawal: csl::Withdrawals,

    pub total_input: Value,
    pub total_output: Value,
}

pub struct TransactionContext {
    pub input_context: InputContext,
    pub fee: i128,
    pub collateral_context: CollateralContext,
    pub tx_body_hash: csl::TransactionHash,
}

pub struct AttachedValidation {
    pub location: String,
}

pub enum WitnessableEntity {
    TxInput(csl::TransactionInput),
    Mint(csl::MintAssets, u32),
    Withdrawal(csl::RewardAddress, u32),
    Certificate(csl::Certificate, u32),
    Vote(csl::Voter, u32),
    VotingProposal(csl::VotingProposal, u32),
}

pub struct NativeScriptWitness {
    native_script: csl::NativeScript,
    script_hash: csl::ScriptHash,
}

pub struct VKeyWitness {
    vkey: csl::Vkey,
    signature: csl::Ed25519Signature,
    vkey_hash: csl::Ed25519KeyHash,
}

pub struct ByronWitness {
    signature: csl::BootstrapWitness,
    key_hash: csl::Ed25519KeyHash,
}

pub enum TransactionWitness {
    PlutusScript(csl::PlutusScript),
    NativeScript(csl::NativeScript),
    ByronSignature(csl::BootstrapWitness),
    VKeySignature(Vkeywitness),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Asset {
    pub policy_id: Vec<u8>,
    pub asset_name: Vec<u8>,
    pub quantity: i128,
}

impl Serialize for Asset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Asset", 3)?;
        state.serialize_field("policy_id", &hex::encode(&self.policy_id))?;
        state.serialize_field("asset_name", &hex::encode(&self.asset_name))?;
        state.serialize_field("quantity", &self.quantity)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            PolicyId,
            AssetName,
            Quantity,
        }

        struct AssetVisitor;
        impl<'de> Visitor<'de> for AssetVisitor {
            type Value = Asset;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Asset with policy_id, asset_name, and quantity")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Asset, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut policy_id = None;
                let mut asset_name = None;
                let mut quantity = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::PolicyId => {
                            if policy_id.is_some() {
                                return Err(de::Error::duplicate_field("policy_id"));
                            }
                            let hex_str: String = map.next_value()?;
                            policy_id = Some(hex::decode(&hex_str).map_err(de::Error::custom)?);
                        }
                        Field::AssetName => {
                            if asset_name.is_some() {
                                return Err(de::Error::duplicate_field("asset_name"));
                            }
                            let hex_str: String = map.next_value()?;
                            asset_name = Some(hex::decode(&hex_str).map_err(de::Error::custom)?);
                        }
                        Field::Quantity => {
                            if quantity.is_some() {
                                return Err(de::Error::duplicate_field("quantity"));
                            }
                            quantity = Some(map.next_value()?);
                        }
                    }
                }

                let policy_id = policy_id.ok_or_else(|| de::Error::missing_field("policy_id"))?;
                let asset_name = asset_name.ok_or_else(|| de::Error::missing_field("asset_name"))?;
                let quantity = quantity.ok_or_else(|| de::Error::missing_field("quantity"))?;

                Ok(Asset {
                    policy_id,
                    asset_name,
                    quantity,
                })
            }
        }

        deserializer.deserialize_struct("Asset", &["policy_id", "asset_name", "quantity"], AssetVisitor)
    }
}
