mod cpu;
mod inst;

pub use cpu::*;
pub use inst::*;

use crate::vm::mmio::{HALT_CONTROL, PPU_CONTROL, SPR_X_POS, SPR_Y_POS};
use crate::vm::{MemoryBank, Word};

pub fn test_gfx_prgm() -> Vec<Word> {
    vec![
        // jump over the data section to program start (r1 is 0 at boot)
        DecodedInst::Lli {
            r_a: 1,
            imm: Word::new_const(8),
        }
        .encode()
        .into(), // 00: r1 = 8
        DecodedInst::Jalr {
            r_a: 0,
            r_b: 1,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 01: jump to r1 (addr 8)
        // data section in the header
        Word::new_const(20), // 02: .fill 20 (initial x)
        Word::new_const(30), // 03: .fill 30 (initial y)
        SPR_X_POS,           // 04: .fill SPR_X_POS
        SPR_Y_POS,           // 05: .fill SPR_Y_POS
        PPU_CONTROL,         // 06: .fill PPU_CONTROL
        HALT_CONTROL,        // 07: .fill HALT_CONTROL
        // program start: load initial x/y into r1/r2
        DecodedInst::Lw {
            r_a: 1,
            r_b: 0,
            imm: Word::new_const(2),
        }
        .encode()
        .into(), // 08: r1 = mem[r0+2] = 20
        DecodedInst::Lw {
            r_a: 2,
            r_b: 0,
            imm: Word::new_const(3),
        }
        .encode()
        .into(), // 09: r2 = mem[r0+3] = 30
        // init constants that persist across loop iterations (all regs are 0 at boot)
        DecodedInst::Lli {
            r_a: 6,
            imm: Word::new_const(3),
        }
        .encode()
        .into(), // 10: r6 = 3 (PPU clear+draw)
        DecodedInst::Lli {
            r_a: 7,
            imm: Word::new_const(1),
        }
        .encode()
        .into(), // 11: r7 = 1 (increment)
        DecodedInst::Lli {
            r_a: 8,
            imm: Word::new_const(13),
        }
        .encode()
        .into(), // 12: r8 = 13 (loop start addr)
        // loop start (addr 13): load MMIO addresses from data section
        DecodedInst::Lw {
            r_a: 3,
            r_b: 0,
            imm: Word::new_const(4),
        }
        .encode()
        .into(), // 13: r3 = SPR_X_POS addr
        DecodedInst::Lw {
            r_a: 4,
            r_b: 0,
            imm: Word::new_const(5),
        }
        .encode()
        .into(), // 14: r4 = SPR_Y_POS addr
        DecodedInst::Lw {
            r_a: 5,
            r_b: 0,
            imm: Word::new_const(6),
        }
        .encode()
        .into(), // 15: r5 = PPU_CONTROL addr
        // write x/y position and issue PPU draw command
        DecodedInst::Sw {
            r_a: 1,
            r_b: 3,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 16: mem[SPR_X_POS] = r1
        DecodedInst::Sw {
            r_a: 2,
            r_b: 4,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 17: mem[SPR_Y_POS] = r2
        DecodedInst::Sw {
            r_a: 6,
            r_b: 5,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 18: mem[PPU_CONTROL] = 3
        // increment sprite position for next frame
        DecodedInst::Add {
            r_a: 1,
            r_b: 1,
            r_c: 7,
        }
        .encode()
        .into(), // 19: r1 += r7 (x_pos++)
        DecodedInst::Add {
            r_a: 2,
            r_b: 2,
            r_c: 7,
        }
        .encode()
        .into(), // 20: r2 += r7 (y_pos++)
        // halt the CPU for the rest of the frame
        DecodedInst::Lw {
            r_a: 9,
            r_b: 0,
            imm: Word::new_const(7),
        }
        .encode()
        .into(), // 21: r9 = HALT_CONTROL addr
        DecodedInst::Sw {
            r_a: 7,
            r_b: 9,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 22: mem[HALT_CONTROL] = 1
        // on wake, jump back to loop start
        DecodedInst::Jalr {
            r_a: 0,
            r_b: 8,
            imm: Word::ZERO,
        }
        .encode()
        .into(), // 23: jump to r8 (addr 13)
    ]
}
