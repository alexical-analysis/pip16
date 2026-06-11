# Pip16 Hardware Specification

## Overview

The Pip16 is a 16-bit fantasy console built around the RiSC-P instruction set.
It targets a programming model where the CPU, PPU, and APU are kept in sync by
having the CPU halt for a fixed number of cycles whenever the PPU or APU is activated.
The console runs at a locked 60 Hz frame rate.

---

## Display

The display is 160x160 pixels, organized as a 20x20 grid of 8x8 pixel tiles.
Colors are encoded as 16-bit RGBA5551 values (5-bit R, 5-bit G, 5-bit B, 1-bit alpha).

The display is palettized. There is a single palette bank of 16 palettes, each with
4 colors. All entries are shared between sprites and backgrounds.

Each palette entry is one 16-bit RGBA5551 color value, stored in Palette Data in PPU
memory. Colors are opaque by default (alpha bit = 1). Setting the alpha bit to 0 marks
a color as transparent, which is useful for sprite outlines or layered effects.

The PPU always uses palette 0 when drawing a single sprite. To change a sprite's colors,
update palette 0 before triggering the draw. Background layers reference any palette
by ID via their tile map entries.

Pixel data is stored as 2-bit palette indices (4 colors per palette per tile).

---

## Input

The console supports a standard gamepad with the following buttons:

| Button   | Description        |
| -------- | ------------------ |
| D-pad    | 4-direction input  |
| A        | Primary action     |
| B        | Secondary action   |
| L        | Left shoulder      |
| R        | Right shoulder     |
| System   | Console menu       |

Button state is exposed as a read-only MMIO register. See MMIO.md.

---

## CPU

The CPU implements the RiSC-P instruction set. See RISC-P.md for the full ISA.

The CPU has 3 memory banks selectable via the Bank Control MMIO register:

| Bank | Description       |
| ---- | ----------------- |
| 0    | Default (Cart)    |
| 1    | PPU               |
| 2    | APU               |

**The program counter always references the Default bank.** Bank control only
affects the SW and LW instructions (load/store). All instruction fetches, jumps,
and branches always read from the Default bank regardless of the bank control register.
This keeps the programming model simple: code always lives in one place.

### Timing Model

The console is locked to 60 Hz. The CPU runs at 15,360,000 Hz, giving exactly
256,000 ticks per frame (15,360,000 / 60).

When the CPU activates the PPU or APU it advances its internal tick counter by a
fixed hardware-defined number of cycles. This models the time the co-processor would
take to operate and keeps the two components in sync without requiring true parallelism.

These costs are hardware constants and are not programmable:

| Operation               | CPU ticks consumed |
| ----------------------- | ------------------ |
| PPU: draw single sprite | 16                 |
| PPU: draw background    | 64                 |
| APU: start playback     | 8                  |
| APU: stop playback      | 8                  |

---

## PPU (Picture Processing Unit)

The PPU has its own 128K address space (bank 1). See MEMORY_MAP.md for the layout.

### Single-Sprite Draw Model

The PPU uses a register-based draw model rather than a traditional sprite attribute table.
To draw a sprite, the CPU:

1. Writes the target X position to the Sprite X register.
2. Writes the target Y position to the Sprite Y register.
3. Writes the tile index to the Sprite Tile register.
4. Writes size, flip, palette, and tile bank to the Sprite Attributes register.
5. Writes 1 to the Sprite Control register to trigger the draw.

On activation the CPU's tick counter advances by the PPU Cycle Cost value, modeling
the time the PPU takes to render the sprite.

### Background Tile Maps

The PPU supports a single scrollable background layer.
The background uses a 32x32 tile map (256x256 pixels), which is larger than the
160x160 display. Scrolling the X/Y offset wraps at the map boundary, enabling
smooth tile streaming for side-scrolling or top-down maps.

Because the PPU can be activated multiple times per frame, more complex layered
scenes can be composed by drawing the background in passes with different scroll
offsets and tile banks.

Each tile map entry is one 16-bit word encoding the tile index, palette, and flip flags.
The background is configured via three MMIO registers. See MMIO.md for details.

### PPU Memory

Tile graphics data is organized into 8K banks of 512 tiles each. The tile bank
field in the sprite attribute and background control registers selects which bank
to read from. See MEMORY_MAP.md for the full PPU address space layout.

---

## APU (Audio Processing Unit)

The APU has its own 128K address space (bank 2), structured the same way as the
PPU bank. The lower ~95K holds waveform and sample data; the upper portion mirrors
RAM and MMIO. See MEMORY_MAP.md for the layout.

The APU supports 4 sound channels:

| Channel | Type         | Description                           |
| ------- | ------------ | ------------------------------------- |
| 0       | Square wave  | Configurable duty cycle               |
| 1       | Triangle     | Fixed triangle waveform               |
| 2       | Custom       | Arbitrary waveform read from APU bank |
| 3       | Noise        | Pseudo-random noise generator         |

Each channel has a frequency register and a control register for volume, envelope,
waveform address (channel 2), and enable/disable. See MMIO.md.

Like the PPU, activating the APU via its control register advances the CPU tick
counter by the APU Cycle Cost value.

---

## Cartridges

A cartridge contains three primary sections:

- **Program Data** — executable code and runtime data (loaded into Cart ROM).
- **GFX Data** — initial tile, palette, and map data (loaded into PPU bank at boot).
- **SFX Data** — waveform and sample data (loaded into APU bank at boot).

On power-on the BIOS copies GFX Data into PPU memory and SFX Data into APU memory
automatically. For simple games with static assets, no further PPU or APU memory
management is required.

---

## SRAM (Save Data)

The Pip16 console has 32K of onboard SRAM. A fraction is allocated to each
cartridge as persistent save storage.

Each cartridge stores a 16-bit hash of its own ROM contents in its header. At boot
the BIOS reads this hash and verifies it against the actual cartridge data as an
integrity check. If the hash does not match, the BIOS halts and reports an error.
If the hash is valid, it is used as the key to look up the cartridge's save slot in
an internal allocation table stored in the console SRAM. If a slot exists for that
hash, it is mapped into the Default bank at the Save SRAM region. If no slot
exists, a new 4K slot is allocated from the remaining free SRAM.

The 4K Save SRAM region in the Default bank is therefore cartridge-specific
persistent memory. It retains its value across power cycles.

Each cartridge sees exactly 4K of save space. The console can hold save data
for multiple cartridges simultaneously, up to the limit of the 32K onboard SRAM.
