/// bank controll is used to set the active memory bank
/// 0 = Cartridge memory space (MMIO Registers, RAM, ROM, Save ROM)
/// 1 = Video memory space (MMIOR Registers, RAM, Backgrounds, Pallets, Sprites, Tile Data)
pub const BANK_CONTROL: usize = 0x00_00;

/// interrupt program counter stores the program counter before the cpu jumps to the interrupt vector
pub const INTERRUPT_PROGRAM_COUNTER: usize = 0x00_01;

/// the interrupt vector is where the CPU jumps to handle an interrupt.
pub const INTERRUPT_VECTOR: u16 = 0xFF_FD;
