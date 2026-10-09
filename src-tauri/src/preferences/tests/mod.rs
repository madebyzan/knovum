use serde_json::json;

use super::Preferences;

#[test]
fn absent_optional_preferences_preserve_saved_choices() {
    let preferences: Preferences = serde_json::from_value(json!({
        "theme": "dark", "density": "comfortable", "textSize": "large", "locale": "zh-CN"
    }))
    .unwrap();
    preferences.validate().unwrap();
    assert_eq!(
        serde_json::to_value(preferences).unwrap(),
        json!({
            "theme": "dark", "density": "comfortable", "textSize": "large", "locale": "zh-CN",
            "backgroundTaskNotifications": true,
            "workbench": {
                "sidebarVisible": true, "sidebarWidth": 224,
                "rightSidebarVisible": false, "rightSidebarWidth": 380
            }
        })
    );
}

#[test]
fn partial_layout_preserves_explicit_false_values() {
    let preferences: Preferences = serde_json::from_value(json!({
        "theme": "system", "density": "compact", "textSize": "default", "locale": "system",
        "backgroundTaskNotifications": false, "workbench": { "sidebarVisible": false }
    }))
    .unwrap();
    preferences.validate().unwrap();
    assert!(!preferences.background_task_notifications);
    assert!(!preferences.workbench.sidebar_visible);
    assert_eq!(preferences.workbench.sidebar_width, 224);
    assert!(!preferences.workbench.right_sidebar_visible);
    assert_eq!(preferences.workbench.right_sidebar_width, 380);
}

#[test]
fn invalid_preferences_remain_errors() {
    let current = json!({
        "theme": "system", "density": "compact", "textSize": "default", "locale": "system"
    });
    for invalid in [
        json!({ "backgroundTaskNotifications": null }),
        json!({ "backgroundTaskNotifications": "true" }),
        json!({ "workbench": null }),
        json!({ "workbench": { "sidebarWidth": 199 } }),
        json!({ "workbench": { "rightSidebarWidth": 481 } }),
        json!({ "workbench": { "sidebarWidth": 224.5 } }),
        json!({ "workbench": { "sidebarVisible": "false" } }),
        json!({ "workbench": { "unknown": true } }),
        json!({ "unknown": true }),
    ] {
        let mut raw = current.clone();
        raw.as_object_mut()
            .unwrap()
            .extend(invalid.as_object().unwrap().clone());
        let result = serde_json::from_value::<Preferences>(raw)
            .map_err(|error| error.to_string())
            .and_then(|preferences| preferences.validate());
        assert!(
            result.is_err(),
            "Invalid preferences were accepted: {invalid}"
        );
    }
    let mut missing_theme = current;
    missing_theme.as_object_mut().unwrap().remove("theme");
    assert!(serde_json::from_value::<Preferences>(missing_theme).is_err());
}
