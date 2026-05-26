use std::fmt::{Display, Result};
use std::ops::{Add, BitAnd, Not};

use macroquad::camera::Camera2D;
use macroquad::texture::RenderTarget;

use crate::vm::cpu::CPU;
use crate::vm::ppu::PPU;

/// A single word in the memory space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(i16);

impl Word {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "Wx{:04X}", self.0)
    }
}

impl From<i16> for Word {
    fn from(value: i16) -> Self {
        Word(value)
    }
}

impl Into<i16> for Word {
    fn into(self) -> i16 {
        self.0
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value as i16)
    }
}

impl Into<u16> for Word {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl Add for Word {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
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
        Self(value as i16)
    }
}

/// The maximum short address that can be represented in the 16-bit address space
const MAX_ADDRESS: usize = u16::MAX as usize;

pub type MemoryBank = [Word; MAX_ADDRESS + 1];

pub struct VM<'a> {
    cpu: CPU,
    ppu: PPU<'a>,
    // +1 here because addresses 0 needs a slot
    cart_memory: MemoryBank,
    video_memory: MemoryBank,
}

impl Display for VM<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "{}\n", self.cpu)?;
        write!(f, "{}\n", self.ppu)?;

        for i in -3..3i32 {
            let idx = self.cpu.get_program_counter() as i32 + i;
            if idx < 0 {
                continue;
            }

            match self.cart_memory.get(idx as usize) {
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

impl<'a> VM<'a> {
    pub fn new(render_target: &'a RenderTarget, camera: &'a Camera2D) -> Self {
        Self {
            cpu: CPU::new(),
            ppu: PPU::new(render_target, camera),
            cart_memory: [Word(0); MAX_ADDRESS + 1],
            video_memory: [Word(0); MAX_ADDRESS + 1],
        }
    }

    pub fn load(&mut self, data: &[Word]) {
        self.cart_memory[..data.len()].copy_from_slice(data);
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.cart_memory, &mut self.video_memory);
        self.ppu.step(&mut self.video_memory);
    }
}
