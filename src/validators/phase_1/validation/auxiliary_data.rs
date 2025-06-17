use cardano_serialization_lib as csl;
use crate::validators::phase_1::errors::{Phase1Error, ValidationError, ValidationResult};

pub struct AuxiliaryDataValidationContext {
    pub auxiliary_data: Option<csl::AuxiliaryData>,
    pub expected_auxiliary_data_hash: Option<csl::AuxiliaryDataHash>,
    pub actual_auxiliary_data_hash: Option<csl::AuxiliaryDataHash>,
}


impl AuxiliaryDataValidationContext {
    pub fn new(tx: &csl::Transaction) -> Self {
        let auxiliary_data: Option<csl::AuxiliaryData> =  tx.auxiliary_data();
        let actual_auxiliary_data_hash = tx.body().auxiliary_data_hash();
        let expected_auxiliary_data_hash = if let Some(auxiliary_data) = &auxiliary_data {
            Some(csl::hash_auxiliary_data(&auxiliary_data))
        } else {
            None
        };

        Self {
            auxiliary_data,
            expected_auxiliary_data_hash,
            actual_auxiliary_data_hash,
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        if let Some(_) = &self.auxiliary_data {
            if let Some(expected_hash) = &self.expected_auxiliary_data_hash {
                match self.actual_auxiliary_data_hash.as_ref() {
                    Some(actual_hash) => {
                        if actual_hash != expected_hash {
                            errors.push(
                                ValidationError::new(
                                    Phase1Error::AuxiliaryDataHashMismatch {
                                        expected_hash: expected_hash.to_hex(),
                                        actual_hash: self.actual_auxiliary_data_hash.as_ref().map(|h| h.to_hex()),
                                    },
                                    "transaction.body.auxiliary_data_hash".to_string(),
                                )
                            );
                        }
                    }
                    None => {
                        errors.push(
                            ValidationError::new(
                                Phase1Error::AuxiliaryDataHashMissing,
                                "transaction.body.auxiliary_data_hash".to_string(),
                            )
                        );
                    }
                };
            } else {
                if let Some(_) = &self.actual_auxiliary_data_hash {
                    errors.push(
                        ValidationError::new(
                            Phase1Error::AuxiliaryDataHashPresentButNotExpected,
                            "transaction.body.auxiliary_data_hash".to_string(),
                        )
                    );
                }
            }
        }
        ValidationResult::new(errors, Vec::new())
    }
}