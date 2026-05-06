// tests to verify that parsing works as expected.
use fe_nmm::enums::NmmFieldDataType;
use fe_nmm::parse::*;
#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    fn test_parse_nmm_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/FE8_Spell_Association_Editor.nmm");
        let result = parse_table_path(&path);
        let table = result.unwrap();

        //TODO: If we get an error back we need to handle that

        // Validate that the header was parsed properly
        assert_eq!("FE8 Spell Association Editor by Vennobennu", table.title);
        assert_eq!(9108440, table.offset);
        assert_eq!(161, table.entry_count);
        assert_eq!(16, table.entry_size);
        assert_eq!(8, table.fields.len());

        // Update these two once we get the txt parsing down
        // assert_eq!(None, table.entry_names_ref);
        // assert_eq!(None, table.source_path);

        // Validate that the body sections were parsed properly.
        // Want to check a Hex and Dec example.

        // Hex example
        let hex_example = &table.fields[2];
        assert_eq!("Ranged Animation to Use", hex_example.label);
        assert_eq!(4, hex_example.offset);
        assert_eq!(1, hex_example.width);
        assert_eq!(NmmFieldDataType::Hex, hex_example.kind);

        // Dec example
        let dec_example = &table.fields[1];
        assert_eq!("No. of Chars to Display (1 or 2)", dec_example.label);
        assert_eq!(2, dec_example.offset);
        assert_eq!(2, dec_example.width);
        assert_eq!(NmmFieldDataType::Decimal, dec_example.kind);

        // Update this once we get text parsing
        // assert_eq!(None, text_example.dropdown_ref);

        // Validate that we parsed the txt file in correctly.
        // Check against one w/hexes and one without
    }
}
