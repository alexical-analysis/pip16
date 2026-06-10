# MMIO Registers

The MMIO block occupies addresses 0xFFC0 – 0xFFFF (64 word addresses = 128 bytes)
in all three memory banks. All registers are 16-bit. Unassigned bit fields are
reserved and should be written as 0; reads may return 0 or undefined values.

Registers marked **read-only** ignore writes.
Registers marked **BIOS-only** always read as 0x0000 in user mode and can only
be written by BIOS code.

---

## System Registers

### System Control — 0xFFC0 (BIOS-only)

Internal BIOS use. Reads as 0x0000 from user mode; writes are ignored.

### Bank Control — 0xFFC1

Selects the active memory bank for SW and LW instructions.
Does not affect instruction fetches; the program counter always reads from the
Default bank.

| Bits  | Description                                 |
| ----- | ------------------------------------------- |
| 0 – 1 | Active bank (0 = Default, 1 = PPU, 2 = APU) |
| 2 – F | Reserved                                    |

### Interrupt Enable — 0xFFC2

Enables individual interrupt sources. When a bit is set and the corresponding
event occurs, the CPU is interrupted and execution jumps to the interrupt handler.

| Bit | Interrupt Source                         |
| --- | ---------------------------------------- |
| 0   | VBlank (display finished one full frame) |
| 1   | HBlank (display finished one scan line)  |
| 2   | PPU Done (sprite draw complete)          |
| 3   | APU Done (APU operation complete)        |
| 4   | Button press (any button state change)   |
| 5–F | Reserved                                 |

### Interrupt Status — 0xFFC3

Each bit is set by hardware when the corresponding interrupt fires.
Write 1 to a bit to acknowledge and clear it.
Read to determine which interrupts are pending.

Bit layout matches Interrupt Enable (0xFFC2).

---

## Display Registers

### Display Control — 0xFFC4

Controls global display state.

| Bits  | Description             |
| ----- | ----------------------- |
| 0     | Display enable (1 = on) |
| 1 – F | Reserved                |

### Display VCount — 0xFFC5 (read-only)

Current scan line being rendered by the PPU.
Counts 0 – 159 during active display and continues through the VBlank period.

| Bits  | Description       |
| ----- | ----------------- |
| 0 – 7 | Current scan line |
| 8 – F | Reserved          |

---

## Sprite Registers

These registers control the single-sprite draw operation.
To draw a sprite: write the position, tile, and attribute registers, then write 1
to Sprite Control. The CPU tick counter advances by the fixed PPU cycle cost on
activation. See HARDWARE.md for the cost values.

### Sprite X — 0xFFC6

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 – F | Signed X position (–128 – 127) |

### Sprite Y — 0xFFC7

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 – F | Signed Y position (–128 – 127) |

### Sprite Tile — 0xFFC8

| Bits  | Description                         |
| ----- | ----------------------------------- |
| 0 – 8 | Tile index within tile bank (0–511) |
| 9 – F | Reserved                            |

### Sprite Attributes — 0xFFC9

| Bits  | Description                              |
| ----- | ---------------------------------------- |
| 0 – 1 | Size (0=8×8, 1=16×16, 2=32×32, 3=64×64) |
| 2     | Horizontal flip                          |
| 3     | Vertical flip                            |
| 4 – 7 | Tile data bank (0–11)                    |
| 8 – 9 | Rotation (0=0°, 1=90°, 2=180°, 3=270°)  |
| A – F | Reserved                                 |

### Sprite Control — 0xFFCA

Write 1 to trigger a sprite draw using the values in the sprite registers above.
The PPU Done interrupt fires when the draw is complete.
Write 0 has no effect.

| Bits  | Description            |
| ----- | ---------------------- |
| 0     | Trigger draw (write 1) |
| 1 – F | Reserved               |

---

## Background Registers

Scroll values wrap at the 32×32 tile map boundary (256 pixels).

### Background Scroll-X — 0xFFCB

| Bits  | Description               |
| ----- | ------------------------- |
| 0 – 7 | Horizontal scroll (0–255) |
| 8 – F | Reserved                  |

### Background Scroll-Y — 0xFFCC

| Bits  | Description             |
| ----- | ----------------------- |
| 0 – 7 | Vertical scroll (0–255) |
| 8 – F | Reserved                |

### Background Control — 0xFFCD

| Bits  | Description                            |
| ----- | -------------------------------------- |
| 0     | Enable                                 |
| 1 – 4 | Tile data bank (0–11, selects 8K bank) |
| 5 – F | Reserved                               |

---

## Input Registers

### Button State — 0xFFCE (read-only)

Each bit reflects the current state of a button (1 = pressed).

| Bit | Button        |
| --- | ------------- |
| 0   | D-pad Up      |
| 1   | D-pad Down    |
| 2   | D-pad Left    |
| 3   | D-pad Right   |
| 4   | A button      |
| 5   | B button      |
| 6   | L shoulder    |
| 7   | R shoulder    |
| 8   | System button |
| 9–F | Reserved      |

---

## APU Registers

The APU has 4 channels. Each channel has a frequency register and a control register.
Activating the APU via APU Control advances the CPU tick counter by a fixed hardware
cost analogously to the PPU timing model. See HARDWARE.md for the fixed tick values.

### APU Control — 0xFFCF

| Bits  | Description                         |
| ----- | ----------------------------------- |
| 0     | APU enable (1 = on)                 |
| 1     | Trigger playback (write 1 to start) |
| 2 – F | Reserved                            |

### APU Channel 0 Frequency — 0xFFD0

Square wave channel. Frequency is specified as a 16-bit divisor applied to the
APU base clock. Lower values produce higher frequencies.

| Bits  | Description       |
| ----- | ----------------- |
| 0 – F | Frequency divisor |

### APU Channel 0 Control — 0xFFD1

| Bits  | Description                                   |
| ----- | --------------------------------------------- |
| 0 – 3 | Volume (0–15)                                 |
| 4 – 5 | Duty cycle (0=12.5%, 1=25%, 2=50%, 3=75%)    |
| 6     | Loop (1 = loop playback)                      |
| 7     | Channel enable                                |
| 8 – F | Reserved                                      |

### APU Channel 1 Frequency — 0xFFD2

Triangle wave channel. Same frequency divisor format as channel 0.

### APU Channel 1 Control — 0xFFD3

| Bits  | Description    |
| ----- | -------------- |
| 0 – 3 | Volume (0–15)  |
| 4     | Loop           |
| 5     | Channel enable |
| 6 – F | Reserved       |

### APU Channel 2 Frequency — 0xFFD4

Custom waveform channel. Frequency at which the waveform address pointer advances.

### APU Channel 2 Control — 0xFFD5

| Bits  | Description    |
| ----- | -------------- |
| 0 – 3 | Volume (0–15)  |
| 4     | Loop           |
| 5     | Channel enable |
| 6 – F | Reserved       |

### APU Channel 2 Waveform Address — 0xFFD6

Start address of the waveform data in APU bank memory (word address).

### APU Channel 2 Waveform Length — 0xFFD7

Length of the waveform in words.

### APU Channel 3 Frequency — 0xFFD8

Noise channel. Controls the rate of the pseudo-random noise generator.

### APU Channel 3 Control — 0xFFD9

| Bits  | Description                        |
| ----- | ---------------------------------- |
| 0 – 3 | Volume (0–15)                      |
| 4     | Short mode (1 = 31-step, 0 = long) |
| 5     | Loop                               |
| 6     | Channel enable                     |
| 7 – F | Reserved                           |

---

## Reserved

0xFFDA – 0xFFFF are reserved for future expansion.
