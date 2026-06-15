use thiserror::Error;

// Error enums to keep track of what went wrong when attempting the lz77 parsing
#[derive(Debug, thiserror::Error)]
pub enum Lz77parseError {
    #[error("unexpected end of file at line {line}")]
    UnexpectedEof { line: usize },
    //TODO: Once we get a good feel for what the data looks like, add more
    // options for error types that we might run into while decompressing
}

// Issue we're goign to run into here is that lz77 is being used to store image data.
// As such it's not stored in the nmm files, those
pub fn decompress_lz77(blob: &[u8]) -> Result<Vec<u8>, Lz77parseError> {
    // This comes in as a blob, let's spit it out in a test and see what we've got.

    println!("lz77 blob: {:?}", blob);
    Ok(Vec::new())
}
