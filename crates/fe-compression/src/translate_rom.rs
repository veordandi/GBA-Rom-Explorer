use std::fs;
use std::path::Path;

use crate::{huffman_decompression::decompress_huffman, lz77_decompression::decompress_lz77};

// important things to be aware of with the overall ROM format.
// Memory ranges from 0x08000000 to 0x08080000 (32MB ROM)
// offset 1 is 0x08000001, so on and so forth. This is how we know where
// unused memory is, since we know the beginning and ending bounds of memory
pub fn translate_rom(path: &Path) -> Result<String, std::io::Error> {
    let rom: Vec<u8> = std::fs::read(path).unwrap();

    let entries = fs::read_dir(&nmm_directory)?;
    for file in entries {
        let entry = file.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(nmm_extension) {
            //If this file is a .nmm file, then we want to go through and do the thing
            // Locate the byte range of a single field within a single entry.
            // `table` and `field` come from the fe-nmm parser; entry_idx is which row.
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

            // Decompress. fe-compression::lz77_decompress is what you'll write.
            let mut payload;

            match blob[0] {
                0x10 => payload = decompress_lz77(blob).unwrap(),
                0x20 | 0x28 => payload = decompress_huffman(blob).unwrap(),
                (_) => None,
            }
        }
    }

    //TODO: Come back to this, I don't know what we'll return from this yet.
    // Theoretically this would be called by fe-tui so that the frontend can display the tables returned here to the frontentd.
    return Ok("".to_string());
}
