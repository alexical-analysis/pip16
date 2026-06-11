mod cpu;
mod inst;

pub use cpu::*;
pub use inst::*;

use crate::vm::mmio::{HALT_CONTROL, SPR_CONTROL, SPR_X, SPR_Y};
use crate::vm::Word;

/// Builds a test ROM padded to 0xB7C0 words. The last 3 words (0xB7BD–0xB7BF)
/// hold the interrupt dispatch sequence; the real handler is at index 24 and
/// ends with RETL.
pub fn test_gfx_prgm() -> Vec<Word> {
    // Interrupt dispatch at 0xB7BD: LUI r15, 0x00 / LLI r15, 24 / JALR r0, r15
    // Handler at index 24: just RETL (resume immediately)
    let dispatch_lui = DecodedInst::Lui {
        r_a: 15,
        imm: Word::ZERO,
    }
    .encode()
    .into();
    let dispatch_lli = DecodedInst::Lli {
        r_a: 15,
        imm: Word::new_const(24),
    }
    .encode()
    .into();
    let dispatch_jalr = DecodedInst::Jalr {
        r_a: 0,
        r_b: 15,
        imm: Word::ZERO,
    }
    .encode()
    .into();

    let mut rom = vec![Word::ZERO; 0xB7C0];

    // --- 00: jump over data section to program start ---
    rom[0] = DecodedInst::Lli {
        r_a: 1,
        imm: Word::new_const(8),
    }
    .encode()
    .into(); // r1 = 8
    rom[1] = DecodedInst::Jalr {
        r_a: 0,
        r_b: 1,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // jump to addr 8

    // --- 02–07: data section ---
    rom[2] = Word::new_const(20); // initial x
    rom[3] = Word::new_const(30); // initial y
    rom[4] = SPR_X;
    rom[5] = SPR_Y;
    rom[6] = SPR_CONTROL;
    rom[7] = HALT_CONTROL;

    // --- 08–11: one-time init ---
    rom[8] = DecodedInst::Lw {
        r_a: 1,
        r_b: 0,
        imm: Word::new_const(2),
    }
    .encode()
    .into(); // r1 = 20 (x)
    rom[9] = DecodedInst::Lw {
        r_a: 2,
        r_b: 0,
        imm: Word::new_const(3),
    }
    .encode()
    .into(); // r2 = 30 (y)
    rom[10] = DecodedInst::Lli {
        r_a: 6,
        imm: Word::new_const(1),
    }
    .encode()
    .into(); // r6 = 1 (SPR_CONTROL draw bit)
    rom[11] = DecodedInst::Lli {
        r_a: 7,
        imm: Word::new_const(1),
    }
    .encode()
    .into(); // r7 = 1 (increment)
    rom[12] = DecodedInst::Lli {
        r_a: 8,
        imm: Word::new_const(13),
    }
    .encode()
    .into(); // r8 = 13 (loop start)

    // --- 13–22: main loop ---
    rom[13] = DecodedInst::Lw {
        r_a: 3,
        r_b: 0,
        imm: Word::new_const(4),
    }
    .encode()
    .into(); // r3 = SPR_X addr
    rom[14] = DecodedInst::Lw {
        r_a: 4,
        r_b: 0,
        imm: Word::new_const(5),
    }
    .encode()
    .into(); // r4 = SPR_Y addr
    rom[15] = DecodedInst::Lw {
        r_a: 5,
        r_b: 0,
        imm: Word::new_const(6),
    }
    .encode()
    .into(); // r5 = SPR_CONTROL addr
    rom[16] = DecodedInst::Sw {
        r_a: 1,
        r_b: 3,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // mem[SPR_X] = r1
    rom[17] = DecodedInst::Sw {
        r_a: 2,
        r_b: 4,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // mem[SPR_Y] = r2
    rom[18] = DecodedInst::Sw {
        r_a: 6,
        r_b: 5,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // mem[SPR_CONTROL] = 1
    rom[19] = DecodedInst::Add {
        r_a: 1,
        r_b: 1,
        r_c: 7,
    }
    .encode()
    .into(); // r1++ (x)
    rom[20] = DecodedInst::Add {
        r_a: 2,
        r_b: 2,
        r_c: 7,
    }
    .encode()
    .into(); // r2++ (y)
    rom[21] = DecodedInst::Lw {
        r_a: 9,
        r_b: 0,
        imm: Word::new_const(7),
    }
    .encode()
    .into(); // r9 = HALT_CONTROL addr
    rom[22] = DecodedInst::Sw {
        r_a: 7,
        r_b: 9,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // mem[HALT_CONTROL] = 1
    rom[23] = DecodedInst::Jalr {
        r_a: 0,
        r_b: 8,
        imm: Word::ZERO,
    }
    .encode()
    .into(); // jump to loop start

    // --- 24: interrupt handler (just return) ---
    rom[24] = DecodedInst::Retl.encode().into();

    // --- 0xB7BD–0xB7BF: interrupt dispatch sequence ---
    rom[0xB7BD] = dispatch_lui;
    rom[0xB7BE] = dispatch_lli;
    rom[0xB7BF] = dispatch_jalr;

    rom
}
