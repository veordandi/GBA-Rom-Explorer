use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

// Error enums to keep track of what went wrong when attempting the lz77 parsing
#[derive(Debug, thiserror::Error)]
pub enum lz77ParseError {
    #[error("unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },
    //TODO: Once we get a good feel for what the data looks like, add more
    // options for error types that we might run into while decompressing
}

// this is the fun part. How do we want to keep track of the decompressed data?
// Once we have all of it, it would need to be kept in buckets based on topic like
// the nmm files are, since it has to be transferred over to that format in order
// to be parsed anyway.
// Really need to find a way to see what the output of the binary file being read in *is*. That
// will help figure out what we have to work with, and what's coming in.
// Have this open the gba file in data/gba and have a test dump the contents? That's gonna be a _lot_ of
// data.
pub fn decompress_lz77(blob: &[u8]) -> Result<Vec<u8>, lz77ParseError> {
    Ok(Vec::new())
}
