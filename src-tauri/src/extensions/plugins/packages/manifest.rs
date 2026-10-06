use super::super::super::{
    content::PackageFiles,
    contracts::{PluginComponent, PluginManifest},
};

pub(super) fn read(
    files: &PackageFiles,
    components: &mut Vec<PluginComponent>,
) -> Result<PluginManifest, String> {
    let mut value = super::json(files, "plugin.json")?;
    let object = value
        .as_object_mut()
        .ok_or("plugin-manifest-must-be-object")?;
    let known = [
        "$schema",
        "name",
        "version",
        "description",
        "author",
        "homepage",
        "repository",
        "license",
        "keywords",
        "extensions",
    ];
    let unknown = object
        .keys()
        .filter(|key| !known.contains(&key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    for key in unknown {
        object.remove(&key);
        diagnostic(
            components,
            &key,
            "manifest",
            "unknown-manifest-field-ignored",
        );
    }
    if object
        .get("extensions")
        .is_some_and(|extensions| !extensions.is_object())
    {
        object.remove("extensions");
        diagnostic(
            components,
            "extensions",
            "manifest",
            "non-object-extensions-ignored",
        );
    }
    let original_extensions = object.remove("extensions");
    super::validate_schema(
        &value,
        include_str!("schemas/plugin.schema.json"),
        "plugin.json",
    )?;
    let mut manifest: PluginManifest =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    manifest.extensions = original_extensions
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok(manifest)
}

pub(super) fn diagnostic(
    components: &mut Vec<PluginComponent>,
    name: &str,
    kind: &str,
    message: impl Into<String>,
) {
    components.push(PluginComponent {
        name: name.into(),
        kind: kind.into(),
        supported: false,
        diagnostic: Some(message.into()),
    });
}
