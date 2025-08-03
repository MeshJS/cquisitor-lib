use crate::common::{UTxO, Asset};
use cardano_serialization_lib as csl;

pub fn convert_utxo_to_csl(utxo: UTxO) -> csl::TransactionUnspentOutput {
    // Create a TransactionInput
    let tx_input = csl::TransactionInput::new(
        &csl::TransactionHash::from_hex(&utxo.input.tx_hash)
            .expect("Invalid transaction hash length"),
        utxo.input.output_index
    );
    
    // Convert address from bech32 string to CSL Address
    let address = csl::Address::from_bech32(&utxo.output.address)
        .expect("Invalid address format");
    
    // Create a Value object for the amount
    let value = create_value_from_assets(utxo.output.amount);
    
    // Create a TransactionOutput
    let mut tx_output = csl::TransactionOutput::new(
        &address,
        &value
    );
    
    // Set datum hash if present
    if let Some(data_hash_hex) = &utxo.output.data_hash {
        let datum_hash = csl::DataHash::from_hex(data_hash_hex)
            .expect("Invalid datum hash length");
        tx_output.set_data_hash(&datum_hash);
    }
    
    // Set inline datum if present
    if let Some(plutus_data_hex) = &utxo.output.plutus_data {
        let plutus_data = csl::PlutusData::from_hex(plutus_data_hex)
            .expect("Invalid plutus data");
        tx_output.set_plutus_data(&plutus_data);
    }
    
    // Set script reference if present
    if let Some(script_ref_hex) = &utxo.output.script_ref {
        let script_ref = csl::ScriptRef::from_hex(&script_ref_hex)
            .expect("Invalid script reference");
        tx_output.set_script_ref(&script_ref);
    }
    
    // Create and return the TransactionUnspentOutput
    csl::TransactionUnspentOutput::new(&tx_input, &tx_output)
}

fn create_value_from_assets(assets: Vec<Asset>) -> csl::Value {
    // Check if we have a single lovelace asset
    if assets.len() == 1 && (assets[0].unit == "lovelace" || assets[0].unit == "") {
        let amount = assets[0].quantity.parse::<u64>()
            .expect("Invalid lovelace amount");
        return csl::Value::new(&csl::BigNum::from_str(&amount.to_string())
            .expect("Failed to create BigNum from amount"));
    }
    
    // Handle multi-asset case
    let mut value = csl::Value::new(&csl::BigNum::zero());
    let mut multi_asset = csl::MultiAsset::new();
    
    for asset in assets {
        if asset.unit == "lovelace" {
            let amount = asset.quantity.parse::<u64>()
                .expect("Invalid lovelace amount");
            value = csl::Value::new(&csl::BigNum::from_str(&amount.to_string())
                .expect("Failed to create BigNum from amount"));
        } else {
            // Parse policy ID and asset name from the unit
            // Format is typically: policyId.assetName
            let parts: Vec<&str> = asset.unit.split('.').collect();
            if parts.len() != 2 {
                continue; // Skip malformed assets
            }
            
            let policy_id_hex = parts[0];
            let policy_id = csl::ScriptHash::from_hex(&policy_id_hex)
                .expect("Invalid policy ID length");
            
            let asset_name_bytes = hex::decode(parts[1])
                .expect("Invalid asset name format");
            let asset_name = csl::AssetName::new(asset_name_bytes)
                .expect("Invalid asset name");
            
            let amount = asset.quantity.parse::<u64>()
                .expect("Invalid asset amount");

            multi_asset.set_asset(&policy_id, &asset_name, &csl::BigNum::from_str(&amount.to_string())
                .expect("Failed to create BigNum from amount"));
        }
    }
    
    // Set the multi-asset part of the value
    if multi_asset.len() != 0 {
        value.set_multiasset(&multi_asset);
    }
    
    value
}

pub(crate) fn pp_cost_model_to_csl(pp_cost_model: &Vec<i64>) -> csl::CostModel {
    let mut cost_model = csl::CostModel::new();
    for (i, cost) in pp_cost_model.iter().enumerate() {
        if *cost < 0 {
            #[allow(unused_must_use)]
            cost_model.set(i, &csl::Int::new_negative(&csl::BigNum::from(cost.abs() as u64)));
        } else {
            #[allow(unused_must_use)]
            cost_model.set(i, &csl::Int::new(&csl::BigNum::from(cost.abs() as u64)));
        }

    }
    cost_model
}