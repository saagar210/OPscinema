use opscinema_types::AnchorCandidate;

pub fn score_anchor(anchor: &AnchorCandidate, drift_px: f32) -> u8 {
    let penalty = (drift_px / 10.0).round() as i32;
    anchor.confidence.saturating_sub(penalty.max(0) as u8)
}
