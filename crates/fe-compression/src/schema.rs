// for a refresher on how the lz77 compression works, check the uefi spec, also gives
// a brief overview of how it works with huffman compression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct lz77_stream {
    original_characters: Vec<String>,
    string_ptrs: Vec<lz77_pointer>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct lz77_pointer {
    position: usize,
    length: usize,
}

impl lz77_stream {
    pub fn new(original_characters: Vec<String>, string_ptrs: Vec<lz77_pointer>) -> Self {
        Self {
            original_characters,
            string_ptrs,
        }
    }
}

impl lz77_pointer {
    pub fn new(position: usize, length: usize) -> Self {
        Self { position, length }
    }
}
