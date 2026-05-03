use crate::enums::{EnumRef, EnumTable};

// This is how we're storing the nightmare + txt files throughout the project
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
