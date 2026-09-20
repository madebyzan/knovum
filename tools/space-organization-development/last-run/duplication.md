# Duplication

Source: `a9c6bb781622093bf6415df2bbbb5eecd1a5a013`. Exit code: 0.

```text
$ jscpd src src-tauri/src --min-lines 6 --min-tokens 80 --threshold 1
Clone found (rust)
 - ai/capabilities/chat/mod.rs [93:17 - 111:40] (19 lines, 96 tokens)
   ai/capabilities/translation/mod.rs [89:17 - 107:40]
Clone found (rust)
 - ai/capabilities/editor/mod.rs [12:55 - 27:11] (16 lines, 94 tokens)
   ai/capabilities/editor/mod.rs [34:1 - 49:11]
Clone found (rust)
 - ai/capabilities/translation/mod.rs [15:51 - 35:8] (21 lines, 122 tokens)
   ai/capabilities/translation/mod.rs [49:1 - 69:8]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [1:1 - 16:11] (16 lines, 112 tokens)
   ai/providers/gemini/chat.rs [1:1 - 16:11]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [1:1 - 16:11] (16 lines, 112 tokens)
   ai/providers/ollama/chat.rs [1:1 - 16:11]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [1:1 - 16:11] (16 lines, 112 tokens)
   ai/providers/openai/chat.rs [1:1 - 16:11]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [18:27 - 28:48] (11 lines, 100 tokens)
   ai/providers/gemini/chat.rs [18:24 - 28:48]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [19:5 - 28:35] (10 lines, 96 tokens)
   ai/providers/openai/chat.rs [25:5 - 34:35]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [55:13 - 88:16] (34 lines, 196 tokens)
   ai/providers/gemini/chat.rs [49:13 - 82:16]
Clone found (rust)
 - ai/providers/dashscope/chat.rs [89:13 - 162:2] (74 lines, 499 tokens)
   ai/providers/gemini/chat.rs [89:13 - 161:2]
Clone found (rust)
 - ai/providers/dashscope/mod.rs [44:39 - 60:2] (17 lines, 94 tokens)
   ai/providers/gemini/mod.rs [40:36 - 56:2]
Clone found (rust)
 - ai/providers/dashscope/mod.rs [44:39 - 60:2] (17 lines, 94 tokens)
   ai/providers/zhipu/mod.rs [40:35 - 56:2]
Clone found (rust)
 - ai/providers/dashscope/upload.rs [24:62 - 32:70] (9 lines, 80 tokens)
   ai/providers/moonshot/upload.rs [23:60 - 33:33]
Clone found (rust)
 - ai/providers/dashscope/upload.rs [36:36 - 52:37] (17 lines, 127 tokens)
   ai/providers/moonshot/upload.rs [37:37 - 53:37]
Clone found (rust)
 - ai/providers/gemini/chat.rs [33:43 - 161:2] (129 lines, 848 tokens)
   ai/providers/ollama/chat.rs [29:43 - 157:2]
Clone found (rust)
 - ai/providers/gemini/chat.rs [33:43 - 161:2] (129 lines, 848 tokens)
   ai/providers/openai/chat.rs [39:43 - 167:2]
Clone found (rust)
 - ai/providers/gemini/chat.rs [39:9 - 51:18] (13 lines, 95 tokens)
   ai/providers/gemini/chat.rs [79:9 - 91:18]
Clone found (rust)
 - ai/providers/gemini/video.rs [62:59 - 73:16] (12 lines, 84 tokens)
   ai/providers/kling/image.rs [46:37 - 57:16]
Clone found (rust)
 - ai/providers/kling/image.rs [42:14 - 57:16] (16 lines, 100 tokens)
   ai/providers/kling/video.rs [57:38 - 72:16]
Clone found (rust)
 - ai/providers/kling/image.rs [90:17 - 102:20] (13 lines, 92 tokens)
   ai/providers/zhipu/video.rs [83:13 - 95:16]
Clone found (rust)
 - ai/providers/kling/video.rs [56:69 - 77:16] (22 lines, 147 tokens)
   ai/providers/zhipu/video.rs [38:69 - 59:16]
Clone found (rust)
 - ai/providers/minimax/video.rs [112:30 - 131:3] (20 lines, 103 tokens)
   ai/providers/zhipu/video.rs [114:30 - 133:3]
Clone found (rust)
 - ai/providers/moonshot/upload.rs [23:45 - 33:33] (11 lines, 86 tokens)
   ai/providers/openai/upload.rs [23:43 - 31:70]
Clone found (rust)
 - ai/providers/moonshot/upload.rs [23:59 - 57:54] (35 lines, 247 tokens)
   ai/providers/zhipu/upload.rs [23:61 - 57:54]
Clone found (rust)
 - ai/providers/moonshot/upload.rs [33:48 - 57:54] (25 lines, 165 tokens)
   ai/providers/openai/upload.rs [31:83 - 55:54]
Clone found (rust)
 - ai/providers/moonshot/upload.rs [43:55 - 58:63] (16 lines, 101 tokens)
   ai/providers/openai/upload.rs [41:55 - 56:61]
Clone found (rust)
 - ai/providers/stepfun/video.rs [77:48 - 87:59] (11 lines, 81 tokens)
   ai/providers/together/video.rs [79:52 - 89:59]
Clone found (rust)
 - ai/providers/stepfun/video.rs [116:47 - 128:52] (13 lines, 86 tokens)
   ai/providers/together/video.rs [118:51 - 131:26]
Clone found (rust)
 - documents/extraction/office/segments.rs [25:9 - 37:33] (13 lines, 88 tokens)
   documents/extraction/office/visuals.rs [21:9 - 33:33]
Clone found (rust)
 - spaces/assets/reclamation/tests/canvas_reference_lifecycle.rs [47:75 - 61:17] (15 lines, 90 tokens)
   spaces/canvas_state/selection/tests/invalidation.rs [36:32 - 50:17]
Clone found (rust)
 - spaces/assets/reclamation/tests/canvas_reference_lifecycle.rs [50:22 - 61:17] (12 lines, 83 tokens)
   spaces/canvas_state/selection/tests/mod.rs [105:35 - 116:17]
Clone found (rust)
 - spaces/canvas_state/selection/tests/invalidation.rs [39:44 - 53:25] (15 lines, 92 tokens)
   spaces/canvas_state/selection/tests/mod.rs [15:64 - 29:25]
Clone found (rust)
 - spaces/canvas_state/selection/tests/mod.rs [15:61 - 29:25] (15 lines, 94 tokens)
   spaces/canvas_state/tests/mod.rs [45:64 - 59:25]
Clone found (rust)
 - spaces/commands/scenes/mod.rs [50:26 - 62:6] (13 lines, 99 tokens)
   spaces/commands/scenes/mod.rs [91:5 - 103:6]
Clone found (rust)
 - spaces/commands/scenes/mod.rs [104:26 - 113:24] (10 lines, 100 tokens)
   spaces/commands/scenes/mod.rs [173:23 - 182:24]
Clone found (rust)
 - spaces/commands/selection_transfer/operations.rs [131:5 - 142:11] (12 lines, 140 tokens)
   spaces/nodes/lifecycle/effects.rs [55:5 - 66:11]
Clone found (rust)
 - spaces/derivation/document_export/lifecycle.rs [69:106 - 81:45] (13 lines, 88 tokens)
   spaces/derivation/image_cutout/mod.rs [60:38 - 75:45]
Clone found (rust)
 - spaces/derivation/grounded_document/persistence.rs [9:22 - 22:8] (14 lines, 95 tokens)
   spaces/derivation/presentation_export.rs [40:15 - 53:8]
Clone found (rust)
 - spaces/derivation/grounded_document/persistence.rs [9:5 - 22:8] (14 lines, 97 tokens)
   spaces/derivation/visualization/persistence.rs [23:17 - 36:8]
Clone found (rust)
 - spaces/derivation/grounded_document/persistence.rs [28:34 - 46:13] (19 lines, 123 tokens)
   spaces/derivation/visualization/persistence.rs [50:66 - 65:13]
Clone found (rust)
 - spaces/derivation/image_cutout/mod.rs [57:229 - 74:13] (18 lines, 97 tokens)
   spaces/derivation/subtitle_extraction.rs [41:256 - 53:13]
Clone found (rust)
 - spaces/derivation/image_cutout/mod.rs [57:229 - 72:10] (16 lines, 90 tokens)
   spaces/derivation/subtitled_video/start.rs [44:240 - 55:14]
Clone found (rust)
 - spaces/derivation/knowledge_video/preparation.rs [159:66 - 170:51] (12 lines, 101 tokens)
   spaces/derivation/presentation_export.rs [112:80 - 122:51]
Clone found (rust)
 - spaces/knowledge/provenance/rows.rs [13:51 - 19:4] (7 lines, 118 tokens)
   spaces/queries/relations.rs [35:64 - 41:2]
Clone found (rust)
 - spaces/knowledge/types/mod.rs [35:32 - 43:47] (9 lines, 102 tokens)
   spaces/knowledge/types/mod.rs [65:32 - 73:47]
Clone found (rust)
 - spaces/knowledge/types/rows.rs [8:5 - 14:18] (7 lines, 139 tokens)
   spaces/queries/export/knowledge_rows.rs [23:13 - 29:26]
Clone found (rust)
 - spaces/package/write/archive.rs [70:41 - 76:16] (7 lines, 89 tokens)
   spaces/package/write/planning.rs [102:29 - 108:15]
Clone found (rust)
 - spaces/previews/thumbnail.rs [9:26 - 19:50] (11 lines, 84 tokens)
   spaces/previews/thumbnail.rs [24:28 - 34:50]
Clone found (rust)
 - spaces/queries/export/rows.rs [47:32 - 53:87] (7 lines, 159 tokens)
   spaces/queries/scenes.rs [30:5 - 36:87]
Clone found (html)
 - features/settings/components/CreationWorkflowEditor.vue:html [74:16 - 82:101] (9 lines, 100 tokens)
   features/settings/components/GenerationPresetEditor.vue:html [60:100 - 68:101]
Clone found (typescript)
 - features/settings/components/CreationWorkflowEditor.vue:typescript [20:67 - 33:50] (14 lines, 82 tokens)
   features/settings/components/GenerationPresetEditor.vue:typescript [20:65 - 33:50]
Clone found (typescript)
 - features/settings/components/panes/AiModelsSettingsPane.vue:typescript [58:1 - 72:17] (15 lines, 85 tokens)
   features/settings/components/panes/DesktopAgentsSettingsPane.vue:typescript [58:1 - 72:17]
Clone found (typescript)
 - features/spaces/__tests__/canvas-persistence-history.test.ts [9:20 - 16:12] (8 lines, 99 tokens)
   features/spaces/__tests__/canvas-session-commit-guard.test.ts [12:18 - 20:8]
Clone found (typescript)
 - features/spaces/__tests__/canvas-surface-host.test.ts [13:1 - 26:35] (14 lines, 138 tokens)
   features/spaces/__tests__/canvas-surface-remount.test.ts [9:32 - 22:35]
Clone found (typescript)
 - features/spaces/__tests__/canvas-surface-host.test.ts [89:49 - 96:55] (8 lines, 83 tokens)
   features/spaces/__tests__/canvas-surface-host.test.ts [108:86 - 115:55]
Clone found (typescript)
 - features/spaces/__tests__/konva-scene-projection.test.ts [21:47 - 34:18] (14 lines, 92 tokens)
   features/spaces/__tests__/konva-scene-projection.test.ts [54:64 - 67:18]
Clone found (typescript)
 - features/spaces/__tests__/konva-scene-projection.test.ts [50:89 - 63:10] (14 lines, 94 tokens)
   features/spaces/__tests__/konva-scene-projection.test.ts [164:86 - 177:10]
Clone found (typescript)
 - features/spaces/__tests__/konva-scene-projection.test.ts [61:39 - 68:97] (8 lines, 87 tokens)
   features/spaces/__tests__/konva-scene-projection.test.ts [181:39 - 188:97]
Clone found (typescript)
 - features/spaces/__tests__/konva-scene-projection.test.ts [80:56 - 101:23] (22 lines, 170 tokens)
   features/spaces/__tests__/konva-scene-projection.test.ts [119:68 - 140:23]
Clone found (typescript)
 - features/spaces/canvas/konva/scene/viewport-culling.ts [19:39 - 31:18] (13 lines, 90 tokens)
   features/spaces/canvas/konva/scene/viewport-culling.ts [35:31 - 47:18]
Clone found (typescript)
 - features/spaces/model/nodes/schema/resource-variants.ts [48:30 - 58:22] (11 lines, 89 tokens)
   features/spaces/model/nodes/schema/resource-variants.ts [59:30 - 69:22]
Clone found (typescript)
 - features/spaces/presentation/SpaceSceneCue.vue:typescript [21:1 - 41:32] (21 lines, 130 tokens)
   features/spaces/presentation/canvas/SpaceCanvasSceneFeedback.vue:typescript [19:2 - 39:32]
Clone found (html)
 - features/spaces/presentation/canvas/nodes/SpaceCanvasTextSelectionBar.vue:html [101:7 - 127:18] (27 lines, 111 tokens)
   features/spaces/presentation/editor/SpaceTiptapInlineMarks.vue:html [53:11 - 38:12]
Clone found (typescript)
 - shared/ui/UiCheckbox.vue:typescript [3:41 - 23:6] (21 lines, 89 tokens)
   shared/ui/UiSwitch.vue:typescript [2:33 - 22:6]
┌────────────┬────────────────┬─────────────┬──────────────┬──────────────┬──────────────────┬───────────────────┐
│ Format     │ Files analyzed │ Total lines │ Total tokens │ Clones found │ Duplicated lines │ Duplicated tokens │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ css        │ 13             │ 1619        │ 11764        │ 0            │ 0 (0.00%)        │ 0 (0.00%)         │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ html       │ 203            │ 17566       │ 49365        │ 2            │ 34 (0.19%)       │ 211 (0.43%)       │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ json       │ 36             │ 2344        │ 8924         │ 0            │ 0 (0.00%)        │ 0 (0.00%)         │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ markdown   │ 2              │ 30          │ 351          │ 0            │ 0 (0.00%)        │ 0 (0.00%)         │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ rust       │ 563            │ 45533       │ 333173       │ 49           │ 968 (2.13%)      │ 7185 (2.16%)      │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ typescript │ 830            │ 50361       │ 353640       │ 13           │ 170 (0.34%)      │ 1328 (0.38%)      │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ vue        │ 228            │ 17969       │ 103462       │ 0            │ 0 (0.00%)        │ 0 (0.00%)         │
├────────────┼────────────────┼─────────────┼──────────────┼──────────────┼──────────────────┼───────────────────┤
│ Total:     │ 1875           │ 135422      │ 860679       │ 64           │ 1172 (0.87%)     │ 8724 (1.01%)      │
└────────────┴────────────────┴─────────────┴──────────────┴──────────────┴──────────────────┴───────────────────┘
Found 64 clones.
time: 319.281ms

```
