# Hardware

This is a descriptoin of the pseudo-hardware that's used by pip16

## Display

The display is 256 pixels wide and 144 pixels tall.
This makes it the same height as the original Game Boy Color but with a 16:9 aspect ratio.
The display supports 16-bit RGB color with a 1-bit of alpha channel.
The system uses a global 32-color palette which is the maximum number of on screen colors.

## Input

They system supports a 4 direction d-pad, an X-button an O-button, L and R buttons, and an system button.

## The PPU (Picture Processing Unit)

The Pip16 supports up to 256 sprites on screen at once.
Sprites can move independenly acorss the screen and can be one of 8 sizes (8x8, 16x16, 32x32, 64x64, 
128x128, 256x256, 512x512, and 1024x1024).
Supporting such large sprite sizes allows the PPU to use a single unified system for drawing maps and
objects.
The PPU loads a single large image with all the sprites for the game at system power on which can not
be overwritten.
The sprites are addressed in 8x8 pixel chunks and the image is a 256x256 tile image (2048x2048).

### Sprites

The pip16 sprites are defined by 4 16-bit registers located in sprite attribute memory.

The first is the X-Attribute Register

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - 9 | Signed x-position (-512 - 511) |
|     A | Horizontal flip                |
|     B | Vertical flip                  |
| C - F | Sprite rotation                |

The second is the Y-Attribute Regiseter

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - 9 | Signed y-position (-512 - 511) |
| A - C | Sprite Size                    |

The third is the Index Register

| Bits  | Description                    |
| ----- | ------------------------------ |
| 0 - F | The sprite index               |
