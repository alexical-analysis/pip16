use std::fmt::{Display, Formatter, Result};

use crate::vm::mmio::{PPU_CONTROL, SPR_ID, SPR_SIZE, SPR_X_POS, SPR_Y_POS};
use crate::vm::{MemoryBank, Word};

use macroquad::prelude::*;

const CLEAR_DISP_MASK: u16 = 0b0000_0000_0000_0001;
const DRAW_SPR_MASK: u16 = 0b0000_0000_0000_0010;

pub struct PPU {
    sprite_sheet: Texture2D,
}

impl Display for PPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[ PPU ]\n")
    }
}

impl PPU {
    pub fn new(sprite_sheet: Texture2D) -> Self {
        Self { sprite_sheet }
    }

    pub fn step(&self, mem: &mut MemoryBank) {
        let ppu_control: u16 = mem.load_word(PPU_CONTROL).into();
        if ppu_control == 0 {
            // if the ppu control register is not set there's nothing for the PPU to draw this cycle.
            return;
        }

        // reset the ppu control register
        mem.store_word(PPU_CONTROL, Word::ZERO);

        if ppu_control & CLEAR_DISP_MASK > 0 {
            clear_background(WHITE);
        }

        if ppu_control & DRAW_SPR_MASK > 0 {
            let x_pos: i16 = mem.load_word(SPR_X_POS).into();
            let y_pos: i16 = mem.load_word(SPR_Y_POS).into();
            let id: u16 = mem.load_word(SPR_ID).into();
            let scale: u16 = mem.load_word(SPR_SIZE).into();
            let scale = scale + 1;

            let pixel_width = self.sprite_sheet.width();
            let tile_width = pixel_width as u32 / 8;
            let tile_x = id as u32 % tile_width;
            let tile_y = id as u32 / tile_width;

            let rect = Rect::new(
                (tile_x * 8) as f32,
                (tile_y * 8) as f32,
                (scale * 8) as f32,
                (scale * 8) as f32,
            );
            draw_texture_ex(
                &self.sprite_sheet,
                x_pos as f32,
                y_pos as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(rect),
                    ..Default::default()
                },
            );
        }
    }
}
