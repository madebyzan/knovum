use yaml_rust2::{Yaml, YamlLoader};

pub(super) struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub included_tools: Vec<String>,
    pub unsupported_requirements: Vec<String>,
}

pub(super) fn parse(markdown: &[u8], directory_name: &str) -> Result<SkillMetadata, String> {
    let text = std::str::from_utf8(markdown).map_err(|_| "skill-markdown-must-be-utf8")?;
    let text = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
        .ok_or("skill-frontmatter-required")?;
    let end = text
        .lines()
        .scan(0usize, |offset, line| {
            let position = *offset;
            *offset += line.len()
                + if text.as_bytes().get(position + line.len()) == Some(&b'\r') {
                    2
                } else {
                    1
                };
            Some((position, line))
        })
        .find(|(_, line)| *line == "---")
        .map(|(position, _)| position)
        .ok_or("skill-frontmatter-unclosed")?;
    if end > 64 * 1024 {
        return Err("skill-frontmatter-size-limit".into());
    }
    let documents = YamlLoader::load_from_str(&text[..end])
        .map_err(|error| format!("skill-frontmatter-invalid: {error}"))?;
    if documents.len() != 1 {
        return Err("skill-frontmatter-requires-one-document".into());
    }
    let document = &documents[0];
    let fields = document
        .as_hash()
        .ok_or("skill-frontmatter-must-be-mapping")?;
    for (key, value) in fields {
        let key = key.as_str().ok_or("skill-frontmatter-key-must-be-string")?;
        match key {
            "name" | "description" | "license" | "compatibility" | "allowed-tools" => {
                if value.as_str().is_none() {
                    return Err(format!("skill-frontmatter-field-must-be-string: {key}"));
                }
            }
            "metadata" => {
                let metadata = value.as_hash().ok_or("skill-metadata-must-be-mapping")?;
                if metadata
                    .iter()
                    .any(|(key, value)| key.as_str().is_none() || value.as_str().is_none())
                {
                    return Err("skill-metadata-must-contain-string-values".into());
                }
            }
            _ => return Err(format!("skill-frontmatter-unknown-field: {key}")),
        }
    }
    let name = string(document, "name")?;
    let description = string(document, "description")?;
    if name != directory_name
        || name.chars().count() > 64
        || name != name.to_lowercase()
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
        || !name
            .chars()
            .all(|character| character.is_alphanumeric() || character == '-')
    {
        return Err("skill-name-must-match-directory-and-standard-format".into());
    }
    if description.chars().count() > 1024 {
        return Err("skill-description-size-limit".into());
    }
    if document["compatibility"]
        .as_str()
        .is_some_and(|value| value.trim().is_empty() || value.chars().count() > 500)
    {
        return Err("skill-compatibility-size-invalid".into());
    }
    let included_tools = document["metadata"]["include_tools"]
        .as_str()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    let unsupported_requirements = document["metadata"]["knovum.requires"]
        .as_str()
        .unwrap_or("")
        .split_whitespace()
        .filter(|requirement| !matches!(*requirement, "mcp" | "knowledge"))
        .map(str::to_owned)
        .collect();
    Ok(SkillMetadata {
        name: name.into(),
        description: description.into(),
        included_tools,
        unsupported_requirements,
    })
}

fn string<'a>(document: &'a Yaml, field: &str) -> Result<&'a str, String> {
    document[field]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("skill-required-field: {field}"))
}

#[cfg(test)]
#[path = "tests/metadata.rs"]
mod tests;
