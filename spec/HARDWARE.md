# Hardware

This is a description of the pseudo-hardware that's used by the pip16.

## Dsiplay

The display is 256 pixels wide and 144 pixels tall.
This makes it the same height as the original Game Boy Color but with a 16:9 aspect ratio.
The display supports 16-bit RGB color with a 1-bit of alpha channel.
However, because the system uses 4-color color paletts the screen can only actually display 112 simultanious 
colors.
3 colors for the 16 sprite palettes (color-0 is always transparent for sprites) and 4 colors for the 16 
background palettes (3\*16 + 4\*16 = 112 colors).

## Input

## The PPU (Picture Processing Unit)

### Sprites

The pip16 supports up to 256 sprites on screen at a once.
Sprites can move independenly acorss the screen and can be one of 4 sizes (8x8, 16x16, 32x32, 64x64)
Sprites can also have 1 of 4 priorities (0 - 3) with 0 being drawn first and 3 being drawn last.
Sprites drawn first will be covered by sprites that are drawn last.
Sprites are controlled by sprite attribute memory and each sprite uses 3 16-bits attribute registers.

The first is the X-Attribute Register

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - 9 | Signed x-position (-512 - 511) |
| A - B | Reserved                       |
|     C | Horizontal flip                |
|     D | Vertical flip                  |
| E - F | Sprite Size (8x8 to 64x64)     |

The second is the Y-Attribute Regiseter

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - 9 | Signed y-position (-512 - 511) |
| A - B | Reserved                       |
| C - F | Sprite rotation                |

The final sprite attribute register is the Tile-Attribute Register.

| Bits  | Description         |
| ----- | ------------------- |
| 0 - 8 | Tile ID (0 - 511)   |
|     9 | Reserved            |
| A - C | VRAM base (0 - 7)   |
| D - F | Palette ID (0 - 15) |

The tile base address is calculated as following:
Tile Data Base Address + (VRAM base * 8K)

### Backgrounds

There are 4 background layers than can be active simultaniously.
Each background can be moved independently and each background is 48x32 tiles (384x256 pixels).
Backgrounds also wrap when they reach the edge of the screen.
This allows backgrounds to implement smooth scrolling with new tiles streaming in as they are needed.

Backgrounds are controlled by using their individual background controll registers.
In addition to the controll registers, each backgroud has memory reserved for it's tile data.

## The APU  (Audio Processing Unit)

## Cartridges

*contains 2 primary sections (maybe more eventually but just 2 for now)*
  - Program Data: this is where your code and data live
  - GFX Data: this is where you initial map, tile, and palete data lives.

This is nice because for simple games where you don't need to make changes to the backgrounds palettes
all the gfx stuff you need is just loaded for you automatically.
