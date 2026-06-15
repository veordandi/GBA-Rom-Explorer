// for a refresher on how the lz77 compression works, check the uefi spec, also gives
// a brief overview of how it works with huffman compression
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lz77Stream {
    original_characters: Vec<String>,
    string_ptrs: Vec<Lz77Pointer>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lz77Pointer {
    position: usize,
    length: usize,
}

impl Lz77Stream {
    pub fn new(original_characters: Vec<String>, string_ptrs: Vec<Lz77Pointer>) -> Self {
        Self {
            original_characters,
            string_ptrs,
        }
    }
}

impl Lz77Pointer {
    pub fn new(position: usize, length: usize) -> Self {
        Self { position, length }
    }
}
