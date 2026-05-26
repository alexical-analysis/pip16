# pip16

A [RiSC-16](https://user.eng.umd.edu/~blj/RiSC/) virtual machine written in Rust.

RiSC-16 is a 16-bit educational ISA with 8 registers and 8 instructions: `ADD`, `ADDI`, `NAND`, `LUI`, `SW`, `LW`, `BEQ`, and `JALR`. Programs are loaded directly into the flat 64K word memory space and executed from address 0.

## Usage

```
cargo run
```

## Pip16 Specs (TODO: remove these once everything is moved into the spec folder)

Display: 16:9 aspect ratio
  256x144 pixels
  32x18 chars
  576 total chars

Backgrounds: 4 total
  384x256 pixels
  48x32 tiles
  1.5 K words/ background
  6 K words total (12K bytes)

Background Control: 32-bits (2 words)
  3-bits VRAM Base (0 - 16)
  4-bits Priority (0 - 16)
  9-bits X-pos (-256, 255)
  9-bits Y-pos (-256, 255)

Background Tile: 16-bits (1 word)
  2-bits flip L<->R and U<->D
  4-bits pallet
  9-bits char id (0 - 511)

Pallets: 16 Background, 16 Sprites
  16-bit color (RRRRR|GGGGG|BBBBB|A)
  5-bit Red
  5-bit Green
  5-bit Blue
  1-bit Alpha
  4 colors / pallet
  1K words
  2K bytes
  

Sprite Memory: 256 Sprites
  1k words
  2K bytes

Sprite Attributes: 64-bits (4 words)
  9-bits X-pos (-256, 255)
  9-bits Y-pos (-256, 255)
  9-bits char id (0 - 511)
  3-bits VRAM base
  4-bit size (8x8 - 64x64)
  4-bit pallet

Char: 8x8 pixels
  2bbp - 16 bytes / 8 words
  
Memory Spaces: 2x128K spaces
    128K total addressable memory (64k address but each address is for 1 word, not 1 byte)
    User Memory Space (Loaded from a cartridge)
    System Memory Space (Used to controll the system, cleared to 0 on startup)

User Memory Space 128K
MMIO Registers 1K
RAM 32K
Save ROM 5K
ROM 90K 

Video Memory Space 128K
MMIO Registers (mirrored from User Memory) 1K 
RAM (mirrored from User Memory) 32K
Backgrounds 12K
Pallet Memory 2K
Sprite Memory 2K
Tile Data 64K
Reserved 15K (system BIOS/ ROM)

## MMIO Registers

### Display
Display Control (16-bits/ 1 word)

### DMA 
the DMA registers allow for fast transfer of data between the User Memory Space and the System Memory Space
Source Address (16-bits / 1 word)
Destination Address (16-bits / 1 word)
Word Count (16-bits / 1 word)
DMA Controll (16-bits / 1 word)
  1-bit enable
  1-bit interrupt on complete
  1-bit status

## The PPU Design

## The System BIOS
