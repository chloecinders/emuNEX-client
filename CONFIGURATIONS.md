# CONFIGURATIONS

This file keeps track of all known working emulator configurations.

## Replacements:

- $rom_name: name of the rom (usually game_id + extension)
- $game_id: id of the game (comes from the server)
- $console: console string
- $data_dir: domain data directory
- $save_dir: domain save file directory
- $temp_dir: temp directory where the rom is moved to while playing a game
- $emu_dir: domain emulator directory

## Windows

### GBA

#### VBA-M

- Binary Path: (binary)
- Run Command: `$rom`
- Save Path: `$temp_dir`

### SNES

#### snes9x

- Binary Path: (binary)
- Run Command: `$rom`
- Save Path: `$emu_dir`

