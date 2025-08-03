use crate::{
    common::TxInput,
    js_error::JsError,
    validators::{
        helpers::normalize_script_ref,
        input_contexts::{UtxoInputContext, ValidationInputContext},
        phase_1::errors::{
            Phase1Error, Phase1Warning, ValidationPhase1Error, ValidationPhase1Warning,
        },
        validation_result::ValidationResult,
    },
};
use cardano_serialization_lib as csl;
use std::collections::HashSet;

const MAX_REFERENCE_SCRIPTS_SIZE: u64 = 200 * 1024;

pub struct TransactionLimitsValidator<'a> {
    pub actual_tx_size: u64,
    pub max_tx_size: u64,
    pub actual_execution_units: (u64, u64), // (memory, steps)
    pub max_execution_units: (u64, u64),
    pub actual_ref_scripts_size: u64,
    pub max_ref_scripts_size: u64,
    pub current_slot: u64,
    pub validity_interval: (Option<u64>, Option<u64>), // (start, end)
    pub inputs_sorted: bool,
    pub inputs_count: usize,
    pub ref_inputs: Vec<TxInput>,
    pub collateral_inputs: Vec<TxInput>,
    pub inputs: Vec<TxInput>,
    pub validation_input_context: &'a ValidationInputContext,
}

impl<'a> TransactionLimitsValidator<'a> {
    pub fn new(
        tx_size: usize,
        tx_body: &csl::TransactionBody,
        tx_witness_set: &csl::TransactionWitnessSet,
        validation_input_context: &'a ValidationInputContext,
    ) -> Result<Self, JsError> {
        let actual_tx_size = tx_size as u64;
        let max_tx_size = validation_input_context
            .protocol_parameters
            .max_transaction_size as u64;

        let redeemers = tx_witness_set.redeemers().unwrap_or(csl::Redeemers::new());
        let total_execution_units = redeemers.total_ex_units().unwrap_or(csl::ExUnits::new(
            &csl::BigNum::from(0u64),
            &csl::BigNum::from(0u64),
        ));

        let actual_execution_units = (
            total_execution_units
                .mem()
                .to_str()
                .parse::<u64>()
                .unwrap_or(0),
            total_execution_units
                .steps()
                .to_str()
                .parse::<u64>()
                .unwrap_or(0),
        );

        let max_execution_units = (
            validation_input_context
                .protocol_parameters
                .max_tx_execution_units
                .mem,
            validation_input_context
                .protocol_parameters
                .max_tx_execution_units
                .steps,
        );

        let utxos = collect_utxos(tx_body, validation_input_context);
        let actual_ref_scripts_size = calculate_total_reference_scripts_size(&utxos)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let current_slot = validation_input_context.slot;
        let validity_interval = get_validity_interval(tx_body);

        let inputs_sorted = check_inputs_sorted(tx_body);
        let inputs_count = tx_body.inputs().len();

        let ref_inputs = tx_body
            .reference_inputs()
            .unwrap_or(csl::TransactionInputs::new());
        let collateral_inputs = tx_body
            .collateral()
            .unwrap_or(csl::TransactionInputs::new());
        let inputs = tx_body.inputs();

        let ref_inputs = ref_inputs
            .into_iter()
            .map(|input| TxInput {
                tx_hash: input.transaction_id().to_hex(),
                output_index: input.index(),
            })
            .collect();
        let collateral_inputs = collateral_inputs
            .into_iter()
            .map(|input| TxInput {
                tx_hash: input.transaction_id().to_hex(),
                output_index: input.index(),
            })
            .collect();
        let inputs = inputs
            .into_iter()
            .map(|input| TxInput {
                tx_hash: input.transaction_id().to_hex(),
                output_index: input.index(),
            })
            .collect();

        Ok(Self {
            actual_tx_size,
            max_tx_size,
            actual_execution_units,
            max_execution_units,
            actual_ref_scripts_size,
            max_ref_scripts_size: MAX_REFERENCE_SCRIPTS_SIZE,
            current_slot,
            validity_interval,
            inputs_sorted,
            inputs_count,
            ref_inputs,
            collateral_inputs,
            inputs,
            validation_input_context,
        })
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        if self.inputs_count == 0 {
            errors.push(ValidationPhase1Error::new(
                Phase1Error::InputSetEmptyUTxO,
                "transaction.body.inputs".to_string(),
            ));
        }

        if self.actual_tx_size > self.max_tx_size {
            errors.push(ValidationPhase1Error::new(
                Phase1Error::MaxTxSizeUTxO {
                    actual_size: self.actual_tx_size,
                    max_size: self.max_tx_size,
                },
                "transaction".to_string(),
            ));
        }

        if self.actual_execution_units.0 > self.max_execution_units.0
            || self.actual_execution_units.1 > self.max_execution_units.1
        {
            errors.push(ValidationPhase1Error::new(
                Phase1Error::ExUnitsTooBigUTxO {
                    actual_memory_units: self.actual_execution_units.0,
                    actual_steps_units: self.actual_execution_units.1,
                    max_memory_units: self.max_execution_units.0,
                    max_steps_units: self.max_execution_units.1,
                },
                "transaction.witness_set.redeemers".to_string(),
            ));
        }

        if self.actual_ref_scripts_size > self.max_ref_scripts_size {
            errors.push(ValidationPhase1Error::new(
                Phase1Error::RefScriptsSizeTooBig {
                    actual_size: self.actual_ref_scripts_size,
                    max_size: self.max_ref_scripts_size,
                },
                "transaction.body.inputs".to_string(),
            ));
        }

        let is_outside_validity = match self.validity_interval {
            (Some(start), Some(end)) => self.current_slot < start || self.current_slot > end,
            (Some(start), None) => self.current_slot < start,
            (None, Some(end)) => self.current_slot > end,
            (None, None) => false,
        };

        if is_outside_validity {
            let (interval_start, interval_end) = match self.validity_interval {
                (Some(start), Some(end)) => (start, end),
                (Some(start), None) => (start, u64::MAX),
                (None, Some(end)) => (0, end),
                (None, None) => (0, u64::MAX),
            };

            errors.push(ValidationPhase1Error::new(
                Phase1Error::OutsideValidityIntervalUTxO {
                    current_slot: self.current_slot,
                    interval_start,
                    interval_end,
                },
                "transaction.body".to_string(),
            ));
        }

        for input in self.ref_inputs.iter() {
            let utxo = self
                .validation_input_context
                .find_utxo(input.tx_hash.clone(), input.output_index);
            if utxo.map(|utxo| utxo.is_spent).unwrap_or(true) {
                errors.push(ValidationPhase1Error::new(
                    Phase1Error::BadInputsUTxO {
                        invalid_input: input.clone(),
                    },
                    "transaction.body.reference_inputs".to_string(),
                ));
            }
        }

        for input in self.collateral_inputs.iter() {
            let utxo = self
                .validation_input_context
                .find_utxo(input.tx_hash.clone(), input.output_index);
            if utxo.map(|utxo| utxo.is_spent).unwrap_or(true) {
                errors.push(ValidationPhase1Error::new(
                    Phase1Error::BadInputsUTxO {
                        invalid_input: input.clone(),
                    },
                    "transaction.body.collateral".to_string(),
                ));
            }
        }

        for input in self.inputs.iter() {
            let utxo = self
                .validation_input_context
                .find_utxo(input.tx_hash.clone(), input.output_index);
            if utxo.map(|utxo| utxo.is_spent).unwrap_or(true) {
                errors.push(ValidationPhase1Error::new(
                    Phase1Error::BadInputsUTxO {
                        invalid_input: input.clone(),
                    },
                    "transaction.body.inputs".to_string(),
                ));
            }
        }

        let mut warnings = Vec::new();

        if !self.inputs_sorted {
            warnings.push(ValidationPhase1Warning::new(
                Phase1Warning::InputsAreNotSorted,
                "transaction.body.inputs".to_string(),
            ));
        }

        for (i, ref_input) in self.ref_inputs.iter().enumerate() {
            if self.inputs.contains(ref_input) {
                errors.push(ValidationPhase1Error::new(
                    Phase1Error::ReferenceInputOverlapsWithInput {
                        input: ref_input.clone(),
                    },
                    format!("transaction.body.reference_inputs.{}", i),
                ));
            }
        }

        ValidationResult::new_phase_1(errors, warnings)
    }
}

fn collect_utxos<'a>(
    tx_body: &csl::TransactionBody,
    validation_input_context: &'a ValidationInputContext,
) -> HashSet<&'a UtxoInputContext> {
    let inputs = tx_body.inputs();
    let mut input_utxos: HashSet<&'a UtxoInputContext> = inputs
        .into_iter()
        .map(|input| validation_input_context.find_utxo(input.to_hex(), input.index()))
        .filter_map(|utxo| utxo)
        .collect();

    let ref_inputs = tx_body.reference_inputs();
    let ref_utxos: Vec<&'a UtxoInputContext> = ref_inputs
        .unwrap_or(csl::TransactionInputs::new())
        .into_iter()
        .map(|input| validation_input_context.find_utxo(input.to_hex(), input.index()))
        .filter_map(|utxo| utxo)
        .collect();

    input_utxos.extend(ref_utxos);
    input_utxos
}

fn calculate_total_reference_scripts_size(
    utxos: &HashSet<&UtxoInputContext>,
) -> Result<u64, String> {
    let mut total_size = 0u64;
    for utxo in utxos.iter() {
        if utxo.utxo.output.script_ref.is_some() {
            if let Some(script_ref) = &utxo.utxo.output.script_ref {
                let normalized_script_ref = normalize_script_ref(script_ref)?;
                let script_ref_size = normalized_script_ref.to_unwrapped_bytes().len() as u64;
                total_size += script_ref_size;
            }
        }
    }
    Ok(total_size)
}

fn get_validity_interval(tx_body: &csl::TransactionBody) -> (Option<u64>, Option<u64>) {
    let ttl = tx_body
        .ttl_bignum()
        .map(|slot| slot.to_str().parse::<u64>().unwrap_or(0));
    let validity_start_interval = tx_body
        .validity_start_interval_bignum()
        .map(|slot| slot.to_str().parse::<u64>().unwrap_or(0));

    (validity_start_interval, ttl)
}

fn check_inputs_sorted(tx_body: &csl::TransactionBody) -> bool {
    let inputs = tx_body.inputs();
    if inputs.len() <= 1 {
        return true;
    }

    for i in 1..inputs.len() {
        let prev_input = inputs.get(i - 1);
        let curr_input = inputs.get(i);

        let prev_tx_id = prev_input.transaction_id();
        let curr_tx_id = curr_input.transaction_id();

        if prev_tx_id > curr_tx_id {
            return false;
        }

        if prev_tx_id == curr_tx_id && prev_input.index() > curr_input.index() {
            return false;
        }
    }

    true
}
