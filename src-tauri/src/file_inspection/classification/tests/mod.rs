use std::path::Path;

use super::{classify_file, ResourceClassification};

#[test]
fn classifies_semantic_resource_categories() {
    assert_eq!(
        classify_file(Path::new("data.csv")),
        ResourceClassification::TextDocument
    );
    assert_eq!(
        classify_file(Path::new("main.rs")),
        ResourceClassification::SourceCode("rust")
    );
    assert_eq!(
        classify_file(Path::new("notes.txt")),
        ResourceClassification::TextDocument
    );
}

#[test]
fn treats_sensitive_files_as_opaque_files() {
    assert_eq!(
        classify_file(Path::new(".npmrc")),
        ResourceClassification::File
    );
}

#[test]
fn treats_archives_as_opaque_archive_resources() {
    for name in ["bundle.zip", "site.tar.gz", "package.jar", "site.epub"] {
        assert_eq!(
            classify_file(Path::new(name)),
            ResourceClassification::Archive
        );
        assert_eq!(ResourceClassification::Archive.node_kind(), "file");
    }
    for name in ["backup.7z", "package.rar", "disk.iso", "backup.gz"] {
        assert_eq!(classify_file(Path::new(name)), ResourceClassification::File);
    }
}

#[test]
fn treats_office_documents_as_file_resources() {
    for name in ["report.docx", "sheet.xlsx", "presentation.pptx"] {
        assert_eq!(classify_file(Path::new(name)), ResourceClassification::File);
        assert_eq!(ResourceClassification::File.node_kind(), "file");
    }
}
