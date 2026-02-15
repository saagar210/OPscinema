use uuid::Uuid;

pub const EVIDENCE_NAMESPACE_UUID: Uuid = Uuid::from_u128(0xd4214da8_7c85_4ff4_84ba_9e6f0bba4a1f);

pub fn deterministic_evidence_id(session_id: Uuid, kind: &str, source_id: &str) -> Uuid {
    let input = format!("{session_id}:{kind}:{source_id}");
    Uuid::new_v5(&EVIDENCE_NAMESPACE_UUID, input.as_bytes())
}
