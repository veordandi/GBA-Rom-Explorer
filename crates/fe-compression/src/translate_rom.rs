use std::fs;
use std::path::Path;

use fe_nmm::enums::TableLocation;

use crate::{huffman_decompression::decompress_huffman, lz77_decompression::decompress_lz77};

const NMM_EXTENSION: &str = "nmm";

// important things to be aware of with the overall ROM format.
// Memory ranges from 0x08000000 to 0x08080000 (32MB ROM)
// offset 1 is 0x08000001, so on and so forth. This is how we know where
// unused memory is, since we know the beginning and ending bounds of memory
pub fn translate_rom(path: &Path) -> Result<String, std::io::Error> {
    // Read the bytes of the ROM
    let rom: Vec<u8> = std::fs::read(path).unwrap();

    // Read the .nmm files in the NMM directory
    let nmm_directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("/data/nmm/FE8NightmareModules");

    let entries = fs::read_dir(&nmm_directory)?;
    for file in entries {
        let entry = file.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(NMM_EXTENSION) {
            //If this file is a .nmm file, then we want to go through and do the thing
            // Locate the byte range of a single field within a single entry.
            // `table` and `field` come from the fe-nmm parser; entry_idx is which row.

            let table = fe_nmm::parse::parse_table_path(&path).unwrap();
            let row_start;

            match table.offset {
                TableLocation::Static(_) => {
                    let mut table_start;
                    if let TableLocation::Static(addr) = table.offset {
                        table_start = addr as usize;
                    }
                    let len = table.entry_size as usize * table.entry_count as usize;
                    let table_bytes: &[u8] = &rom[table_start..table_start + len];
                }
                TableLocation::IndirectFrom(_) => {
                    // There will be a 4 byte pointer at the given offset. Read that to find the location of the table
                    let mut ptr_loc;
                    if let TableLocation::IndirectFrom(ptr_addr) = table.offset {
                        ptr_loc = ptr_addr as usize;
                    }

                    let ptr_bytes: [u8; 4] = rom[ptr_loc..ptr_loc + 4].try_into().expect("4 bytes");

                    // Hop 2: those 4 bytes are a GBA pointer; convert to file offset
                    let start = gba_ptr_to_file_offset(ptr_bytes).expect("valid pointer");
                    let len = table.entry_size as usize * table.entry_count as usize;
                    let table_bytes: &[u8] = &rom[start..start + len];
                }
                TableLocation::DiscoverByPattern(_) => {
                    //TODO: Offset will be a Vec<u8>, we're gonna crawl the rom for a section matching that vec
                }
                _ => {}
            }

            // for each field in the table, go through the rom data and parse it
            for field in table.fields {
                let field_start = field.offset as usize;
                let field_end = row_start + field.width;
                let field_bytes = &rom[field_start..field_end];

                // Sanity-check the marker byte. If this isn't 0x10, the pointer doesn't
                // point at LZ77 data — could be raw bytes, Huffman, a sub-table, or junk.
                assert_eq!(blob[0], 0x10, "expected LZ77 marker, got {:#x}", blob[0]);

                let mut payload;

                match blob[0] {
                    0x10 => payload = decompress_lz77(blob).unwrap(),
                    0x20 | 0x28 => payload = decompress_huffman(blob).unwrap(),
                    _ => (),
                }
            }
        }
    }

    //TODO: Come back to this, I don't know what we'll return from this yet.
    // Theoretically this would be called by fe-tui so that the frontend can display the tables returned here to the frontentd.
    return Ok("".to_string());
}

/// Convert a raw little-endian GBA pointer (4 bytes from ROM) into a file offset.
/// Returns None if the bytes aren't a plausible ROM pointer.
fn gba_ptr_to_file_offset(bytes: [u8; 4]) -> Option<usize> {
    let ptr = u32::from_le_bytes(bytes);
    // Sanity check: ROM region is 0x08000000 or 0x09000000
    let region = ptr & 0xFF00_0000;
    if region != 0x0800_0000 && region != 0x0900_0000 {
        return None;
    }
    Some((ptr & 0x01FF_FFFF) as usize)
}
