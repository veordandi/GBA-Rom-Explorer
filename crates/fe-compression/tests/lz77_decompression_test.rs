// tests to verify that lz77 works as expected.
use fe_compression::lz77_decompression::*;
#[cfg(test)]
mod lz77_decompression_test {
    use super::*;

    #[test]
    fn test_file_dump() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../data/gba/Fire Emblem - the Sacred Stones # GBA.GBA");
        decompress_lz77(&path);
    }
}
