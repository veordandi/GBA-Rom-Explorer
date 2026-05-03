# `fe-nmm` — Recommended Rust Data Schema

A concrete proposal for the Rust types the `fe-nmm` crate should produce when parsing the `.nmm` and accompanying `.txt` files under `data/nmm/FE8NightmareModules/`. This is scoped to match the project layout in `README.md`: `fe-nmm` produces *schemas*, `fe-rom` applies those schemas to a loaded ROM, and `fe-tui` consumes both.

## 1. What an `.nmm` actually contains (recap)

Each `.nmm` is a line-oriented text file. After any leading `#`-comment lines and a single literal `1` (the module count — every file in the FE8 set uses `1`), the file is a fixed seven-line **table header** followed by repeated five-line **field blocks** separated by blank lines:

```
1                                ← module count (always 1 in our corpus)
FE8 Character Editor by SpyroDi  ← title
0x803D30                         ← absolute ROM offset of the table
256                              ← number of entries (decimal OR 0x-prefixed hex)
52                               ← bytes per entry
FE8 Character Editor.txt         ← per-entry name list, or NULL
NULL                             ← reserved (always NULL in this corpus)

Name value                       ← field label
0                                ← byte offset within an entry
2                                ← field width in bytes
NEHU                             ← display/type tag
NULL                             ← dropdown source filename, or NULL
```

Type tags observed across all 280 modules: **`NEHU`, `NEDU`, `NEDS`, `NDHU`, `NDDU`** (no `HEXA` actually appears in this dataset, despite the stub in `crates/fe-nmm/src/lib.rs` listing it). They decompose cleanly into two axes:

| Tag    | Widget   | Display | Signedness |
|--------|----------|---------|------------|
| `NEHU` | Editbox  | Hex     | Unsigned   |
| `NEDU` | Editbox  | Decimal | Unsigned   |
| `NEDS` | Editbox  | Decimal | Signed     |
| `NDHU` | Dropdown | Hex     | Unsigned   |
| `NDDU` | Dropdown | Decimal | Unsigned   |

Two important subtleties the schema must handle:

- **Field offsets are not monotonic and do not need to cover every byte of the entry.** In `FE8 Character Editor.nmm`, `Base Luck` lives at offset 18 between fields at offset 15 and 16 — the file's *author order* is not the *byte layout order*. Both views matter (UI lists fields in author order; decoding reads bytes by offset), so preserve author order in the parsed schema and let consumers sort by offset on demand.
- **Many modules contain `***UNKNOWN***`-named fields.** The parser must accept them as ordinary fields, not reject them.

There are two distinct kinds of `.txt` sidecar:

- **Indexed dropdown** (e.g. `Class List.txt`, `Item List.txt`): first line is a count, followed by `0xHH Label` lines. Indices may have gaps (some indices have no row).
- **Entry-name list** (e.g. `FE8 Character Editor.txt`): one label per line with no index prefix; position in the file is the entry index.

A note on `Item List.txt` — some lines contain a `/` mid-line (`0x00 All Swords/   Separator`). Treat the `/`-suffix as commentary or a secondary view label and ignore it for the primary index→label mapping; flag it for later if a consumer cares.

## 2. Recommended types in `fe-nmm`

The headline pitch: split the crate into three small modules — `schema` (the parsed `.nmm`), `enums` (the parsed `.txt` sidecars), and `parse` (the actual reader) — and keep all of them owned-string and `serde`-friendly so the TUI and any future linter can serialize/cache them trivially.

### 2.1 The schema types

```rust
// crates/fe-nmm/src/schema.rs

/// One parsed Nightmare module — the description of a single fixed-size
/// table somewhere in the ROM. Owns its strings; cheap to clone the metadata
/// since field counts are small (typical: 10–60 fields).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NmmTable {
    /// Display title from the module header.
    pub title: String,
    /// Absolute ROM offset where the table begins.
    pub offset: u32,
    /// Number of entries in the table.
    pub entry_count: u32,
    /// Size of one entry in bytes. Field offsets must be < this.
    pub entry_size: u32,
    /// Sidecar file giving a human label per entry, if any.
    /// Filename is verbatim from the .nmm, resolved relative to the .nmm dir.
    pub entry_names_ref: Option<EnumRef>,
    /// Field definitions in *author order* (not byte order).
    pub fields: Vec<NmmField>,
    /// Path the schema was parsed from, for diagnostics and for resolving
    /// EnumRef paths. Optional so schemas can be constructed in tests.
    pub source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NmmField {
    pub label: String,
    /// Byte offset within an entry.
    pub offset: u32,
    /// Width in bytes. Always 1, 2, or 4 in this corpus.
    pub width: u8,
    pub kind: NmmFieldKind,
    /// For Dropdown fields, the sidecar enum file.
    pub dropdown_ref: Option<EnumRef>,
}

/// Two-axis decomposition of the five flat tags. Avoids a flat 5-variant
/// enum that mixes "is this a dropdown" with "how do I render the number".
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NmmFieldKind {
    Editbox { display: NumDisplay, signed: bool },
    Dropdown { display: NumDisplay },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumDisplay { Hex, Decimal }

/// Reference to a sidecar .txt file. Stores the verbatim filename plus an
/// optional resolved absolute path. The parser fills `name`; resolution
/// happens lazily so loading is opt-in and testable.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumRef {
    pub name: String,
    #[serde(skip)]
    pub resolved: Option<std::path::PathBuf>,
}
```

Why `NmmFieldKind` is shaped like this rather than a flat 5-variant enum: every place the TUI renders a value it answers two independent questions — "does this open a picker?" and "decimal or hex?" — and a flat enum forces a `match` over all five tags every time. The two-axis form lets the editor logic match on `Dropdown { .. }` once and the formatter match on `display` once. Signed only applies to editboxes (no `NDDS`/`NDHS` exist in the corpus), so it stays inside the `Editbox` variant.

### 2.2 The enum-table types

```rust
// crates/fe-nmm/src/enums.rs

/// An indexed dropdown table from a sidecar .txt (Class List.txt, etc.).
/// Sparse: not every index in 0..count necessarily has a label.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumTable {
    pub declared_count: u32,
    pub labels: std::collections::BTreeMap<u32, String>,
    pub source_path: std::path::PathBuf,
}

/// An entry-name list from a sidecar .txt (FE8 Character Editor.txt, etc.).
/// One label per row index, dense.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryNames {
    pub labels: Vec<String>,
    pub source_path: std::path::PathBuf,
}
```

Two distinct types is deliberate — they look similar but their access patterns differ (sparse keyed lookup vs. dense positional lookup) and conflating them makes consumers branch on a flag at every call site.

### 2.3 The parser entry points

```rust
// crates/fe-nmm/src/parse.rs

pub fn parse_table(input: &str) -> Result<NmmTable, NmmParseError>;
pub fn parse_table_path(path: &std::path::Path) -> Result<NmmTable, NmmParseError>;

pub fn parse_enum_table(input: &str) -> Result<EnumTable, NmmParseError>;
pub fn parse_entry_names(input: &str) -> Result<EntryNames, NmmParseError>;

#[derive(Debug, thiserror::Error)]
pub enum NmmParseError {
    #[error("unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },
    #[error("expected {expected} on line {line}, got {found:?}")]
    BadLine { line: usize, expected: &'static str, found: String },
    #[error("invalid integer on line {line}: {source}")]
    BadInt { line: usize, #[source] source: std::num::ParseIntError },
    #[error("unknown field kind on line {line}: {tag}")]
    UnknownKind { line: usize, tag: String },
    #[error("io error reading {path}: {source}")]
    Io { path: std::path::PathBuf, #[source] source: std::io::Error },
}
```

Two non-obvious choices worth flagging: (1) numeric lines must accept both decimal and `0x`-hex (the "0x31" in `Random Item Group Editor.nmm` is a real entry-count value), so use a single `parse_uint` helper that does `s.strip_prefix("0x").map(|h| u32::from_str_radix(h, 16))` first; (2) attach line numbers to every error since these files are hand-edited and a useful error message is what makes the parser pleasant to work with.

### 2.4 Bundling: a loaded module set

`fe-rom` will need to load *all* of FE8's modules at once. Provide a thin loader that walks a directory and returns a registry:

```rust
// crates/fe-nmm/src/registry.rs

/// All schemas + sidecars discovered under a directory tree.
/// Sidecars are loaded lazily and memoized.
pub struct NmmRegistry {
    tables: Vec<NmmTable>,
    enums: std::sync::OnceLock<HashMap<PathBuf, EnumTable>>,
    entry_names: std::sync::OnceLock<HashMap<PathBuf, EntryNames>>,
}

impl NmmRegistry {
    pub fn load_dir(root: &Path) -> Result<Self, NmmParseError>;
    pub fn tables(&self) -> &[NmmTable];
    pub fn lookup_enum(&self, r: &EnumRef) -> Result<&EnumTable, NmmParseError>;
    pub fn lookup_names(&self, r: &EnumRef) -> Result<&EntryNames, NmmParseError>;
}
```

Lazy loading matters because the same `.txt` (e.g. `Class List.txt`) is referenced by dozens of modules; eager loading per `EnumRef::resolved` would re-parse it dozens of times. Memoize on resolved path.

## 3. How this layers with the rest of the workspace

This is what unlocks the README's claim that "every fixed-size data table in the ROM can be described by an `.nmm`, so a good parser means most tables come for free":

- **`fe-nmm`** — pure text parser. Outputs `NmmTable` (schema), `EnumTable` (dropdown), `EntryNames`. No ROM bytes touched.
- **`fe-rom`** — owns `NmmRegistry` plus the loaded ROM bytes. Defines the *runtime* counterparts:

  ```rust
  // crates/fe-rom/src/decode.rs (sketch — not part of fe-nmm)
  pub enum NmmValue<'a> {
      Int { raw: u64, signed: bool, display: NumDisplay, width: u8 },
      Enum { raw: u64, label: Option<&'a str>, display: NumDisplay },
  }

  pub fn decode_field<'a>(rom: &Rom, table: &NmmTable, field: &NmmField,
                          entry_index: u32, registry: &'a NmmRegistry)
      -> Result<NmmValue<'a>, RomError>;
  ```

  This is the boundary where little-endian widths, ROM offsets, and dropdown lookup all live. Keep it out of `fe-nmm` so the parser stays trivially testable without ROM fixtures.
- **`fe-tui`** — for the selected table, renders fields in `NmmTable.fields` order; for each field calls `decode_field` and uses `NmmFieldKind` to pick a renderer (hex vs decimal, label vs raw number). The hex dump under the field view doesn't need the schema at all.
- **`fe-lint`** (stretch, per README) — walks `NmmRegistry`, decodes every entry, and reports e.g. dropdown values whose raw byte has no matching label. This works because schemas are pure data.

The other crates (`fe-compression`, `fe-gfx`, `fe-text`, `fe-map`) don't depend on `fe-nmm` at all — they handle compressed blobs and graphics that aren't fixed-size tables. The README's roadmap puts `.nmm` first for exactly this reason: it's a self-contained text-grammar warmup with no binary parsing.

## 4. Suggested file layout for the crate

```
crates/fe-nmm/
├── Cargo.toml         # add: thiserror, serde (optional), once_cell-style
├── src/
│   ├── lib.rs         # re-exports
│   ├── schema.rs      # NmmTable, NmmField, NmmFieldKind, NumDisplay, EnumRef
│   ├── enums.rs       # EnumTable, EntryNames
│   ├── parse.rs       # parse_table, parse_enum_table, parse_entry_names, errors
│   └── registry.rs    # NmmRegistry, directory walker
└── tests/
    ├── fixtures/      # 3–4 representative .nmm files copied from data/
    └── parse.rs       # round-trip and golden-output tests
```

Drop the existing stub's `HEXA` variant — it's unused. Replace `nmmContents` (which currently has `Vec<NmmBody>` for what should be a flat list of fields, not nested bodies) with the single `NmmTable` above. Rename `byte_modified` → `offset` and `label_byte_length` → `width` to match the format docs. Use `u32` for offsets and counts since ROM offsets exceed `u16` and Rust's `usize` carries platform-dependent baggage you don't want in a serializable schema.

## 5. Concrete example — what parsing `FE8 Character Editor.nmm` produces

```rust
NmmTable {
    title: "FE8 Character Editor by SpyroDi".into(),
    offset: 0x0080_3D30,
    entry_count: 256,
    entry_size: 52,
    entry_names_ref: Some(EnumRef { name: "FE8 Character Editor.txt".into(), resolved: None }),
    fields: vec![
        NmmField { label: "Name value".into(),  offset: 0,  width: 2,
                   kind: NmmFieldKind::Editbox { display: Hex, signed: false },
                   dropdown_ref: None },
        // ... 51 more fields, including `Base Luck` at offset 18 between
        // `Base Speed` (15) and `Base Defense` (16)
        NmmField { label: "Class (support viewer only)".into(), offset: 5, width: 1,
                   kind: NmmFieldKind::Dropdown { display: Hex },
                   dropdown_ref: Some(EnumRef { name: "Class List.txt".into(), resolved: None }) },
        // ...
    ],
    source_path: Some(/* …/Class & Character editors/FE8 Character Editor.nmm */),
}
```

A `decode_field` call against this schema for entry index 0 (Eirika), field "Class (support viewer only)" reads one byte at ROM offset `0x0080_3D30 + 0*52 + 5`, gets `0x02`, and looks it up in the registry's resolved `Class List.txt` → `"Eirika Lord"`.

## 6. Things to defer

- **Multi-module `.nmm` files.** The leading `1` is a module count. The format theoretically allows >1, but no FE8 file in this corpus uses it. Parse the count, assert it's 1, surface a `MultiModuleUnsupported` error otherwise — don't build for it speculatively.
- **`HEXA` and other exotic tags.** Not present in the FE8 corpus. Add when a future FE6/FE7 module set requires it; the `NmmFieldKind` enum extends cleanly.
- **Write-back.** Per the README, the project is read-only to start. The schema types should be `Clone` and `Serialize` so a future writer can take owned values without touching the parser.
- **`/`-separated multi-column labels in some `Item List.txt` rows.** Parse the first label, store the rest as an opaque `aux: Option<String>` if needed later, but don't model them as first-class until something consumes them.

## 7. References

The format details above were cross-checked against:

- [Nightmare module format explained — Fire Emblem Universe](https://feuniverse.us/t/nightmare-module-format-explained/267)
- [Appendix: Nightmare Module Format — ultimate-tutorial-2](https://tutorial.feuniverse.us/nightmare/appendix)
- The 280 `.nmm` and 116 `.txt` files under `data/nmm/FE8NightmareModules/`
- [README.md](../README.md) — for the crate roles and the read-only-first scope

Sources for this recommendation are the FEU tutorial threads above plus direct inspection of the data directory.
