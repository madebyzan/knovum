use super::{speech, HashMap, IdMaps};
use serde_json::json;

#[test]
fn separation_remaps_source_run_and_stem_assets_as_one_publication() {
    let maps = IdMaps {
        source_space: "old".into(),
        target_space: "new".into(),
        id_map_by_collection: HashMap::from([
            (
                "nodes",
                HashMap::from([("source".into(), "new-source".into())]),
            ),
            ("runs", HashMap::from([("run".into(), "new-run".into())])),
        ]),
        path_by_token: HashMap::from([
            ("source".into(), "/new/source.wav".into()),
            ("vocals".into(), "/new/vocals.wav".into()),
            ("background".into(), "/new/background.wav".into()),
        ]),
    };
    let mut value = json!({
        "voiceProfiles":[],"voiceRevisions":[],"speechProjects":[],
        "speechProjectRevisions":[],"speechSegments":[],"speechRenders":[],
        "speechSourceSeparations":[{"id":"run","runId":"run","sourceNodeId":"source",
            "sourceAssetPath":"knovum-package-asset:source",
            "vocalsAssetPath":"knovum-package-asset:vocals",
            "backgroundAssetPath":"knovum-package-asset:background"}]
    });
    speech::remap(&mut value, &maps).unwrap();
    let publication = &value["speechSourceSeparations"][0];
    assert_eq!(publication["id"], "new-run");
    assert_eq!(publication["runId"], "new-run");
    assert_eq!(publication["sourceNodeId"], "new-source");
    assert_eq!(publication["sourceAssetPath"], "/new/source.wav");
    assert_eq!(publication["vocalsAssetPath"], "/new/vocals.wav");
    assert_eq!(publication["backgroundAssetPath"], "/new/background.wav");
}
