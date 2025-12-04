#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const WHITE: Color = Color {
    r: 0xFF,
    g: 0xFF,
    b: 0xFF,
};

pub const BLACK: Color = Color {
    r: 0x00,
    g: 0x00,
    b: 0x00,
};
