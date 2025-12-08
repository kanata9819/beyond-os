use crate::color::Color;
use crate::font;
use crate::frame_buffer::BeyondFrameBufferTrait;

pub struct Renderer;

impl Renderer {
    pub fn draw_char<T: BeyondFrameBufferTrait>(
        fb: &mut T,
        x: usize,
        y: usize,
        glyph: &[u8; 8],
        color: Color,
    ) {
        const SCALE: usize = 2;
        for (row, line) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (line >> (7 - col)) & 1 == 1 {
                    for sy in 0..SCALE {
                        for sx in 0..SCALE {
                            fb.put_pixel(x + col * SCALE + sx, y + row * SCALE + sy, color);
                        }
                    }
                }
            }
        }
    }

    pub fn glyph_for(c: char) -> Option<&'static [u8; 8]> {
        match c {
            'A' => Some(&font::GLYPH_A),
            'B' => Some(&font::GLYPH_B),
            'C' => Some(&font::GLYPH_C),
            'D' => Some(&font::GLYPH_D),
            'E' => Some(&font::GLYPH_E),
            'F' => Some(&font::GLYPH_F),
            'G' => Some(&font::GLYPH_G),
            'H' => Some(&font::GLYPH_H),
            'I' => Some(&font::GLYPH_I),
            'J' => Some(&font::GLYPH_J),
            'K' => Some(&font::GLYPH_K),
            'L' => Some(&font::GLYPH_L),
            'M' => Some(&font::GLYPH_M),
            'N' => Some(&font::GLYPH_N),
            'O' => Some(&font::GLYPH_O),
            'P' => Some(&font::GLYPH_P),
            'Q' => Some(&font::GLYPH_Q),
            'R' => Some(&font::GLYPH_R),
            'S' => Some(&font::GLYPH_S),
            'T' => Some(&font::GLYPH_T),
            'U' => Some(&font::GLYPH_U),
            'V' => Some(&font::GLYPH_V),
            'W' => Some(&font::GLYPH_W),
            'X' => Some(&font::GLYPH_X),
            'Y' => Some(&font::GLYPH_Y),
            'Z' => Some(&font::GLYPH_Z),
            'a' => Some(&font::GLYPH_A_SMALL),
            'b' => Some(&font::GLYPH_B_SMALL),
            'c' => Some(&font::GLYPH_C_SMALL),
            'd' => Some(&font::GLYPH_D_SMALL),
            'e' => Some(&font::GLYPH_E_SMALL),
            'f' => Some(&font::GLYPH_F_SMALL),
            'g' => Some(&font::GLYPH_G_SMALL),
            'h' => Some(&font::GLYPH_H_SMALL),
            'i' => Some(&font::GLYPH_I_SMALL),
            'j' => Some(&font::GLYPH_J_SMALL),
            'k' => Some(&font::GLYPH_K_SMALL),
            'l' => Some(&font::GLYPH_L_SMALL),
            'm' => Some(&font::GLYPH_M_SMALL),
            'n' => Some(&font::GLYPH_N_SMALL),
            'o' => Some(&font::GLYPH_O_SMALL),
            'p' => Some(&font::GLYPH_P_SMALL),
            'q' => Some(&font::GLYPH_Q_SMALL),
            'r' => Some(&font::GLYPH_R_SMALL),
            's' => Some(&font::GLYPH_S_SMALL),
            't' => Some(&font::GLYPH_T_SMALL),
            'u' => Some(&font::GLYPH_U_SMALL),
            'v' => Some(&font::GLYPH_V_SMALL),
            'w' => Some(&font::GLYPH_W_SMALL),
            'x' => Some(&font::GLYPH_X_SMALL),
            'y' => Some(&font::GLYPH_Y_SMALL),
            'z' => Some(&font::GLYPH_Z_SMALL),
            '!' => Some(&font::GLYPH_EXCL),
            ' ' => Some(&font::GLYPH_SPACE),
            '?' => Some(&font::GLYPH_QUESTION),
            '.' => Some(&font::GLYPH_DOT),
            ',' => Some(&font::GLYPH_COMMA),
            ':' => Some(&font::GLYPH_COLON),
            ';' => Some(&font::GLYPH_SEMICOLON),
            '-' => Some(&font::GLYPH_MINUS),
            '_' => Some(&font::GLYPH_UNDERSCORE),
            '+' => Some(&font::GLYPH_PLUS),
            '*' => Some(&font::GLYPH_ASTERISK),
            '/' => Some(&font::GLYPH_SLASH),
            '(' => Some(&font::GLYPH_LPAREN),
            ')' => Some(&font::GLYPH_RPAREN),
            '[' => Some(&font::GLYPH_LBRACKET),
            ']' => Some(&font::GLYPH_RBRACKET),
            '{' => Some(&font::GLYPH_LBRACE),
            '}' => Some(&font::GLYPH_RBRACE),
            '=' => Some(&font::GLYPH_EQUAL),
            '<' => Some(&font::GLYPH_LT),
            '>' => Some(&font::GLYPH_GT),
            '0' => Some(&font::GLYPH_0),
            '1' => Some(&font::GLYPH_1),
            '2' => Some(&font::GLYPH_2),
            '3' => Some(&font::GLYPH_3),
            '4' => Some(&font::GLYPH_4),
            '5' => Some(&font::GLYPH_5),
            '6' => Some(&font::GLYPH_6),
            '7' => Some(&font::GLYPH_7),
            '8' => Some(&font::GLYPH_8),
            '9' => Some(&font::GLYPH_9),
            _ => None,
        }
    }
}
