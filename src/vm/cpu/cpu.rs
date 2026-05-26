use std::fmt::{Display, Formatter, Result};

use crate::vm::cpu::inst::{self, DecInst, EncInst};
use crate::vm::mmio::{BANK_CONTROL, INTERRUPT_PROGRAM_COUNTER, INTERRUPT_VECTOR};
use crate::vm::{MemoryBank, Word};

// The first 32k (16k words) are mirrored between address spaces
const MAX_MIRROR_ADDRESS: u16 = 16 * 1024;

/// This is represents a RiSC-16 register
#[derive(Clone, Copy)]
pub struct Reg(Word);

impl Reg {
    pub fn new() -> Self {
        Reg(Word::new())
    }

    pub fn load(&self) -> Word {
        self.0
    }

    pub fn store(&mut self, word: Word) {
        self.0 = word
    }
}

pub struct CPU {
    program_counter: u16,
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

impl CPU {
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            regs: [Reg::new(); 8],
            interrupt_pending: false,
            handling_interrupt: false,
        }
    }

    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }

    pub fn step(&mut self, cart_memory: &mut MemoryBank, video_memory: &mut MemoryBank) {
        // add this in so the compiler knows it can remove the bounds check below. In a release build
        // this whole if check should get optimized away since it's marked as unreachable
        if self.program_counter as usize >= cart_memory.len() {
            unreachable!("this is not possible")
        }

        // load an instruction for cart_memory the CPU only reads instructions for cart_memory, it does
        // not respect the bank controll register
        let inst = cart_memory[self.program_counter as usize];
        self.program_counter += 1;

        // decode the instruction
        let inst = EncInst::from(inst);
        if inst.is_noop() {
            return;
        }
        let inst = self.decode(inst);

        // execut the instruction
        self.exec(cart_memory, video_memory, inst);

        // check for and handle any interrupts
        if self.interrupt_pending {
            self.handle_interupt(cart_memory);
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
            _ => unreachable!("this is not a valid opcode"),
        }
    }

    fn load_reg(&self, reg: u8) -> Word {
        // register 0 always contains the value 0 as per the spec
        if reg == 0 {
            return Word::new();
        }

        self.regs[reg as usize].load()
    }

    fn store_reg(&mut self, reg: u8, value: Word) {
        self.regs[reg as usize].store(value);
    }

    fn load_memory(
        &self,
        cart_memory: &mut MemoryBank,
        video_memory: &mut MemoryBank,
        addr: u16,
    ) -> Word {
        // the lower 16k addresses are mirrored between the 2 address spaces. If that's the case we just
        // read/write from cart memory directly
        if addr < MAX_MIRROR_ADDRESS {
            return cart_memory[addr as usize];
        }

        let space: i16 = cart_memory[BANK_CONTROL].into();
        if space == 0 {
            cart_memory[addr as usize]
        } else {
            video_memory[addr as usize]
        }
    }

    fn store_memory(
        &mut self,
        cart_memory: &mut MemoryBank,
        video_memory: &mut MemoryBank,
        addr: u16,
        value: Word,
    ) {
        // the lower 16k addresses are mirrored between the 2 address spaces. If that's the case we just
        // read/write from cart memory directly
        if addr < MAX_MIRROR_ADDRESS {
            cart_memory[addr as usize] = value;
            return;
        }

        let space: i16 = cart_memory[BANK_CONTROL].into();
        if space == 0 {
            cart_memory[addr as usize] = value;
        } else {
            video_memory[addr as usize] = value;
        }
    }

    fn exec(&mut self, cart_memory: &mut MemoryBank, video_memory: &mut MemoryBank, inst: DecInst) {
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
                let addr: u16 = addr.into();

                let value = self.load_reg(r_a);
                self.store_memory(cart_memory, video_memory, addr, value);
            }
            DecInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let addr: u16 = addr.into();

                let value = self.load_memory(cart_memory, video_memory, addr);
                self.store_reg(r_a, value);
            }
            DecInst::Beq { r_a, r_b, imm } => {
                if self.load_reg(r_a) == self.load_reg(r_b) {
                    self.program_counter = self.program_counter.wrapping_add_signed(imm.into())
                }
            }
            DecInst::Jalr { r_a, r_b } => {
                let value = Word::from(self.program_counter as i16);
                self.store_reg(r_a, value);
                let reg_value = self.load_reg(r_b);
                self.program_counter = reg_value.into();
            }
            DecInst::Reti => {
                let return_addr = cart_memory[INTERRUPT_PROGRAM_COUNTER];
                self.program_counter = return_addr.into();
                self.handling_interrupt = false;
            }
        }
    }

    fn handle_interupt(&mut self, cart_memory: &mut MemoryBank) {
        self.handling_interrupt = true;
        self.interrupt_pending = false;

        cart_memory[INTERRUPT_PROGRAM_COUNTER] = Word::from(self.program_counter);
        self.program_counter = INTERRUPT_VECTOR;
    }
}
