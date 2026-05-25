use std::fmt::{Display, Result};
use std::ops::{Add, BitAnd, Not};

/// The maximum short address that can be represented by the cpu
const MAX_ADDRESS: usize = u16::MAX as usize;

/// A single word in the memory space.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(i16);

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

/// This is the encoded instruction passed to the VM
#[derive(Clone, Copy)]
pub struct EncInst(u16);

impl EncInst {
    fn opcode(&self) -> u16 {
        let mask = 0b1110_0000_0000_0000;
        self.0 & mask
    }

    pub fn new_add(r_a: u8, r_b: u8, r_c: u8) -> EncInst {
        DecInst::Add { r_a, r_b, r_c }.encode()
    }

    pub fn new_addi(r_a: u8, r_b: u8, imm: i16) -> EncInst {
        debug_assert!(
            imm <= 63 && imm >= -64,
            "imm can only be between -64 and 63"
        );

        DecInst::Addi {
            r_a,
            r_b,
            imm: Word(imm),
        }
        .encode()
    }

    pub fn new_nand(r_a: u8, r_b: u8, r_c: u8) -> EncInst {
        DecInst::Nand { r_a, r_b, r_c }.encode()
    }

    pub fn new_lui(r_a: u8, imm: u16) -> EncInst {
        // lui can only load values between 1 << 16 and 1 << 6
        debug_assert!(
            imm >= 1 << 6 || imm == 0,
            "imm can only be between {} and {} or 0",
            1 << 6,
            u16::MAX
        );
        DecInst::Lui { r_a, imm: imm >> 6 }.encode()
    }

    pub fn new_sw(r_a: u8, r_b: u8, imm: i16) -> EncInst {
        debug_assert!(
            imm <= 63 && imm >= -64,
            "imm can only be between -64 and 63"
        );

        DecInst::Sw {
            r_a,
            r_b,
            imm: Word(imm),
        }
        .encode()
    }

    pub fn new_lw(r_a: u8, r_b: u8, imm: i16) -> EncInst {
        debug_assert!(
            imm <= 63 && imm >= -64,
            "imm can only be between -64 and 63"
        );

        DecInst::Lw {
            r_a,
            r_b,
            imm: Word(imm),
        }
        .encode()
    }

    pub fn new_beq(r_a: u8, r_b: u8, imm: i16) -> EncInst {
        debug_assert!(
            imm <= 63 && imm >= -64,
            "imm can only be between -64 and 63"
        );

        DecInst::Beq {
            r_a,
            r_b,
            imm: Word(imm),
        }
        .encode()
    }

    pub fn new_jalr(r_a: u8, r_b: u8) -> EncInst {
        DecInst::Jalr { r_a, r_b }.encode()
    }
}

impl From<Word> for EncInst {
    fn from(value: Word) -> Self {
        Self(value.0 as u16)
    }
}

impl Into<Word> for EncInst {
    fn into(self) -> Word {
        Word(self.0 as i16)
    }
}

/// These are the decoded instructions which mean they may be longer than 16-bits for conviencne
pub enum DecInst {
    Add { r_a: u8, r_b: u8, r_c: u8 },
    Addi { r_a: u8, r_b: u8, imm: Word },
    Nand { r_a: u8, r_b: u8, r_c: u8 },
    Lui { r_a: u8, imm: u16 },
    Sw { r_a: u8, r_b: u8, imm: Word },
    Lw { r_a: u8, r_b: u8, imm: Word },
    Beq { r_a: u8, r_b: u8, imm: Word },
    Jalr { r_a: u8, r_b: u8 },
}

impl DecInst {
    pub fn encode(self) -> EncInst {
        let bits = match self {
            DecInst::Add { r_a, r_b, r_c } => {
                (0b000u16 << 13) | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (r_c as u16)
            }
            DecInst::Addi { r_a, r_b, imm } => {
                (0b001u16 << 13)
                    | ((r_a as u16) << 10)
                    | ((r_b as u16) << 7)
                    | (imm.0 as u16 & 0x7F)
            }
            DecInst::Nand { r_a, r_b, r_c } => {
                (0b010u16 << 13) | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (r_c as u16)
            }
            DecInst::Lui { r_a, imm } => (0b011u16 << 13) | ((r_a as u16) << 10) | (imm & 0x3FF),
            DecInst::Sw { r_a, r_b, imm } => {
                (0b100u16 << 13)
                    | ((r_a as u16) << 10)
                    | ((r_b as u16) << 7)
                    | (imm.0 as u16 & 0x7F)
            }
            DecInst::Lw { r_a, r_b, imm } => {
                (0b101u16 << 13)
                    | ((r_a as u16) << 10)
                    | ((r_b as u16) << 7)
                    | (imm.0 as u16 & 0x7F)
            }
            DecInst::Beq { r_a, r_b, imm } => {
                (0b110u16 << 13)
                    | ((r_a as u16) << 10)
                    | ((r_b as u16) << 7)
                    | (imm.0 as u16 & 0x7F)
            }
            DecInst::Jalr { r_a, r_b } => {
                (0b111u16 << 13) | ((r_a as u16) << 10) | ((r_b as u16) << 7)
            }
        };
        EncInst(bits)
    }
}

pub struct VM {
    program_counter: u16,
    regs: [Reg; 8],
    // +1 here because address 0 needs a slot
    memory: [Word; MAX_ADDRESS + 1],
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

            match self.memory.get(idx as usize) {
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
            memory: [Word(0); MAX_ADDRESS + 1],
        }
    }

    pub fn load(&mut self, data: &[Word]) {
        self.memory[..data.len()].copy_from_slice(data);
    }

    pub fn step(&mut self) {
        // add this in so the compiler knows it can remove the bounds check below. In a release build
        // this whole if check should get optimized away since it's marked as unreachable
        if self.program_counter as usize >= self.memory.len() {
            unreachable!("this is not possible")
        }

        // convert the memory address into an instruction
        let inst = self.memory[self.program_counter as usize];
        self.program_counter += 1;

        // decode the instruction
        let inst = EncInst::from(inst);
        let inst = self.decode(inst);

        // execut the instruction
        self.exec(inst);
    }

    pub fn decode(&self, inst: EncInst) -> DecInst {
        match inst.opcode() {
            0b0000_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let r_c = (inst.0 & 0b0000_0000_0000_0111) as u8;
                DecInst::Add { r_a, r_b, r_c }
            }
            0b0010_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let imm = sign_imm(inst);
                DecInst::Addi { r_a, r_b, imm }
            }
            0b0100_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let r_c = (inst.0 & 0b0000_0000_0000_0111) as u8;
                DecInst::Nand { r_a, r_b, r_c }
            }
            0b0110_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let imm = inst.0 & 0b0000_0011_1111_1111;
                DecInst::Lui { r_a, imm }
            }
            0b1000_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let imm = sign_imm(inst);
                DecInst::Sw { r_a, r_b, imm }
            }
            0b1010_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let imm = sign_imm(inst);
                DecInst::Lw { r_a, r_b, imm }
            }
            0b1100_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                let imm = sign_imm(inst);
                DecInst::Beq { r_a, r_b, imm }
            }
            0b1110_0000_0000_0000 => {
                let r_a = mask_and_shift(inst, 0b0001_1100_0000_0000, 10);
                let r_b = mask_and_shift(inst, 0b0000_0011_1000_0000, 7);
                DecInst::Jalr { r_a, r_b }
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

    pub fn exec(&mut self, inst: DecInst) {
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
                let value = Word((imm << 6) as i16);
                self.store_reg(r_a, value);
            }
            DecInst::Sw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let addr = addr.0 as u16;

                let value = self.load_reg(r_a);
                self.memory[addr as usize] = value;
            }
            DecInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let addr = addr.0 as u16;

                let value = self.memory[addr as usize];
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
        }
    }
}

fn mask_and_shift(inst: EncInst, mask: u16, shift: u16) -> u8 {
    ((inst.0 & mask) >> shift) as u8
}

fn sign_imm(inst: EncInst) -> Word {
    // convert the value to a signed 8-bit number instead of a 7 bit number
    let value = ((inst.0 & 0b0000_0000_0111_1111) << 1) as i8;
    // fix the magnitued
    let value = value >> 1;

    Word(value as i16)
}
