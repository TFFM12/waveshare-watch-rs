use embedded_graphics::geometry::Point as EgPoint;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::apps::{App, AppInput, AppResult};

const W: i32 = 410;
const H: i32 = 502;

const BTN_X: i32 = 30;
const BTN_W: i32 = W - BTN_X * 2;
const BTN_H: i32 = 56;
const BTN_GAP: i32 = 12;
const BTN_Y0: i32 = 150;

#[derive(Clone, Copy, PartialEq)]
pub enum OutputMode {
    Auto,
    Speaker,
    Headphones,
}

pub struct BluetoothAudioApp {
    connected: bool,
    connecting: bool,
    target_label: [u8; 18],
    target_len: usize,
    output_mode: OutputMode,
    headphones_connected: bool,
    request_connect: bool,
    request_disconnect: bool,
}

impl BluetoothAudioApp {
    pub fn new() -> Self {
        Self {
            connected: false,
            connecting: false,
            target_label: *b"--:--:--:--:--:--",
            target_len: 17,
            output_mode: OutputMode::Auto,
            headphones_connected: false,
            request_connect: false,
            request_disconnect: false,
        }
    }

    pub fn set_target_label(&mut self, label: &str) {
        let bytes = label.as_bytes();
        let len = bytes.len().min(self.target_label.len());
        self.target_label[..len].copy_from_slice(&bytes[..len]);
        self.target_len = len;
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        self.connecting = false;
    }

    pub fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    pub fn headphones_connected(&self) -> bool {
        self.headphones_connected
    }

    pub fn take_connect_request(&mut self) -> bool {
        let req = self.request_connect;
        self.request_connect = false;
        req
    }

    pub fn take_disconnect_request(&mut self) -> bool {
        let req = self.request_disconnect;
        self.request_disconnect = false;
        req
    }

    pub fn handle_tap(&mut self, x: u16, y: u16) {
        let xi = x as i32;
        let yi = y as i32;
        for idx in 0..3i32 {
            let by = BTN_Y0 + idx * (BTN_H + BTN_GAP);
            if xi >= BTN_X && xi <= BTN_X + BTN_W && yi >= by && yi <= by + BTN_H {
                match idx {
                    0 => {
                        if self.connected || self.connecting {
                            self.request_disconnect = true;
                            self.connecting = false;
                            self.connected = false;
                        } else {
                            self.request_connect = true;
                            self.connecting = true;
                        }
                    }
                    1 => {
                        self.output_mode = match self.output_mode {
                            OutputMode::Auto => OutputMode::Speaker,
                            OutputMode::Speaker => OutputMode::Headphones,
                            OutputMode::Headphones => OutputMode::Auto,
                        };
                    }
                    2 => {
                        self.headphones_connected = !self.headphones_connected;
                    }
                    _ => {}
                }
                break;
            }
        }
    }

    fn target_str(&self) -> &str {
        core::str::from_utf8(&self.target_label[..self.target_len]).unwrap_or("--")
    }

    pub fn active_output_label(&self) -> &'static str {
        match self.output_mode {
            OutputMode::Auto => {
                if self.headphones_connected {
                    "HEADPHONES (AMP)"
                } else {
                    "SPEAKER"
                }
            }
            OutputMode::Speaker => "SPEAKER",
            OutputMode::Headphones => "HEADPHONES (AMP)",
        }
    }
}

impl App for BluetoothAudioApp {
    fn name(&self) -> &str { "Bluetooth Audio" }

    fn setup(&mut self) {
        self.request_connect = false;
        self.request_disconnect = false;
    }

    fn update(&mut self, _input: &AppInput) -> AppResult {
        AppResult::Continue
    }

    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(EgPoint::zero(), Size::new(W as u32, H as u32))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(d);

        let title = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
        let white = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let dim = MonoTextStyle::new(&FONT_10X20, Rgb565::CSS_GRAY);
        let green = MonoTextStyle::new(&FONT_10X20, Rgb565::GREEN);
        let yellow = MonoTextStyle::new(&FONT_10X20, Rgb565::YELLOW);

        let _ = Text::with_alignment("BLUETOOTH AUDIO", EgPoint::new(W / 2, 38), title, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Target", EgPoint::new(W / 2, 72), dim, Alignment::Center).draw(d);
        let _ = Text::with_alignment(self.target_str(), EgPoint::new(W / 2, 98), white, Alignment::Center).draw(d);

        let status = if self.connected {
            "CONNECTED"
        } else if self.connecting {
            "CONNECTING..."
        } else {
            "DISCONNECTED"
        };
        let status_style = if self.connected { green } else { yellow };
        let _ = Text::with_alignment(status, EgPoint::new(W / 2, 124), status_style, Alignment::Center).draw(d);

        draw_button(
            d,
            BTN_Y0,
            if self.connected || self.connecting { "DISCONNECT" } else { "CONNECT" },
            if self.connected || self.connecting { Rgb565::new(16, 4, 2) } else { Rgb565::new(2, 12, 20) },
        );
        draw_button(
            d,
            BTN_Y0 + BTN_H + BTN_GAP,
            match self.output_mode {
                OutputMode::Auto => "OUTPUT: AUTO",
                OutputMode::Speaker => "OUTPUT: SPEAKER",
                OutputMode::Headphones => "OUTPUT: HEADPHONES",
            },
            Rgb565::new(3, 8, 3),
        );
        draw_button(
            d,
            BTN_Y0 + (BTN_H + BTN_GAP) * 2,
            if self.headphones_connected { "HP LINK: ON" } else { "HP LINK: OFF" },
            if self.headphones_connected { Rgb565::new(2, 16, 6) } else { Rgb565::new(6, 6, 6) },
        );

        let _ = Text::with_alignment("ACTIVE ROUTE", EgPoint::new(W / 2, 408), dim, Alignment::Center).draw(d);
        let _ = Text::with_alignment(self.active_output_label(), EgPoint::new(W / 2, 434), white, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Tap buttons to control BT audio", EgPoint::new(W / 2, H - 20), dim, Alignment::Center).draw(d);
    }
}

fn draw_button<D: DrawTarget<Color = Rgb565>>(d: &mut D, y: i32, label: &str, bg: Rgb565) {
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(EgPoint::new(BTN_X, y), Size::new(BTN_W as u32, BTN_H as u32)),
        Size::new(10, 10),
    )
    .into_styled(PrimitiveStyle::with_fill(bg))
    .draw(d);
    let txt = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let _ = Text::with_alignment(label, EgPoint::new(W / 2, y + 34), txt, Alignment::Center).draw(d);
}
