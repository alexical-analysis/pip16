use std::fmt::{Display, Formatter, Result};

use crate::vm::cpu::inst::{self, DecInst, EncInst};
use crate::vm::mmio::{INTERRUPT_PROGRAM_COUNTER, INTERRUPT_VECTOR};
use crate::vm::{MemoryBank, Word};

/// This is represents a RiSC-16 register
#[derive(Clone, Copy)]
pub struct Reg(Word);

impl Reg {
    pub fn new() -> Self {
        Reg(Word::ZERO)
    }

    pub fn load(&self) -> Word {
        self.0
    }

    pub fn store(&mut self, word: Word) {
        self.0 = word
    }
}

pub struct CPU {
    program_counter: Word,
    regs: [Reg; 8],
    interrupt_pending: bool,
    handling_interrupt: bool,
}

impl Display for CPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut headers = vec![];
        let mut values = vec![];
        for (i, &reg) in self.regs.iter().enumerate() {
            headers.push(format!("|  Reg_{}  ", i));

            let val: i16 = reg.load().into();
            let sign = if val < 0 { '-' } else { '+' };
            let val = format!("| {}0x{:04X} ", sign, val.unsigned_abs());
            values.push(val);
        }

        write!(f, "[ CPU ]\n")?;
        write!(f, "    PC: {}\n", self.program_counter)?;
        write!(f, "    {} |\n", headers.join(""))?;
        write!(f, "    {} |\n\n", values.join(""))?;

        Ok(())
    }
}

pub const CPU_HZ: usize = 15_360_000;

impl CPU {
    pub fn new() -> Self {
        Self {
            program_counter: Word::ZERO,
            regs: [Reg::new(); 8],
            interrupt_pending: false,
            handling_interrupt: false,
        }
    }

    pub fn get_program_counter(&self) -> Word {
        self.program_counter
    }

    pub fn step(&mut self, mem: &mut MemoryBank) {
        let inst = mem.load_word(self.program_counter);
        self.program_counter = self.program_counter + Word::ONE;

        // decode the instruction
        let inst = EncInst::from(inst);
        if inst.is_noop() {
            return;
        }
        let inst = self.decode(inst);

        // execut the instruction
        self.exec(mem, inst);

        // check for and handle any interrupts
        if self.interrupt_pending {
            self.handle_interupt(mem);
        }
    }

    pub fn decode(&self, inst: EncInst) -> DecInst {
        match inst.opcode() {
            inst::ADD => DecInst::Add {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                r_c: inst.reg_c(),
            },
            inst::ADDI => DecInst::Addi {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            inst::NAND => DecInst::Nand {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                r_c: inst.reg_c(),
            },
            inst::LUI => DecInst::Lui {
                r_a: inst.reg_a(),
                imm: inst.imm(),
            },
            inst::SW => DecInst::Sw {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            inst::LW => DecInst::Lw {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            inst::BEQ => DecInst::Beq {
                r_a: inst.reg_a(),
                r_b: inst.reg_b(),
                imm: inst.simm(),
            },
            inst::JALR => {
                if inst.is_reti_imm() {
                    return DecInst::Reti;
                }

                DecInst::Jalr {
                    r_a: inst.reg_a(),
                    r_b: inst.reg_b(),
                }
            }
            _ => unreachable!("not a valid opcode"),
        }
    }

    fn load_reg(&self, reg: u8) -> Word {
        // register 0 always contains the value 0 as per the spec
        if reg == 0 {
            return Word::ZERO;
        }

        self.regs[reg as usize].load()
    }

    fn store_reg(&mut self, reg: u8, value: Word) {
        self.regs[reg as usize].store(value);
    }

    fn exec(&mut self, mem: &mut MemoryBank, inst: DecInst) {
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
                let value = self.load_reg(r_a);
                mem.store_word(addr, value);
            }
            DecInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let value = mem.load_word(addr);
                self.store_reg(r_a, value);
            }
            DecInst::Beq { r_a, r_b, imm } => {
                if self.load_reg(r_a) == self.load_reg(r_b) {
                    let pc: u16 = self.program_counter.into();
                    let pc = pc.wrapping_add_signed(imm.into());
                    self.program_counter = Word::from(pc);
                }
            }
            DecInst::Jalr { r_a, r_b } => {
                let value = Word::from(self.program_counter);
                self.store_reg(r_a, value);
                let reg_value = self.load_reg(r_b);
                self.program_counter = reg_value.into();
            }
            DecInst::Reti => {
                let return_addr = mem.load_word(INTERRUPT_PROGRAM_COUNTER);
                self.program_counter = return_addr.into();
                self.handling_interrupt = false;
            }
        }
    }

    fn handle_interupt(&mut self, mem: &mut MemoryBank) {
        self.handling_interrupt = true;
        self.interrupt_pending = false;

        let pc = Word::from(self.program_counter);
        mem.store_word(INTERRUPT_PROGRAM_COUNTER, pc);
        self.program_counter = INTERRUPT_VECTOR;
    }
}
