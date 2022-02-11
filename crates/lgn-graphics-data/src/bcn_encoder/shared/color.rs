use crate::Color;

#[derive(Default, Clone, Copy)]
pub(crate) struct ColorRgb565 {
    pub(crate) data: u16,
}

impl ColorRgb565 {
    const RED_MASK: u16 = 0b1111_1000_0000_0000;
    const RED_SHIFT: u16 = 11;
    const GREEN_MASK: u16 = 0b0000_0111_1110_0000;
    const GREEN_SHIFT: u16 = 5;
    const BLUE_MASK: u16 = 0b0000_0000_0001_1111;

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let mut color = Self::default();
        color.set_r(r);
        color.set_g(g);
        color.set_b(b);

        color
    }

    pub fn get_r(self) -> u8 {
        let r5 = (self.data & Self::RED_MASK) >> Self::RED_SHIFT;
        ((r5 << 3) | (r5 >> 2)) as u8
    }

    pub fn set_r(&mut self, value: u8) {
        let r5 = u16::from(value >> 3);
        self.data &= !Self::RED_MASK;
        self.data |= r5 << Self::RED_SHIFT;
    }

    pub fn get_g(self) -> u8 {
        let g6 = (self.data & Self::GREEN_MASK) >> Self::GREEN_SHIFT;
        ((g6 << 2) | (g6 >> 4)) as u8
    }

    pub fn set_g(&mut self, value: u8) {
        let g6 = u16::from(value >> 2);
        self.data &= !Self::GREEN_MASK;
        self.data |= g6 << Self::GREEN_SHIFT;
    }

    pub fn get_b(self) -> u8 {
        let b5 = self.data & Self::BLUE_MASK;
        ((b5 << 3) | (b5 >> 2)) as u8
    }

    pub fn set_b(&mut self, value: u8) {
        let b5 = u16::from(value >> 3);
        self.data &= !Self::BLUE_MASK;
        self.data |= b5;
    }

    pub fn get_raw_r(self) -> u16 {
        (self.data & Self::RED_MASK) >> Self::RED_SHIFT
    }

    pub fn set_raw_r(&mut self, mut value: i32) {
        if value > 31 {
            value = 31;
        };
        if value < 0 {
            value = 0;
        };

        self.data &= !Self::RED_MASK;
        self.data |= (value << Self::RED_SHIFT) as u16;
    }

    pub fn get_raw_g(self) -> u16 {
        (self.data & Self::GREEN_MASK) >> Self::GREEN_SHIFT
    }

    pub fn set_raw_g(&mut self, mut value: i32) {
        if value > 63 {
            value = 63;
        };
        if value < 0 {
            value = 0;
        };

        self.data &= !Self::GREEN_MASK;
        self.data |= (value << Self::GREEN_SHIFT) as u16;
    }

    pub fn get_raw_b(self) -> u16 {
        self.data & Self::BLUE_MASK
    }

    pub fn set_raw_b(&mut self, mut value: i32) {
        if value > 31 {
            value = 31;
        };
        if value < 0 {
            value = 0;
        };

        self.data &= !Self::BLUE_MASK;
        self.data |= value as u16;
    }

    pub fn to_color_rgba_32(self) -> Color {
        Color::from((self.get_r(), self.get_g(), self.get_b()))
    }
}

pub(crate) fn interpolate_color(c0: Color, c1: Color, mut factor: f32) -> Color {
    factor = if factor < 0.0 {
        0.0
    } else if factor > 1.0 {
        1.0
    } else {
        factor
    };

    let lerp_r = f32::from(c0.r) * factor + f32::from(c1.r) * (1.0 - factor);
    let lerp_g = f32::from(c0.g) * factor + f32::from(c1.g) * (1.0 - factor);
    let lerp_b = f32::from(c0.b) * factor + f32::from(c1.b) * (1.0 - factor);

    Color::from((lerp_r as u8, lerp_g as u8, lerp_b as u8))
}
