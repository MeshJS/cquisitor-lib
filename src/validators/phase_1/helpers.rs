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