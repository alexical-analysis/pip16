# Memory Map

There are 2 distinct memory spaces in the pip16.
Each memory space supports 128k of addressable memory.
Memory is addressed in 2-byte sections (or 1 word).
This means that while each memory space has 128k of total memory, there are only 64k of addresses.

The memory is broken up into Cart Memory and Video Memory.
Cart Memory contains MMIO Registers, RAM, Save ROM, and Cart ROM.
Video Memory contains a mirror of MMIO Registers, a mirror of RAM, VRAM, and the BIOS.
The MMIO Registers and RAM are mirrored between the two memory spaces for convience.

The bank control register is used to control where instructions load and store data to.
When the register is 0, Cart Memory is in use, regiser 1 is Video Memory.

## Cart Memory

Cart Memory is loaded from the user cartrige on startup.
The CPU can only read instructions from the Cart Memory and the program counter does not respect the 
bank controll regiser.
The 32K of general purpose RAM is also located here.
90K of ROM is loaded directly from the user cartridge and repesents the executable code to run.
At the bottom of the memory space there is 5K of persisten save data that can be written to.

| Starting Address | Size | Addresses | Description         |
| ---------------- | ---- | --------- | ------------------- |
|          0x00_00 | 1K   | 512       | BIOS                |
|          0x02_00 | 1K   | 512       | MMIO Registers      |
|          0x04_00 | 32K  | 16K       | General purpose RAM |
|          0x44_00 | 4K   | 4K        | Save SRAM           |
|          0x54_00 | 90K  | 45K       | Cart ROM            |

## Video Memory

Video Memory is designed for use by the PPU.
It holds all the data for sprites, tiles, and backgrounds.
The largest chunk of data is the 64k of tile data that provide the underlying graphics for both sprits 
and background tile maps.
This memory space also stores pallet data, sprite attirbute data, and the BIOS usesd on startup.

While it's true that normal execution can only read instructions from Cart Memory, the opposite is actually
true durring startup.
The CPUs program counter is initalized at 0xE2_00 pointing at the start of the BIOS and reads instructions
directly from the Video Memory segment.
Once all the initialization code has run the BIOS switches the CPU into user mode, the PC jumps to 0x54_00,
and the CPU begins reading instructions from the Cart Memory.

| Starting Address | Size | Addresses | Description                    |
| ---------------- | ---- | --------- | ------------------------------ |
|          0x00_00 | 1k   | 512       | MMIO Registers (mirrored)      |
|          0x02_00 | 32K  | 16K       | General purpose RAM (mirrored) |
|          0x42_00 | 12K  | 6K        | Background Tile Data           |
|          0x5A_00 | 2K   | 1K        | Pallet Data                    |
|          0x5E_00 | 2K   | 1K        | Sprite Attribute Data          |
|          0x62_00 | 64K  | 32K       | Tile Data                      |
|          0xE2_00 | 15K  | 7.5K      | BIOS (read only)               |
