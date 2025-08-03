use super::errors::{Phase2Error, Phase2Warning};

pub fn get_error_hint(_error: &Phase2Error) -> Option<String> {
    return None;
}

pub fn get_warning_hint(_warning: &Phase2Warning) -> Option<String> {
    return None;
}