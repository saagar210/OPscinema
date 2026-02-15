use opscinema_types::{Step, StepModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepsCandidatesGeneratedPayload {
    pub schema_version: u32,
    pub steps: Vec<Step>,
}

pub fn initial_step_model(steps: Vec<Step>) -> StepModel {
    StepModel {
        schema_version: 1,
        steps,
    }
}
