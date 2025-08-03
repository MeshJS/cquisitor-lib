use crate::validators::{
    input_contexts::ValidationInputContext,
    phase_1::errors::{Phase1Error, ValidationPhase1Error},
    validation_result::ValidationResult,
};
use cardano_serialization_lib::{self as csl, BigNum};

pub struct OutputValidator {
    pub oversized_outputs: Vec<(usize, u32)>, // (index, value_size)
    pub outputs_below_min_ada: Vec<(usize, i128, i128)>, // (index, actual_amount, min_amount)
    pub max_value_size: u32,
}

impl OutputValidator {
    pub fn new(
        tx_body: &csl::TransactionBody,
        validation_input_context: &ValidationInputContext,
    ) -> Self {
        let outputs = tx_body.outputs();
        let max_value_size = validation_input_context.protocol_parameters.max_value_size;

        let mut oversized_outputs = Vec::new();
        let mut outputs_below_min_ada = Vec::new();

        // Check each output
        for i in 0..outputs.len() {
            let output = outputs.get(i);
            let output_index = i;

            // Check value size
            let value_size = output.amount().to_bytes().len() as u32;
            if value_size > max_value_size {
                oversized_outputs.push((output_index, value_size));
            }

            // Check minimum ADA
            let coins_per_byte = validation_input_context
                .protocol_parameters
                .ada_per_utxo_byte;
            let data_cost = csl::DataCost::new_coins_per_byte(&BigNum::from(coins_per_byte));

            if let Ok(min_ada) = csl::min_ada_for_output(&output, &data_cost) {
                let min_ada_amount = min_ada.to_str().parse::<i128>().unwrap_or(0);
                let actual_ada_amount =
                    output.amount().coin().to_str().parse::<i128>().unwrap_or(0);

                if actual_ada_amount < min_ada_amount {
                    outputs_below_min_ada.push((output_index, actual_ada_amount, min_ada_amount));
                }
            }
        }

        Self {
            oversized_outputs,
            outputs_below_min_ada,
            max_value_size,
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        for (index, value_size) in &self.oversized_outputs {
            // Check for oversized outputs
            if !self.oversized_outputs.is_empty() {
                errors.push(ValidationPhase1Error::new(
                    Phase1Error::OutputTooBigUTxO {
                        actual_size: *value_size,
                        max_size: self.max_value_size,
                    },
                    format!("transaction.body.outputs.{}", index),
                ));
            }
        }

        // Check for outputs below minimum ADA
        for (index, actual_amount, min_amount) in &self.outputs_below_min_ada {
            errors.push(ValidationPhase1Error::new(
                Phase1Error::OutputTooSmallUTxO {
                    output_amount: *actual_amount,
                    min_amount: *min_amount,
                },
                format!("transaction.body.outputs.{}", index),
            ));
        }

        ValidationResult::new_phase_1(errors, Vec::new())
    }
}
