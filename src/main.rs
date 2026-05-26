mod vm;

use vm::cpu::EncInst;
use vm::{VM, Word};

use macroquad::prelude::*;

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

    let mut vm = VM::new(&render_target, &camera);

    vm.load(&[
        EncInst::new_lw(1, 0, 7).into(),   // 0: r1 = memory[0+7] = count (5)
        EncInst::new_lw(2, 1, 3).into(),   // 1: r2 = memory[r1+2] = memory[8] = -1 (neg1)
        EncInst::new_add(1, 1, 2).into(),  // 2 (start): r1 = r1 + r2 (decrement)
        EncInst::new_beq(0, 1, 1).into(),  // 3: if r1==0, jump to done (addr 5)
        EncInst::new_beq(0, 0, -3).into(), // 4: jump to start (addr 2)
        EncInst::new_addi(7, 0, 6).into(), // 5: r7 = r0 + 6 (i.e. 6)
        EncInst::new_jalr(0, 7).into(),    // 6: halt (just loop forever)
        Word::from(5i16),                  // 7: (count): .fill 5
        Word::from(-1i16),                 // 8: (neg1): .fill -1
    ]);

    let mut i = 0;
    loop {
        i += 1;
        vm.step();
        next_frame().await;
        if i % 120 == 0 {
            print!("---{}---\n{}\n", i, vm);
        }
    }
}
