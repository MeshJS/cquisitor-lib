use crate::js_error::JsError;
use crate::validators::input_contexts::ValidationInputContext;
use crate::validators::phase_2::data_mapper::{to_pallas_cost_modesl, to_pallas_utxos};
use crate::validators::phase_2::errors::{Phase2Error, Phase2Warning, ValidationPhase2Error, ValidationPhase2Warning};
use crate::validators::phase_2::eval_redeemer::{eval_redeemer, slot_config_network};
use crate::validators::validation_result::{EvalRedeemerResult, ValidationResult};
use pallas_primitives::conway::{MintedTx, Redeemer};
use pallas_traverse::{Era, MultiEraTx};
use std::collections::HashSet;
use uplc::machine::cost_model::ExBudget;
use uplc::tx::{ResolvedInput, SlotConfig};
use uplc::tx::{iter_redeemers, DataLookupTable};
use crate::validators::validation_result::RedeemerTag as ValidatorRedeemerTag;

pub fn phase_2_validation(
    tx_hex: &str,
    validation_input_context: &ValidationInputContext,
) -> Result<ValidationResult, JsError> {
    let tx_bytes = hex::decode(tx_hex).map_err(|e| JsError::new(&e.to_string()))?;
    let mtx = MultiEraTx::decode_for_era(Era::Conway, &tx_bytes)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let tx = match mtx {
        MultiEraTx::Conway(tx) => tx.into_owned(),
        _ => return Err(JsError::new("Invalid transaction type")),
    };

    // Gather all input identifiers from the transaction.
    let request_utxos = collect_inputs(&tx);

    // Deserialize UTxO data and convert to the internal representation.
    let utxos = to_pallas_utxos(&validation_input_context.utxo_set)?;

    check_missed_utxos(&request_utxos, &utxos)?;

    let slot_config = slot_config_network(&validation_input_context.network_type);

    let cost_models = to_pallas_cost_modesl(&validation_input_context.protocol_parameters.cost_models);
    let exec_result = eval_all_redeemers(&tx, &utxos, Some(&cost_models), &slot_config);

    Ok(exec_result)
}

/// Collects all input identifiers (including reference inputs and collateral)
/// from the given transaction.
fn collect_inputs(tx: &MintedTx) -> Vec<String> {
    let mut inputs = tx
        .transaction_body
        .inputs
        .iter()
        .map(input_to_request_format)
        .collect::<Vec<_>>();

    if let Some(ref_inputs) = &tx.transaction_body.reference_inputs {
        inputs.extend(ref_inputs.iter().map(input_to_request_format));
    }
    if let Some(collaterals) = &tx.transaction_body.collateral {
        inputs.extend(collaterals.iter().map(input_to_request_format));
    }
    inputs
}

/// Checks whether the UTXOs requested in the transaction are present in the API response.
fn check_missed_utxos(request_utxos: &[String], utxos: &[ResolvedInput]) -> Result<(), JsError> {
    let utxo_keys: HashSet<String> = utxos
        .iter()
        .map(|u| format!("{}#{}", hex::encode(u.input.transaction_id), u.input.index))
        .collect();
    let missed_utxos: Vec<String> = request_utxos
        .iter()
        .filter(|u| !utxo_keys.contains(*u))
        .cloned()
        .collect();

    if !missed_utxos.is_empty() {
        return Err(JsError::new(&format!(
            "Can't get these UTXOs from API, check the network type: {}",
            missed_utxos.join(", ")
        )));
    }
    Ok(())
}

/// Formats a transaction input into the expected "txhash#index" string format.
fn input_to_request_format(input: &pallas_primitives::TransactionInput) -> String {
    format!("{}#{}", hex::encode(input.transaction_id), input.index)
}

/// Evaluates all redeemers in the transaction.
fn eval_all_redeemers(
    tx: &MintedTx,
    utxos: &[ResolvedInput],
    cost_mdls: Option<&pallas_primitives::conway::CostModels>,
    slot_config: &SlotConfig,
) -> ValidationResult {
    let mut phase_2_errors = vec![];
    let mut phase_2_warnings = vec![];
    let mut eval_results = vec![];

    let lookup_table = DataLookupTable::from_transaction(tx, utxos);

    if let Some(redeemers) = tx.transaction_witness_set.redeemer.as_ref() {
        let remaining_budget = ExBudget {
            mem: i64::MAX,
            cpu: i64::MAX,
        };
        for (redeemer_index, (r_key, r_value, r_ex_units)) in iter_redeemers(redeemers).enumerate() {
            let redeemer = Redeemer {
                tag: r_key.tag,
                index: r_key.index,
                data: r_value.clone(),
                ex_units: r_ex_units,
            };
            let (eval_redeemer_result, error) = eval_redeemer(
                tx,
                utxos,
                slot_config,
                &redeemer,
                &lookup_table,
                cost_mdls,
                &remaining_budget,
            );

            if let Some(error) = error {
                phase_2_errors.push(ValidationPhase2Error::new_with_locations(error, &redeemer_to_tx_locations(&eval_redeemer_result, redeemer_index)));
            }

            eval_results.push(eval_redeemer_result.clone());

            let estimated_budget = &eval_redeemer_result.calculated_ex_units;
            let redeemer_budget = &eval_redeemer_result.provided_ex_units;

            if estimated_budget.mem > redeemer_budget.mem || estimated_budget.steps > redeemer_budget.steps {
                phase_2_errors.push(ValidationPhase2Error::new_with_locations(Phase2Error::NoEnoughBudget {
                    expected_budget: estimated_budget.clone(),
                    actual_budget: redeemer_budget.clone(),
                }, &redeemer_to_tx_locations(&eval_redeemer_result, redeemer_index)));
            } else if estimated_budget.mem < redeemer_budget.mem || estimated_budget.steps < redeemer_budget.steps {
                phase_2_warnings.push(ValidationPhase2Warning::new_with_locations(Phase2Warning::BudgetIsBiggerThanExpected {
                    expected_budget: estimated_budget.clone(),
                    actual_budget: redeemer_budget.clone(),
                }, &redeemer_to_tx_locations(&eval_redeemer_result, redeemer_index)));
            }
        }
    }
    ValidationResult::new_phase_2(phase_2_errors, phase_2_warnings, eval_results)
}

fn redeemer_to_tx_locations(redeemer: &EvalRedeemerResult, redeemer_index: usize) -> Vec<String> {
    let mut locations = vec![];
    let body_location = redeemer_tag_to_tx_location(&redeemer.tag, redeemer.index);
    let redeemer_location = format!("transaction.witness_set.redeemers.{}", redeemer_index);
    locations.push(body_location);
    locations.push(redeemer_location);
    locations
}

fn redeemer_tag_to_tx_location(redeemer_tag: &ValidatorRedeemerTag, redeemer_index: u64) -> String {
    match redeemer_tag {
        ValidatorRedeemerTag::Mint => format!("transaction.body.mint.{}", redeemer_index),
        ValidatorRedeemerTag::Spend => format!("transaction.body.inputs.{}", redeemer_index),
        ValidatorRedeemerTag::Cert => format!("transaction.body.certs.{}", redeemer_index),
        ValidatorRedeemerTag::Propose => format!("transaction.body.voting_proposals.{}", redeemer_index),
        ValidatorRedeemerTag::Vote => format!("transaction.body.voting_procedures.{}", redeemer_index),
        ValidatorRedeemerTag::Reward => format!("transaction.body.withdrawals.{}", redeemer_index),
    }
}