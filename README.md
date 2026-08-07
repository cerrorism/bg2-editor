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
  - Items and spells can be typed by raw resource code (e.g. `SW1H01`, `SPWI112`, with the resolved name shown alongside) **or** picked by name via a "🔍" search popup — available on every item/spell row, plus dedicated "Add via Search" buttons for adding a new item, known spell, or memorized spell straight from the catalog.
- Save writes back with an automatic `baldur.gam.bak` backup.
- **Game-data name resolution** ("🎮 Set Game Folder…" in the toolbar, remembered across launches): reads `chitin.key` + the referenced `.bif` archives + `dialog.tlk` to resolve item/spell resrefs to their in-game names (and build full searchable item/spell catalogs — 1423 items / 802 spells in a real BG1:EE install, built in ~3ms), and `.IDS` tables (`CLASS.IDS`, `RACE.IDS`, `GENDER.IDS`, `ALIGNMEN.IDS`, `EA.IDS`, `KIT.IDS`) to turn Class/Race/Gender/Alignment/Allegiance/Kit into dropdowns instead of raw numeric IDs. Falls back to raw codes/numbers whenever no game folder is set (or a resref/ID isn't found — e.g. a modded item).

**Validated against real save data and a real game install** (three BG1:EE saves from an actual playthrough, cross-checked against a real local BG1:EE `chitin.key`+`dialog.tlk`; one save has 1 party member + 36 embedded non-party creatures, each with their own spells/items): parsing, in-memory round trip, and the full load → edit → `save_with_backup` → reload write path all produce **byte-for-byte identical** output to the original `baldur.gam` aside from the intended edit. The inventory/spellbook mutation helpers (add/remove item, add/remove known spell, add/remove memorized spell, add/remove a whole spell-level block — each of which has to keep index/offset bookkeeping consistent across the CRE's variable-length sections) are also exercised against real save data. Name resolution and catalog-building were checked against a real character (a Cavalier Paladin) and a real install: every equipped item, known spell, and memorized spell resolved correctly, as did Class/Race/Gender/Alignment/Allegiance/Kit, and the full item/spell catalogs came out clean (a handful of internal "&lt;NO TEXT&gt;" stub records are filtered out). See `examples/inspect.rs`, `examples/test_save.rs`, `examples/test_mutations.rs`, and `examples/test_gamedata.rs`.

The item/spell picker itself (the popup window, search box, click-to-select) is standard native-GUI interaction that couldn't be scripted/automated the way the rest of this was verified — it compiles cleanly, reuses the same catalog-building logic already validated above, and the app launches/runs without error, but hasn't been click-tested by hand yet.

**Also fixed, from real user-reported issues:**
- **CJK text rendering** — egui's bundled fonts only cover Latin script, so a Chinese party member name rendered as empty boxes. `src/fonts.rs` now loads a system CJK font (e.g. Microsoft YaHei) as a fallback if one is found. Verified with `ab_glyph` (the same font-parsing crate egui uses internally) that the font it picks actually contains glyphs for a real Chinese name from a real save — see `examples/test_fonts.rs`.
- **Folder persistence** — both the save folder and game folder are now remembered across launches (previously only the game folder was), and the auto-detect for the default save folder now also checks OneDrive-redirected `Documents` folders, not just the plain one.
- **A real, confirmed data-corruption bug** — several `DragValue` widgets used a `.range()` narrower than the field's legitimate values (e.g. spell level `1..=9` when `0` is a valid real placeholder; per-creature reputation `0..=20` when real data can be up to 255). egui's `DragValue` silently clamps an out-of-range *pre-existing* value into range the moment it's rendered, with no user interaction needed — so just opening the Spells tab flipped every unused "level 0" slot to "level 1" (including one *active* slot holding 7 real memorized spells) and clamped a reputation value of 120 down to 20. This is almost certainly what crashed the game on load. Confirmed via byte-level diffing of the user's actual save file across two edits, then reproduced and fixed at the widget level in `examples/test_dragvalue_fix.rs` (headlessly renders the exact widgets with the exact real values and asserts they survive unchanged). While fixing this, also corrected several fields' signedness (resistances, saves, THAC0, luck, turn-undead-level were typed `u8` but are actually signed `i8` in the format — meaning e.g. a vulnerability's legitimately-negative resistance would have been misread *and* then clamped into a positive range by the same bug). All numeric ranges in the UI are now either the field's true full native range, or a closed enum's exact valid set (e.g. attacks/round) — never a "typical gameplay value" guess.

**Not yet implemented:**
- Nothing on the original request list — attributes, inventory, spellbook, and name resolution (including a picker) are all in place. Possible next steps: verifying an edited save actually loads in-game now that the corruption bug is fixed, and BG2:EE-specific validation (everything so far has been tested against BG1:EE, since that's the save data available).

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
cargo run --example test_fonts
cargo run --example test_dragvalue_fix
cargo run --example diff_gam -- "<a.gam>" "<b.gam>"
cargo run --example dump_cre -- "<save-folder>"
```

`inspect` is read-only: loads a real save, prints key fields, and reports whether an in-memory serialize+reparse is byte-identical to the original file. `test_save` exercises the actual write path (edits a stat, calls `save_with_backup`, verifies the backup matches the pre-edit original and the reload picks up the edit) — always run it against a scratch copy, never a save folder you care about. `test_mutations` is read-only: runs the inventory/spellbook add/remove helpers against a real character and checks the result still round-trips consistently. `test_gamedata` is read-only: loads a real game install's `chitin.key`/`dialog.tlk` and resolves every item/spell/class/race/alignment/kit for a real character. `test_fonts` checks the system CJK font `fonts::setup` would pick actually contains the needed glyphs. `test_dragvalue_fix` headlessly reproduces (and verifies the fix for) the DragValue range-clamping corruption bug described above. `diff_gam`/`dump_cre` are read-only ad-hoc diagnostics used to investigate that bug — a byte-level diff between two `.gam` files, and a full field dump of a save's first party member.

Not yet verified: loading an edited save back into the actual game, and hands-on testing of the item/spell picker popup itself.

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
