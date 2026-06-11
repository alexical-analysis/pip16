use std::fmt::{Display, Result};
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr, Sub};

use macroquad::prelude::{clear_background, BLACK};
use macroquad::texture::Texture2D;

use crate::vm::apu::APU;
use crate::vm::mmio::{BANK_CONTROL, HALT_CONTROL, INT_ENABLE, INT_STATUS, INT_VBLANK};
use crate::vm::ppu::PPU;
use crate::vm::riscp::CPU;

/// A single word in the memory space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(u16);

impl Word {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub const fn new_const(value: i32) -> Self {
        Self(value as u16)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "Wx{:04X}", self.0)
    }
}

impl From<i16> for Word {
    fn from(value: i16) -> Self {
        Word(value as u16)
    }
}

impl Into<i16> for Word {
    fn into(self) -> i16 {
        self.0 as i16
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value)
    }
}

impl Into<u16> for Word {
    fn into(self) -> u16 {
        self.0
    }
}

impl Into<usize> for Word {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Add for Word {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl Sub for Word {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl Mul for Word {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.wrapping_mul(rhs.0))
    }
}

impl BitXor for Word {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Shl for Word {
    type Output = Self;
    fn shl(self, rhs: Self) -> Self::Output {
        Self(self.0 << rhs.0)
    }
}

impl Shr for Word {
    type Output = Self;
    fn shr(self, rhs: Self) -> Self::Output {
        Self(self.0 >> rhs.0)
    }
}

impl BitOr for Word {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for Word {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl Not for Word {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl From<i8> for Word {
    fn from(value: i8) -> Self {
        Self(value as u16)
    }
}

const MAX_ADDRESS: usize = u16::MAX as usize;

pub struct MemoryBank([Word; MAX_ADDRESS + 1]);

impl MemoryBank {
    pub fn new() -> Self {
        Self([Word(0); MAX_ADDRESS + 1])
    }

    pub fn load(&mut self, data: &[Word]) {
        self.0[..data.len()].copy_from_slice(data);
    }

    pub fn load_word(&self, addr: Word) -> Word {
        let addr: usize = addr.into();
        if addr >= self.0.len() {
            unreachable!("address out of bounds")
        }
        self.0[addr]
    }

    pub fn store_word(&mut self, addr: Word, value: Word) {
        let addr: usize = addr.into();
        if addr >= self.0.len() {
            unreachable!("address out of bounds")
        }
        self.0[addr] = value
    }
}

/// Addresses at or above this boundary are in the mirrored RAM/MMIO region and
/// always resolve to the default bank, regardless of the active bank setting.
const MIRRORED_BASE: u16 = 0xBFC0;

pub struct BankedMemory {
    default: MemoryBank,
    ppu: MemoryBank,
    apu: MemoryBank,
}

impl BankedMemory {
    pub fn new() -> Self {
        Self {
            default: MemoryBank::new(),
            ppu: MemoryBank::new(),
            apu: MemoryBank::new(),
        }
    }

    fn active_bank(&self) -> u8 {
        let ctrl: u16 = self.default.load_word(BANK_CONTROL).into();
        (ctrl & 0x3) as u8
    }

    /// Instruction fetch — always reads from the default bank.
    pub fn fetch(&self, addr: Word) -> Word {
        self.default.load_word(addr)
    }

    /// Banked load (LW instruction). Addresses in the mirrored region always
    /// resolve to the default bank.
    pub fn load(&self, addr: Word) -> Word {
        let addr_u: u16 = addr.into();
        if addr_u >= MIRRORED_BASE {
            return self.default.load_word(addr);
        }
        match self.active_bank() {
            1 => self.ppu.load_word(addr),
            2 => self.apu.load_word(addr),
            _ => self.default.load_word(addr),
        }
    }

    /// Banked store (SW instruction). Addresses in the mirrored region always
    /// resolve to the default bank.
    pub fn store(&mut self, addr: Word, value: Word) {
        let addr_u: u16 = addr.into();
        if addr_u >= MIRRORED_BASE {
            self.default.store_word(addr, value);
            return;
        }
        match self.active_bank() {
            1 => self.ppu.store_word(addr, value),
            2 => self.apu.store_word(addr, value),
            _ => self.default.store_word(addr, value),
        }
    }

    /// Direct MMIO access for internal use by the VM, PPU, and APU. Always
    /// reads/writes the default bank; MMIO lives in the mirrored region.
    pub fn load_mmio(&self, addr: Word) -> Word {
        self.default.load_word(addr)
    }

    pub fn store_mmio(&mut self, addr: Word, value: Word) {
        self.default.store_word(addr, value);
    }

    /// Load cartridge data into the default bank starting at address 0.
    pub fn load_default(&mut self, data: &[Word]) {
        self.default.load(data);
    }
}

const CPU_HZ: usize = 15_360_000;

pub struct VM {
    cpu: CPU,
    ppu: PPU,
    apu: APU,
    mem: BankedMemory,
    first_frame: bool,
}

impl Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}\n", self.cpu)?;
        write!(f, "{}\n", self.ppu)?;

        for i in -3..3i32 {
            let pc: u16 = self.cpu.get_program_counter().into();
            let idx = pc as i32 + i;
            if idx < 0 || idx > u16::MAX as i32 {
                continue;
            }
            let w = self.mem.fetch(Word::from(idx as u16));
            if i == 0 {
                write!(f, ">  0x{:04X} : 0x{:04X}\n", idx, w.0)?;
            } else {
                write!(f, "  0x{:04X} : 0x{:04X}\n", idx, w.0)?;
            }
        }

        Ok(())
    }
}

impl VM {
    pub fn new(sprite_sheet: Texture2D) -> Self {
        Self {
            cpu: CPU::new(),
            ppu: PPU::new(sprite_sheet),
            apu: APU::new(),
            mem: BankedMemory::new(),
            first_frame: true,
        }
    }

    pub fn load(&mut self, data: &[Word]) {
        self.mem.load_default(data);
    }

    pub fn step_frame(&mut self) {
        let cycles_per_frame = CPU_HZ / 60;

        // Always clear HALT at the start of a new frame so the CPU can run.
        self.mem.store_mmio(HALT_CONTROL, Word::ZERO);

        // Fire VBlank interrupt if enabled. Skipped on the very first frame so
        // boot code runs cleanly before any interrupt can fire.
        if !self.first_frame {
            let int_enable: u16 = self.mem.load_mmio(INT_ENABLE).into();
            if int_enable & INT_VBLANK != 0 {
                let status: u16 = self.mem.load_mmio(INT_STATUS).into();
                self.mem
                    .store_mmio(INT_STATUS, Word::from(status | INT_VBLANK));
                self.cpu.trigger_interrupt();
            }
        }
        self.first_frame = false;

        clear_background(BLACK);

        let mut cycles = 0usize;
        while cycles < cycles_per_frame {
            if self.mem.load_mmio(HALT_CONTROL) != Word::ZERO {
                break;
            }

            self.cpu.step(&mut self.mem);
            cycles += 1;

            let ppu_cost = self.ppu.step(&mut self.mem);
            cycles += ppu_cost;

            let apu_cost = self.apu.step(&mut self.mem);
            cycles += apu_cost;

            self.check_pending_interrupts();
        }
    }

    fn check_pending_interrupts(&mut self) {
        let int_enable: u16 = self.mem.load_mmio(INT_ENABLE).into();
        let int_status: u16 = self.mem.load_mmio(INT_STATUS).into();
        if int_enable & int_status != 0 {
            self.cpu.trigger_interrupt();
        }
    }
}
