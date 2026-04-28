# GBA-Rom-Explorer

# Fire Emblem GBA ROM Explorer — Project Design

## What it is

A Rust TUI application for browsing and inspecting the internals of GBA Fire Emblem ROMs (FE6, FE7, FE8). Load a ROM, and get an interactive terminal interface showing character stats, class data, item tables, map layouts, sprites, and text — all decoded from the raw binary. The underlying parsing crate is reusable as a library for other tools (linters, diff tools, randomizers, etc.), and there's a genuine gap in the Rust ecosystem for this — no existing crate covers it.

The scope is deliberately *read-only* to start. Writing back into ROMs opens a large can of worms (repointing, free-space management, checksum handling) that's worth deferring until the parser side is solid.

## Components

### `fe-nmm` — Nightmare module parser

Standalone crate that parses `.nmm` files into a structured schema (offset, entry count, entry size, field definitions with type and optional dropdown enum). This is the foundation — every fixed-size data table in the ROM can be described by an `.nmm`, so a good parser means most tables come for free without hand-writing per-field Rust structs. Small, self-contained, good first target.

### `fe-compression` — GBA BIOS codecs

Pure-Rust implementations of LZ77 (SWI 0x11/0x12) and Huffman (SWI 0x13) decompression. LZ77 is used for graphics and palettes throughout the ROM; Huffman covers the text system. Both formats are well-specified — the byte-level description in the `gba` crate's `bios::LZ77UnCompReadNormalWrite8bit` docs is essentially a ready-made spec. Validate output against Nintenlord's LZ77 tool as an oracle.

### `fe-gfx` — GBA graphics primitives

Decoders for 4bpp tile data, 16-color palettes (GBA 15-bit BGR555 → RGB888), and TSA (tile screen arrangement) entries with their packed H-flip/V-flip/palette-index bits. Renders tiles to an `image::RgbaImage` or equivalent. This is where sprites and map tiles become actually visible.

### `fe-text` — Huffman-compressed text decoder

The GBA FE text system uses a custom Huffman tree stored "upside-down" (leaves first, root last) with 2-byte symbols. There's a global text array of pointers, each pointing to a compressed null-terminated string. Needed for character names, item descriptions, and dialogue to show up as anything other than pointers.

### `fe-rom` — Game detection and the main ROM type

Identifies FE6/FE7U/FE7J/FE8U/FE8J by CRC32 (the approach used by Universal-FE-Randomizer), exposes a `Rom` wrapper with pointer resolution helpers (the `0x08XXXXXX`-prefixed little-endian pointer dance), and loads the appropriate set of Nightmare modules for the detected game. Depends on `fe-nmm` and validates against the FE8U decomp's struct definitions where available.

### `fe-tui` — The `ratatui` frontend

Left pane: tree of tables (Characters, Classes, Items, Chapters, Maps). Right pane: decoded fields for the selected entry, with a hex dump underneath. A dedicated graphics preview pane (either `ratatui-image` with Kitty/iTerm2 protocol support, or Unicode block-art fallback) shows sprites and map tiles inline. Keyboard-driven navigation, no mouse required.

### `fe-map` — Map/chapter renderer

Ties `fe-compression` + `fe-gfx` + the chapter-data tables together to produce a rendered map image. This is the "wow, it actually works" demo and a natural mid-project milestone — when you can pull up Chapter 1 of FE8 and see the map rendered correctly in the terminal, everything under it is probably right.

## Stretch components

- **`fe-patch`** — wrap the `flips` crate to apply and create `.ups`/`.ips` patches. One-day project once the ROM loader is solid.
- **`fe-lint`** — structural validator. Walks chapter data and reports dangling pointers, references to nonexistent unit IDs, unreachable map tiles, null animation pointers, etc. A tool you'd actually use on your own hacks.
- **`fe-diff`** — semantic diff between two ROMs or between a ROM and a patched version. "Eliwood's STR went 5→6, Chapter 3 gained an event at turn 4." Way more useful than `xdelta` for collaborating on hacks.

## Suggested roadmap

1. `.nmm` parser — warmup, small text grammar.
2. LZ77 decoder — first binary format, validate against Nintenlord's tool.
3. Tile + palette renderer — first visual output, render a character portrait to PNG.
4. ROM detection + loading + character/class tables exposed through `.nmm`.
5. `ratatui` skeleton wired up to the table browser.
6. Huffman text decoder — names and descriptions become readable.
7. Map renderer — the big payoff.
8. Stretch features as interest dictates.

## Resources

### Byte-level specs

- `gba` crate docs — `bios::LZ77UnCompReadNormalWrite8bit` and `HuffUnCompReadNormal` contain complete format specs: <https://docs.rs/gba/latest/gba/bios/>
- GBATEK BIOS decompression functions: <https://problemkaputt.de/gbatek-bios-decompression-functions.htm>
- TONC — the standard reference for GBA hardware (tiles, palettes, tilemaps): <https://www.coranac.com/tonc/text/toc.htm>

### FE-specific format docs

- femodding wiki ROM Maps: <https://femodding.fandom.com/wiki/ROM_Maps>
- Nightmare module format explained (FEU): <https://feuniverse.us/t/nightmare-module-format-explained/267>
- EA Table Formatting for the Modern Hacker: <https://feuniverse.us/t/ea-table-formatting-for-the-modern-hacker/19412>
- Blazer's Ultimate Tutorial (FE7-oriented, older, still useful): <http://www.feshrine.net/ultimatetutorial/>

### Machine-readable schemas and ground truth

- Nightmare modules collection: <https://github.com/laqieer/Fire-Emblem-Nightmare-Modules>
- FE8U decompilation — the single most authoritative reference: <https://github.com/FireEmblemUniverse/fireemblem8u>
- FE6 decompilation: <https://github.com/StanHash/fe6-decomp>

### Reference implementations to study

- FEBuilderGBA (C#) — the tool itself, look for `LZ77.cs`, `ImageUtil*.cs`, `TextForm.cs`: <https://github.com/FEBuilderGBA/FEBuilderGBA>
- Universal-FE-Randomizer (Java) — closest functional analog, excellent reference for ROM-identification-by-CRC32 and data-table modeling: <https://github.com/lushen124/Universal-FE-Randomizer>
- Emblem-Magic (C#) — smaller than FEBuilder, gentler read: <https://github.com/LexouDuck/Emblem-Magic>

### Rust crates to depend on (not reimplement)

- `flips` — IPS/BPS/UPS patching: <https://lib.rs/crates/flips>
- `ratatui` — TUI framework: <https://ratatui.rs>
- `ratatui-image` — inline image rendering in terminals that support it: <https://crates.io/crates/ratatui-image>
- `binrw` or `nom` — binary parsing. `binrw` with its derive macros is probably the right fit for fixed-layout structs; `nom` if you want more combinator-style parsing practice.
- `image` — for producing PNGs during development and validation

### Community

- FEUniverse Discord (linked from the FEBuilder README) — active, knowledgeable, the place to ask "is my LZ77 decoder's output correct for this blob"
- Serenes Forest forums: <https://serenesforest.net/forums/>
