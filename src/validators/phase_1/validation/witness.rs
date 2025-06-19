use crate::validators::phase_1::{
    common::LocalCredential,
    errors::{Phase1Error, ValidationError, ValidationResult},
    helpers::string_to_csl_address,
    ValidationInputContext,
};
use cardano_serialization_lib as csl;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessRequirement {
    /// Verification key signature required
    VKeySignature(csl::Ed25519KeyHash),
    /// Native script required
    NativeScript(csl::ScriptHash),
    /// Plutus script required
    PlutusScript(csl::ScriptHash),
    /// Redeemer required
    Redeemer { tag: csl::RedeemerTag, index: u32 },
    /// Datum required
    Datum(csl::DataHash),
    /// Unknown script type (not found in provided witnesses)
    UnknownScript(csl::ScriptHash),
}

#[derive(Debug, Clone)]
pub enum WitnessSource {
    /// Witness provided in witness set
    WitnessSet,
    /// Script available through reference input
    ReferenceInput(csl::TransactionInput),
}

#[derive(Debug, Clone)]
pub struct RequiredVKeyWitness {
    pub key_hash: csl::Ed25519KeyHash,
    pub location: String,
    pub entity_index: u32,
}

#[derive(Debug, Clone)]
pub struct RequiredScriptWitness {
    pub script_hash: csl::ScriptHash,
    pub location: String,
    pub entity_index: u32,
}

#[derive(Debug, Clone)]
pub struct RequiredRedeemerWitness {
    pub tag: csl::RedeemerTag,
    pub index: u32,
    pub location: String,
    pub entity_index: u32,
}

#[derive(Debug, Clone)]
pub struct RequiredDatumWitness {
    pub datum_hash: csl::DataHash,
    pub location: String,
    pub entity_index: u32,
}

pub struct WitnessValidator<'a> {
    /// Required VKey witnesses
    pub required_vkey_witnesses: Vec<RequiredVKeyWitness>,
    /// Required native script witnesses
    pub required_native_script_witnesses: Vec<RequiredScriptWitness>,
    /// Required plutus script witnesses
    pub required_plutus_script_witnesses: Vec<RequiredScriptWitness>,
    /// Required redeemer witnesses
    pub required_redeemer_witnesses: Vec<RequiredRedeemerWitness>,
    /// Required datum witnesses
    pub required_datum_witnesses: Vec<RequiredDatumWitness>,
    /// Required unknown script witnesses
    pub required_unknown_script_witnesses: Vec<RequiredScriptWitness>,
    /// Set of provided VKey witnesses
    pub provided_vkey_witnesses: HashSet<csl::Ed25519KeyHash>,
    /// Map of native script hashes to their sources
    pub native_script_sources: HashMap<csl::ScriptHash, WitnessSource>,
    pub native_scripts_signature_candidates: HashSet<csl::Ed25519KeyHash>,
    /// Map of plutus script hashes to their sources
    pub plutus_script_sources: HashMap<csl::ScriptHash, WitnessSource>,
    /// Map of datum hashes to their sources
    pub datum_sources: HashMap<csl::DataHash, WitnessSource>,
    pub output_datums_hashes: HashSet<csl::DataHash>,
    /// Set of provided redeemers by (tag, index)
    pub provided_redeemers: HashSet<(csl::RedeemerTag, u32)>,
    /// Validation context
    pub validation_input_context: &'a ValidationInputContext,
}

impl<'a> WitnessValidator<'a> {
    pub fn new(
        tx: &csl::Transaction,
        validation_input_context: &'a ValidationInputContext,
    ) -> Self {
        let mut context = Self {
            required_vkey_witnesses: Vec::new(),
            required_native_script_witnesses: Vec::new(),
            required_plutus_script_witnesses: Vec::new(),
            required_redeemer_witnesses: Vec::new(),
            required_datum_witnesses: Vec::new(),
            required_unknown_script_witnesses: Vec::new(),
            provided_vkey_witnesses: HashSet::new(),
            native_script_sources: HashMap::new(),
            native_scripts_signature_candidates: HashSet::new(),
            plutus_script_sources: HashMap::new(),
            datum_sources: HashMap::new(),
            output_datums_hashes: HashSet::new(),
            provided_redeemers: HashSet::new(),
            validation_input_context,
        };

        // Collect all provided witnesses
        context.collect_provided_witnesses(tx);

        // Collect all required witnesses
        context.collect_required_witnesses(tx);

        // Fill native_scripts_signature_candidates
        context.collect_native_scripts_signature_candidates(tx);

        context.collect_output_datums_hashes(tx);

        context
    }

    fn collect_output_datums_hashes(&mut self, tx: &csl::Transaction) {
        let outputs = tx.body().outputs();
        for i in 0..outputs.len() {
            let output = outputs.get(i);
            if let Some(datum_hash) = output.data_hash() {
                self.output_datums_hashes.insert(datum_hash);
            }
        }
    }

    fn collect_provided_witnesses(&mut self, tx: &csl::Transaction) {
        let witness_set = tx.witness_set();

        // VKey witnesses
        if let Some(vkey_witnesses) = witness_set.vkeys() {
            for i in 0..vkey_witnesses.len() {
                let vkey_witness = vkey_witnesses.get(i);
                let key_hash = vkey_witness.vkey().public_key().hash();
                self.provided_vkey_witnesses.insert(key_hash);
            }
        }

        // Native scripts
        if let Some(native_scripts) = witness_set.native_scripts() {
            for i in 0..native_scripts.len() {
                let script = native_scripts.get(i);
                let script_hash = script.hash();
                self.native_script_sources
                    .insert(script_hash, WitnessSource::WitnessSet);
            }
        }

        // Plutus scripts
        if let Some(plutus_scripts) = witness_set.plutus_scripts() {
            for i in 0..plutus_scripts.len() {
                let script = plutus_scripts.get(i);
                let script_hash = script.hash();
                self.plutus_script_sources
                    .insert(script_hash, WitnessSource::WitnessSet);
            }
        }

        // Plutus data (datums)
        if let Some(plutus_data) = witness_set.plutus_data() {
            for i in 0..plutus_data.len() {
                let datum = plutus_data.get(i);
                let datum_hash = csl::hash_plutus_data(&datum);
                self.datum_sources
                    .insert(datum_hash, WitnessSource::WitnessSet);
            }
        }

        // Redeemers
        if let Some(redeemers) = witness_set.redeemers() {
            for i in 0..redeemers.len() {
                let redeemer = redeemers.get(i);
                if let Ok(index_u32) = redeemer.index().to_str().parse::<u32>() {
                    self.provided_redeemers.insert((redeemer.tag(), index_u32));
                }
            }
        }

        // Collect scripts and datums from reference inputs
        let ref_inputs = tx
            .body()
            .reference_inputs()
            .unwrap_or(csl::TransactionInputs::new());
        let inputs = tx.body().inputs();
        let all_inputs = inputs.into_iter().chain(ref_inputs.into_iter());

        for input in all_inputs {
            if let Some(utxo) = self
                .validation_input_context
                .find_utxo(input.transaction_id().to_hex(), input.index())
            {
                // Check script_ref
                if let Some(script_ref_hex) = &utxo.utxo.output.script_ref {
                    if let Ok(script_ref_bytes) = hex::decode(script_ref_hex) {
                        if let Ok(script_ref) = csl::ScriptRef::from_bytes(script_ref_bytes) {
                            // ScriptRef contains either NativeScript or PlutusScript
                            // Try to get as NativeScript
                            if let Some(native_script) = script_ref.native_script() {
                                let script_hash = native_script.hash();
                                self.native_script_sources.insert(
                                    script_hash,
                                    WitnessSource::ReferenceInput(input.clone()),
                                );
                            }
                            // Try to get as PlutusScript
                            else if let Some(plutus_script) = script_ref.plutus_script() {
                                let script_hash = plutus_script.hash();
                                self.plutus_script_sources.insert(
                                    script_hash,
                                    WitnessSource::ReferenceInput(input.clone()),
                                );
                            }
                        }
                    }
                }

                // Check inline datum
                if let Some(datum_hex) = &utxo.utxo.output.plutus_data {
                    if let Ok(datum_bytes) = hex::decode(datum_hex) {
                        if let Ok(datum) = csl::PlutusData::from_bytes(datum_bytes) {
                            let datum_hash = csl::hash_plutus_data(&datum);
                            self.datum_sources
                                .insert(datum_hash, WitnessSource::ReferenceInput(input.clone()));
                        }
                    }
                }
            }
        }
    }

    /// Determines script type by its hash
    fn determine_script_type(&self, script_hash: &csl::ScriptHash) -> Option<WitnessRequirement> {
        if self.native_script_sources.contains_key(script_hash) {
            Some(WitnessRequirement::NativeScript(script_hash.clone()))
        } else if self.plutus_script_sources.contains_key(script_hash) {
            Some(WitnessRequirement::PlutusScript(script_hash.clone()))
        } else {
            None
        }
    }

    /// Adds required witnesses for script based on its type
    /// If script is unknown, adds UnknownScript requirement
    fn add_script_witness_requirement(
        &mut self,
        script_hash: csl::ScriptHash,
        location: String,
        entity_index: u32,
        redeemer_tag: Option<csl::RedeemerTag>,
        datum_hash: Option<csl::DataHash>,
    ) {
        if let Some(script_type) = self.determine_script_type(&script_hash) {
            match script_type {
                WitnessRequirement::PlutusScript(_) => {
                    // Plutus script
                    self.required_plutus_script_witnesses
                        .push(RequiredScriptWitness {
                            script_hash,
                            location: location.clone(),
                            entity_index,
                        });

                    // Redeemer (if tag provided)
                    if let Some(tag) = redeemer_tag {
                        self.required_redeemer_witnesses
                            .push(RequiredRedeemerWitness {
                                tag,
                                index: entity_index,
                                location: location.clone(),
                                entity_index,
                            });
                    }

                    // Datum (if provided)
                    if let Some(datum) = datum_hash {
                        self.required_datum_witnesses.push(RequiredDatumWitness {
                            datum_hash: datum,
                            location,
                            entity_index,
                        });
                    }
                }
                WitnessRequirement::NativeScript(_) => {
                    // Native script
                    self.required_native_script_witnesses
                        .push(RequiredScriptWitness {
                            script_hash,
                            location,
                            entity_index,
                        });
                }
                _ => {}
            }
        } else {
            // If we can't determine script type, mark as unknown
            self.required_unknown_script_witnesses
                .push(RequiredScriptWitness {
                    script_hash,
                    location,
                    entity_index,
                });
        }
    }

    fn collect_required_witnesses(&mut self, tx: &csl::Transaction) {
        // 1. Inputs
        self.collect_input_witnesses(tx);

        // 2. Collateral inputs
        self.collect_collateral_witnesses(tx);

        // 3. Withdrawals
        self.collect_withdrawal_witnesses(tx);

        // 4. Certificates
        self.collect_certificate_witnesses(tx);

        // 5. Voting proposals
        self.collect_voting_proposal_witnesses(tx);

        // 6. Votes
        self.collect_vote_witnesses(tx);

        // 7. Mint
        self.collect_mint_witnesses(tx);

        // 8. Required signers
        self.collect_required_signer_witnesses(tx);
    }

    fn collect_native_scripts_signature_candidates(&mut self, tx: &csl::Transaction) {
        // Iterate through all required native scripts
        for required in &self.required_native_script_witnesses.clone() {
            let script_hash = &required.script_hash;
            // Check if native script exists in witness set
            if let Some(witness_set) = tx.witness_set().native_scripts() {
                for i in 0..witness_set.len() {
                    let native_script = witness_set.get(i);
                    if native_script.hash() == *script_hash {
                        // Get key hashes from native script
                        let key_hashes = get_native_script_key_hashes(&native_script);
                        for key_hash in key_hashes {
                            self.native_scripts_signature_candidates.insert(key_hash);
                        }
                        break;
                    }
                }
            }

            // Check native scripts from reference inputs
            if let Some(source) = self.native_script_sources.get(script_hash) {
                if let WitnessSource::ReferenceInput(ref_input) = source {
                    if let Some(utxo) = self
                        .validation_input_context
                        .find_utxo(ref_input.transaction_id().to_hex(), ref_input.index())
                    {
                        if let Some(script_ref_hex) = &utxo.utxo.output.script_ref {
                            if let Ok(script_ref_bytes) = hex::decode(script_ref_hex) {
                                if let Ok(script_ref) = csl::ScriptRef::from_bytes(script_ref_bytes)
                                {
                                    if let Some(native_script) = script_ref.native_script() {
                                        if native_script.hash() == *script_hash {
                                            let key_hashes =
                                                get_native_script_key_hashes(&native_script);
                                            for key_hash in key_hashes {
                                                self.native_scripts_signature_candidates
                                                    .insert(key_hash);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_input_witnesses(&mut self, tx: &csl::Transaction) {
        let inputs = tx.body().inputs();
        for i in 0..inputs.len() {
            let input = inputs.get(i);
            if let Some(utxo) = self
                .validation_input_context
                .find_utxo(input.transaction_id().to_hex(), input.index())
            {
                if let Ok(address) = string_to_csl_address(&utxo.utxo.output.address) {
                    if let Some(payment_cred) = address.payment_cred() {
                        match payment_cred.kind() {
                            csl::CredKind::Key => {
                                if let Some(key_hash) = payment_cred.to_keyhash() {
                                    self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                        key_hash,
                                        location: format!("transaction.body.inputs.{}", i),
                                        entity_index: i as u32,
                                    });
                                }
                            }
                            csl::CredKind::Script => {
                                if let Some(script_hash) = payment_cred.to_scripthash() {
                                    // Get datum hash for Plutus script inputs
                                    let datum_hash = if let Some(data_hash_hex) =
                                        &utxo.utxo.output.data_hash
                                    {
                                        if let Ok(data_hash_bytes) = hex::decode(data_hash_hex) {
                                            csl::DataHash::from_bytes(data_hash_bytes).ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };

                                    self.add_script_witness_requirement(
                                        script_hash,
                                        format!("transaction.body.inputs.{}", i),
                                        i as u32,
                                        Some(csl::RedeemerTag::new_spend()),
                                        datum_hash,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_collateral_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(collateral) = tx.body().collateral() {
            for i in 0..collateral.len() {
                let input = collateral.get(i);
                if let Some(utxo) = self
                    .validation_input_context
                    .find_utxo(input.transaction_id().to_hex(), input.index())
                {
                    if let Ok(address) = string_to_csl_address(&utxo.utxo.output.address) {
                        if let Some(payment_cred) = address.payment_cred() {
                            if payment_cred.kind() == csl::CredKind::Key {
                                if let Some(key_hash) = payment_cred.to_keyhash() {
                                    self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                        key_hash,
                                        location: format!("transaction.body.collateral.{}", i),
                                        entity_index: i as u32,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_withdrawal_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(withdrawals) = tx.body().withdrawals() {
            let withdrawal_keys = withdrawals.keys();
            for i in 0..withdrawal_keys.len() {
                let reward_address = withdrawal_keys.get(i);
                let stake_cred = reward_address.payment_cred();

                match stake_cred.kind() {
                    csl::CredKind::Key => {
                        if let Some(key_hash) = stake_cred.to_keyhash() {
                            self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                key_hash,
                                location: format!("transaction.body.withdrawals.{}", i),
                                entity_index: i as u32,
                            });
                        }
                    }
                    csl::CredKind::Script => {
                        if let Some(script_hash) = stake_cred.to_scripthash() {
                            // Determine script type by collected witnesses
                            self.add_script_witness_requirement(
                                script_hash,
                                format!("transaction.body.withdrawals.{}", i),
                                i as u32,
                                Some(csl::RedeemerTag::new_reward()),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }

    fn collect_certificate_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(certs) = tx.body().certs() {
            for i in 0..certs.len() {
                let cert = certs.get(i);
                self.collect_certificate_witness(&cert, i as u32);
            }
        }
    }

    fn collect_certificate_witness(&mut self, cert: &csl::Certificate, index: u32) {
        match cert.kind() {
            csl::CertificateKind::StakeRegistration => {
                if let Some(stake_reg) = cert.as_stake_registration() {
                    let stake_cred = stake_reg.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::StakeDeregistration => {
                if let Some(stake_dereg) = cert.as_stake_deregistration() {
                    let stake_cred = stake_dereg.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::StakeDelegation => {
                if let Some(stake_del) = cert.as_stake_delegation() {
                    let stake_cred = stake_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::PoolRegistration => {
                if let Some(pool_reg) = cert.as_pool_registration() {
                    let pool_params = pool_reg.pool_params();
                    let operator = pool_params.operator();
                    self.required_vkey_witnesses.push(RequiredVKeyWitness {
                        key_hash: operator,
                        location: format!("transaction.body.certs.{}", index),
                        entity_index: index,
                    });
                    let owners = pool_params.pool_owners();
                    for owner in owners.into_iter() {
                        self.required_vkey_witnesses.push(RequiredVKeyWitness {
                            key_hash: owner.clone(),
                            location: format!("transaction.body.certs.{}", index),
                            entity_index: index,
                        });
                    }
                }
            }
            csl::CertificateKind::PoolRetirement => {
                if let Some(pool_ret) = cert.as_pool_retirement() {
                    let pool_keyhash = pool_ret.pool_keyhash();
                    self.required_vkey_witnesses.push(RequiredVKeyWitness {
                        key_hash: pool_keyhash,
                        location: format!("transaction.body.certs.{}", index),
                        entity_index: index,
                    });
                }
            }
            csl::CertificateKind::DRepRegistration => {
                if let Some(drep_reg) = cert.as_drep_registration() {
                    let voting_cred = drep_reg.voting_credential();
                    self.add_certificate_credential_witness(
                        voting_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::DRepDeregistration => {
                if let Some(drep_dereg) = cert.as_drep_deregistration() {
                    let voting_cred = drep_dereg.voting_credential();
                    self.add_certificate_credential_witness(
                        voting_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::DRepUpdate => {
                if let Some(drep_update) = cert.as_drep_update() {
                    let voting_cred = drep_update.voting_credential();
                    self.add_certificate_credential_witness(
                        voting_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::CommitteeHotAuth => {
                if let Some(committee_auth) = cert.as_committee_hot_auth() {
                    let committee_cold_cred = committee_auth.committee_cold_credential();
                    self.add_certificate_credential_witness(
                        committee_cold_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::CommitteeColdResign => {
                if let Some(committee_resign) = cert.as_committee_cold_resign() {
                    let committee_cold_cred = committee_resign.committee_cold_credential();
                    self.add_certificate_credential_witness(
                        committee_cold_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::StakeAndVoteDelegation => {
                if let Some(stake_vote_del) = cert.as_stake_and_vote_delegation() {
                    let stake_cred = stake_vote_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::StakeRegistrationAndDelegation => {
                if let Some(stake_reg_del) = cert.as_stake_registration_and_delegation() {
                    let stake_cred = stake_reg_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::StakeVoteRegistrationAndDelegation => {
                if let Some(stake_vote_reg_del) = cert.as_stake_vote_registration_and_delegation() {
                    let stake_cred = stake_vote_reg_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::VoteDelegation => {
                if let Some(vote_del) = cert.as_vote_delegation() {
                    let stake_cred = vote_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::VoteRegistrationAndDelegation => {
                if let Some(vote_reg_del) = cert.as_vote_registration_and_delegation() {
                    let stake_cred = vote_reg_del.stake_credential();
                    self.add_certificate_credential_witness(
                        stake_cred,
                        format!("transaction.body.certs.{}", index),
                        index,
                    );
                }
            }
            csl::CertificateKind::GenesisKeyDelegation => {}
            csl::CertificateKind::MoveInstantaneousRewardsCert => {}
        }
    }

    fn add_certificate_credential_witness(
        &mut self,
        credential: csl::Credential,
        location: String,
        index: u32,
    ) {
        match credential.kind() {
            csl::CredKind::Key => {
                if let Some(key_hash) = credential.to_keyhash() {
                    self.required_vkey_witnesses.push(RequiredVKeyWitness {
                        key_hash,
                        location,
                        entity_index: index,
                    });
                }
            }
            csl::CredKind::Script => {
                if let Some(script_hash) = credential.to_scripthash() {
                    self.add_script_witness_requirement(
                        script_hash,
                        location,
                        index,
                        Some(csl::RedeemerTag::new_cert()),
                        None,
                    );
                }
            }
        }
    }

    fn collect_voting_proposal_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(voting_proposals) = tx.body().voting_proposals() {
            for i in 0..voting_proposals.len() {
                let proposal = voting_proposals.get(i);
                let gov_action = proposal.governance_action();
                let action_kind = gov_action.kind();
                if action_kind == csl::GovernanceActionKind::ParameterChangeAction {
                    let pp_change_action = gov_action.as_parameter_change_action().unwrap();
                    if let Some(script_hash) = pp_change_action.policy_hash() {
                        self.add_script_witness_requirement(
                            script_hash,
                            format!("transaction.body.voting_proposals.{}", i),
                            i as u32,
                            Some(csl::RedeemerTag::new_voting_proposal()),
                            None,
                        );
                    }
                }
            }
        }
    }

    fn collect_vote_witnesses(&mut self, tx: &csl::Transaction) {
        let votes = tx.body().voting_procedures();
        if let Some(votes) = votes {
            let voters = votes.get_voters();
            let voters_count = voters.len();
            for i in 0..voters_count {
                let voter = voters.get(i).unwrap();
                let voter_kind = voter.kind();
                match voter_kind {
                    csl::VoterKind::ConstitutionalCommitteeHotKeyHash => {
                        if let Some(voter_cred) = voter
                            .to_constitutional_committee_hot_credential()
                            .unwrap()
                            .to_keyhash()
                        {
                            self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                key_hash: voter_cred,
                                location: format!("transaction.body.voting_procedures.{}", i),
                                entity_index: i as u32,
                            });
                        }
                    }
                    csl::VoterKind::ConstitutionalCommitteeHotScriptHash => {
                        if let Some(voter_cred) = voter
                            .to_constitutional_committee_hot_credential()
                            .unwrap()
                            .to_scripthash()
                        {
                            self.add_script_witness_requirement(
                                voter_cred,
                                format!("transaction.body.voting_procedures.{}", i),
                                i as u32,
                                Some(csl::RedeemerTag::new_vote()),
                                None,
                            );
                        }
                    }
                    csl::VoterKind::DRepKeyHash => {
                        if let Some(voter_cred) = voter.to_drep_credential().unwrap().to_keyhash() {
                            self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                key_hash: voter_cred,
                                location: format!("transaction.body.voting_procedures.{}", i),
                                entity_index: i as u32,
                            });
                        }
                    }
                    csl::VoterKind::DRepScriptHash => {
                        if let Some(voter_cred) = voter
                            .to_drep_credential()
                            .map(|cred| cred.to_scripthash())
                            .flatten()
                        {
                            self.add_script_witness_requirement(
                                voter_cred,
                                format!("transaction.body.voting_procedures.{}", i),
                                i as u32,
                                Some(csl::RedeemerTag::new_vote()),
                                None,
                            );
                        }
                    }
                    csl::VoterKind::StakingPoolKeyHash => {
                        if let Some(voter_cred) = voter.to_stake_pool_key_hash() {
                            self.required_vkey_witnesses.push(RequiredVKeyWitness {
                                key_hash: voter_cred,
                                location: format!("transaction.body.voting_procedures.{}", i),
                                entity_index: i as u32,
                            });
                        }
                    }
                }
            }
        }
    }

    fn collect_mint_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(mint) = tx.body().mint() {
            let policy_ids = mint.keys();
            for i in 0..policy_ids.len() {
                let policy_id = policy_ids.get(i);
                let script_hash = csl::ScriptHash::from_bytes(policy_id.to_bytes()).unwrap();

                // Определяем тип скрипта по собранным витнесам
                self.add_script_witness_requirement(
                    script_hash,
                    format!("transaction.body.mint.{}", i),
                    i as u32,
                    Some(csl::RedeemerTag::new_mint()),
                    None,
                );
            }
        }
    }

    fn collect_required_signer_witnesses(&mut self, tx: &csl::Transaction) {
        if let Some(required_signers) = tx.body().required_signers() {
            for i in 0..required_signers.len() {
                let key_hash = required_signers.get(i);
                self.required_vkey_witnesses.push(RequiredVKeyWitness {
                    key_hash,
                    location: format!("transaction.body.required_signers.{}", i),
                    entity_index: i as u32,
                });
            }
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Проверяем VKey witnesses
        for required in &self.required_vkey_witnesses {
            let found = self.provided_vkey_witnesses.contains(&required.key_hash);

            if !found {
                errors.push(ValidationError::new(
                    Phase1Error::MissingVKeyWitnesses {
                        missing_key_hash: hex::encode(required.key_hash.to_bytes()),
                    },
                    required.location.clone(),
                ));
            }
        }

        // Проверяем Native Script witnesses
        for required in &self.required_native_script_witnesses {
            let found = self
                .native_script_sources
                .contains_key(&required.script_hash);

            if !found {
                errors.push(ValidationError::new(
                    Phase1Error::MissingScriptWitnesses {
                        missing_script_hash: hex::encode(required.script_hash.to_bytes()),
                    },
                    required.location.clone(),
                ));
            }
        }

        // Проверяем Plutus Script witnesses
        for required in &self.required_plutus_script_witnesses {
            let script_found = self
                .plutus_script_sources
                .contains_key(&required.script_hash);

            if !script_found {
                errors.push(ValidationError::new(
                    Phase1Error::MissingScriptWitnesses {
                        missing_script_hash: hex::encode(required.script_hash.to_bytes()),
                    },
                    required.location.clone(),
                ));
            }
        }

        // Проверяем Redeemer witnesses
        for required in &self.required_redeemer_witnesses {
            let redeemer_found = self
                .provided_redeemers
                .contains(&(required.tag.clone(), required.index));

            if !redeemer_found {
                errors.push(ValidationError::new(
                    Phase1Error::MissingRedeemer {
                        tag: format!("{:?}", required.tag),
                        index: required.index,
                    },
                    required.location.clone(),
                ));
            }
        }

        // Проверяем Datum witnesses
        for required in &self.required_datum_witnesses {
            let datum_found = self.datum_sources.contains_key(&required.datum_hash);

            if !datum_found {
                // TODO: добавить соответствующую ошибку для отсутствующих датумов
                // Пока используем MissingScriptWitnesses как заглушку
                errors.push(ValidationError::new(
                    Phase1Error::MissingScriptWitnesses {
                        missing_script_hash: hex::encode(required.datum_hash.to_bytes()),
                    },
                    required.location.clone(),
                ));
            }
        }

        // Проверяем Unknown Script witnesses
        for required in &self.required_unknown_script_witnesses {
            // Неизвестный тип скрипта - скрипт не был предоставлен
            errors.push(ValidationError::new(
                Phase1Error::MissingScriptWitnesses {
                    missing_script_hash: hex::encode(required.script_hash.to_bytes()),
                },
                required.location.clone(),
            ));
        }

        // Проверяем на лишние витнесы (опционально)
        let mut extraneous_scripts = HashSet::new();

        // Проверяем native scripts
        for (script_hash, _) in &self.native_script_sources {
            let required = self
                .required_native_script_witnesses
                .iter()
                .any(|req| &req.script_hash == script_hash);

            if !required {
                extraneous_scripts.insert(hex::encode(script_hash.to_bytes()));
            }
        }

        // Проверяем plutus scripts
        for (script_hash, _) in &self.plutus_script_sources {
            let required = self
                .required_plutus_script_witnesses
                .iter()
                .any(|req| &req.script_hash == script_hash);

            if !required {
                extraneous_scripts.insert(hex::encode(script_hash.to_bytes()));
            }
        }

        if !extraneous_scripts.is_empty() {
            errors.push(ValidationError::new(
                Phase1Error::ExtraneousScriptWitnesses { extraneous_scripts },
                "transaction.witness_set".to_string(),
            ));
        }

        ValidationResult::new(errors, vec![])
    }
}

fn get_native_script_key_hashes(native_script: &csl::NativeScript) -> HashSet<csl::Ed25519KeyHash> {
    let mut key_hashes = HashSet::new();
    get_native_script_key_hashes_internal(native_script, &mut key_hashes);
    key_hashes
}

fn get_native_script_key_hashes_internal(
    native_script: &csl::NativeScript,
    key_hashes: &mut HashSet<csl::Ed25519KeyHash>,
) {
    let script_kind = native_script.kind();
    match script_kind {
        csl::NativeScriptKind::ScriptPubkey => {
            if let Some(script_pubkey) = native_script.as_script_pubkey() {
                key_hashes.insert(script_pubkey.addr_keyhash());
            }
        }
        csl::NativeScriptKind::ScriptAll => {
            if let Some(script_all) = native_script.as_script_all() {
                for script in script_all.native_scripts() {
                    get_native_script_key_hashes_internal(&script, key_hashes);
                }
            }
        }
        csl::NativeScriptKind::ScriptAny => {
            if let Some(script_any) = native_script.as_script_any() {
                for script in script_any.native_scripts() {
                    get_native_script_key_hashes_internal(&script, key_hashes);
                }
            }
        }
        csl::NativeScriptKind::ScriptNOfK => {
            if let Some(script_n_of_k) = native_script.as_script_n_of_k() {
                for script in script_n_of_k.native_scripts() {
                    get_native_script_key_hashes_internal(&script, key_hashes);
                }
            }
        }
        csl::NativeScriptKind::TimelockStart => {}
        csl::NativeScriptKind::TimelockExpiry => {}
    }
}
