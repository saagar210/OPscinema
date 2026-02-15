pub mod error;
pub mod ipc;
pub mod models;

pub use error::*;
pub use ipc::*;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn roundtrip<T>(value: &T)
    where
        T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_string(value).expect("serialize");
        let back: T = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(&back, value);
    }

    #[test]
    fn phase0_contract_roundtrip_smoke() {
        roundtrip(&AppError {
            code: AppErrorCode::PermissionDenied,
            message: "m".to_string(),
            details: Some("d".to_string()),
            recoverable: true,
            action_hint: Some("hint".to_string()),
        });
        roundtrip(&SessionCreateRequest {
            label: "session".to_string(),
            metadata: std::collections::BTreeMap::new(),
        });
        roundtrip(&StepsApplyEditRequest {
            session_id: Uuid::new_v4(),
            base_seq: 1,
            op: StepEditOp::Delete {
                step_id: Uuid::new_v4(),
            },
        });
        roundtrip(&ExportVerifyRequest {
            bundle_path: "/tmp/bundle".to_string(),
        });
    }

    #[test]
    fn phase5_step_schema_byte_lock() {
        let schema = schema_for!(StepModel);
        let mut generated = serde_json::to_string_pretty(&schema).expect("schema json");
        generated.push('\n');
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema/step_model.v1.json");
        let committed = fs::read_to_string(path).expect("read committed schema");
        assert_eq!(
            generated, committed,
            "step schema changed; requires versioned migration"
        );
    }
}
