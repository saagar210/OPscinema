use opscinema_types::{AppError, AppErrorCode, ExportWarning, Step, TextBlockProvenance};

#[derive(Debug, Clone)]
pub struct ExportGateInput {
    pub steps: Vec<Step>,
    pub missing_evidence: Vec<String>,
    pub degraded_anchor_ids: Vec<String>,
    pub warnings: Vec<ExportWarning>,
}

pub fn ensure_generated_blocks_have_evidence(steps: &[Step]) -> Result<(), AppError> {
    for step in steps {
        for block in &step.body.blocks {
            if block.provenance == TextBlockProvenance::Generated && block.evidence_refs.is_empty()
            {
                return Err(AppError {
                    code: AppErrorCode::ExportGateFailed,
                    message: "Generated text block missing evidence refs".to_string(),
                    details: Some(format!("step={} block={}", step.step_id, block.block_id)),
                    recoverable: false,
                    action_hint: Some(
                        "Attach evidence or mark block as human-authored".to_string(),
                    ),
                });
            }
        }
    }
    Ok(())
}

pub fn tutorial_pack_gate(input: &ExportGateInput) -> Result<(), AppError> {
    ensure_generated_blocks_have_evidence(&input.steps)?;
    if !input.missing_evidence.is_empty()
        || !input.degraded_anchor_ids.is_empty()
        || !input.warnings.is_empty()
    {
        return Err(AppError {
            code: AppErrorCode::ExportGateFailed,
            message: "TutorialPack strict gate failed".to_string(),
            details: Some(format!(
                "missing_evidence={} degraded_anchors={} warnings={}",
                input.missing_evidence.len(),
                input.degraded_anchor_ids.len(),
                input.warnings.len()
            )),
            recoverable: true,
            action_hint: Some("Resolve evidence coverage, anchors, and warnings".to_string()),
        });
    }
    Ok(())
}

pub fn proof_bundle_gate(input: &ExportGateInput) -> Result<(), AppError> {
    ensure_generated_blocks_have_evidence(&input.steps)?;
    if !input.missing_evidence.is_empty() {
        return Err(AppError {
            code: AppErrorCode::ExportGateFailed,
            message: "ProofBundle blocked due to missing evidence coverage".to_string(),
            details: None,
            recoverable: true,
            action_hint: Some("Attach evidence refs".to_string()),
        });
    }
    Ok(())
}
