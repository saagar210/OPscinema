use opscinema_types::{EvidenceItem, EvidenceSet, Step};
use uuid::Uuid;

pub fn for_step(step: &Step, all: &EvidenceSet) -> EvidenceSet {
    let ids: std::collections::BTreeSet<Uuid> = step
        .body
        .blocks
        .iter()
        .flat_map(|b| b.evidence_refs.iter().copied())
        .collect();
    let evidence = all
        .evidence
        .iter()
        .filter(|e| ids.contains(&e.evidence_id))
        .cloned()
        .collect::<Vec<EvidenceItem>>();
    EvidenceSet { evidence }
}
