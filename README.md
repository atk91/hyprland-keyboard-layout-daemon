# Keyboard layout daemon for Hyprland

## Description
Fully non-blocking single-threaded per-window keyboard layout daemon for Hyprland.

## Build
```
$ cargo build --release
```
The executable will be at `target/release/layout-daemon`. You might want to `strip` it.


## How to use
Launch it at hyprland start - for example, add `exec-once = /path/to/layout-daemon`
to your hyprland.conf.

The daemon will switch layout to last used on each window switch.
On new windows the layout will be set to default which is now hardcoded to be `English (Us)`.
Also while switching to [Rofi](https://github.com/davatorium/rofi) layout is enforced to be `English (Us)`.
