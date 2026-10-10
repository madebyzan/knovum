use super::*;
use std::collections::BTreeSet;

#[test]
fn catalog_packages_have_supported_components_and_progressive_resources() {
    let mut plugin_names = BTreeSet::new();
    let mut plugin_ids = BTreeSet::new();
    let mut skill_names = BTreeSet::new();
    for package in PACKAGES {
        assert!(plugin_names.insert(package.name));
        assert!(plugin_ids.insert(package.id));
        let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/agent-plugins")
            .join(package.name);
        let files =
            super::super::sources::directory_files(&directory, &CancellationToken::new()).unwrap();
        let inspected =
            super::super::packages::inspect(&files, content::files_digest(&files)).unwrap();
        assert_eq!(inspected.inspection.manifest.name, package.name);
        assert_eq!(inspected.skills.len(), 1);
        assert!(
            inspected
                .inspection
                .components
                .iter()
                .all(|component| component.supported),
            "{}: {:?}",
            package.name,
            inspected.inspection.components
        );
        for skill in inspected.skills {
            assert!(skill_names.insert(skill.name.clone()));
            assert!(skill.unsupported_requirements.is_empty());
            let prefix = format!("skills/{}/", skill.name);
            let paths: Vec<_> = files
                .keys()
                .filter_map(|path| path.strip_prefix(&prefix))
                .collect();
            assert!(paths.iter().any(|path| path.starts_with("references/")));
            assert!(paths.iter().any(|path| path.starts_with("templates/")));
            let markdown = std::str::from_utf8(&files[&format!("{prefix}SKILL.md")]).unwrap();
            for path in paths {
                if path == "SKILL.md" {
                    continue;
                }
                assert!(
                    markdown.contains(path),
                    "{}: {} is undiscoverable",
                    skill.name,
                    path
                );
            }
        }
    }
    assert_eq!(skill_names.len(), PACKAGES.len());
}

#[test]
fn bundled_publication_cannot_impersonate_another_package_or_publisher() {
    let release = &marketplace().releases[0];
    let mut publication = PluginPublication {
        marketplace_id: MARKETPLACE_ID.into(),
        publisher_id: release.publisher_id.clone(),
        plugin_name: release.name.clone(),
    };
    assert!(require_publication(&publication, &release.source).is_ok());
    publication.publisher_id = "other".into();
    assert!(require_publication(&publication, &release.source).is_err());
    publication.publisher_id = release.publisher_id.clone();
    assert!(require_publication(
        &publication,
        &PluginSource::Bundled {
            name: "meeting-notes".into()
        }
    )
    .is_err());
}
