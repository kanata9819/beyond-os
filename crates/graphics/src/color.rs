#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn white() -> Color {
        Color {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 0x00,
            g: 0x00,
            b: 0x00,
        }
    }
    // 背景：濃い青
    pub fn deep_blue() -> Color {
        Color {
            r: 0x10,
            g: 0x40,
            b: 0x60,
        }
    }
}
