# bg2-editor

A save file editor for **Baldur's Gate II: Enhanced Edition** (and, incidentally, BG1:EE — see below), focused on character attributes, inventory, and spellbook — not the whole Infinity Engine resource set. Built with Rust + [egui](https://github.com/emilk/egui), in the same spirit as [pal-editor](../pal-editor): a small, single-purpose tool with a UI you don't need a wiki to understand.

The on-disk formats (GAM save file, embedded CRE creature structure) are reverse-engineered from [NearInfinity](https://github.com/InfinityTools/NearInfinity)'s source — NearInfinity is the reference implementation for Infinity Engine formats, but its UI is a dense generic resource browser; this project reuses its format knowledge without inheriting that UI.

## Status

**Implemented:**
- Full binary parser/serializer for the GAM V2.0 save format (BG1:EE/BG2/BG2:EE), including embedded CRE V1.0 party-member creature data (attributes, proficiencies, inventory, spellbook).
- UI, four tabs per party member:
  - **Abilities & Combat** — ability scores, HP, AC, THAC0, saves, resistances.
  - **Class & Skills** — levels, weapon proficiency pips, thief skills.
  - **Inventory** — all 38 equipped slots (helmet/armor/weapons/rings/etc., picked from the character's item list) plus the full item list itself (add/remove, quantities/charges, identified flag).
  - **Spells** — known spells (add/remove) and memorized spells grouped by spell-level/class block (add/remove whole levels, add/remove/toggle individual memorized spells, edit max/current memorizable counts).
  - Items and spells are edited by raw resource code (e.g. `SW1H01`, `SPWI112`); the resolved display name is shown alongside once a game folder is set (see below).
- Save writes back with an automatic `baldur.gam.bak` backup.
- **Game-data name resolution** ("🎮 Set Game Folder…" in the toolbar, remembered across launches): reads `chitin.key` + the referenced `.bif` archives + `dialog.tlk` to resolve item/spell resrefs to their in-game names, and `.IDS` tables (`CLASS.IDS`, `RACE.IDS`, `GENDER.IDS`, `ALIGNMEN.IDS`, `EA.IDS`, `KIT.IDS`) to turn Class/Race/Gender/Alignment/Allegiance/Kit into searchable dropdowns instead of raw numeric IDs. Falls back to raw codes/numbers whenever no game folder is set (or a resref/ID isn't found — e.g. a modded item).

**Validated against real save data and a real game install** (three BG1:EE saves from an actual playthrough, cross-checked against a real local BG1:EE `chitin.key`+`dialog.tlk`; one save has 1 party member + 36 embedded non-party creatures, each with their own spells/items): parsing, in-memory round trip, and the full load → edit → `save_with_backup` → reload write path all produce **byte-for-byte identical** output to the original `baldur.gam` aside from the intended edit. The inventory/spellbook mutation helpers (add/remove item, add/remove known spell, add/remove memorized spell, add/remove a whole spell-level block — each of which has to keep index/offset bookkeeping consistent across the CRE's variable-length sections) are also exercised against real save data. Name resolution was checked against a real character (a Cavalier Paladin) and correctly resolved every equipped item, known spell, and memorized spell, plus Class/Race/Gender/Alignment/Allegiance/Kit. See `examples/inspect.rs`, `examples/test_save.rs`, `examples/test_mutations.rs`, and `examples/test_gamedata.rs`.

**Not yet implemented:**
- A searchable item/spell *picker* (autocomplete over the full game catalog) — for now you type the resource code and the resolved name confirms it's right, rather than searching by name to find the code.

## Build

```powershell
cargo build --release
```

Output: `target\release\bg2-editor.exe`.

Note: if building from a shell where Git for Windows' `usr\bin` precedes the Visual Studio tools on `PATH`, the MSVC linker gets shadowed by Git's POSIX `link.exe` and the build fails with a linker error. Build from a shell where the VS toolchain's `link.exe` resolves first (e.g. plain PowerShell rather than a Git Bash-derived PATH).

## Verification

```powershell
cargo test
```

Runs structural round-trip tests (`parse(serialize(x)) == x`) for both the CRE and GAM layers against synthetic sample data.

```powershell
cargo run --example inspect -- "<save-folder>"
cargo run --example test_save -- "<scratch-copy-of-a-save-folder>"
cargo run --example test_mutations -- "<save-folder>"
cargo run --example test_gamedata -- "<game-install-root>" "<save-folder>"
```

`inspect` is read-only: loads a real save, prints key fields, and reports whether an in-memory serialize+reparse is byte-identical to the original file. `test_save` exercises the actual write path (edits a stat, calls `save_with_backup`, verifies the backup matches the pre-edit original and the reload picks up the edit) — always run it against a scratch copy, never a save folder you care about. `test_mutations` is read-only: runs the inventory/spellbook add/remove helpers against a real character and checks the result still round-trips consistently. `test_gamedata` is read-only: loads a real game install's `chitin.key`/`dialog.tlk` and resolves every item/spell/class/race/alignment/kit for a real character.

Not yet verified: loading an edited save back into the actual game.

## Project structure

```
src/
  main.rs        — eframe entry point
  app.rs         — egui UI: toolbar, save list, per-character tabs
  config.rs       — remembers the last-used game install folder across launches
  save_file.rs     — save-folder discovery, load/save with backup
  format/
    primitives.rs — LE read/write helpers, ResRef, Writer
    gam.rs         — GAM V2.0 file: header, party/non-party members, globals, journal, familiar, stored locations
    cre.rs         — embedded CRE V1.0 creature structure: attributes, proficiencies, inventory, spellbook
  gamedata/
    key.rs         — chitin.key: BIFF-entry table, resource-entry table, locator bit-packing
    bif.rs         — .bif archives: plain BIFFV1, whole-file-compressed BIF V1.0, block-compressed BIFCV1.0
    tlk.rs          — dialog.tlk string table (strref -> text)
    ids.rs          — .IDS symbol table text format (CLASS.IDS, RACE.IDS, etc.)
    mod.rs           — GameData: ties the above together into item/spell/IDS name resolution, with caching
```

## Binary format notes

- A BG2:EE save is a **folder** containing `baldur.gam` as a plain, uncompressed file — no zlib container involved (that's only used for the save's area-state `.sav` files, which this editor doesn't touch).
- Party members are embedded directly in `baldur.gam` as full CRE V1.0 structures (not separate `.cre` files); editing a save means re-serializing the whole GAM file with every section offset recomputed, since inventory/spellbook edits change section sizes.
- Byte offsets were verified directly against `NearInfinity`'s `GamResource.java`, `PartyNPC.java`, and `CreResource.java` (`readOther()`), not just its wiki documentation.

## BG1:EE

BG1:EE saves use the same GAM V2.0 / CRE V1.0 format as BG2:EE (NearInfinity's `Profile.java` groups `BG1 || BG2 || EE` under the same CRE version support), so this editor should work for BG1:EE saves too, modulo game-specific content (handled dynamically via the game install once the `gamedata` resolver exists, rather than hardcoded).
