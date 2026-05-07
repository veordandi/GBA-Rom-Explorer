// tests to verify that parsing works as expected.
use fe_nmm::enums::NmmFieldDataType;
use fe_nmm::parse::*;
use std::num::IntErrorKind;
#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    fn test_parse_nmm_file_successful() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/FE8_Spell_Association_Editor.nmm");
        let result = parse_table_path(&path);
        let table = result.unwrap();

        // Validate that the header was parsed properly
        assert_eq!("FE8 Spell Association Editor by Vennobennu", table.title);
        assert_eq!(9108440, table.offset);
        assert_eq!(161, table.entry_count);
        assert_eq!(16, table.entry_size);
        assert_eq!(8, table.fields.len());
        assert_eq!(None, table.entry_names_ref);
        assert_eq!(path.to_str(), table.source_path.unwrap().to_str());

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
    }

    #[test]
    fn parse_nmm_file_invalid_digit_failure() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/malformed_header.nmm");
        let result = parse_table_path(&path);
        // Not sure how the error message comes back?
        assert!(result.is_err());
        let unwrapped_err = result.unwrap_err();
        match unwrapped_err {
            NmmParseError::BadInt { line, source } => {
                assert_eq!(1, line);
                assert_eq!(IntErrorKind::InvalidDigit, *source.kind())
            }
            _ => panic!("unexpected error: {:?}", unwrapped_err),
        }
    }

    #[test]
    fn parse_nmm_file_unknown_kind_failure() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unknown_kind.nmm");
        let result = parse_table_path(&path);
        // Not sure how the error message comes back?
        assert!(result.is_err());
        let unwrapped_err = result.unwrap_err();
        match unwrapped_err {
            NmmParseError::UnknownKind { line, tag } => {
                assert_eq!(14, line);
                assert_eq!("ABCD", tag)
            }
            _ => panic!("unexpected error: {:?}", unwrapped_err),
        }
    }
}
