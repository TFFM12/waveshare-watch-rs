use alloc::string::String;
use alloc::vec::Vec;

use embedded_graphics::geometry::Point as EgPoint;
use embedded_graphics::mono_font::ascii::{FONT_8X13, FONT_10X20};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

const W: i32 = 410;
const H: i32 = 502;
const X: i32 = 14;
const ROW_W: i32 = W - 28;
const ROW_H: i32 = 34;
const GAP: i32 = 6;

const ROW_WIFI: i32 = 58;
const ROW_BLE: i32 = ROW_WIFI + (ROW_H + GAP);
const ROW_CPU: i32 = ROW_BLE + (ROW_H + GAP);
const ROW_BRI: i32 = ROW_CPU + (ROW_H + GAP);
const ROW_WALL: i32 = ROW_BRI + (ROW_H + GAP);
const ROW_TIME: i32 = ROW_WALL + (ROW_H + GAP);
const ROW_BTN: i32 = ROW_TIME + (ROW_H + GAP);
const ROW_GYRO: i32 = ROW_BTN + (ROW_H + GAP);

pub struct SettingsApp {
    wifi_on: bool,
    ble_on: bool,
    cpu_mhz: u16,
    brightness: u8,
    manual_hour: u8,
    manual_minute: u8,
    manual_dirty: bool,
    wallpaper_names: Vec<String>,
    wallpaper_index: usize,
    req_wifi_toggle: bool,
    req_ble_toggle: bool,
    req_cpu_cycle: bool,
    req_brightness: Option<u8>,
    req_apply_time: bool,
    req_open_gyro: bool,
    req_wallpaper: Option<usize>,
}

impl SettingsApp {
    pub fn new() -> Self {
        Self {
            wifi_on: false,
            ble_on: false,
            cpu_mhz: 160,
            brightness: 0xA0,
            manual_hour: 12,
            manual_minute: 0,
            manual_dirty: false,
            wallpaper_names: Vec::new(),
            wallpaper_index: 0,
            req_wifi_toggle: false,
            req_ble_toggle: false,
            req_cpu_cycle: false,
            req_brightness: None,
            req_apply_time: false,
            req_open_gyro: false,
            req_wallpaper: None,
        }
    }

    pub fn update(&mut self, _dt_ms: u32) {}

    pub fn set_runtime_state(&mut self, wifi_on: bool, ble_on: bool, cpu_mhz: u16, brightness: u8) {
        self.wifi_on = wifi_on;
        self.ble_on = ble_on;
        self.cpu_mhz = cpu_mhz;
        self.brightness = brightness;
    }

    pub fn set_current_time(&mut self, hour: u8, minute: u8) {
        if !self.manual_dirty {
            self.manual_hour = hour.min(23);
            self.manual_minute = minute.min(59);
        }
    }

    pub fn set_wallpaper_options(&mut self, names: Vec<String>) {
        self.wallpaper_names = names;
        if self.wallpaper_index >= self.wallpaper_names.len() {
            self.wallpaper_index = 0;
        }
    }

    pub fn set_wallpaper_index(&mut self, idx: usize) {
        if self.wallpaper_names.is_empty() {
            self.wallpaper_index = 0;
            return;
        }
        self.wallpaper_index = idx.min(self.wallpaper_names.len() - 1);
    }

    pub fn wallpaper_index(&self) -> usize { self.wallpaper_index }

    pub fn handle_tap(&mut self, x: u16, y: u16) -> bool {
        let xi = x as i32;
        let yi = y as i32;

        if in_row(yi, ROW_WIFI) {
            self.req_wifi_toggle = true;
            self.wifi_on = !self.wifi_on;
            return true;
        }
        if in_row(yi, ROW_BLE) {
            self.req_ble_toggle = true;
            self.ble_on = !self.ble_on;
            return true;
        }
        if in_row(yi, ROW_CPU) {
            self.req_cpu_cycle = true;
            self.cpu_mhz = match self.cpu_mhz { 80 => 160, 160 => 240, _ => 80 };
            return true;
        }
        if in_row(yi, ROW_BRI) {
            let new_bri = if xi < W / 2 {
                self.brightness.saturating_sub(16).max(0x10)
            } else {
                self.brightness.saturating_add(16).min(0xFF)
            };
            self.brightness = new_bri;
            self.req_brightness = Some(new_bri);
            return true;
        }
        if in_row(yi, ROW_WALL) {
            if !self.wallpaper_names.is_empty() {
                self.wallpaper_index = (self.wallpaper_index + 1) % self.wallpaper_names.len();
                self.req_wallpaper = Some(self.wallpaper_index);
            }
            return true;
        }
        if in_row(yi, ROW_TIME) {
            self.manual_dirty = true;
            if xi < 145 {
                self.manual_hour = (self.manual_hour + 23) % 24;
            } else if xi < 270 {
                self.manual_hour = (self.manual_hour + 1) % 24;
            } else if xi < 340 {
                self.manual_minute = (self.manual_minute + 59) % 60;
            } else {
                self.manual_minute = (self.manual_minute + 1) % 60;
            }
            return true;
        }
        if in_row(yi, ROW_BTN) {
            self.req_apply_time = true;
            return true;
        }
        if in_row(yi, ROW_GYRO) {
            self.req_open_gyro = true;
            return true;
        }
        false
    }

    pub fn take_wifi_toggle_request(&mut self) -> bool {
        let v = self.req_wifi_toggle;
        self.req_wifi_toggle = false;
        v
    }

    pub fn take_ble_toggle_request(&mut self) -> bool {
        let v = self.req_ble_toggle;
        self.req_ble_toggle = false;
        v
    }

    pub fn take_cpu_cycle_request(&mut self) -> bool {
        let v = self.req_cpu_cycle;
        self.req_cpu_cycle = false;
        v
    }

    pub fn take_brightness_request(&mut self) -> Option<u8> {
        self.req_brightness.take()
    }

    pub fn take_open_gyro_request(&mut self) -> bool {
        let v = self.req_open_gyro;
        self.req_open_gyro = false;
        v
    }

    pub fn take_apply_time_request(&mut self) -> Option<(u8, u8)> {
        if self.req_apply_time {
            self.req_apply_time = false;
            self.manual_dirty = false;
            Some((self.manual_hour, self.manual_minute))
        } else {
            None
        }
    }

    pub fn take_wallpaper_request(&mut self) -> Option<usize> {
        self.req_wallpaper.take()
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(EgPoint::zero(), Size::new(W as u32, H as u32))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::new(1, 2, 2)))
            .draw(d);

        let title = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
        let row_label = MonoTextStyle::new(&FONT_8X13, Rgb565::CSS_GRAY);
        let row_val = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let row_hint = MonoTextStyle::new(&FONT_8X13, Rgb565::new(12, 20, 24));

        let _ = Text::with_alignment("SETTINGS", EgPoint::new(W / 2, 34), title, Alignment::Center).draw(d);

        draw_row(d, ROW_WIFI, "WiFi", if self.wifi_on { "ON" } else { "OFF" }, self.wifi_on);
        draw_row(d, ROW_BLE, "Bluetooth", if self.ble_on { "ON" } else { "OFF" }, self.ble_on);
        draw_row(d, ROW_CPU, "CPU", match self.cpu_mhz { 80 => "80MHz", 240 => "240MHz", _ => "160MHz" }, true);

        draw_row_bg(d, ROW_BRI, Rgb565::new(2, 4, 8));
        let _ = Text::new("Brightness", EgPoint::new(X + 10, ROW_BRI + 14), row_label).draw(d);
        let mut bri_buf = [0u8; 8];
        let bri_s = fmt_pct(&mut bri_buf, self.brightness);
        let _ = Text::with_alignment(bri_s, EgPoint::new(W - 92, ROW_BRI + 25), row_val, Alignment::Center).draw(d);
        let _ = Text::new("- tap left / + tap right", EgPoint::new(X + 10, ROW_BRI + 30), row_hint).draw(d);

        draw_row_bg(d, ROW_WALL, Rgb565::new(3, 3, 8));
        let _ = Text::new("Wallpaper", EgPoint::new(X + 10, ROW_WALL + 14), row_label).draw(d);
        let wall = if self.wallpaper_names.is_empty() {
            "sem imagens"
        } else {
            self.wallpaper_names[self.wallpaper_index].as_str()
        };
        let _ = Text::new(wall, EgPoint::new(X + 10, ROW_WALL + 30), row_val).draw(d);

        draw_row_bg(d, ROW_TIME, Rgb565::new(8, 4, 0));
        let _ = Text::new("Hora manual", EgPoint::new(X + 10, ROW_TIME + 14), row_label).draw(d);
        let mut tbuf = [0u8; 6];
        let ts = fmt_hhmm(&mut tbuf, self.manual_hour, self.manual_minute);
        let _ = Text::with_alignment(ts, EgPoint::new(W - 72, ROW_TIME + 25), row_val, Alignment::Center).draw(d);
        let _ = Text::new("H- H+ M- M+ (esq -> dir)", EgPoint::new(X + 10, ROW_TIME + 30), row_hint).draw(d);

        draw_row_bg(d, ROW_BTN, Rgb565::new(2, 10, 8));
        let _ = Text::with_alignment("APPLY MANUAL TIME", EgPoint::new(W / 2, ROW_BTN + 24), row_val, Alignment::Center).draw(d);

        draw_row_bg(d, ROW_GYRO, Rgb565::new(4, 8, 4));
        let _ = Text::with_alignment("OPEN GYRO MENU", EgPoint::new(W / 2, ROW_GYRO + 24), row_val, Alignment::Center).draw(d);
    }
}

fn in_row(y: i32, row_top: i32) -> bool {
    y >= row_top && y < row_top + ROW_H
}

fn draw_row_bg<D: DrawTarget<Color = Rgb565>>(d: &mut D, y: i32, bg: Rgb565) {
    let _ = RoundedRectangle::with_equal_corners(
        Rectangle::new(EgPoint::new(X, y), Size::new(ROW_W as u32, ROW_H as u32)),
        Size::new(8, 8),
    )
    .into_styled(PrimitiveStyle::with_fill(bg))
    .draw(d);
}

fn draw_row<D: DrawTarget<Color = Rgb565>>(d: &mut D, y: i32, label: &str, value: &str, active: bool) {
    let bg = if active { Rgb565::new(2, 10, 12) } else { Rgb565::new(3, 4, 5) };
    draw_row_bg(d, y, bg);
    let row_label = MonoTextStyle::new(&FONT_8X13, Rgb565::CSS_GRAY);
    let row_val = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    let _ = Text::new(label, EgPoint::new(X + 10, y + 14), row_label).draw(d);
    let _ = Text::with_alignment(value, EgPoint::new(W - 38, y + 25), row_val, Alignment::Center).draw(d);
}

fn fmt_hhmm<'a>(buf: &'a mut [u8; 6], h: u8, m: u8) -> &'a str {
    buf[0] = b'0' + (h / 10);
    buf[1] = b'0' + (h % 10);
    buf[2] = b':';
    buf[3] = b'0' + (m / 10);
    buf[4] = b'0' + (m % 10);
    core::str::from_utf8(&buf[..5]).unwrap_or("00:00")
}

fn fmt_pct<'a>(buf: &'a mut [u8; 8], v: u8) -> &'a str {
    let pct = ((v as u16 * 100) / 255) as u8;
    let mut p = 0;
    if pct >= 100 {
        buf[p] = b'1'; p += 1;
        buf[p] = b'0'; p += 1;
        buf[p] = b'0'; p += 1;
    } else {
        if pct >= 10 {
            buf[p] = b'0' + pct / 10;
            p += 1;
        }
        buf[p] = b'0' + pct % 10;
        p += 1;
    }
    buf[p] = b'%';
    p += 1;
    core::str::from_utf8(&buf[..p]).unwrap_or("0%")
}
