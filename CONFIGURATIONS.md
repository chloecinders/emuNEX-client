# CONFIGURATIONS

This file keeps track of all known working emulator configurations.

## Magic Values

numeric: used in save file extensions to save all files with a numeric extension. Used for snes9x which saves its save states as .000 to .099

## Replacements:

- $rom_name: name of the rom (usually game_id + extension)
- $game_id: id of the game (comes from the server)
- $console: console string
- $data_dir: domain data directory
- $save_dir: domain save file directory
- $temp_dir: temp directory where the rom is moved to while playing a game
- $emu_dir: domain emulator directory
- $documents: documents folder on the users device (probably user/Documents but can also be user/OneDrive/Documents or user/My Documents, etc.)

## Windows

#### VBA-M

- Run Command: `$rom`
- Assigned Consoles: GB, GBA, GBC
- Save Path: `$temp_dir`
- Save File Extension: .sav, .sgm
- Input Config File: `%LocalAppData%/visualboyadvance-m/vbam.ini`
- Input Mapper: `vbam`

#### mGBA

- Run Command: `$rom`
- Assigned Consoles: GB, GBA, GBC
- Save Path: `$temp_dir`
- Save File Extension: .sav, .sgm
- Input Config File: `$emu_dir/config.ini`
- Input Mapper: `mgba`

#### Mesen

- Run Command: `$rom`
- Assigned Consoles: NES
- Save Path: `$emu_dir/Saves`
- Save File Extension: .sav
- Input Config File: `$emu_dir/settings.xml`
- Input Mapper: `mesen`

#### snes9x

- Run Command: `$rom`
- Assigned Consoles: SNES
- Save Path: `$emu_dir`
- Save File Extension: .srm, numeric
- Input Config File: $emu_dir/snes9x.conf
- Input Mapper: snes9x

Snes9x needs a special config file that forces its directory to be used instead of subdirectories due to the way it loads game files. When uploading it as an emulator please use a bundle and include a config file which sets all "Dir:" prefixed settings to `.\`.
