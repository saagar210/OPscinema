use opscinema_types::EvidenceLocator;

pub fn detect_drift(previous: &[EvidenceLocator], current: &[EvidenceLocator]) -> bool {
    if previous.len() != current.len() {
        return true;
    }

    for (prev, curr) in previous.iter().zip(current.iter()) {
        if prev.locator_type != curr.locator_type {
            return true;
        }
        if prev.asset_id != curr.asset_id || prev.frame_ms != curr.frame_ms {
            return true;
        }
        if prev.text_offset != curr.text_offset {
            return true;
        }

        match (&prev.bbox_norm, &curr.bbox_norm) {
            (Some(a), Some(b)) => {
                if distance(a.x, b.x) > 250
                    || distance(a.y, b.y) > 250
                    || distance(a.w, b.w) > 300
                    || distance(a.h, b.h) > 300
                {
                    return true;
                }
            }
            (None, None) => {}
            _ => return true,
        }
    }

    false
}

fn distance(a: u32, b: u32) -> u32 {
    a.abs_diff(b)
}
