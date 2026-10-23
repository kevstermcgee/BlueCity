//! Shared isometric projection for drawing, terrain picking, and placement.
use crate::State;
use vesper3d::two_d::Point;
pub fn tile_size(zoom: i32) -> (i32, i32) {
    match zoom {
        1 => (8, 4),
        3 => (18, 9),
        _ => (12, 6),
    }
}
pub fn project(cam_x: i32, cam_y: i32, zoom: i32, x: i32, y: i32, elevation: i16) -> Point {
    let (w, h) = tile_size(zoom);
    Point::new(
        400 + (x - cam_x - y + cam_y) * w,
        225 + (x - cam_x + y - cam_y) * h - i32::from(elevation) * h / 16,
    )
}
pub fn project_flat(st: &State, x: i32, y: i32) -> Point {
    project(st.cam_x, st.cam_y, st.zoom, x, y, 0)
}
pub fn project_tile(st: &State, x: i32, y: i32) -> Point {
    let elevation = st
        .city
        .idx(x, y)
        .and_then(|i| st.city.ext.cells.get(i))
        .map(|c| c.elevation)
        .unwrap_or(0);
    project(st.cam_x, st.cam_y, st.zoom, x, y, elevation)
}
pub fn pick(st: &State, p: Point) -> Option<[i32; 2]> {
    let (w, h) = tile_size(st.zoom);
    let a = (p.x - 400) as f64 / f64::from(w);
    let b = (p.y - 225) as f64 / f64::from(h);
    let approximate = [
        ((a + b) / 2.).round() as i32 + st.cam_x,
        ((b - a) / 2.).round() as i32 + st.cam_y,
    ];
    // Reverse painter order resolves elevated diamonds that overlap on screen.
    let mut candidates = Vec::new();
    for y in approximate[1] - 8..=approximate[1] + 8 {
        for x in approximate[0] - 8..=approximate[0] + 8 {
            if st.city.idx(x, y).is_some() {
                let q = project_tile(st, x, y);
                if (p.x - q.x).abs() * h + (p.y - q.y).abs() * w <= w * h {
                    candidates.push([x, y]);
                }
            }
        }
    }
    candidates.into_iter().max_by_key(|[x, y]| (x + y, *x))
}
