use crate::vm::Word;

const BASE: i32 = 0xFF_C0;

pub const SYSTEM_CONTROL: Word = Word::new_const(BASE + 0x00); // BIOS-only
pub const BANK_CONTROL: Word = Word::new_const(BASE + 0x01);
pub const INT_ENABLE: Word = Word::new_const(BASE + 0x02);
pub const INT_STATUS: Word = Word::new_const(BASE + 0x03);
pub const DISPLAY_CONTROL: Word = Word::new_const(BASE + 0x04);
pub const DISPLAY_VCOUNT: Word = Word::new_const(BASE + 0x05);
pub const SPR_X: Word = Word::new_const(BASE + 0x06);
pub const SPR_Y: Word = Word::new_const(BASE + 0x07);
pub const SPR_TILE: Word = Word::new_const(BASE + 0x08);
pub const SPR_ATTR: Word = Word::new_const(BASE + 0x09);
pub const SPR_CONTROL: Word = Word::new_const(BASE + 0x0A);
pub const BG_SCROLL_X: Word = Word::new_const(BASE + 0x0B);
pub const BG_SCROLL_Y: Word = Word::new_const(BASE + 0x0C);
pub const BG_CONTROL: Word = Word::new_const(BASE + 0x0D);
pub const BUTTON_STATE: Word = Word::new_const(BASE + 0x0E);
pub const APU_CONTROL: Word = Word::new_const(BASE + 0x0F);
pub const APU_CH0_FREQ: Word = Word::new_const(BASE + 0x10);
pub const APU_CH0_CTRL: Word = Word::new_const(BASE + 0x11);
pub const APU_CH1_FREQ: Word = Word::new_const(BASE + 0x12);
pub const APU_CH1_CTRL: Word = Word::new_const(BASE + 0x13);
pub const APU_CH2_FREQ: Word = Word::new_const(BASE + 0x14);
pub const APU_CH2_CTRL: Word = Word::new_const(BASE + 0x15);
pub const APU_CH2_ADDR: Word = Word::new_const(BASE + 0x16);
pub const APU_CH2_LEN: Word = Word::new_const(BASE + 0x17);
pub const APU_CH3_FREQ: Word = Word::new_const(BASE + 0x18);
pub const APU_CH3_CTRL: Word = Word::new_const(BASE + 0x19);
pub const HALT_CONTROL: Word = Word::new_const(BASE + 0x1A); // reserved slot, internal use

// Interrupt enable/status bit masks
pub const INT_VBLANK: u16 = 1 << 0;
pub const INT_HBLANK: u16 = 1 << 1;
pub const INT_PPU_DONE: u16 = 1 << 2;
pub const INT_APU_DONE: u16 = 1 << 3;
pub const INT_BUTTON: u16 = 1 << 4;
