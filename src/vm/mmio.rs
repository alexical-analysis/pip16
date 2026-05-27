use crate::vm::Word;

/// the interrupt vector is where the CPU jumps to handle an interrupt.
pub const INTERRUPT_VECTOR: Word = Word::new_const(0xFF_FD);

/// the base address of the MMIO Register memory space
const BASE: i32 = 0x02_00;
pub const MMIO_BASE: Word = Word::new_const(BASE);

/// interrupt program counter stores the program counter before the cpu jumps to the interrupt vector
pub const INTERRUPT_PROGRAM_COUNTER: Word = Word::new_const(BASE + 0x00_00);

/// the signed x-position of where to draw the sprite
pub const SPR_X_POS: Word = Word::new_const(BASE + 0x00_01);

/// the signed y-position of where to draw the sprite
pub const SPR_Y_POS: Word = Word::new_const(BASE + 0x00_02);

/// the unsigned id for the 8x8 sprite in the sprite map
pub const SPR_ID: Word = Word::new_const(BASE + 0x00_03);

/// the ppu control register
pub const PPU_CONTROL: Word = Word::new_const(BASE + 0x00_04);

/// the sie of the sprite to draw
pub const SPR_SIZE: Word = Word::new_const(BASE + 0x00_05);

/// controller input register
pub const CONTROLLER_INPUT: Word = Word::new_const(BASE + 0x00_06);

/// cycle count high is the upper word of the 32-bit cycle counter
pub const CYCLE_COUNT_HIGH: Word = Word::new_const(BASE + 0x00_07);

/// cycle count low is the lower word of the 32-bit cycle counter
pub const CYCLE_COUNT_LOW: Word = Word::new_const(BASE + 0x00_08);

/// halt control to turn the CPU off until the start of the next frame
pub const HALT_CONTROL: Word = Word::new_const(BASE + 0x00_09);
