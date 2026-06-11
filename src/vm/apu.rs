use std::fmt::{Display, Formatter, Result};

use crate::vm::mmio::{APU_CONTROL, INT_APU_DONE, INT_ENABLE, INT_STATUS};
use crate::vm::{BankedMemory, Word};

pub struct APU;

impl Display for APU {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[ APU ]\n")
    }
}

impl APU {
    pub fn new() -> Self {
        Self
    }

    /// Steps the APU. Returns the number of CPU cycles consumed this step.
    pub fn step(&self, mem: &mut BankedMemory) -> usize {
        let apu_ctrl: u16 = mem.load_mmio(APU_CONTROL).into();
        if apu_ctrl & 0x01 == 0 {
            return 0;
        }

        mem.store_mmio(APU_CONTROL, Word::ZERO);

        let int_enable: u16 = mem.load_mmio(INT_ENABLE).into();
        if int_enable & INT_APU_DONE != 0 {
            let status: u16 = mem.load_mmio(INT_STATUS).into();
            mem.store_mmio(INT_STATUS, Word::from(status | INT_APU_DONE));
        }

        8
    }
}
