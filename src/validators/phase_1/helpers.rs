use cardano_serialization_lib as csl;

use crate::validators::phase_1::common::{LocalCredential, NetworkType};

pub fn string_to_csl_address(address_str: &String) -> Result<csl::Address, String> {
    match csl::Address::from_bech32(&address_str) {
        Ok(address) => Ok(address),
        Err(_) => match csl::Address::from_hex(&address_str) {
            Ok(address) => Ok(address),
            Err(_) => match csl::ByronAddress::from_base58(&address_str) {
                Ok(byron_address) => Ok(byron_address.to_address()),
                Err(e) => Err(format!("Error converting address {}: {:?}", address_str, e)),
            }
        }
    }
}

pub fn csl_tx_input_to_string(tx_input: &csl::TransactionInput) -> String {
    format!("{}#{}", tx_input.transaction_id().to_hex(), tx_input.index())
}

pub fn credential_to_bech32_reward_address(credential: &csl::Credential, network_type: &NetworkType) -> String {
    let network_id = match network_type {
        NetworkType::Mainnet =>  csl::NetworkInfo::mainnet().network_id(),
        NetworkType::Testnet =>  csl::NetworkInfo::testnet_preprod().network_id(),
    };
    let address = csl::RewardAddress::new(network_id, credential).to_address().to_bech32(None);
    address.unwrap_or_else(|_| "".to_string())
}

pub fn csl_credential_to_local_credential(credential: &csl::Credential) -> LocalCredential {
    match credential.kind() {
        csl::CredKind::Key => {
            if let Some(key_hash) = credential.to_keyhash() {
                LocalCredential::KeyHash(key_hash.to_bytes())
            } else {
                LocalCredential::KeyHash(vec![])
            }
        }
        csl::CredKind::Script => {
            if let Some(script_hash) = credential.to_scripthash() {
                LocalCredential::ScriptHash(script_hash.to_bytes())
            } else {
                LocalCredential::ScriptHash(vec![])
            }
        }
    }
}

pub fn normalize_script_ref(
    script_ref: &String,
) -> Result<csl::ScriptRef, String> {
    if script_ref.starts_with("82") {
        let bytes = hex::decode(script_ref.clone())
            .map_err(|e| format!("Failed to decode script ref hex: {}", e))?;
        let mut encoder = pallas_codec::minicbor::Encoder::new(Vec::new());
        encoder
            .tag(pallas_codec::minicbor::data::Tag::new(24))
            .map_err(|e| "Failed to write tag")?;
        encoder
            .bytes(&bytes)
            .map_err(|e| format!("Failed to encode script ref bytes: {}", e))?;
        let write_buffer = encoder.writer().clone();
        csl::ScriptRef::from_bytes(write_buffer)
            .map_err(|_| "Failed to decode script ref hex".to_string())
    } else {
        csl::ScriptRef::from_hex(&script_ref).map_err(|e| {
            format!(
                "Failed to parse script ref: {:?} - with ref: {}",
                e,
                script_ref
            )
        })
    }
}