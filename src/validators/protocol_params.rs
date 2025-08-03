use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::convert::TryFrom;
use crate::common::{CostModels, ExUnitPrices, ExUnits, SubCoin};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolParameters {
    /// Linear factor for the minimum fee calculation formula
    pub min_fee_coefficient_a: u64,
    /// Constant factor for the minimum fee calculation formula
    pub min_fee_constant_b: u64,
    
    // Block size limits
    /// Maximum block body size in bytes
    pub max_block_body_size: u32,
    /// Maximum transaction size in bytes
    pub max_transaction_size: u32,
    /// Maximum block header size in bytes
    pub max_block_header_size: u32,
    
    // Deposit parameters
    /// Deposit amount required for registering a stake key
    pub stake_key_deposit: u64,
    /// Deposit amount required for registering a stake pool
    pub stake_pool_deposit: u64,
    
    // Stake pool parameters
    /// Maximum number of epochs that can be used for pool retirement ahead
    pub max_epoch_for_pool_retirement: u32,

    
    // Version information
    /// Protocol version (major, minor)
    pub protocol_version: (u32, u32),
    
    // Cost parameters
    /// Minimum pool cost in lovelace
    pub min_pool_cost: u64,
    /// Cost per UTxO byte in lovelace
    pub ada_per_utxo_byte: u64,
    /// Cost models for Plutus script execution
    pub cost_models: CostModels,
    /// Price of execution units for script execution
    pub execution_prices: ExUnitPrices,
    /// Maximum execution units allowed for a transaction
    pub max_tx_execution_units: ExUnits,
    /// Maximum execution units allowed for a block
    pub max_block_execution_units: ExUnits,
    
    // Value size
    /// Maximum size of a Value in bytes
    pub max_value_size: u32,
    
    // Collateral parameters
    /// Percentage of transaction fee required as collateral
    pub collateral_percentage: u32,
    /// Maximum number of collateral inputs
    pub max_collateral_inputs: u32,
    
    /// Deposit amount required for submitting a governance action
    pub governance_action_deposit: u64,
    /// Deposit amount required for registering as a DRep
    pub drep_deposit: u64,
    
    // Reference scripts
    /// Coins per byte for reference scripts
    pub reference_script_cost_per_byte: SubCoin,
}
