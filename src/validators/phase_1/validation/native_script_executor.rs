use std::collections::HashSet;

use crate::js_error::JsError;
use cardano_serialization_lib as csl;

#[derive(Debug)]
pub struct NativeScriptExecutor<'a> {
    script: &'a csl::NativeScript,
    signatures: &'a HashSet<csl::Ed25519KeyHash>,
    slot: u64,
}

impl<'a> NativeScriptExecutor<'a> {
    pub fn new(
        script: &'a csl::NativeScript,
        signatures: &'a HashSet<csl::Ed25519KeyHash>,
        slot: u64,
    ) -> Self {
        Self {
            script,
            signatures,
            slot,
        }
    }

    pub fn execute(&self) -> Result<bool, JsError> {
        self.execute_internal(self.script)
    }

    fn execute_internal(&self, script: &csl::NativeScript) -> Result<bool, JsError> {
        match script.kind() {
            csl::NativeScriptKind::ScriptPubkey => {
                self.execute_pubkey_script(script.as_script_pubkey().unwrap())
            }
            csl::NativeScriptKind::ScriptAll => {
                self.execute_all_script(self.script.as_script_all().unwrap())
            }
            csl::NativeScriptKind::ScriptAny => {
                self.execute_any_script(self.script.as_script_any().unwrap())
            }
            csl::NativeScriptKind::ScriptNOfK => {
                self.execute_nofk_script(self.script.as_script_n_of_k().unwrap())
            }
            csl::NativeScriptKind::TimelockStart => {
                self.execute_invalid_before_script(self.script.as_timelock_start().unwrap())
            }
            csl::NativeScriptKind::TimelockExpiry => {
                self.execute_invalid_hereafter_script(self.script.as_timelock_expiry().unwrap())
            }
        }
    }

    fn execute_pubkey_script(&self, script_pubkey: csl::ScriptPubkey) -> Result<bool, JsError> {
        Ok(self.signatures.contains(&script_pubkey.addr_keyhash()))
    }

    fn execute_all_script(&self, script_all: csl::ScriptAll) -> Result<bool, JsError> {
        let native_scripts = script_all.native_scripts();
        for i in 0..native_scripts.len() {
            let script = native_scripts.get(i);
            if !self.execute_internal(&script)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn execute_any_script(&self, script_any: csl::ScriptAny) -> Result<bool, JsError> {
        let native_scripts = script_any.native_scripts();
        for i in 0..native_scripts.len() {
            let script = native_scripts.get(i);
            if self.execute_internal(&script)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn execute_nofk_script(&self, script_nofk: csl::ScriptNOfK) -> Result<bool, JsError> {
        let native_scripts = script_nofk.native_scripts();
        let mut valid_count = 0;
        for i in 0..native_scripts.len() {
            let script = native_scripts.get(i);
            if self.execute_internal(&script)? {
                valid_count += 1;
            }
        }
        Ok(valid_count >= script_nofk.n())
    }

    fn execute_invalid_before_script(
        &self,
        timelock_start: csl::TimelockStart,
    ) -> Result<bool, JsError> {
        let slot = timelock_start
            .slot_bignum()
            .to_str()
            .parse::<u64>()
            .map_err(|_| JsError::new("Failed to parse slot as u64"))?;
        Ok(self.slot > slot)
    }

    fn execute_invalid_hereafter_script(
        &self,
        timelock_expiry: csl::TimelockExpiry,
    ) -> Result<bool, JsError> {
        let slot = timelock_expiry
            .slot_bignum()
            .to_str()
            .parse::<u64>()
            .map_err(|_| JsError::new("Failed to parse slot as u64"))?;
        Ok(self.slot <= slot)
    }
}
