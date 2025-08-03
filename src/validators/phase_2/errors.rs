use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::{common::ExUnits, validators::phase_2::hints::{get_error_hint, get_warning_hint}};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ValidationPhase2Error {
    pub error: Phase2Error,
    pub error_message: String,
    pub locations: Vec<String>,
    pub hint: Option<String>,
}

impl ValidationPhase2Error {
    pub fn new(error: Phase2Error, location: String) -> Self {
        let error_message = error.to_string();
        let hint = get_error_hint(&error);
        Self {
            error,
            error_message,
            locations: vec![location],
            hint,
        }
    }

    pub fn new_with_locations(error: Phase2Error, locations: &[String]) -> Self {
        let error_message = error.to_string();
        let hint = get_error_hint(&error);
        Self {
            error,
            error_message,
            locations: locations.to_vec(),
            hint,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ValidationPhase2Warning {
    pub warning: Phase2Warning,
    pub locations: Vec<String>,
    pub hint: Option<String>,
}

impl ValidationPhase2Warning {
    pub fn new(warning: Phase2Warning, location: String) -> Self {
        let hint = get_warning_hint(&warning);
        Self {
            warning,
            locations: vec![location],
            hint,
        }
    }

    pub fn new_with_locations(warning: Phase2Warning, locations: &[String]) -> Self {
        let hint = get_warning_hint(&warning);
        Self {
            warning,
            locations: locations.to_vec(),
            hint,
        }
    }
}

/// Phase 1 validation errors
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub enum Phase2Error {
    NoEnoughBudget {
        expected_budget: ExUnits,
        actual_budget: ExUnits,
    },
    InvalidRedeemerIndex {
        tag: String,
        index: u64,
    },
    MachineError { error: String },
    NativeScriptIsReferencedByRedeemer,
    CostModelNotFound { language: String },
    ScriptDecodeError { error: String },
    BuildTxContextError { error: String },
    MissingScriptForRedeemer { error: String },
}

impl Phase2Error {
    pub fn to_string(&self) -> String {
        match self {
            Phase2Error::NoEnoughBudget { expected_budget, actual_budget } => {
                format!(
                    "Not enough budget available. Expected: {:?}, Actual: {:?}",
                    expected_budget, actual_budget
                )
            }
            Phase2Error::InvalidRedeemerIndex { tag, index } => {
                format!("Invalid redeemer index for tag '{}': {}", tag, index)
            }
            Phase2Error::MachineError { error } => {
                format!("Plutus machine error: {}", error)
            }
            Phase2Error::NativeScriptIsReferencedByRedeemer => {
                "Native script cannot be referenced by redeemer".to_string()
            }
            Phase2Error::CostModelNotFound { language } => {
                format!("Cost model not found for language: {}", language)
            }
            Phase2Error::ScriptDecodeError { error } => {
                format!("Failed to decode script: {}", error)
            }
            Phase2Error::BuildTxContextError { error } => {
                format!("Failed to build transaction context: {}", error)
            }
            Phase2Error::MissingScriptForRedeemer { error } => {
                format!("Missing script for redeemer: {}", error)
            }
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub enum Phase2Warning {
    BudgetIsBiggerThanExpected {
        expected_budget: ExUnits,
        actual_budget: ExUnits,
    },
}

impl Phase2Warning {
    pub fn to_string(&self) -> String {
        match self {
            Phase2Warning::BudgetIsBiggerThanExpected { expected_budget, actual_budget } => {
                format!(
                    "Budget is bigger than expected. Expected: {:?}, Actual: {:?}",
                    expected_budget, actual_budget
                )
            }
        }
    }
}