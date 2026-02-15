use opscinema_types::BBoxNorm;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawPoint {
    pub x: f64,
    pub y: f64,
}

pub fn normalize_point(raw: RawPoint, width: f64, height: f64) -> (u32, u32) {
    let x = ((raw.x / width).clamp(0.0, 1.0) * 10_000.0).round() as u32;
    let y = ((raw.y / height).clamp(0.0, 1.0) * 10_000.0).round() as u32;
    (x, y)
}

pub fn normalize_bbox(x: f64, y: f64, w: f64, h: f64, width: f64, height: f64) -> BBoxNorm {
    let (nx, ny) = normalize_point(RawPoint { x, y }, width, height);
    let nw = ((w / width).clamp(0.0, 1.0) * 10_000.0).round() as u32;
    let nh = ((h / height).clamp(0.0, 1.0) * 10_000.0).round() as u32;
    BBoxNorm {
        x: nx,
        y: ny,
        w: nw,
        h: nh,
    }
}
