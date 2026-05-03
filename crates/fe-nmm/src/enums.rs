// File contains enums relating to the structure of the nmm data as stored in the application.
// Mainly correlates to the txt files

// values for nmm entries, either straight dec/hex entry or dec/hex dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NmmFieldKind {
    Editbox { display: NumDisplay, signed: bool },
    Dropdown { display: NumDisplay },
}

// Enum for value type field type
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumDisplay {
    Hex,
    Decimal,
}

// tracks txt files that have hexes and labels
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EnumTable {
    pub declared_count: u32,
    pub labels: std::collections::BTreeMap<u32, String>,
    pub source_path: std::path::PathBuf,
}

// txt files that are just plain labels, no 0x[yy] values included
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryNames {
    pub labels: Vec<String>,
    pub source_path: std::path::PathBuf,
}
