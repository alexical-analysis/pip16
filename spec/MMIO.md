# MMIO Registers

The MMIO Registers are how the CPU and the hardware interact to drive the pip16.
All registers are 16-bit values and the bits are named 0 - F.

## System Registers

### System Control ( 0x00_00 )

Internal BIOS use only, can not be written to and will always return 0x00_00 when read from user space.

### Bank Control ( 0x00_01 )

Controlls the active memory bank.
0 = Cartridge memory.
1 = Video memory.

| Bits  | Description                | 
| ----- | -------------------------- |
|     0 | Swap the active bank       |
| 1 - F | reserved                   |

## Display Registers

Display registers are used to controll the behavior of the system display.

### Display Control ( 0x00_02 )

This controls the general display status including interupts and turning the display on and off

| Bits  | Description                | 
| ----- | -------------------------- |
|     0 | enable/disable the display |
| 1 - F | reserved                   |

### Display VCount ( 0x00_03 )

Tracks the current scan line being rendered by the PPU

| Bits  | Description | 
| ----- | ----------- |
| 0 - F | scan line   |

## Background Registers

Background registers control and configure the 4 background layers

### Background Control ( 0x00_04 )

This register can be used to enable and disable the various backgrounds

| Bits  | Description                 |
| ----- | --------------------------- |
|     0 | enable/disable background 0 |
|     1 | enable/disable background 1 |
|     2 | enable/disable background 2 |
|     3 | enable/disable background 3 |

### Background 0 Control-X ( 0x00_05 )

Control the 0th backgrounds draw order as well as it's x position

| Bits  | Description                     |
| ----- | ------------------------------- |
| 0 - 8 | Signed x-position (-256 - 255)  |
| 9 - B | Reserved                        |
| C - F | Draw order back(0) to front(16) |

### Background 0 Control-Y ( 0x00_06 )

Control the 0th backgrounds tile base address as well as it's y position.
The tile base address is calculated the following forumula.
Tile Data Base Address + (Base Index * 8k)

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - 8 | Signed y-position (-256 - 255) |
| 9 - B | Reserved                       |
| C - F | VRAM base (0 - 8)              |

### Background 1 Control-X ( 0x00_07 )

Same as the above Control-X register but for background 1

### Background 1 Control-Y ( 0x00_08 )

Same as the above Control-Y register but for background 1

### Background 2 Control-X ( 0x00_09 )

Same as the above Control-X register but for background 2

### Background 2 Control-Y ( 0x00_0A )

Same as the above Control-Y register but for background 2

### Background 3 Control-X ( 0x00_0B )

Same as the above Control-X register but for background 3

### Background 3 Control-Y ( 0x00_0C )

Same as the above Control-Y register but for background 3
