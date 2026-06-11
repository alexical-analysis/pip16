use std::fmt::{Display, Formatter, Result};

use crate::vm::mmio::{
    BG_CONTROL, INT_ENABLE, INT_PPU_DONE, INT_STATUS, SPR_ATTR, SPR_CONTROL, SPR_TILE, SPR_X,
    SPR_Y,
};
use crate::vm::{BankedMemory, Word};

use macroquad::prelude::*;

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

    /// Steps the PPU. Returns the number of CPU cycles consumed this step.
    pub fn step(&self, mem: &mut BankedMemory) -> usize {
        let spr_ctrl: u16 = mem.load_mmio(SPR_CONTROL).into();
        if spr_ctrl & 0x01 == 0 {
            return 0;
        }

        mem.store_mmio(SPR_CONTROL, Word::ZERO);

        let mut cost = 0usize;

        // Background draw (stub — no actual render yet)
        let bg_ctrl: u16 = mem.load_mmio(BG_CONTROL).into();
        if bg_ctrl & 0x01 != 0 {
            cost += 64;
        }

        // Sprite draw
        let x_pos: i16 = mem.load_mmio(SPR_X).into();
        let y_pos: i16 = mem.load_mmio(SPR_Y).into();
        let tile_id: u16 = mem.load_mmio(SPR_TILE).into();
        let tile_id = tile_id & 0x1FF; // bits 0-8

        let attr: u16 = mem.load_mmio(SPR_ATTR).into();
        let size_idx = attr & 0x3; // bits 0-1
        let h_flip = (attr >> 2) & 0x1 != 0; // bit 2
        let v_flip = (attr >> 3) & 0x1 != 0; // bit 3
        // bits 4-7: tile data bank — ignored while using texture approach
        let rotation_idx = (attr >> 8) & 0x3; // bits 8-9

        let pixel_size = 8u32 << size_idx; // 8, 16, 32, or 64

        let tile_cols = self.sprite_sheet.width() as u32 / 8;
        let tile_x = tile_id as u32 % tile_cols;
        let tile_y = tile_id as u32 / tile_cols;

        let rotation = match rotation_idx {
            1 => std::f32::consts::FRAC_PI_2,
            2 => std::f32::consts::PI,
            3 => 3.0 * std::f32::consts::FRAC_PI_2,
            _ => 0.0,
        };

        draw_texture_ex(
            &self.sprite_sheet,
            x_pos as f32,
            y_pos as f32,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    (tile_x * 8) as f32,
                    (tile_y * 8) as f32,
                    pixel_size as f32,
                    pixel_size as f32,
                )),
                rotation,
                flip_x: h_flip,
                flip_y: v_flip,
                pivot: Some(vec2(
                    x_pos as f32 + pixel_size as f32 / 2.0,
                    y_pos as f32 + pixel_size as f32 / 2.0,
                )),
                ..Default::default()
            },
        );

        cost += 16;

        // Set PPU Done interrupt status if enabled
        let int_enable: u16 = mem.load_mmio(INT_ENABLE).into();
        if int_enable & INT_PPU_DONE != 0 {
            let status: u16 = mem.load_mmio(INT_STATUS).into();
            mem.store_mmio(INT_STATUS, Word::from(status | INT_PPU_DONE));
        }

        cost
    }
}
