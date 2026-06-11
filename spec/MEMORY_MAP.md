# Memory Map

## Overview

The Pip16 has three distinct 128K address spaces, selected by the Bank Control MMIO register.
Each address space is word-addressed (1 address = 1 16-bit word = 2 bytes), giving
64K addresses per bank (0x0000 – 0xFFFF) for a total of 128K bytes per bank.

| Bank | Name    | Selected by Bank Control |
| ---- | ------- | ------------------------ |
| 0    | Default | 0                        |
| 1    | PPU     | 1                        |
| 2    | APU     | 2                        |

**The program counter always references the Default bank.** Bank control only
affects SW (store) and LW (load) instructions.

### Mirrored Regions

To avoid requiring a bank switch for common operations, the 32K RAM block and the
128-byte MMIO block occupy the same top-of-address-space positions in all three banks.
Writes through any bank to these regions affect the same physical memory.

| Region      | Addresses         | Size |
| ----------- | ----------------- | ---- |
| MMIO        | 0xFFC0 – 0xFFFF   | 128B |
| General RAM | 0xBFC0 – 0xFFBF   | 32K  |

---

## Default Bank (Bank 0)

Loaded from the cartridge at power-on.
All instruction fetches happen here regardless of the active bank.

| Start  | End    | Size  | Description                  |
| ------ | ------ | ----- | ---------------------------- |
| 0x0000 | 0xB7BF | ~90K  | Cart ROM                     |
| 0xB7C0 | 0xBFBF | 4K    | Save SRAM (cartridge-unique) |
| 0xBFC0 | 0xFFBF | 32K   | General Purpose RAM          |
| 0xFFC0 | 0xFFFF | 128B  | MMIO Registers               |

### Cart ROM (0x0000 – 0xB7BF)

46,016 word addresses = ~90K bytes of read-only program and data storage.
This region is mapped directly from the cartridge Program Data section at boot.
The CPU begins execution at 0x0000 after the BIOS hands off control.

### Save SRAM (0xB7C0 – 0xBFBF)

4K of persistent per-cartridge storage backed by console onboard SRAM.
At boot the BIOS maps the cartridge's allocated save slot here using the
cartridge ROM hash as the key. See HARDWARE.md for the full SRAM model.

### General Purpose RAM (0xBFC0 – 0xFFBF)

32K of volatile read/write memory. Available in all three banks via mirroring.
The stack and heap live here by convention.

### MMIO Registers (0xFFC0 – 0xFFFF)

128 bytes of memory-mapped I/O. Available in all three banks via mirroring.
See MMIO.md for the complete register reference.

---

## PPU Bank (Bank 1)

Holds all graphics data. Switched in via Bank Control for PPU data access.

| Start  | End    | Size   | Description                    |
| ------ | ------ | ------ | ------------------------------ |
| 0x0000 | 0x00FF | 512B   | Palette Data                   |
| 0x0100 | 0x04FF | 2K     | Background Tile Map            |
| 0x0500 | 0xBFBF | ~93.5K | Tile Graphics Data             |
| 0xBFC0 | 0xFFBF | 32K    | General Purpose RAM (mirrored) |
| 0xFFC0 | 0xFFFF | 128B   | MMIO Registers (mirrored)      |

### Palette Data (0x0000 – 0x00FF)

512 bytes (256 word addresses). Holds all 16 palette definitions.

| Offset        | Description                             |
| ------------- | --------------------------------------- |
| 0x0000–0x003F | Palettes 0–15 (4 colors × 16 palettes) |
| 0x0040–0x00FF | Reserved                                |

Each palette entry is one 16-bit RGBA5551 word. Colors default to opaque (alpha=1).
Setting alpha=0 marks a color as transparent. The PPU always uses palette 0 for
single-sprite draws; background tile maps reference palettes by ID.

### Background Tile Map (0x0100 – 0x04FF)

2K holding a single 32×32 tile map (1024 word addresses = 32×32 entries).

Each tile map entry is one 16-bit word:

| Bits  | Description                              |
| ----- | ---------------------------------------- |
| 0 – 8 | Tile index within tile data bank (0–511) |
| 9     | Horizontal flip                          |
| A     | Vertical flip                            |
| B – E | Palette ID (0–15)                        |
| F     | Reserved                                 |

### Tile Graphics Data (0x0500 – 0xBFBF)

~93.5K holding all tile pixel data.
Organized as 8K banks of 512 tiles each. The tile bank field in sprite attributes
and background control registers selects the active bank (0–11).

Each 8×8 tile is stored as 16 bytes (8 words) of 2-bit-per-pixel palette indices,
packed 4 pixels per byte, row by row.

---

## APU Bank (Bank 2)

Holds waveform and sample data for the APU. Structured analogously to the PPU bank.

| Start  | End    | Size  | Description                    |
| ------ | ------ | ----- | ------------------------------ |
| 0x0000 | 0xBFBF | ~96K  | Waveform / Sample Data         |
| 0xBFC0 | 0xFFBF | 32K   | General Purpose RAM (mirrored) |
| 0xFFC0 | 0xFFFF | 128B  | MMIO Registers (mirrored)      |

### Waveform / Sample Data (0x0000 – 0xBFBF)

~96K of raw waveform data used by APU channel 2 (custom waveform channel).
The APU channel 2 control register specifies the start address and length of
the waveform to play. All other channels (square, triangle, noise) are fully
register-driven and do not consume space here.
