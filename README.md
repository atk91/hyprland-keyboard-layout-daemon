# Keyboard layout daemon for Hyprland
## Build
```
$ cargo build --release
```
The executable will be at `target/release/layout-daemon`

## How to use
Launch it at hyprland start - for example, add `exec-once = /path/to/layout-daemon`
to your hyprland.conf.
For now only Us/Ru layouts are supported, so make sure you have 
`kb_layout=us,ru` configured in `input` section of hyprland config

The daemon will switch layout to last used on each window switch.
On new windows the layout will be set to default which is now hardcoded to be `Us`.
