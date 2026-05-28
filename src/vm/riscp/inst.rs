use crate::vm::Word;

const ADD: u16 = 0;
const SUB: u16 = 1 << 12;
const MUL: u16 = 2 << 12;
const XOR: u16 = 3 << 12;
const NAND: u16 = 4 << 12;
const SHL: u16 = 5 << 12;
const SHR: u16 = 6 << 12;
const LUI: u16 = 7 << 12;
const LLI: u16 = 8 << 12;
const SW: u16 = 9 << 12;
const LW: u16 = 10 << 12;
const JALR: u16 = 11 << 12;
const BEQ: u16 = 12 << 12;
const BNE: u16 = 13 << 12;
const BLT: u16 = 14 << 12;

const OPCODE_MASK: u16 = 0xF0_00;
const REG_A_MASK: u16 = 0x0F_00;
const REG_B_MASK: u16 = 0x00_F0;
const REG_C_MASK: u16 = 0x00_0F;
const WIDE_IMM_MASK: u16 = 0x00_FF;
const IMM_MASK: u16 = 0x00_0F;

/// This is the encoded instruction passed to the VM
#[derive(Clone, Copy)]
pub struct EncodedInst(u16);

impl From<Word> for EncodedInst {
    fn from(value: Word) -> Self {
        Self(value.into())
    }
}

impl EncodedInst {
    pub fn is_noop(&self) -> bool {
        self.0 == 0
    }

    fn imm_gt_0(&self) -> bool {
        (self.0 & IMM_MASK) > 0
    }

    fn opcode(&self) -> u16 {
        self.0 & OPCODE_MASK
    }

    fn reg_a(&self) -> u8 {
        ((self.0 & REG_A_MASK) >> 8) as u8
    }

    fn reg_b(&self) -> u8 {
        ((self.0 & REG_B_MASK) >> 4) as u8
    }

    fn reg_c(&self) -> u8 {
        (self.0 & REG_C_MASK) as u8
    }

    fn wide_imm(&self) -> Word {
        Word::from(self.0 & WIDE_IMM_MASK)
    }

    fn imm(&self) -> Word {
        Word::from(self.0 & IMM_MASK)
    }

    pub fn decode(self) -> DecodedInst {
        if self.0 == 0 {
            return DecodedInst::Noop;
        }

        match self.opcode() {
            ADD => DecodedInst::Add {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            SUB => DecodedInst::Sub {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            MUL => DecodedInst::Mul {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            XOR => DecodedInst::Xor {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            NAND => DecodedInst::Nand {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            SHL => DecodedInst::Shl {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                imm: self.imm(),
            },
            SHR => DecodedInst::Shr {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                imm: self.imm(),
            },
            LUI => DecodedInst::Lui {
                r_a: self.reg_a(),
                imm: self.wide_imm(),
            },
            LLI => DecodedInst::Lui {
                r_a: self.reg_a(),
                imm: self.wide_imm(),
            },
            SW => DecodedInst::Sw {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                imm: self.imm(),
            },
            LW => DecodedInst::Lw {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                imm: self.imm(),
            },
            JALR => {
                let imm = self.imm();
                if imm > Word::ZERO {
                    DecodedInst::Retl
                } else {
                    DecodedInst::Jalr {
                        r_a: self.reg_a(),
                        r_b: self.reg_b(),
                        imm,
                    }
                }
            }
            BEQ => DecodedInst::Beq {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            BNE => DecodedInst::Bne {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            BLT => DecodedInst::Blt {
                r_a: self.reg_a(),
                r_b: self.reg_b(),
                r_c: self.reg_c(),
            },
            _ => unreachable!("unknown instruction opcode"),
        }
    }
}

/// These are the decoded instructions which mean they may be longer than 16-bits for conviencne
pub enum DecodedInst {
    Add { r_a: u8, r_b: u8, r_c: u8 },
    Sub { r_a: u8, r_b: u8, r_c: u8 },
    Mul { r_a: u8, r_b: u8, r_c: u8 },
    Xor { r_a: u8, r_b: u8, r_c: u8 },
    Nand { r_a: u8, r_b: u8, r_c: u8 },
    Shl { r_a: u8, r_b: u8, imm: Word },
    Shr { r_a: u8, r_b: u8, imm: Word },
    Lui { r_a: u8, imm: Word },
    Lli { r_a: u8, imm: Word },
    Sw { r_a: u8, r_b: u8, imm: Word },
    Lw { r_a: u8, r_b: u8, imm: Word },
    Jalr { r_a: u8, r_b: u8, imm: Word },
    Beq { r_a: u8, r_b: u8, r_c: u8 },
    Bne { r_a: u8, r_b: u8, r_c: u8 },
    Blt { r_a: u8, r_b: u8, r_c: u8 },
    Noop,
    Retl,
}

impl DecodedInst {
    pub fn encode(self) -> EncodedInst {
        let bits = match self {
            DecodedInst::Add { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                ADD | r_a | r_b | r_c as u16
            }
            DecodedInst::Sub { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                SUB | r_a | r_b | r_c as u16
            }
            DecodedInst::Mul { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                MUL | r_a | r_b | r_c as u16
            }
            DecodedInst::Xor { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                XOR | r_a | r_b | r_c as u16
            }
            DecodedInst::Nand { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                NAND | r_a | r_b | r_c as u16
            }
            DecodedInst::Shl { r_a, r_b, imm } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                let imm: u16 = imm.into();
                SHL | r_a | r_b | imm
            }
            DecodedInst::Shr { r_a, r_b, imm } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                let imm: u16 = imm.into();
                SHR | r_a | r_b | imm
            }
            DecodedInst::Lui { r_a, imm } => {
                let r_a = (r_a as u16) << 8;
                let imm: u16 = imm.into();
                LUI | r_a | imm
            }
            DecodedInst::Lli { r_a, imm } => {
                let r_a = (r_a as u16) << 8;
                let imm: u16 = imm.into();
                LLI | r_a | imm
            }
            DecodedInst::Sw { r_a, r_b, imm } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                let imm: u16 = imm.into();
                SW | r_a | r_b | imm
            }
            DecodedInst::Lw { r_a, r_b, imm } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                let imm: u16 = imm.into();
                LW | r_a | r_b | imm
            }
            DecodedInst::Jalr { r_a, r_b, imm } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                let imm: u16 = imm.into();
                JALR | r_a | r_b | imm
            }
            DecodedInst::Beq { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                BEQ | r_a | r_b | r_c as u16
            }
            DecodedInst::Bne { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                BNE | r_a | r_b | r_c as u16
            }
            DecodedInst::Blt { r_a, r_b, r_c } => {
                let r_a = (r_a as u16) << 8;
                let r_b = (r_b as u16) << 4;
                BLT | r_a | r_b | r_c as u16
            }
            DecodedInst::Noop => 0x00_00,
            DecodedInst::Retl => JALR | 0x00_01,
        };
        EncodedInst(bits)
    }
}
