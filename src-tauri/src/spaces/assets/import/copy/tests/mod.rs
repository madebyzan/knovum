    use super::reject_source;

    #[test]
    fn directories_are_not_space_import_sources() {
        let metadata =
            std::fs::metadata(std::env::temp_dir()).expect("temporary directory metadata");
        assert_eq!(
            reject_source(&metadata).unwrap_err(),
            "Only regular files can be imported into the Space"
        );
    }
