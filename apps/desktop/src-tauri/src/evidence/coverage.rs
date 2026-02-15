use opscinema_types::{EvidenceCoverageResponse, Step, TextBlockProvenance};

pub fn evaluate(steps: &[Step]) -> EvidenceCoverageResponse {
    let mut missing_step_ids = Vec::new();
    let mut missing_generated_block_ids = Vec::new();

    for step in steps {
        let mut step_missing = false;
        for block in &step.body.blocks {
            if block.provenance == TextBlockProvenance::Generated && block.evidence_refs.is_empty()
            {
                missing_generated_block_ids.push(block.block_id.clone());
                step_missing = true;
            }
        }
        if step_missing {
            missing_step_ids.push(step.step_id);
        }
    }

    EvidenceCoverageResponse {
        pass: missing_generated_block_ids.is_empty(),
        missing_step_ids,
        missing_generated_block_ids,
    }
}
