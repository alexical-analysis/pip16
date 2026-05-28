mod vm;

use std::time::{Duration, Instant};

use macroquad::prelude::*;

use vm::VM;

use crate::vm::riscp::test_gfx_prgm;

const TARGET_FRAME_TIME: Duration = Duration::from_micros(16_667); // ~60fps

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 144;
const SCREEN_SCALE: u32 = 4;

fn window_conf() -> Conf {
    Conf {
        window_title: "pip16".to_string(),
        window_width: (SCREEN_WIDTH * SCREEN_SCALE) as i32,
        window_height: (SCREEN_HEIGHT * SCREEN_SCALE) as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // setup the render target
    let render_target = render_target(SCREEN_WIDTH, SCREEN_HEIGHT);
    render_target.texture.set_filter(FilterMode::Nearest);

    // set up the 2d camera so that we scale the immage correctly
    let mut camera = Camera2D::from_display_rect(Rect::new(
        0.0,
        0.0,
        SCREEN_WIDTH as f32,
        SCREEN_HEIGHT as f32,
    ));
    camera.render_target = Some(render_target.clone());

    // snag the sprite sheet
    let sprite_sheet = load_texture("assets/tilemap_packed.png").await.unwrap();
    sprite_sheet.set_filter(FilterMode::Nearest);

    let mut vm = VM::new(sprite_sheet);

    // test program
    let prgm = test_gfx_prgm();
    vm.load(&prgm);

    loop {
        let frame_start = Instant::now();

        set_camera(&camera);
        vm.step_frame();

        set_default_camera();
        draw_texture_ex(
            &render_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true, // macroquad render targets are flipped
                ..Default::default()
            },
        );

        draw_text(&format!("{:.0} fps", get_fps()), 5.0, 15.0, 16.0, BLACK);
        next_frame().await;

        // lock the maximum frame rate for that juci retro feel
        let elapsed = frame_start.elapsed();
        if elapsed < TARGET_FRAME_TIME {
            // sleep is not precise and so we need to sleep slightly less than the remaining time
            let sleep_time = TARGET_FRAME_TIME - elapsed;
            if sleep_time > Duration::from_millis(2) {
                std::thread::sleep(sleep_time - Duration::from_millis(2));
            }

            // then we just spin the CPU for the last few milllis since that's much more accurate
            let sleep_until = frame_start + TARGET_FRAME_TIME;
            while Instant::now() < sleep_until {}
        }
    }
}
