use crate::vm::vm::Word;

#[derive(PartialEq)]
pub struct Opcode(u16);

pub const ADD: Opcode = Opcode(0);
pub const ADDI: Opcode = Opcode(1 << 13);
pub const NAND: Opcode = Opcode(2 << 13);
pub const LUI: Opcode = Opcode(3 << 13);
pub const SW: Opcode = Opcode(4 << 13);
pub const LW: Opcode = Opcode(5 << 13);
pub const BEQ: Opcode = Opcode(6 << 13);
pub const JALR: Opcode = Opcode(7 << 13);

pub const OPCODE_MASK: u16 = 0b1110_0000_0000_0000;
pub const REG_A_MASK: u16 = 0b0001_1100_0000_0000;
pub const REG_B_MASK: u16 = 0b0000_0011_1000_0000;
pub const REG_C_MASK: u16 = 0b0000_0000_0000_0111;
pub const SIMM_MASK: u16 = 0b0000_0000_0111_1111;
pub const IMM_MASK: u16 = 0b0000_0011_1111_1111;
pub const RETI_MASK: u16 = 0b0000_0000_0111_1111;

/// This is the encoded instruction passed to the VM
#[derive(Clone, Copy)]
pub struct EncInst(u16);

impl EncInst {
    pub fn opcode(&self) -> Opcode {
        Opcode(self.0 & OPCODE_MASK)
    }

    pub fn is_noop(&self) -> bool {
        // add 0,0,0 is the noop pusedo-instruction
        self.0 == 0
    }

    pub fn is_reti_imm(&self) -> bool {
        (self.0 & RETI_MASK) > 0
    }

    pub fn reg_a(&self) -> u8 {
        ((self.0 & REG_A_MASK) >> 10) as u8
    }

    pub fn reg_b(&self) -> u8 {
        ((self.0 & REG_B_MASK) >> 7) as u8
    }

    pub fn reg_c(&self) -> u8 {
        (self.0 & REG_C_MASK) as u8
    }

    pub fn simm(&self) -> Word {
        let value = ((self.0 & SIMM_MASK) << 1) as i8;
        let value = value >> 1;
        Word::from(value)
    }

    pub fn imm(&self) -> Word {
        Word::from((self.0 & IMM_MASK) << 6)
    }

    pub fn new_noop() -> EncInst {
        DecInst::Add {
            r_a: 0,
            r_b: 0,
            r_c: 0,
        }
        .encode()
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
            imm: Word::from(imm),
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

        DecInst::Lui {
            r_a,
            imm: Word::from(imm >> 6),
        }
        .encode()
    }

    pub fn new_sw(r_a: u8, r_b: u8, imm: i16) -> EncInst {
        debug_assert!(
            imm <= 63 && imm >= -64,
            "imm can only be between -64 and 63"
        );

        DecInst::Sw {
            r_a,
            r_b,
            imm: Word::from(imm),
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
            imm: Word::from(imm),
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
            imm: Word::from(imm),
        }
        .encode()
    }

    pub fn new_jalr(r_a: u8, r_b: u8) -> EncInst {
        DecInst::Jalr { r_a, r_b }.encode()
    }
}

impl From<Word> for EncInst {
    fn from(value: Word) -> Self {
        Self(value.into())
    }
}

impl Into<Word> for EncInst {
    fn into(self) -> Word {
        Word::from(self.0 as i16)
    }
}

/// These are the decoded instructions which mean they may be longer than 16-bits for conviencne
pub enum DecInst {
    Add { r_a: u8, r_b: u8, r_c: u8 },
    Addi { r_a: u8, r_b: u8, imm: Word },
    Nand { r_a: u8, r_b: u8, r_c: u8 },
    Lui { r_a: u8, imm: Word },
    Sw { r_a: u8, r_b: u8, imm: Word },
    Lw { r_a: u8, r_b: u8, imm: Word },
    Beq { r_a: u8, r_b: u8, imm: Word },
    Jalr { r_a: u8, r_b: u8 },
    Reti,
}

impl DecInst {
    pub fn encode(self) -> EncInst {
        let bits = match self {
            DecInst::Add { r_a, r_b, r_c } => {
                ADD.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (r_c as u16)
            }
            DecInst::Addi { r_a, r_b, imm } => {
                let imm: u16 = imm.into();
                ADDI.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (imm & 0x7Fu16)
            }
            DecInst::Nand { r_a, r_b, r_c } => {
                NAND.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (r_c as u16)
            }
            DecInst::Lui { r_a, imm } => {
                let imm: u16 = imm.into();
                LUI.0 | ((r_a as u16) << 10) | (imm & 0x3FF)
            }
            DecInst::Sw { r_a, r_b, imm } => {
                let imm: u16 = imm.into();
                SW.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (imm & 0x7F)
            }
            DecInst::Lw { r_a, r_b, imm } => {
                let imm: u16 = imm.into();
                LW.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (imm & 0x7F)
            }
            DecInst::Beq { r_a, r_b, imm } => {
                let imm: u16 = imm.into();
                BEQ.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7) | (imm & 0x7F)
            }
            DecInst::Jalr { r_a, r_b } => JALR.0 | ((r_a as u16) << 10) | ((r_b as u16) << 7),
            DecInst::Reti => 0x00_01,
        };
        EncInst(bits)
    }
}
