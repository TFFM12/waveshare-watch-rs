use embedded_graphics::geometry::Point as EgPoint;
use embedded_graphics::mono_font::ascii::{FONT_8X13, FONT_10X20};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::apps::{App, AppInput, AppResult};

const W: i32 = 410;
const H: i32 = 502;
const CX: i32 = W / 2;
const CY: i32 = H / 2 + 10;
const R: i32 = 92;
const BALL_R: i32 = 12;

pub struct GyroApp {
    ax: i16,
    ay: i16,
}

impl GyroApp {
    pub fn new() -> Self {
        Self { ax: 0, ay: 0 }
    }
}

impl App for GyroApp {
    fn name(&self) -> &str { "Gyroscope" }

    fn setup(&mut self) {}

    fn update(&mut self, input: &AppInput) -> AppResult {
        self.ax = (input.accel.0 * 100.0) as i16;
        self.ay = (input.accel.1 * 100.0) as i16;
        AppResult::Continue
    }

    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(EgPoint::zero(), Size::new(W as u32, H as u32))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(d);

        let title = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
        let dim = MonoTextStyle::new(&FONT_8X13, Rgb565::CSS_GRAY);
        let white = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);

        let _ = Text::with_alignment("GYRO MENU", EgPoint::new(CX, 36), title, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Roda movida pelo acelerometro", EgPoint::new(CX, 56), dim, Alignment::Center).draw(d);

        let _ = Circle::new(EgPoint::new(CX - R, CY - R), (R * 2) as u32)
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_DARK_GRAY, 2))
            .draw(d);

        let max_off = R - BALL_R - 6;
        let bx = (-(self.ay as i32) * max_off / 100).clamp(-max_off, max_off);
        let by = ((self.ax as i32) * max_off / 100).clamp(-max_off, max_off);
        let _ = Rectangle::new(
            EgPoint::new(CX + bx - BALL_R, CY + by - BALL_R),
            Size::new((BALL_R * 2) as u32, (BALL_R * 2) as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(d);

        let mut buf = [0u8; 24];
        let s = fmt_axes(&mut buf, self.ax, self.ay);
        let _ = Text::with_alignment(s, EgPoint::new(CX, CY + R + 34), white, Alignment::Center).draw(d);
    }
}

fn fmt_axes<'a>(buf: &'a mut [u8; 24], x: i16, y: i16) -> &'a str {
    let mut p = 0;
    for &c in b"X:" { buf[p] = c; p += 1; }
    p = write_i16(buf, p, x);
    for &c in b" Y:" { buf[p] = c; p += 1; }
    p = write_i16(buf, p, y);
    core::str::from_utf8(&buf[..p]).unwrap_or("X:0 Y:0")
}

fn write_i16(buf: &mut [u8; 24], mut p: usize, v: i16) -> usize {
    if v < 0 { buf[p] = b'-'; p += 1; }
    let a = v.unsigned_abs();
    if a >= 100 { buf[p] = b'0' + (a / 100) as u8; p += 1; }
    if a >= 10 { buf[p] = b'0' + ((a / 10) % 10) as u8; p += 1; }
    buf[p] = b'0' + (a % 10) as u8; p + 1
}
