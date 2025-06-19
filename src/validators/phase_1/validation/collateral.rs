use crate::validators::phase_1::{
    errors::{Phase1Error, Phase1Warning, ValidationError, ValidationResult, ValidationWarning},
    helpers::{csl_tx_input_to_string, string_to_csl_address},
    value::Value,
    ValidationInputContext,
};
use cardano_serialization_lib::{self as csl, BigNum};

#[derive(Debug, Clone, PartialEq, Eq)]
enum InvalidInputType {
    PaymentCredentialIsScript,
    AddressIsReward,
    HasNonAdaAssets,
}

pub struct CollateralValidator {
    pub invalid_inputs: Vec<(csl::TransactionInput, u32, InvalidInputType)>,
    pub total_input: Value,
    pub collateral_return: Option<Value>,
    pub total_collateral: Option<i128>,
    pub actual_collateral: Value,
    pub estimated_minimal_collateral: i128,
    pub actual_number_of_inputs: u32,
    pub max_number_of_inputs: u32,
    pub need_collateral: bool,
    pub min_ada_for_collateral_return: Option<i128>,
}

impl CollateralValidator {
    pub fn new(tx: &csl::Transaction, validation_input_context: &ValidationInputContext) -> Self {
        let total_input = calculate_total_input(tx, validation_input_context);
        let collateral_return = calculate_total_output(tx);
        let total_collateral = get_total_collateral(tx);
        let actual_collateral = if let Some(collateral_return) = &collateral_return {
            &total_input - &collateral_return
        } else {
            total_input.clone()
        };
        let estimated_minimal_collateral =
            calculate_estimated_minimal_collateral(tx, validation_input_context);
        let mut invalid_inputs = find_script_payment_inputs(tx, validation_input_context);
        invalid_inputs.extend(find_reward_address_inputs(tx, validation_input_context));
        invalid_inputs.extend(find_non_ada_inputs(tx, validation_input_context));

        let actual_number_of_inputs = tx.body().inputs().len() as u32;
        let max_number_of_inputs = validation_input_context
            .protocol_parameters
            .max_collateral_inputs;
        let need_collateral = is_need_collateral(tx);
        let min_ada_for_collateral_return = calculate_min_ada_for_collateral_return(tx, validation_input_context);
        Self {
            invalid_inputs,
            total_input,
            collateral_return,
            total_collateral,
            actual_collateral,
            estimated_minimal_collateral,
            actual_number_of_inputs,
            max_number_of_inputs,
            need_collateral,
            min_ada_for_collateral_return,
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if self.actual_number_of_inputs > self.max_number_of_inputs {
            errors.push(ValidationError::new(
                Phase1Error::TooManyCollateralInputs {
                    actual_count: self.actual_number_of_inputs,
                    max_count: self.max_number_of_inputs,
                },
                "transaction.body.collateral".to_string(),
            ));
        }

        if !self.need_collateral
            && (self.actual_number_of_inputs > 0 || self.total_collateral.is_some())
        {
            warnings.push(ValidationWarning::new(
                Phase1Warning::CollateralIsUnnecessary,
                "transaction.body.collateral".to_string(),
            ));
        }

        if let Some(total_collateral) = self.total_collateral {
            if total_collateral < self.estimated_minimal_collateral {
                errors.push(ValidationError::new(
                    Phase1Error::InsufficientCollateral {
                        total_collateral,
                        required_collateral: self.estimated_minimal_collateral,
                    },
                    "transaction.body.total_collateral".to_string(),
                ));
            }

            if self.actual_collateral.coins != total_collateral {
                errors.push(ValidationError::new(
                    Phase1Error::IncorrectTotalCollateralField {
                        declared_total: total_collateral,
                        actual_sum: self.actual_collateral.coins,
                    },
                    "transaction.body.total_collateral".to_string(),
                ));
            }
        } else {
            if self.total_input.coins < self.estimated_minimal_collateral {
                errors.push(ValidationError::new(
                    Phase1Error::InsufficientCollateral {
                        total_collateral: self.total_input.coins,
                        required_collateral: self.estimated_minimal_collateral,
                    },
                    "transaction.body.collateral".to_string(),
                ));
            }
            if self.collateral_return.is_some() {
                warnings.push(ValidationWarning::new(
                    Phase1Warning::TotalCollateralIsNotDeclared,
                    "transaction.body.total_collateral".to_string(),
                ));
            }
        }

        if self.collateral_return.is_some() {
            if self.actual_collateral.has_assets() {
                errors.push(ValidationError::new(
                    Phase1Error::CalculatedCollateralContainsNonAdaAssets,
                    "transaction.body.collateral".to_string(),
                ));
            }
        } else {
            for input in self.invalid_inputs.iter() {
                if input.2 == InvalidInputType::HasNonAdaAssets {
                    errors.push(ValidationError::new(
                        Phase1Error::CollateralInputContainsNonAdaAssets {
                            collateral_input: csl_tx_input_to_string(&input.0),
                        },
                        format!("transaction.body.collateral.{}", input.1),
                    ));
                }
            }
        }

        if let Some(min_ada_for_collateral_return) = self.min_ada_for_collateral_return {
            if self.actual_collateral.coins < min_ada_for_collateral_return {
                errors.push(ValidationError::new(
                    Phase1Error::CollateralReturnTooSmall {
                        output_amount: self.actual_collateral.coins,
                        min_amount: min_ada_for_collateral_return,
                    },
                    "transaction.body.collateral_return".to_string(),
                ));
            }
        }

        for invalid_input in self.invalid_inputs.iter() {
            if invalid_input.2 == InvalidInputType::PaymentCredentialIsScript {
                errors.push(ValidationError::new(
                    Phase1Error::CollateralIsLockedByScript {
                        invalid_collateral: csl_tx_input_to_string(&invalid_input.0),
                    },
                    format!("transaction.body.collateral.{}", invalid_input.1),
                ));
            } else if invalid_input.2 == InvalidInputType::AddressIsReward {
                warnings.push(ValidationWarning::new(
                    Phase1Warning::CollateralInputUsesRewardAddress {
                        invalid_collateral: csl_tx_input_to_string(&invalid_input.0),
                    },
                    format!("transaction.body.collateral.{}", invalid_input.1),
                ));
            }
        }
        ValidationResult::new(errors, warnings)
    }
}

fn calculate_total_input(
    tx: &csl::Transaction,
    validation_input_context: &ValidationInputContext,
) -> Value {
    tx.body()
        .collateral()
        .unwrap_or(csl::TransactionInputs::new())
        .into_iter()
        .map(|input| {
            let utxo =
                validation_input_context.find_utxo(input.transaction_id().to_hex(), input.index());
            if let Some(utxo) = utxo {
                Value::new_from_common_assets(&utxo.utxo.output.amount)
            } else {
                Value::new_from_coins(0)
            }
        })
        .fold(Value::new_from_coins(0), |acc, value| acc + value)
}

fn calculate_total_output(tx: &csl::Transaction) -> Option<Value> {
    if let Some(collateral_return) = tx.body().collateral_return() {
        Some(Value::new_from_csl_value(&collateral_return.amount()))
    } else {
        None
    }
}

fn get_total_collateral(tx: &csl::Transaction) -> Option<i128> {
    if let Some(total_collateral) = tx.body().total_collateral() {
        Some(total_collateral.to_str().parse::<i128>().unwrap_or(0))
    } else {
        None
    }
}

fn calculate_estimated_minimal_collateral(
    tx: &csl::Transaction,
    validation_input_context: &ValidationInputContext,
) -> i128 {
    let tx_fee: i128 = tx.body().fee().to_str().parse::<i128>().unwrap_or(0);
    let collateral_percentage: i128 = validation_input_context
        .protocol_parameters
        .collateral_percentage
        .into();
    let collateral_amount: i128 = tx_fee * collateral_percentage / 100;
    collateral_amount
}

fn is_need_collateral(tx: &csl::Transaction) -> bool {
    let redeemers_count = tx
        .witness_set()
        .redeemers()
        .map_or(0, |redeemers| redeemers.len());
    redeemers_count > 0
}

fn find_script_payment_inputs(
    tx: &csl::Transaction,
    validation_input_context: &ValidationInputContext,
) -> Vec<(csl::TransactionInput, u32, InvalidInputType)> {
    tx.body()
        .collateral()
        .unwrap_or(csl::TransactionInputs::new())
        .into_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let utxo =
                validation_input_context.find_utxo(input.transaction_id().to_hex(), input.index());

            if let Some(utxo) = utxo {
                if let Ok(csl_address) = string_to_csl_address(&utxo.utxo.output.address) {
                    if csl_address.kind() != csl::AddressKind::Reward {
                        if let Some(payment_cred) = csl_address.payment_cred() {
                            if payment_cred.kind() == csl::CredKind::Script {
                                return Some((
                                    input.clone(),
                                    index as u32,
                                    InvalidInputType::PaymentCredentialIsScript,
                                ));
                            }
                        }
                    }
                }
            }

            None
        })
        .collect()
}

fn find_reward_address_inputs(
    tx: &csl::Transaction,
    validation_input_context: &ValidationInputContext,
) -> Vec<(csl::TransactionInput, u32, InvalidInputType)> {
    tx.body()
        .collateral()
        .unwrap_or(csl::TransactionInputs::new())
        .into_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let utxo =
                validation_input_context.find_utxo(input.transaction_id().to_hex(), input.index());

            if let Some(utxo) = utxo {
                if let Ok(csl_address) = string_to_csl_address(&utxo.utxo.output.address) {
                    if csl_address.kind() == csl::AddressKind::Reward {
                        return Some((
                            input.clone(),
                            index as u32,
                            InvalidInputType::AddressIsReward,
                        ));
                    }
                }
            }

            None
        })
        .collect()
}

fn find_non_ada_inputs(
    tx: &csl::Transaction,
    validation_input_context: &ValidationInputContext,
) -> Vec<(csl::TransactionInput, u32, InvalidInputType)> {
    tx.body()
        .collateral()
        .unwrap_or(csl::TransactionInputs::new())
        .into_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let utxo =
                validation_input_context.find_utxo(input.transaction_id().to_hex(), input.index());

            if let Some(utxo) = utxo {
                if utxo.utxo.output.has_non_ada_assets() {
                    return Some((
                        input.clone(),
                        index as u32,
                        InvalidInputType::HasNonAdaAssets,
                    ));
                }
            }

            None
        })
        .collect()
}

fn calculate_min_ada_for_collateral_return(tx: &csl::Transaction, validation_input_context: &ValidationInputContext) -> Option<i128> {
    if let Some(collateral_return) = tx.body().collateral_return() {
        let coins_per_byte = validation_input_context.protocol_parameters.ada_per_utxo_byte;
        let data_cost = csl::DataCost::new_coins_per_byte(&BigNum::from(coins_per_byte));
        let min_ada = csl::min_ada_for_output(&collateral_return, &data_cost);
        match min_ada {
            Ok(min_ada) => Some(min_ada.to_str().parse::<i128>().unwrap_or(0)),
            Err(_) => None,
        }
    } else {
        None
    }
}
