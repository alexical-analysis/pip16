use std::fmt::{Display, Formatter, Result};

use crate::vm::MemoryBank;

use macroquad::prelude::*;

pub struct PPU<'a> {
    render_target: &'a RenderTarget,
    camera: &'a Camera2D,
}

impl Display for PPU<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[ PPU ]\n")
    }
}

impl<'a> PPU<'a> {
    pub fn new(render_target: &'a RenderTarget, camera: &'a Camera2D) -> Self {
        Self {
            render_target,
            camera,
        }
    }

    pub fn step(&self, vidoe_memory: &mut MemoryBank) {
        set_camera(self.camera);
        clear_background(BLACK);
        draw_rectangle(10.0, 10.0, 20.0, 20.0, RED);
        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);

        // Scale up to the window
        set_default_camera();
        draw_texture_ex(
            &self.render_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true, // macroquad render targets are flipped
                ..Default::default()
            },
        );
    }
}
