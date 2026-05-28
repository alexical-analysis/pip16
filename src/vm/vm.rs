use std::fmt::{Display, Result};
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr, Sub};

use macroquad::texture::Texture2D;

use crate::vm::mmio::HALT_CONTROL;
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

/// The maximum short address that can be represented in the 16-bit address space
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
        // add this in so the compiler knows it can remove the bounds check below. In a release build
        // this whole if check should get optimized away since it's marked as unreachable
        let addr: usize = addr.into();
        if addr >= self.0.len() {
            unreachable!("address out of bounds")
        }

        self.0[addr]
    }
    pub fn store_word(&mut self, addr: Word, value: Word) {
        // add this in so the compiler knows it can remove the bounds check below. In a release build
        // this whole if check should get optimized away since it's marked as unreachable
        let addr: usize = addr.into();
        if addr >= self.0.len() {
            unreachable!("address out of bounds")
        }

        self.0[addr] = value
    }
}

/// The clock rate that the vm runs the CPU at
const CPU_HZ: usize = 15_360_000;

pub struct VM {
    cpu: CPU,
    ppu: PPU,
    // +1 here because addresses 0 needs a slot
    mem: MemoryBank,
}

impl Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}\n", self.cpu)?;
        write!(f, "{}\n", self.ppu)?;

        for i in -3..3i32 {
            let pc: u16 = self.cpu.get_program_counter().into();
            let idx = pc as i32 + i;
            if idx < 0 {
                continue;
            }

            match self.mem.0.get(idx as usize) {
                Some(&w) => {
                    if i == 0 {
                        write!(f, ">  0x{:04X} : 0x{:04X}\n", idx, w.0)?;
                    } else {
                        write!(f, "  0x{:04X} : 0x{:04X}\n", idx, w.0)?;
                    }
                }
                None => break,
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
            mem: MemoryBank::new(),
        }
    }

    pub fn load(&mut self, data: &[Word]) {
        self.mem.load(data);
    }

    pub fn step_frame(&mut self) {
        let cycles_per_frame = CPU_HZ / 60;

        self.mem.store_word(HALT_CONTROL, Word::ZERO);

        for i in 0..cycles_per_frame {
            self.cpu.step(&mut self.mem);
            // TODO: the ppu might take more than 1 cycle if it has drawing work to do so that needs
            // to be acounted for here. We should have the ppu return the "cost of work" and then skip
            // through that many cycles.
            self.ppu.step(&mut self.mem);

            // check if the CPU_HALT register is set
            let halt = self.mem.load_word(HALT_CONTROL);
            if halt != Word::ZERO {
                break;
            }
        }
    }
}
