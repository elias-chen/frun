//! Geometry and pagination for the pinned-program home screen.
pub const COLUMNS: usize = 4;
pub const CELL: f32 = 92.0;
pub const HEADER: f32 = 32.0;

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
