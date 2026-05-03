// NMM FILE REPRESENTATIONS
use crate::enums::*;
// Essentially one parsed nightmare module, header, body, and reference to txt file if applicable
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NmmTable {
    pub title: String, // Header title
    pub offset: u32,   // Offset from the start of the ROM
    pub entry_count: u32,
    pub entry_size: u32,       // Size in bytes
    pub fields: Vec<NmmField>, // Field definitions from the body in *author order* (not byte order).

    // Associated txt file giving a human label per entry, if any
    pub entry_names_ref: Option<EnumRef>,

    /// Path the schema was parsed from, for diagnostics and for resolving
    /// EnumRef paths. Optional so schemas can be constructed in tests.
    pub source_path: Option<std::path::PathBuf>,
}

// describes one body module from an nmm file
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NmmField {
    pub label: String,
    pub offset: u32,                   // Byte offset inside the entry
    pub width: u8,                     // width in bytes, always either 1/2/4
    pub kind: NmmFieldKind,            // Whether dropdown or free entry
    pub dropdown_ref: Option<EnumRef>, // hex or dec
}

// TXT FILE REFERENCES
// reference to the txts, label and the absolute path to file
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumRef {
    pub name: String,
    #[serde(skip)]
    pub resolved: Option<std::path::PathBuf>,
}
