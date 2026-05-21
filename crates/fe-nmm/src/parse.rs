use crate::enums::*;
use crate::schema::*;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io::Error;
use std::path::Path;
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

pub fn parse_table_path(path: &Path) -> Result<NmmTable, NmmParseError> {
    let contents = read_to_string(path).map_err(|source| NmmParseError::Io {
        path: path.to_owned(),
        source,
    })?;
    let mut table = parse_table(&contents)?;
    table.source_path = Some(path.to_owned());
    Ok(table)
}

/* The header we're parsing will look like this:
    1       -> indicator that header has begun
    FE8 Spell Association Editor by Vennobennu  -> label
    0x8AFBD8 -> offset
    161     -> entry count
    16      -> entry byte size
    NULL    -> Type of values (dropdown/input, hex/dec)
    NULL    -> txt file name
*/

// Parsing a generic body entry for the .nmm file
/* Formatted like so:
   Weapon      -> Label
   0           -> Offset
   2           -> width
   NDHU        -> Type of values (dropdown/input, hex/dec)
   Item List.txt   -> txt file name (Can be NULL)
*/
pub fn parse_table(input: &str) -> Result<NmmTable, NmmParseError> {
    let mut cursor = LineCursor::new(input);

    // This is really just here to grab the leading 1 in the file that signifies the header starting. We don't
    // want to put the handling of this in the loop logic in the wrapper, because then it will check for an extra
    // if statement _every_ time it looks at contents as opposed to just running the one time
    let module_count = cursor.expect_uint()?;
    if module_count != 1 {
        return Err(NmmParseError::BadLine {
            line: cursor.last_consumed_line,
            expected: "module count of 1",
            found: module_count.to_string(),
        });
    }

    // The cursor handles iterating over these, which means we don't need to put it in a loop here,
    // it handles all of that by +=1 the position
    let mut table = NmmTable::new();
    table.title = cursor.expect()?.to_string();
    table.offset = parse_table_location(cursor.expect()?)?;
    table.entry_count = cursor.expect_uint()?;
    table.entry_size = cursor.expect_uint()?;
    table.entry_names_ref = parse_enum_ref(cursor.expect()?);
    let _reserved = cursor.expect()?; // This one is a bit weird. It will _always_ be null, but like the section above,
    // it saves us a lot more time to handle it here than inside the cursor loop

    // Can do this as opposed to doing while cursor.position < cursor.lines.len()
    // Looks cleaner, and means we're checking the value we'll be using before going into the loop
    // as opposed to just assuming the numbers will work
    while cursor.peek_meaningful().is_some() {
        let mut field = NmmField::new();
        field.label = cursor.expect()?.to_string();
        field.offset = cursor.expect_uint()?;
        field.width = cursor.expect_uint()? as u8; // Nifty trick here, you can cast u32 down to a u8. (would truncate if value was high enough)
        // in other contexts, could do let x = y.try_into().expect("Value out of range!"); to cast w/error handling
        field.kind = cursor.expect_data_type()?;
        field.dropdown_ref = parse_enum_ref(cursor.expect()?);
        table.fields.push(field);
    }

    Ok(table)
}

fn parse_table_location(s: &str) -> Result<TableLocation, NmmParseError> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Ok(TableLocation::Static(
            u32::from_str_radix(hex, 16).map_err(|source| NmmParseError::UnknownKind {
                line: 1,
                tag: s.to_string(),
            })?,
        ))
    } else {
        Ok(TableLocation::Static(s.parse::<u32>().map_err(
            |source| NmmParseError::UnknownKind {
                line: 1,
                tag: s.to_string(),
            },
        )?))
    }
}

// We're using this to parse the enum as oppposed to parsing it in the LineCursor struct due to how the expect() structure works.
// Essentially we need to return an Option<EnumRef> out of this for proper assignment to an EnumRef type field. The Cursor methods
// only return a Result<T,E> object, so we'll have the code above grab that, and pass it here if no error surfaces.
fn parse_enum_ref(s: &str) -> Option<EnumRef> {
    if s.trim() == "NULL" {
        None
    } else {
        Some(EnumRef {
            name: s.trim().to_string(),
            resolved: None,
        })
    }
}

// Oddly enough, this seems to be a fairly common design pattern. Instead of using a numerical iterator,
// We're gonna use a wrapper on an iterator.
struct LineCursor<'a> {
    lines: Vec<&'a str>,
    position: usize,           // index of next line to read
    last_consumed_line: usize, // most recently used line
}

impl<'a> LineCursor<'a> {
    // Implementing a new instance, pass in the file input
    fn new(input: &'a str) -> Self {
        Self {
            // Fun fact! .lines() is a String method, not specific to reading in files.
            lines: input.lines().collect(),
            position: 0,
            last_consumed_line: 0,
        }
    }

    // Consume blanks and `#`-comments and return the next meaningful line, trimmed.
    fn next_meaningful(&mut self) -> Option<&'a str> {
        // This one is pretty simple, we just skip over a line if it's a comment or blank
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim();
            self.position += 1;
            if line.starts_with('#') || line == "" {
                continue; // We want to skip these lines, and they won't count as last consumed, because there was no meaningful data there
            } else {
                self.last_consumed_line += 1;
                return Some(self.lines[self.position - 1].trim());
            }
        }
        None
    }

    // Peek without consuming. Need to use this to determine whether the file has ended,
    // to avoid getting an EOF
    fn peek_meaningful(&self) -> Option<&'a str> {
        self.lines[self.position..]
            .iter()
            .copied() // this is a neat trick, because you're using .copied you're not consuming the iterator
            .map(str::trim)
            .find(|t| !t.is_empty() && !t.starts_with('#'))
    }

    // Checks to verify that the next line is a real value, returns an error if not.
    fn expect(&mut self) -> Result<&'a str, NmmParseError> {
        let line = self.position + 1;
        match self.next_meaningful() {
            Some(s) => Ok(s),
            None => Err(NmmParseError::UnexpectedEof { line }),
        }
    }

    // Same as above, but we're specifically checking that the value we're parsing
    // is a u32! (neat trick, if you want a u8, do expect_uint() as u8)
    fn expect_uint(&mut self) -> Result<u32, NmmParseError> {
        let s = self.expect()?;
        // ? means that we propagate error if it returns, from here on we assume we have a real value;

        // If let syntax still hurts my brain. Read right side conditional, then left side assignment to
        // the value in the {}.
        let parsed = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u32::from_str_radix(hex, 16)
        } else {
            s.parse::<u32>()
        };

        // need to return either the parse number or a parse error.
        // Because the parsed value comes back as a Result<u32, ParseIntError>, we can run
        // map_err, because that's a method on the Result object
        parsed.map_err(|source| NmmParseError::BadInt {
            line: self.last_consumed_line,
            source,
        })
    }

    // Checks the data type for the field. As commented elsewhere, it's either going to be a (hexa)decimal value w/a txt file or without
    // we COULD add this in to the schema, but on the other hand, we'll probably only ever need to do this once while parsing to/from nmm format
    fn expect_data_type(&mut self) -> Result<NmmFieldDataType, NmmParseError> {
        let s = self.expect()?;
        match s {
            "NDDU" | "NEDU" | "NEDS" => Ok(NmmFieldDataType::Decimal),
            "NDHU" | "NEHU" => Ok(NmmFieldDataType::Hex),
            _ => Err(NmmParseError::UnknownKind {
                line: self.position,
                tag: s.to_string(),
            }),
        }
    }
}
