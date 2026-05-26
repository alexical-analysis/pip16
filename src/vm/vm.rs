use std::fmt::{Display, Result};
use std::ops::{Add, BitAnd, Not};

use crate::vm::inst::{DecInst, EncInst, REG_A_MASK, REG_B_MASK, REG_C_MASK};
use crate::vm::mmio::{BANK_CONTROL, INTERRUPT_PROGRAM_COUNTER, INTERRUPT_VECTOR};

/// The maximum short address that can be represented by the cpu
const MAX_ADDRESS: usize = u16::MAX as usize;

// The first 32k (16k words) are mirrored between address spaces
const MAX_MIRROR_ADDRESS: u16 = 16 * 1024;

/// A single word in the memory space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(i16);

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        write!(f, "Wx{:04X}", self.0)
    }
}

impl Into<u16> for Word {
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl From<i16> for Word {
    fn from(value: i16) -> Self {
        Word(value)
    }
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value as i16)
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

/// This is represents a RiSC-16 register
#[derive(Clone, Copy)]
pub struct Reg(Word);

impl Reg {
    fn load(&self) -> Word {
        self.0
    }

    fn store(&mut self, word: Word) {
        self.0 = word
    }
}

pub struct VM {
    program_counter: u16,
    regs: [Reg; 8],
    // +1 here because addresses 0 needs a slot
    cart_memory: [Word; MAX_ADDRESS + 1],
    video_memory: [Word; MAX_ADDRESS + 1],
    interrupt_pending: bool,
    handling_interrupt: bool,
}

impl Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let mut headers = vec![];
        let mut values = vec![];
        for (i, &reg) in self.regs.iter().enumerate() {
            headers.push(format!("|  Reg_{}  ", i));

            let val = reg.load().0;
            let sign = if val < 0 { '-' } else { '+' };
            let val = format!("| {}0x{:04X} ", sign, val.unsigned_abs());
            values.push(val);
        }

        write!(f, "{} |\n", headers.join(""))?;
        write!(f, "{} |\n\n", values.join(""))?;
        write!(f, "  [ PC: {} ]\n", self.program_counter)?;

        for i in -3..3i32 {
            let idx = self.program_counter as i32 + i;
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

impl VM {
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            regs: [Reg(Word(0)); 8],
            cart_memory: [Word(0); MAX_ADDRESS + 1],
            video_memory: [Word(0); MAX_ADDRESS + 1],
            interrupt_pending: false,
            handling_interrupt: false,
        }
    }

    pub fn load(&mut self, data: &[Word]) {
        self.cart_memory[..data.len()].copy_from_slice(data);
    }

    pub fn step(&mut self) {
        // add this in so the compiler knows it can remove the bounds check below. In a release build
        // this whole if check should get optimized away since it's marked as unreachable
        if self.program_counter as usize >= self.cart_memory.len() {
            unreachable!("this is not possible")
        }

        // load an instruction for cart_memory the CPU only reads instructions for cart_memory, it does
        // not respect the bank controll register
        let inst = self.cart_memory[self.program_counter as usize];
        self.program_counter += 1;

        // decode the instruction
        let inst = EncInst::from(inst);
        if inst.is_noop() {
            return;
        }
        let inst = self.decode(inst);

        // execut the instruction
        self.exec(inst);

        // check for and handle any interrupts
        if self.interrupt_pending {
            self.handle_interupt();
        }
    }

    pub fn decode(&self, inst: EncInst) -> DecInst {
        match inst.opcode() {
            ADD => DecInst::Add {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                r_c: inst.reg_c(),
            },
            ADDI => DecInst::Addi {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            NAND => DecInst::Nand {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                r_c: inst.reg_c(),
            },
            LUI => DecInst::Lui {
                r_a: inst.reg_a(),
                imm: inst.imm(),
            },
            SW => DecInst::Sw {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            LW => DecInst::Lw {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            BEQ => DecInst::Beq {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            JALR => {
                if inst.is_reti_imm() {
                    return DecInst::Reti;
                }

                DecInst::Jalr {
                    r_a: inst.reg_a(),
                    r_b: inst.reg_b(),
                }
            }
            _ => unreachable!("this is not a valid opcode"),
        }
    }

    fn load_reg(&self, reg: u8) -> Word {
        // register 0 always contains the value 0 as per the spec
        if reg == 0 {
            return Word(0);
        }

        self.regs[reg as usize].load()
    }

    fn store_reg(&mut self, reg: u8, value: Word) {
        self.regs[reg as usize].store(value);
    }

    fn load_memory(&self, addr: u16) -> Word {
        // the lower 16k addresses are mirrored between the 2 address spaces. If that's the case we just
        // read/write from cart memory directly
        if addr < MAX_MIRROR_ADDRESS {
            return self.cart_memory[addr as usize];
        }

        let space = self.cart_memory[BANK_CONTROL];
        if space.0 == 0 {
            self.cart_memory[addr as usize]
        } else {
            self.video_memory[addr as usize]
        }
    }

    fn store_memory(&mut self, addr: u16, value: Word) {
        // the lower 16k addresses are mirrored between the 2 address spaces. If that's the case we just
        // read/write from cart memory directly
        if addr < MAX_MIRROR_ADDRESS {
            self.cart_memory[addr as usize] = value;
            return;
        }

        let space = self.cart_memory[BANK_CONTROL];
        if space.0 == 0 {
            self.cart_memory[addr as usize] = value;
        } else {
            self.video_memory[addr as usize] = value;
        }
    }

    fn exec(&mut self, inst: DecInst) {
        match inst {
            DecInst::Add { r_a, r_b, r_c } => {
                let value = self.load_reg(r_b) + self.load_reg(r_c);
                self.store_reg(r_a, value);
            }
            DecInst::Addi { r_a, r_b, imm } => {
                let value = self.load_reg(r_b) + imm;
                self.store_reg(r_a, value);
            }
            DecInst::Nand { r_a, r_b, r_c } => {
                let value = !(self.load_reg(r_b) & self.load_reg(r_c));
                self.store_reg(r_a, value);
            }
            DecInst::Lui { r_a, imm } => {
                self.store_reg(r_a, imm);
            }
            DecInst::Sw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let addr = addr.0 as u16;

                let value = self.load_reg(r_a);
                self.store_memory(addr, value);
            }
            DecInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let addr = addr.0 as u16;

                let value = self.load_memory(addr);
                self.store_reg(r_a, value);
            }
            DecInst::Beq { r_a, r_b, imm } => {
                if self.load_reg(r_a) == self.load_reg(r_b) {
                    self.program_counter = self.program_counter.wrapping_add_signed(imm.0)
                }
            }
            DecInst::Jalr { r_a, r_b } => {
                let value = Word(self.program_counter as i16);
                self.store_reg(r_a, value);
                self.program_counter = self.load_reg(r_b).0 as u16;
            }
            DecInst::Reti => {
                let return_addr = self.cart_memory[INTERRUPT_PROGRAM_COUNTER];
                self.program_counter = return_addr.into();
                self.handling_interrupt = false;
            }
        }
    }

    fn handle_interupt(&mut self) {
        self.handling_interrupt = true;
        self.interrupt_pending = false;

        self.cart_memory[INTERRUPT_PROGRAM_COUNTER] = Word::from(self.program_counter);
        self.program_counter = INTERRUPT_VECTOR;
    }
}
