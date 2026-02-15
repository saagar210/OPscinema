use opscinema_types::{AppError, AppErrorCode, Step, StepEditOp};

pub fn apply_edit(steps: &mut Vec<Step>, op: &StepEditOp) -> Result<(), AppError> {
    match op {
        StepEditOp::InsertAfter {
            after_step_id,
            step,
        } => {
            let pos = steps
                .iter()
                .position(|s| &s.step_id == after_step_id)
                .ok_or(AppError {
                    code: AppErrorCode::NotFound,
                    message: "after_step_id not found".to_string(),
                    details: None,
                    recoverable: true,
                    action_hint: None,
                })?;
            steps.insert(pos + 1, step.clone());
            renumber(steps);
            Ok(())
        }
        StepEditOp::UpdateTitle { step_id, title } => {
            let step = steps
                .iter_mut()
                .find(|s| &s.step_id == step_id)
                .ok_or(AppError {
                    code: AppErrorCode::NotFound,
                    message: "step_id not found".to_string(),
                    details: None,
                    recoverable: true,
                    action_hint: None,
                })?;
            step.title = title.clone();
            Ok(())
        }
        StepEditOp::ReplaceBody { step_id, body } => {
            let step = steps
                .iter_mut()
                .find(|s| &s.step_id == step_id)
                .ok_or(AppError {
                    code: AppErrorCode::NotFound,
                    message: "step_id not found".to_string(),
                    details: None,
                    recoverable: true,
                    action_hint: None,
                })?;
            step.body = body.clone();
            Ok(())
        }
        StepEditOp::Delete { step_id } => {
            let before = steps.len();
            steps.retain(|s| &s.step_id != step_id);
            if steps.len() == before {
                return Err(AppError {
                    code: AppErrorCode::NotFound,
                    message: "step_id not found".to_string(),
                    details: None,
                    recoverable: true,
                    action_hint: None,
                });
            }
            renumber(steps);
            Ok(())
        }
        StepEditOp::Reorder { step_id, new_index } => {
            let pos = steps
                .iter()
                .position(|s| &s.step_id == step_id)
                .ok_or(AppError {
                    code: AppErrorCode::NotFound,
                    message: "step_id not found".to_string(),
                    details: None,
                    recoverable: true,
                    action_hint: None,
                })?;
            let step = steps.remove(pos);
            let idx = (*new_index as usize).min(steps.len());
            steps.insert(idx, step);
            renumber(steps);
            Ok(())
        }
    }
}

fn renumber(steps: &mut [Step]) {
    for (i, step) in steps.iter_mut().enumerate() {
        step.order_index = i as u32;
    }
}
