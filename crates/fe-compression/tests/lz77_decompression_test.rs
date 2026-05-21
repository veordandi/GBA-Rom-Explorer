// tests to verify that lz77 works as expected.
use fe_compression::lz77_decompression::*;
#[cfg(test)]
mod lz77_decompression_test {
    use super::*;

    // This is an odd test. The nmm module location will always be the same, and is checked directly
    // in the translate_rom file. So we're going to have to bypass part of the function to test the lz77 directly.
    #[test]
    fn lz77_file_dump() {
        let rom_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/data/gba/Fire Emblem - the Sacred Stones # GBA.GBA");

        let rom: Vec<u8> = std::fs::read(rom_path).unwrap();

        // Read the .nmm files in the NMM directory
        let nmm_directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/crates/fe-compression/tests/fixtures/");

        let entries = fs::read_dir(&nmm_directory)?;
        for file in entries {
            let entry = file.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(NMM_EXTENSION) {
                //If this file is a .nmm file, then we want to go through and do the thing
                // Locate the byte range of a single field within a single entry.
                // `table` and `field` come from the fe-nmm parser; entry_idx is which row.

                let table = fe_nmm::parse::parse_all_nmm_files(path);
                let row_start =
                    (table.offset as usize) + (entry_idx as usize) * (table.entry_size as usize);
                let field_start = row_start + field.offset as usize;
                let field_bytes = &rom[field_start..field_start + field.width as usize];

                // For a 4-byte "… Pointer" field, interpret it as a little-endian u32
                // GBA pointer and convert to a file offset by stripping 0x08000000.
                let gba_ptr = u32::from_le_bytes(field_bytes.try_into()?);
                let blob_offset = (gba_ptr & 0x01FF_FFFF) as usize; // == gba_ptr - 0x08000000 for valid ROM pointers

                // Take a slice from there to the end of the ROM. The decoder will
                // figure out where it ends from the header's declared decompressed size.
                let blob = &rom[blob_offset..];

                // Sanity-check the marker byte. If this isn't 0x10, the pointer doesn't
                // point at LZ77 data — could be raw bytes, Huffman, a sub-table, or junk.
                assert_eq!(blob[0], 0x10, "expected LZ77 marker, got {:#x}", blob[0]);

                let mut payload;

                match blob[0] {
                    0x10 => payload = decompress_lz77(blob).unwrap(),
                    // 0x20 | 0x28 => payload = decompress_huffman(blob).unwrap(),
                    (_) => (),
                }
            }
    }
}
