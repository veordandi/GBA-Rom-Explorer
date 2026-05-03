// tests to verify that parsing works as expected
#[cfg(test)]
mod parse_test {
    use super::*;

    #[test]
    fn test_parse_table() {
        //TODO: Update the path here after you move real files in there
        let path = std::path::Path::new("tests/parse_test_data.txt");
        let table = parse_table_path(&path);
        // assert!() Add your asserts here
    }
}
