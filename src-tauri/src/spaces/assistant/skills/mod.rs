use std::collections::BTreeMap;

pub(super) fn snapshots() -> BTreeMap<String, String> {
    BTreeMap::from([(
        "/knowledge-v1/SKILL.md".into(),
        include_str!("knowledge-v1/SKILL.md").into(),
    )])
}
