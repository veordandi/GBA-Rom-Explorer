// tests to verify that parsing works as expected.
use fe_nmm::parse::*;
#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    fn test_parse_nmm_file() {
        // This is a fun one, where does this path go? Based on where this file is, or the execution file?
        // is there a way to get it to start from the repo root?
        let path = std::path::Path::new("../fixtures/FE* Spell Association Editor.nmm");
        let result = parse_table_path(path);
        let table = result.unwrap();

        //TODO: If we get an error back we need to handle that

        // Validate that the header was parsed properly
        assert_eq!("FE8 Spell Association Editor by Vennobennu", table.title);
        assert_eq!(0x8AFBD8, table.offset);
        assert_eq!(161, table.entry_count);
        assert_eq!(16, table.entry_size);

        // Update these two once we get the txt parsing down
        assert_eq!(None, table.entry_names_ref);
        assert_eq!(None, table.source_path);

        // Validate that the body sections were parsed properly.
        // Want to check a Hex and Dec example.

        // Validate that we parsed the txt file in correctly.
        // Check against one w/hexes and one without
    }
}
