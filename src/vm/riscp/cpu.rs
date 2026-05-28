use crate::vm::{
    MemoryBank, Word,
    riscp::{DecodedInst, EncodedInst},
};

/// This is represents a RiSC-P register
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

/// This is where the CPU jumps to when it needs to handle an interrupt. It's FF_FC because the minimum
/// number of instructions needed to jump to an arbitrary address where handler code is located is 3.
/// LUI+LLI to set up the address vector and JALR to jump to that address
pub const INTERRUPT_VECTOR: Word = Word::new_const(0xFF_FC);

pub struct CPU {
    program_counter: Word,
    interrupt_return: Reg,
    regs: [Reg; 16],
    interrupt_pending: bool,
    handling_interrupt: bool,
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

    pub fn get_porgram_counter(&self) -> Word {
        self.program_counter
    }

    pub fn step(&mut self, mem: &mut MemoryBank) {
        let inst = mem.load_word(self.program_counter);
        self.program_counter = self.program_counter + Word::ONE;

        // decode the instruction
        let inst = EncodedInst::from(inst);
        if inst.is_noop() {
            return;
        }
        let inst = inst.decode();

        // execut the instruction
        self.exec(mem, inst);

        // check for and handle any interrupts
        if self.interrupt_pending {
            self.handle_interupt();
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

    fn exec(&mut self, mem: &mut MemoryBank, inst: DecodedInst) {
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
                mem.store_word(addr, value);
            }
            DecodedInst::Lw { r_a, r_b, imm } => {
                let addr = self.load_reg(r_b) + imm;
                let value = mem.load_word(addr);
                self.store_reg(r_a, value);
            }
            DecodedInst::Jalr { r_a, r_b, .. } => {
                self.store_reg(r_a, self.program_counter);

                let reg_value = self.load_reg(r_b);
                self.program_counter = reg_value.into();
            }
            DecodedInst::Beq { r_a, r_b, r_c } => {
                let r_a = self.load_reg(r_a);
                let r_b = self.load_reg(r_b);
                if r_a == r_b {
                    let r_c: i16 = self.load_reg(r_c).into();
                    let pc: u16 = self.program_counter.into();
                    let pc = pc.wrapping_add_signed(r_c);

                    self.program_counter = Word::from(pc);
                }
            }
            DecodedInst::Bne { r_a, r_b, r_c } => {
                let r_a = self.load_reg(r_a);
                let r_b = self.load_reg(r_b);
                if r_a != r_b {
                    let r_c: i16 = self.load_reg(r_c).into();
                    let pc: u16 = self.program_counter.into();
                    let pc = pc.wrapping_add_signed(r_c);

                    self.program_counter = Word::from(pc);
                }
            }
            DecodedInst::Blt { r_a, r_b, r_c } => {
                let r_a = self.load_reg(r_a);
                let r_b = self.load_reg(r_b);
                if r_a < r_b {
                    let r_c: i16 = self.load_reg(r_c).into();
                    let pc: u16 = self.program_counter.into();
                    let pc = pc.wrapping_add_signed(r_c);

                    self.program_counter = Word::from(pc);
                }
            }
            DecodedInst::Noop => {}
            DecodedInst::Retl => {
                self.handling_interrupt = false;
                self.interrupt_return.store(self.program_counter);

                let addr = self.interrupt_return.load();
                self.program_counter = addr;
            }
        }
    }

    fn handle_interupt(&mut self) {
        self.interrupt_pending = false;
        self.handling_interrupt = true;

        self.interrupt_return.store(self.program_counter);
        self.program_counter = INTERRUPT_VECTOR;
    }
}
