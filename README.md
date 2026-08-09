# bg2-editor

A save file editor for **Baldur's Gate II: Enhanced Edition** (and BG1:EE, which shares the same save format), focused on character attributes, inventory, and spellbook — not the whole Infinity Engine resource set. Built with Rust + [egui](https://github.com/emilk/egui): a small, single-purpose tool with a UI you don't need a wiki to understand.

The on-disk formats (GAM save file, embedded CRE creature structure, KEY/BIF/TLK/IDS/ITM/SPL/BAM game data) are reverse-engineered from [NearInfinity](https://github.com/InfinityTools/NearInfinity)'s source — the reference implementation for Infinity Engine formats — without inheriting its dense generic resource-browser UI.

## Features

- Edit ability scores, HP, AC, THAC0, saves, resistances, class/level, weapon proficiencies, and thief skills for every party member.
- Full inventory editing: all 38 equipped slots plus the item list (add/remove, quantities/charges, identified flag).
- Full spellbook editing: known and memorized spells, grouped by level/class, with add/remove/toggle.
- Items and spells can be typed by resource code or picked by name from a searchable, categorized catalog with icons, stats, and descriptions pulled from the actual game install.
- Character portraits, and class/race/kit/item/spell names shown in the game's real localized text (not raw internal IDs), in whatever language your game is installed in.
- Safe writes: an automatic `.bak` backup is made before every save.

## Build

```powershell
cargo build --release
```

Output: `target\release\bg2-editor.exe`.

> If building from a shell where Git for Windows' `usr\bin` precedes the Visual Studio tools on `PATH`, the MSVC linker gets shadowed by Git's POSIX `link.exe`. Build from a shell where the VS toolchain's `link.exe` resolves first (e.g. plain PowerShell).

## Testing

```powershell
cargo test
```

Runs structural round-trip tests for the CRE and GAM format layers. There's also a set of `examples/` that exercise the parser against real save/game-install data — see [CLAUDE.md](CLAUDE.md) for details.

## Project structure

```
src/
  main.rs, app.rs      — eframe entry point and egui UI
  config.rs             — persisted settings (last-used folders)
  save_file.rs           — save-folder discovery, load/save with backup
  theme.rs, fonts.rs      — UI theme and CJK font support
  format/                — GAM save file + embedded CRE creature structure
  gamedata/               — game install readers: KEY/BIF archives, dialog.tlk,
                            .IDS/.2DA tables, ITM/SPL/BAM resources, name resolution
```

## Status

Actively developed and validated against real BG1:EE save data and a real game install (parsing, editing, and the full write path have been verified byte-for-byte). BG2:EE support uses the same format but hasn't been tested against a real BG2:EE save yet. See [CLAUDE.md](CLAUDE.md) for the detailed changelog, verification notes, and known gaps.
