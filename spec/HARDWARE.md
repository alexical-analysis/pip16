# Hardware

This is a description of the pseudo-hardware that's used by the pip16.

## Dsiplay

The display is 256 pixels wide and 144 pixels tall.
This makes it the same height as the original Game Boy Color but with a 16:9 aspect ratio.
The display supports 16-bit RGB color with a 1-bit of alpha channel.
However, because the system uses 4-color color paletts the screen can only actually display 112 simultanious 
colors.
3 colors for the 16 sprite pallets (color-0 is always transparent for sprites) and 4 colors for the 16 
background pallets (3\*16 + 4\*16 = 112 colors).

## Input

## Sound

## Cartridges

*contains 2 primary sections (maybe more eventually but just 2 for now)*
  - Program Data: this is where your code and data live
  - GFX Data: this is where you initial map, tile, and pallet data lives.

This is nice because for simple games where you don't need to make changes to the backgrounds palletes
all the gfx stuff you need is just loaded for you automatically.
