/*
 * We'll be implementing this at a future date, it's much easier
 * to implement the lz77 decompression first, based on the order of therom
 * compression algorithm.
 */

use thiserror::Error;

// Error enums to keep track of what went wrong when attempting the lz77 parsing
#[derive(Debug, thiserror::Error)]
pub enum HuffmanParseError {
    #[error("unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },
    //TODO: Once we get a good feel for what the data looks like, add more
    // options for error types that we might run into while decompressing
}

pub fn decompress_huffman(blob: &[u8]) -> Result<Vec<u8>, HuffmanParseError> {
    //TODO: Implement this later!
    // println!("Huffman blob!: {:?}", blob);
    Ok(Vec::new())
}
