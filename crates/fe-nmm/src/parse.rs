use crate::enums::*;
use crate::schema::*;
use std::fs::read_to_string;
use std::io::Error;
use thiserror;

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

// Allows us to translate err:Error into NmmParseError
impl From<Error> for NmmParseError {
    fn from(err: Error) -> Self {
        NmmParseError::Io {
            path: std::path::PathBuf::from(""),
            source: err.into(),
        }
    }
}

pub fn parse_table_path(path: &str) -> Result<NmmTable, NmmParseError> {
    // read lines in one by one
    let mut table = NmmTable::new();

    let mut line_number = 0;
    let mut read_header = false;
    let mut read_body = false;

    'line_read: for line in read_to_string(path).unwrap().lines() {
        if read_header == true {
            // The Headers will _always_ have the same structure. There's almost certainly a clearner way to do this,
            // but for no we're going to get a working implementation, and clean it up w/better error handlng later
            //
            /* The header we're parsing will look like this:
                1       -> indicator that header has begun
                FE8 Spell Association Editor by Vennobennu  -> label
                0x8AFBD8 -> offset
                161     -> entry count
                16      -> entry byte size
                NULL    -> Type of values (dropdown/input, hex/dec)
                NULL    -> txt file name
            */
            match line_number {
                0 => table.title = line.to_string(),
                1 => table.offset = line.parse::<u32>().unwrap(),
                2 => table.entry_count = line.parse::<u32>().unwrap(),
                3 => table.entry_size = line.parse::<u32>().unwrap(),
                4 => {
                    if line.to_string() != "NULL" {
                        //TODO: At this point we need to parse the txt files
                        // parse_txt_file(line)
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

        // Parsing a generic body entry for the .nmm file
        /* Formatted like so:
           Weapon      -> Label
           0           -> Offset
           2           -> width
           NDHU        -> Type of values (dropdown/input, hex/dec)
           Item List.txt   -> txt file name (Can be NULL)
        */
        if read_body {
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

    Ok(table)
}

pub fn parse_txt_file(input: &str) -> Result<EnumTable, NmmParseError> {
    // for the given str ptr, read file w/name.
    // Note that based on what we have in enums.rs, the file will
    // either be a plain list, or look like "0x00 Off"
}

// pub fn parse_enum_table(input: &str) -> Result<EnumTable, NmmParseError>;
// pub fn parse_entry_names(input: &str) -> Result<EntryNames, NmmParseError>;
