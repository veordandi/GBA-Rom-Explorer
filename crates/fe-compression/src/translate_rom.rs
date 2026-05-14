use std::path::Path;

// important things to be aware of with the overall ROM format.
// Memory ranges from 0x08000000 to 0x08080000 (32MB ROM)
// offset 1 is 0x08000001, so on and so forth. This is how we know where
// unused memory is, since we know the beginning and ending bounds of memory
pub fn translate_rom(path: &Path) -> String {
    // This has to take forever, right? Surely there's a quicker way.
    let rom: Vec<u8> = std::fs::read(path).unwrap();

    // Locate the byte range of a single field within a single entry.
    // `table` and `field` come from the fe-nmm parser; entry_idx is which row.
    let row_start = (table.offset as usize) + (entry_idx as usize) * (table.entry_size as usize);
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
    let payload: Vec<u8> = lz77_decompress(blob)?;
}
