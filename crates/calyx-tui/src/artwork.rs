use image::imageops::FilterType;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub const ART_WIDTH: usize = 20;
pub const ART_HEIGHT_PIXELS: usize = 20;

#[derive(Debug, Clone)]
pub struct TerminalCoverArt {
    pub grid: Vec<Vec<(u8, u8, u8)>>,
    pub is_real_image: bool,
}

impl TerminalCoverArt {
    pub fn from_jpeg_or_seed(bytes: Option<&[u8]>, seed_text: &str) -> Self {
        if let Some(raw) = bytes {
            if let Ok(img) = image::load_from_memory(raw) {
                let resized = img
                    .resize_exact(ART_WIDTH as u32, ART_HEIGHT_PIXELS as u32, FilterType::Lanczos3)
                    .to_rgb8();
                let mut grid = vec![vec![(0u8, 0u8, 0u8); ART_WIDTH]; ART_HEIGHT_PIXELS];
                for y in 0..ART_HEIGHT_PIXELS {
                    for x in 0..ART_WIDTH {
                        let p = resized.get_pixel(x as u32, y as u32);
                        grid[y][x] = (p[0], p[1], p[2]);
                    }
                }
                return Self {
                    grid,
                    is_real_image: true,
                };
            }
        }
        Self::procedural_fallback(seed_text)
    }

    fn procedural_fallback(seed_text: &str) -> Self {
        let mut grid = vec![vec![(24u8, 28u8, 44u8); ART_WIDTH]; ART_HEIGHT_PIXELS];
        let hash = fnv1a_hash(seed_text.as_bytes());

        let r_base = ((hash & 0xFF) as u8).wrapping_mul(3);
        let g_base = (((hash >> 8) & 0xFF) as u8).wrapping_mul(5);
        let b_base = (((hash >> 16) & 0xFF) as u8).wrapping_mul(7);

        let r_accent = ((hash >> 24) & 0xFF) as u8;
        let g_accent = ((hash >> 32) & 0xFF) as u8;
        let b_accent = ((hash >> 40) & 0xFF) as u8;

        for y in 0..ART_HEIGHT_PIXELS {
            for x in 0..ART_WIDTH {
                let dx = (x as isize - (ART_WIDTH as isize / 2)).unsigned_abs();
                let dy = (y as isize - (ART_HEIGHT_PIXELS as isize / 2)).unsigned_abs();
                let dist = dx + dy;

                if dist <= 3 {
                    grid[y][x] = (r_accent, g_accent, b_accent);
                } else if dist <= 8 {
                    let groove = (dist * 24) as u8;
                    grid[y][x] = (
                        r_base.wrapping_add(groove),
                        g_base.wrapping_add(groove / 2),
                        b_base.wrapping_add(groove),
                    );
                } else {
                    let gx = (x * 11) as u8;
                    let gy = (y * 11) as u8;
                    grid[y][x] = (
                        r_base.wrapping_add(gx),
                        g_accent.wrapping_add(gy),
                        b_base.wrapping_add(gx ^ gy),
                    );
                }
            }
        }

        Self {
            grid,
            is_real_image: false,
        }
    }

    pub fn to_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::with_capacity(ART_HEIGHT_PIXELS / 2);
        for y in (0..ART_HEIGHT_PIXELS).step_by(2) {
            let mut spans = Vec::with_capacity(ART_WIDTH);
            for x in 0..ART_WIDTH {
                let top = self.grid[y][x];
                let bottom = self.grid[y + 1][x];
                spans.push(Span::styled(
                    "▀",
                    Style::default()
                        .fg(Color::Rgb(top.0, top.1, top.2))
                        .bg(Color::Rgb(bottom.0, bottom.1, bottom.2)),
                ));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}