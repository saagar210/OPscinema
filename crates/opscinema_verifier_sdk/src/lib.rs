use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierCapabilitySpec {
    pub verifier_id: String,
    pub allow_read_paths: Vec<String>,
    pub allow_commands: Vec<String>,
    pub timeout_secs: u32,
    pub allow_network_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifierExecutionResult {
    pub status: String,
    pub output: String,
    pub warnings: Vec<String>,
}
