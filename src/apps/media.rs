use alloc::string::String;
use alloc::vec::Vec;

use embedded_graphics::geometry::Point as EgPoint;
use embedded_graphics::mono_font::ascii::{FONT_8X13, FONT_10X20};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::apps::{App, AppInput, AppResult};
use crate::peripherals::touch::SwipeDirection;

const W: i32 = 410;
const H: i32 = 502;
const LIST_X: i32 = 20;
const LIST_Y: i32 = 86;
const LIST_W: i32 = W - 40;
const ITEM_H: i32 = 44;
const MAX_VISIBLE: usize = 8;

pub struct MediaApp {
    images: Vec<String>,
    selected: usize,
    scroll: usize,
}

impl MediaApp {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            selected: 0,
            scroll: 0,
        }
    }

    pub fn set_images(&mut self, images: Vec<String>) {
        self.images = images;
        if self.selected >= self.images.len() {
            self.selected = 0;
        }
        self.scroll = self.selected.min(self.images.len().saturating_sub(1));
    }

    pub fn set_selected_index(&mut self, idx: usize) {
        if self.images.is_empty() {
            self.selected = 0;
            self.scroll = 0;
            return;
        }
        self.selected = idx.min(self.images.len() - 1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + MAX_VISIBLE {
            self.scroll = self.selected + 1 - MAX_VISIBLE;
        }
    }

    pub fn selected_index(&self) -> usize { self.selected }
    pub fn image_count(&self) -> usize { self.images.len() }

    fn move_selection(&mut self, down: bool) {
        if self.images.is_empty() {
            return;
        }
        if down {
            self.selected = (self.selected + 1).min(self.images.len() - 1);
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + MAX_VISIBLE {
            self.scroll = self.selected + 1 - MAX_VISIBLE;
        }
    }
}

impl App for MediaApp {
    fn name(&self) -> &str { "Media" }

    fn setup(&mut self) {}

    fn update(&mut self, input: &AppInput) -> AppResult {
        if let Some(swipe) = input.swipe {
            match swipe {
                SwipeDirection::Up => self.move_selection(true),
                SwipeDirection::Down => self.move_selection(false),
                _ => {}
            }
        }
        AppResult::Continue
    }

    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(EgPoint::zero(), Size::new(W as u32, H as u32))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(d);

        let title = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
        let white = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let dim = MonoTextStyle::new(&FONT_8X13, Rgb565::CSS_GRAY);
        let row_text = MonoTextStyle::new(&FONT_8X13, Rgb565::WHITE);

        let _ = Text::with_alignment("MEDIA SD", EgPoint::new(W / 2, 34), title, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Imagens para wallpaper", EgPoint::new(W / 2, 54), dim, Alignment::Center).draw(d);

        if self.images.is_empty() {
            let _ = Text::with_alignment("Nenhuma imagem encontrada", EgPoint::new(W / 2, H / 2), white, Alignment::Center).draw(d);
            let _ = Text::with_alignment("Use /media, /wallpaper ou /images no SD", EgPoint::new(W / 2, H / 2 + 24), dim, Alignment::Center).draw(d);
        } else {
            for i in 0..MAX_VISIBLE {
                let idx = self.scroll + i;
                if idx >= self.images.len() { break; }
                let y = LIST_Y + i as i32 * (ITEM_H + 4);
                let active = idx == self.selected;
                let bg = if active { Rgb565::new(4, 12, 16) } else { Rgb565::new(2, 4, 6) };
                let _ = RoundedRectangle::with_equal_corners(
                    Rectangle::new(EgPoint::new(LIST_X, y), Size::new(LIST_W as u32, ITEM_H as u32)),
                    Size::new(8, 8),
                ).into_styled(PrimitiveStyle::with_fill(bg)).draw(d);

                let name = self.images[idx].as_str();
                let _ = Text::new(name, EgPoint::new(LIST_X + 10, y + 16), row_text).draw(d);
                if active {
                    let _ = Text::new("selecionada", EgPoint::new(LIST_X + 10, y + 32), dim).draw(d);
                }
            }
        }

        let _ = Text::with_alignment("Swipe ↑↓ para navegar", EgPoint::new(W / 2, H - 34), dim, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Escolha no Settings > Wallpaper", EgPoint::new(W / 2, H - 18), dim, Alignment::Center).draw(d);
    }
}
