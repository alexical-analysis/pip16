use std::fmt::{Display, Formatter, Result};

use crate::vm::{
    BankedMemory, Word,
    riscp::{DecodedInst, EncodedInst},
};

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

/// The CPU jumps here when an interrupt fires. The last 3 words of Cart ROM
/// (0xB7BD–0xB7BF) hold a 3-instruction dispatch sequence:
///   LUI rX, <high byte of handler>
///   LLI rX, <low byte of handler>
///   JALR r0, rX
/// The actual handler ends with RETL to resume interrupted code.
pub const INTERRUPT_VECTOR: Word = Word::new_const(0xB7BD);

pub struct CPU {
    program_counter: Word,
    interrupt_return: Reg,
    regs: [Reg; 16],
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
        write!(f, "    interrupt pending: {}\n", self.interrupt_pending)?;
        write!(f, "    handling interrupt: {}\n", self.handling_interrupt)?;
        write!(f, "    interrupt return: {}\n", self.interrupt_return.load())?;
        write!(f, "    PC: {}\n", self.program_counter)?;
        write!(f, "    {} |\n", headers.join(""))?;
        write!(f, "    {} |\n\n", values.join(""))?;

        Ok(())
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            program_counter: Word::ZERO,
            interrupt_return: Reg::new(),
            regs: [Reg::new(); 16],
            interrupt_pending: false,
            handling_interrupt: false,
        }
    }

    pub fn get_program_counter(&self) -> Word {
        self.program_counter
    }

    /// Signals an interrupt to the CPU. Ignored if already handling one.
    pub fn trigger_interrupt(&mut self) {
        if !self.handling_interrupt {
            self.interrupt_pending = true;
        }
    }

    pub fn step(&mut self, mem: &mut BankedMemory) {
        // Instruction fetch always reads from the default bank.
        let inst = mem.fetch(self.program_counter);
        self.program_counter = self.program_counter + Word::ONE;

        let inst = EncodedInst::from(inst);
        if inst.is_noop() {
            return;
        }
        let inst = inst.decode();

        self.exec(mem, inst);

        if self.interrupt_pending {
            self.handle_interrupt();
        }
    }

    fn load_reg(&self, reg: u8) -> Word {
        if reg == 0 {
            return Word::ZERO;
        }
        self.regs[reg as usize].load()
    }

    fn store_reg(&mut self, reg: u8, value: Word) {
        self.regs[reg as usize].store(value);
    }

    fn exec(&mut self, mem: &mut BankedMemory, inst: DecodedInst) {
        match inst {
            DecodedInst::Add { r_a, r_b, r_c } => {
                let value = self.load_reg(r_b) + self.load_reg(r_c);
                self.store_reg(r_a, value);
            }
            DecodedInst::Sub { r_a, r_b, r_c } => {
                let value = self.load_reg(r_b) - self.load_reg(r_c);
                self.store_reg(r_a, value);
            }
            DecodedInst::Mul { r_a, r_b, r_c } => {
                let value = self.load_reg(r_b) * self.load_reg(r_c);
                self.store_reg(r_a, value);
            }
            DecodedInst::Xor { r_a, r_b, r_c } => {
                let value = self.load_reg(r_b) ^ self.load_reg(r_c);
                self.store_reg(r_a, value);
            }
            DecodedInst::Nand { r_a, r_b, r_c } => {
                let value = !(self.load_reg(r_b) & self.load_reg(r_c));
                self.store_reg(r_a, value);
            }
            DecodedInst::Shl { r_a, r_b, imm } => {
                let value = self.load_reg(r_b) << imm;
                self.store_reg(r_a, value);
            }
            DecodedInst::Shr { r_a, r_b, imm } => {
                let value = self.load_reg(r_b) >> imm;
                self.store_reg(r_a, value);
            }
            DecodedInst::Lui { r_a, imm } => {
                self.store_reg(r_a, imm << Word::new_const(8));
            }
            DecodedInst::Lli { r_a, imm } => {
                let value = self.load_reg(r_a) | imm;
                self.store_reg(r_a, value);
            }
            DecodedInst::Sw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let value = self.load_reg(r_a);
                mem.store(addr, value);
            }
            DecodedInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let value = mem.load(addr);
                self.store_reg(r_a, value);
            }
            DecodedInst::Jalr { r_a, r_b, .. } => {
                self.store_reg(r_a, self.program_counter);
                let reg_value = self.load_reg(r_b);
                self.program_counter = reg_value.into();
            }
            DecodedInst::Beq { r_a, r_b, r_c } => {
                if self.load_reg(r_b) == self.load_reg(r_c) {
                    self.program_counter = self.load_reg(r_a);
                }
            }
            DecodedInst::Bne { r_a, r_b, r_c } => {
                if self.load_reg(r_b) != self.load_reg(r_c) {
                    self.program_counter = self.load_reg(r_a);
                }
            }
            DecodedInst::Blt { r_a, r_b, r_c } => {
                if self.load_reg(r_b) < self.load_reg(r_c) {
                    self.program_counter = self.load_reg(r_a);
                }
            }
            DecodedInst::Noop => {}
            DecodedInst::Retl => {
                self.handling_interrupt = false;
                self.program_counter = self.interrupt_return.load();
            }
        }
    }

    fn handle_interrupt(&mut self) {
        self.interrupt_pending = false;
        self.handling_interrupt = true;
        self.interrupt_return.store(self.program_counter);
        self.program_counter = INTERRUPT_VECTOR;
    }
}
