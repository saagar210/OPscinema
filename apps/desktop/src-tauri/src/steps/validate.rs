use crate::evidence::coverage;
use opscinema_types::{StepModel, StepsValidateExportResponse};

pub fn validate_for_export(model: &StepModel) -> StepsValidateExportResponse {
    let mut errors = Vec::new();
    if model.schema_version != 1 {
        errors.push("unsupported schema_version".to_string());
    }
    let coverage = coverage::evaluate(&model.steps);
    if !coverage.pass {
        errors.push("evidence coverage failed".to_string());
    }

    StepsValidateExportResponse {
        schema_valid: model.schema_version == 1,
        evidence_valid: coverage.pass,
        errors,
    }
}
