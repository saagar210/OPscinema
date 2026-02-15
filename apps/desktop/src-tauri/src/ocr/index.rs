use opscinema_types::{OcrBlock, OcrSearchHit};

pub fn search(session_blocks: &[OcrBlock], query: &str, frame_ms: i64) -> Vec<OcrSearchHit> {
    let q = query.to_lowercase();
    session_blocks
        .iter()
        .filter(|block| block.text.to_lowercase().contains(&q))
        .map(|block| OcrSearchHit {
            frame_ms,
            block_id: block.ocr_block_id.clone(),
            snippet: block.text.clone(),
        })
        .collect()
}
