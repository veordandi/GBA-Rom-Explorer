// File contains enums relating to the structure of the nmm data as stored in the application.
// Mainly correlates to the txt files

// values for nmm entries, either straight dec/hex entry or dec/hex dropdown
// Note that there are 5 values in the nmm files for this. NDDU/NDHU/NEDU/NEDS/NEHU
// All of these come down to either a Hex or Decimal value, and will either be free input (no txt)
// or a drowpdown that has available options stored in the correlating txt file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NmmFieldDataType {
    Hex,
    Decimal,
}

// Need a better way to store the offset than a u8. We might end up with an odd offset when we bring
// in modified roms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TableLocation<'a> {
    Static(u32),
    IndirectFrom(u32),
    DiscoverByPattern(&'a [u8]),
}

// tracks txt files that have hexes as well as labels.
// hex values stored as the key in the BTreeMap, with the label as the value.
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
