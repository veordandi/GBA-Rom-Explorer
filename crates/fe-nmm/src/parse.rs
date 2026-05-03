use crate::schema::*;

// Error enums that we'll be using in our parsing functions when things go wrong
#[derive(Debug, thiserror::Error)]
pub enum NmmParseError {
    #[error("unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },
    #[error("expected {expected} on line {line}, got {found:?}")]
    BadLine {
        line: usize,
        expected: &'static str,
        found: String,
    },
    #[error("invalid integer on line {line}: {source}")]
    BadInt {
        line: usize,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("unknown field kind on line {line}: {tag}")]
    UnknownKind { line: usize, tag: String },
    #[error("io error reading {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// Parses regular nmm module given a string input
pub fn parse_table(input: &str) -> Result<NmmTable, NmmParseError>;

pub fn parse_table_path(path: &std::path::Path) -> Result<NmmTable, NmmParseError> {
    // read lines in one by one
    let table = NmmTable::new();

    //TODO: Somehow we need an Option<std::path::PathBuf>
    // table.source_path = filename.to_string();

    let mut line_number = 0;
    let mut read_header = false;
    let mut read_body = false;

    'line_read: for line in read_to_string(filename).unwrap().lines() {
        if read_header == true {
            match line_number {
                0 => table.title = line.to_string(),
                1 => table.offset = line.to_u32().unwrap(),
                2 => table.entry_count = line.to_u32().unwrap(),
                3 => table.entry_size = line.to_u32().unwrap(),
                4 => {
                    if line.to_string() != "NULL" {
                        //TODO: HAHAHA FUCK YOU HAVE FUN PARSING THE TXT
                        // parse_txt_file(&line.to_string())
                    }
                }
                _ => {
                    line_number = 0;
                    read_header = false;
                    read_body = true;
                    continue 'line_read;
                }
            }
        }

        if (read_body) {
            match line_number {
                0 => {}
                _ => {
                    line_number = 0;
                    read_body = false;
                }
            }
        }

        if line.starts_with("1") && !read_header {
            read_header = true;
        }
    }

    table
}

pub fn parse_enum_table(input: &str) -> Result<EnumTable, NmmParseError>;
pub fn parse_entry_names(input: &str) -> Result<EntryNames, NmmParseError>;
