mod vm;

use std::time::{Duration, Instant};

use macroquad::prelude::*;

use vm::cpu::EncInst;
use vm::{VM, Word};

use crate::vm::cpu::CPU_HZ;
use crate::vm::mmio::{HALT_CONTROL, PPU_CONTROL, SPR_X_POS, SPR_Y_POS};

const TARGET_FRAME_TIME: Duration = Duration::from_micros(16_667); // ~60fps

const WIDTH: u32 = 256;
const HEIGHT: u32 = 144;
const SCALE: u32 = 4;

fn window_conf() -> Conf {
    Conf {
        window_title: "pip16".to_string(),
        window_width: (WIDTH * SCALE) as i32,
        window_height: (HEIGHT * SCALE) as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // setup the render target
    let render_target = render_target(WIDTH, HEIGHT);
    render_target.texture.set_filter(FilterMode::Nearest);

    // set up the 2d camera so that we scale the immage correctly
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32));
    camera.render_target = Some(render_target.clone());

    // snag the sprite sheet
    let sprite_sheet = load_texture("assets/tilemap_packed.png").await.unwrap();
    sprite_sheet.set_filter(FilterMode::Nearest);

    let mut vm = VM::new(sprite_sheet);
    // load_count_script(&mut vm);
    load_gfx_script(&mut vm);

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
            let sleep_until = frame_start + TARGET_FRAME_TIME;

            // sleep is not precise and so we need to sleep slightly less than the remaining time
            let sleep_time = TARGET_FRAME_TIME - elapsed;
            if sleep_time > Duration::from_millis(2) {
                std::thread::sleep(sleep_time - Duration::from_millis(2));
            }

            // then we just spin the CPU for the last few milllis since that's much more accurate
            while Instant::now() < sleep_until {}
        }
    }
}

fn load_count_script(vm: &mut VM) {
    vm.load(&[
        EncInst::new_lw(1, 0, 7).into(),   // 0: r1 = memory[0+7] = count (5)
        EncInst::new_lw(2, 1, 3).into(),   // 1: r2 = memory[r1+2(8)] = neg1 (-1)
        EncInst::new_add(1, 1, 2).into(),  // 2 (start): r1 = r1 + r2 (decrement)
        EncInst::new_beq(0, 1, 1).into(),  // 3: if r1==0, jump to done (addr 5)
        EncInst::new_beq(0, 0, -3).into(), // 4: jump to start (addr 2)
        EncInst::new_addi(7, 0, 6).into(), // 5: r7 = r0 + 6 (i.e. 6)
        EncInst::new_jalr(0, 7).into(),    // 6: halt (just loop forever)
        Word::from(5i16),                  // 7: (count): .fill 5
        Word::from(-1i16),                 // 8: (neg1): .fill -1
    ]);
}

fn load_gfx_script(vm: &mut VM) {
    vm.load(&[
        // jump over the data section to program start
        EncInst::new_addi(1, 0, 8).into(), // 00: r1 = r0(0) + 8
        EncInst::new_jalr(0, 1).into(),    // 01: jump to addr in r1 (16)
        // data setction in the header
        Word::new_const(20), // 02: .fill 20
        Word::new_const(30), // 03: .fill 30
        SPR_X_POS,           // 04: .fill SPR_X_POS (register address)
        SPR_Y_POS,           // 05: .fill SPR_Y_POS (register address)
        PPU_CONTROL,         // 06: .fill PPU_CONTROL (register address)
        HALT_CONTROL,        // 07: .file HALT_CONTROL (register address)
        // actual program start (r1 and r2 are the persistent x and y positions)
        EncInst::new_lw(1, 0, 2).into(), // 08: r1 = mem[r0(0) + 2] (20)
        EncInst::new_lw(2, 0, 3).into(), // 09: r2 = mem[r0(0) + 3] (30)
        // this is the loop start
        EncInst::new_lw(3, 0, 4).into(), // 10: r3 = mem[r0(0) + 4] (SPR_X_POS)
        EncInst::new_lw(4, 0, 5).into(), // 11: r4 = mem[r0(0) + 5] (SPR_Y_POS)
        EncInst::new_lw(5, 0, 6).into(), // 12: r5 = mem[r0(0) + 6] (PPU_CONTROL)
        EncInst::new_addi(6, 0, 3).into(), // 13: r6 = r0(0) + 3
        // set the sprite x/y position and tell the ppu to draw a sprite
        EncInst::new_sw(1, 3, 0).into(), // 14: mem[r3(SPR_X_POS) + 0] = r1(20)
        EncInst::new_sw(2, 4, 0).into(), // 15: mem[r4(SPR_Y_POS) + 0] = r2(30)
        EncInst::new_sw(6, 5, 0).into(), // 16: mem[r5(PPU_CONTROL) + 0] = r6(2) (clear screen + draw sprite)
        // increment the x and y position of the sprite
        EncInst::new_addi(1, 1, 1).into(), // 17: r1 = r1 + 1
        EncInst::new_addi(2, 2, 1).into(), // 18: r2 = r2 + 1
        // halt the CPU for the rest of the frame
        EncInst::new_addi(3, 0, 1).into(), // 19: r3 = r0(0) + 1
        EncInst::new_lw(4, 0, 7).into(),   // 20: r4 = mem[r0(0) + 7] (HALT_CONTROL)
        EncInst::new_sw(3, 4, 0).into(),   // 21: mem[r4(HALT_CONTROL) + 0] = r3(1)
        // on wake, execution resumes here, we jump back to the program start
        EncInst::new_addi(3, 0, 10).into(), // 22: r3 = r0(0) + 10
        EncInst::new_jalr(0, 3).into(),     // 23: jump to addr in r3 (10)
    ]);
}
