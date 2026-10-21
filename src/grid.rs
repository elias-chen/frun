//! Geometry and pagination for the pinned-program home screen.
pub const COLUMNS: usize = 4;
pub const CELL: f32 = 92.0;
pub const HEADER: f32 = 32.0;

/// Put the search field at the cursor and keep the entire panel in the work area.
pub fn anchored(
    at: crate::gesture::Point,
    work: [i32; 4],
    size: (i32, i32),
    scale: f32,
) -> (i32, i32) {
    let x = at.x - (54.0 * scale) as i32;
    let y = at.y - (28.0 * scale) as i32;
    (
        x.clamp(work[0], (work[2] - size.0).max(work[0])),
        y.clamp(work[1], (work[3] - size.1).max(work[1])),
    )
}

pub fn page_size(body_height: f32) -> usize {
    let rows = ((body_height - HEADER) / CELL).floor().clamp(1.0, 4.0) as usize;
    rows * COLUMNS
}

pub fn pages(count: usize, size: usize) -> usize {
    count.div_ceil(size).max(1)
}

pub fn page(count: usize, size: usize, requested: usize) -> std::ops::Range<usize> {
    let start = requested.min(pages(count, size) - 1) * size;
    start..(start + size).min(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_anchor_handles_negative_monitors_dpi_and_screen_edges() {
        use crate::gesture::Point;
        let work = [-1920, 0, 0, 1080];
        assert_eq!(
            anchored(Point { x: -1000, y: 300 }, work, (960, 600), 1.5),
            (-1081, 258)
        );
        for at in [Point { x: -1920, y: 0 }, Point { x: -1, y: 1079 }] {
            let (x, y) = anchored(at, work, (960, 600), 1.5);
            assert!(x >= -1920 && x + 960 <= 0);
            assert!(y >= 0 && y + 600 <= 1080);
        }
    }

    #[test]
    fn pins_remain_reachable_on_small_screens_and_partial_pages() {
        let size = page_size(260.0);
        assert_eq!(size, 8);
        let visited: Vec<usize> = (0..pages(23, size))
            .flat_map(|p| page(23, size, p))
            .collect();
        assert_eq!(visited, (0..23).collect::<Vec<_>>());
        assert_eq!(page(23, size, 20), 16..23);
        assert_eq!(page(0, size, 0), 0..0);
        assert_eq!(page_size(20.0), 4);
        assert_eq!(page_size(2000.0), 16);
    }
}
