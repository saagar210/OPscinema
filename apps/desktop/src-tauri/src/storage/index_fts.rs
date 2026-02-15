use opscinema_types::{OcrBlock, OcrSearchHit};

pub fn search_blocks(blocks: &[OcrBlock], query: &str, frame_ms: i64) -> Vec<OcrSearchHit> {
    let q = query.to_lowercase();
    blocks
        .iter()
        .filter(|b| b.text.to_lowercase().contains(&q))
        .map(|b| OcrSearchHit {
            frame_ms,
            block_id: b.ocr_block_id.clone(),
            snippet: b.text.clone(),
        })
        .collect()
}
