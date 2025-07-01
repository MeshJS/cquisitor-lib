use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use crate::{common::{TxInput, UTxO}, validators::phase_1::common::{GovernanceActionId, GovernanceActionType, NetworkType}};
use schemars::JsonSchema;

use super::{common::LocalCredential, ProtocolParameters};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NecessaryInputData {
    pub utxos: Vec<TxInput>,
    pub accounts: Vec<String>,
    pub pools: Vec<String>,
    pub d_reps: Vec<String>,
    pub gov_actions: Vec<GovernanceActionId>,
    pub last_enacted_gov_action: Vec<GovernanceActionType>,
    pub committee_members: Vec<LocalCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GovActionInputContext {
    pub action_id: GovernanceActionId,
    pub action_type: GovernanceActionType,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DrepInputContext {
    pub bech32_drep: String,
    pub is_registered: bool,
    pub payed_deposit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountInputContext {
    pub bech32_address: String,
    pub is_registered: bool,
    pub payed_deposit: Option<u64>,
    pub delegated_to_drep: Option<String>,
    pub delegated_to_pool: Option<String>,
    pub balance: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PoolInputContext {
    pub pool_id: String,
    pub is_registered: bool,
    pub retirement_epoch: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UtxoInputContext {
    pub utxo: UTxO,
    pub is_spent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommitteeInputContext {
    pub active_committee_members: Vec<LocalCredential>,
    pub potential_committee_members: Vec<LocalCredential>,
    pub resigned_committee_members: Vec<LocalCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidationInputContext {
    pub utxo_set: Vec<UtxoInputContext>,
    pub protocol_parameters: ProtocolParameters,
    pub slot: u64,
    pub account_contexts: Vec<AccountInputContext>,
    pub drep_contexts: Vec<DrepInputContext>,
    pub pool_contexts: Vec<PoolInputContext>,
    pub gov_action_contexts: Vec<GovActionInputContext>,
    pub last_enacted_gov_action: Vec<GovActionInputContext>,
    pub committee_context: CommitteeInputContext,
    pub treasury_value: u64,
    pub network_type: NetworkType,
}

impl ValidationInputContext {
    pub fn new(
        utxo_set: Vec<UtxoInputContext>,
        protocol_parameters: ProtocolParameters,
        slot: u64,
        account_contexts: Vec<AccountInputContext>,
        drep_contexts: Vec<DrepInputContext>,
        pool_contexts: Vec<PoolInputContext>,
        gov_action_contexts: Vec<GovActionInputContext>,
        last_enacted_gov_action: Vec<GovActionInputContext>,
        treasury_value: u64,
        network_type: NetworkType,
        committee_context: CommitteeInputContext,
    ) -> Self {
        Self {
            utxo_set,
            protocol_parameters,
            slot,
            account_contexts,
            drep_contexts,
            pool_contexts,
            gov_action_contexts,
            last_enacted_gov_action,
            treasury_value,
            network_type,
            committee_context,
        }
    }

    pub fn find_utxo(&self, tx_hash: String, tx_index: u32) -> Option<&UtxoInputContext> {
        self.utxo_set.iter().find(|utxo| utxo.utxo.input.tx_hash == tx_hash && utxo.utxo.input.output_index == tx_index)
    }

    pub fn find_account_context(&self, bech32_address: &String) -> Option<&AccountInputContext> {
        self.account_contexts.iter().find(|account| &account.bech32_address == bech32_address)
    }

    pub fn find_drep_context(&self, bech32_drep: &String) -> Option<&DrepInputContext> {
        self.drep_contexts.iter().find(|drep| &drep.bech32_drep == bech32_drep)
    }

    pub fn find_pool_context(&self, pool_id: &String) -> Option<&PoolInputContext> {
        self.pool_contexts.iter().find(|pool| &pool.pool_id == pool_id)
    }

    pub fn find_gov_action_context(&self, action_id: GovernanceActionId) -> Option<&GovActionInputContext> {
        self.gov_action_contexts.iter().find(|action| action.action_id == action_id)
    }

    pub fn find_last_enacted_gov_action(&self, action_type: GovernanceActionType) -> Option<&GovActionInputContext> {
        self.last_enacted_gov_action.iter().find(|action| action.action_type == action_type)
    }

    pub fn find_committee_member(&self, credential: &LocalCredential) -> Option<&LocalCredential> {
        self.committee_context.active_committee_members.iter().find(|member| member == &credential)
    }

    pub fn find_potential_committee_member(&self, credential: &LocalCredential) -> Option<&LocalCredential> {
        self.committee_context.potential_committee_members.iter().find(|member| member == &credential)
    }
    
    pub fn find_resigned_committee_member(&self, credential: &LocalCredential) -> Option<&LocalCredential> {
        self.committee_context.resigned_committee_members.iter().find(|member| member == &credential)
    }

    pub fn is_active_committee_member(&self, credential: &LocalCredential) -> bool {
        self.committee_context.active_committee_members.iter().any(|member| member == credential)
    }

    pub fn is_potential_committee_member(&self, credential: &LocalCredential) -> bool {
        self.committee_context.potential_committee_members.iter().any(|member| member == credential)
    }
    
    pub fn is_resigned_committee_member(&self, credential: &LocalCredential) -> bool {
        self.committee_context.resigned_committee_members.iter().any(|member| member == credential)
    }
}