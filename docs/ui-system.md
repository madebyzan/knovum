# Knovum desktop design system

## Product language

Knovum is a desktop workbench. The target is Apple-like platform sensibility plus Linear-like information density, not a web dashboard and not a visual clone of either product.

The design language is **quiet, layered, native-aware and keyboard-first**:

- navigation/control chrome may use native material; content stays calm and opaque;
- hierarchy comes from surface level, spacing, typography and selection before borders or shadows;
- primary text is readable at normal desktop distance; tiny type is exceptional metadata only;
- controls are compact but not cramped;
- separators are structural exceptions, not the default way to group every region;
- normal system state is silent: no permanent "Online" or "Ready" chrome;
- no hero sections, marketing cards, giant headings, pill-heavy navigation or dashboard grids inside workspaces.

## Surface hierarchy

The visual system has four layers:

1. **Window / navigation chrome** — the unified toolbar and feature sidebar share one quiet opaque plane.
2. **Content** — document, canvas, diff, conversation and package documentation. Content uses an opaque `bg-canvas` surface.
3. **Elevated** — menus, popovers, command palette and transient task UI use `bg-overlay`, a strong hairline boundary and restrained `shadow-popover` separation.
4. **Modal** — dialogs use `bg-overlay` + `shadow-dialog` and a restrained scrim.

Elevation belongs primarily to floating layers. Space canvas objects use only their dedicated micro-elevation:
a short contact shadow that separates movable content from the infinite canvas without making it read as a
popover. They never borrow panel, popover or dialog elevation.

Do not use blur or glass effects inside normal content cards. Native material belongs to navigation/control layers.

## Tailwind CSS v4 contract

`src/styles/tokens.css` is the only design-token source and uses Tailwind v4 `@theme` namespaces.

Prefer semantic utilities:

```html
<div class="bg-canvas text-body text-foreground">
  <p class="text-caption text-foreground-secondary">...</p>
  <button class="h-(--knovum-control-height-md) rounded-control bg-accent text-on-accent">...</button>
</div>
```

Do not bypass an existing token:

```text
NO  bg-[var(--color-canvas)]
NO  text-[var(--color-foreground-secondary)]
NO  text-[11px]
NO  rounded-[6px]
NO  shadow-[...]

YES bg-canvas
YES text-foreground-secondary
YES text-caption
YES rounded-control
YES shadow-popover
```

Arbitrary values are only for genuinely instance-specific geometry or runtime values owned by a primitive.

Repeated product geometry must have a semantic radius token. Current specialized roles include
`rounded-section` / `rounded-section-inner` for Space
canvas sections, and `rounded-grid-cell` for compact editor grid pickers. These are product roles, not
permission to add one-off radius values.

Component-local Vue `<style scoped>` blocks must not create aliases by `@apply`-ing Knovum semantic utilities. Tailwind v4 processes scoped styles separately, so theme access there requires an explicit `@reference`; repeating utility composition in the template is the default and keeps the token contract visible. If CSS is genuinely required for selectors that cannot be expressed in markup, reference the main stylesheet deliberately rather than relying on implicit theme scope.

## Color roles

| Utility                                          | Role                                      |
| ------------------------------------------------ | ----------------------------------------- |
| `bg-chrome`                                      | unified window, sidebar and toolbar plane |
| `bg-canvas`                                      | opaque primary content                    |
| `bg-surface`                                     | grouped/settings/embedded surface         |
| `bg-raised`                                      | selected control or local elevation       |
| `bg-overlay`                                     | floating surface                          |
| `bg-field`                                       | quiet input/search surface                |
| `bg-(--knovum-sidebar-item-background-hover)`    | navigation hover                          |
| `bg-(--knovum-sidebar-item-background-selected)` | navigation selection                      |
| `border-divider`                                 | rare structural separator                 |
| `border-window-hairline`                         | crisp window/workspace edge               |
| `text-foreground`                                | primary text                              |
| `text-foreground-secondary`                      | secondary text                            |
| `text-foreground-tertiary`                       | tertiary metadata                         |
| `text-accent` / `bg-accent`                      | restrained primary accent                 |
| `bg-(--knovum-space-guide-spacing-color)`        | Space canvas equal-spacing measurement    |
| `border-border-hover`                            | one deeper object boundary on hover       |

Features must not invent their own gray palette or generic accent palette. Measurement guides use their own
token because a span that reads as selection blue or destructive red would misstate what the drag is doing.

Navigation and transient chrome use the semantic surface utilities defined in `src/styles/app.css`:

- `material-navigation` for the feature sidebar plane;
- `material-toolbar` for integrated window toolbars;
- `material-overlay` for transient floating chrome that genuinely overlays content.

Chrome remains monochromatic and reserves accent color for selection, focus, status, and primary actions.
It must preserve its hierarchy when contrast is increased, the window becomes inactive, or dark appearance
is active. Content surfaces and canvas objects must not use navigation or overlay surfaces.

The light appearance uses one exact `rgb(239 239 240)` chrome plane for the window background, toolbar and
sidebar. This role stays in sRGB because its one-channel cool bias must remain exact; a zero-chroma OKLCH
neutral would resolve to `rgb(239 239 239)`. The canvas is an opaque near-white work surface. Separation comes
from the canvas contrast and restrained elevation, not persistent divider lines. Navigation selection stays
neutral; blue is reserved for focus, active selection, and primary actions. Window activation may soften text
and selection, but it never changes the shared chrome surface color.

Editor/document content may need semantic colors that do not belong to application chrome (for example,
highlight marks). Those colors must live in an editor/content token namespace when reused; a content color
is not permission to hard-code a new application accent or neutral.

## Focus presentation

The global base stylesheet resets WebView default `:focus` outlines. Pointer focus does
not draw an outline; `:focus-visible` owns keyboard focus presentation using a restrained
inset hairline. Controls with a dedicated keyboard indicator may override this base role.
Do not blur controls, prevent pointer focus, or add input-modality event listeners to hide
focus. Reka continues to own focus movement and restoration. Editable-field focus borders,
menu highlighting and canvas-object selection are separate states, not button focus rings.

## Motion

Desktop motion is short, interruptible, and spatially truthful. Use Vue's built-in `<Transition>` and CSS for
shell and component enter/leave states; do not add a motion runtime for simple opacity and transform effects.

- transient sidebar: horizontal transform plus opacity, 200ms enter and 150ms exit;
- scrim: opacity only, 200ms enter and 150ms exit;
- view swaps: at most 150ms with a 4px directional offset;
- controls: 100–150ms paint transitions; toolbar controls never change geometry when pressed;
- never animate canvas-object geometry, sidebar resize, or layout dimensions, and never interpolate a persisted
  node position while it is dragged, undone or restored; the single exception is a command-initiated rearrange
  (align, distribute, auto layout), which may transition the rendered transform for 150ms and stays subject to
  the global reduced-motion rule;
- `prefers-reduced-motion` disables all nonessential transitions globally.

Canvas node work uses one quiet silver-light language. The node payload and feature session own lifecycle
state; the canvas projects those states into presentation without persisting animation state. An unavailable
content region remains a plain panel with one broad, slow diffuse light and a centered status capsule.
Available content stays legible; its chrome uses a three-stroke breathing mark, while an image edit being
applied shows the same diffuse light only over the edited image region. Completion replaces the placeholder
after real content is ready; image, media and markdown surfaces fade out the outgoing state. Failure is static
and keeps a short semantic label. Do not add a dot field,
spinner, bottom progress bar or percentage unless the operation reports real progress. Reduced motion
stops the light and mark animations.

## Typography roles

The type scale is semantic; components choose a role, never a pixel size.

| Utility         | Default | Use                                                    |
| --------------- | ------: | ------------------------------------------------------ |
| `text-micro`    |    11px | keyboard hints/debug metadata only                     |
| `text-caption`  |    12px | secondary labels and descriptions                      |
| `text-label`    |    13px | controls, rows and navigation                          |
| `text-body`     |    13px | normal reading content                                 |
| `text-body-lg`  |    14px | emphasized labels and compact headings                 |
| `text-title-sm` |    15px | subview/section title                                  |
| `text-title`    |    17px | workspace/settings title                               |
| `text-display`  |    24px | settings root, onboarding and exceptional empty states |

`Settings -> Text size` changes the values behind these roles. Features continue using the same utilities.

## Geometry and density

Global roles include:

- `h-(--knovum-control-height-sm)`, `h-(--knovum-control-height-md)`, `h-(--knovum-control-height-lg)`;
- `h-window-chrome` for integrated titlebar/toolbar geometry;
- `h-(--knovum-compact-item-height)` for compact navigation and menu density;
- `size-(--knovum-icon-size-xs)/sm/default/toolbar/lg` for symbols. Toolbar symbols use the dedicated 18px optical role inside 32px controls.

`data-platform` changes titlebar/chrome geometry where the operating systems differ. `data-density` changes interaction density without changing typography hierarchy. `h-window-chrome` is the stable native titlebar band and intentionally does not shrink with content density; density-sensitive in-workspace toolbars use their own toolbar/control geometry.
On macOS the integrated Tab Bar is 40px high, matching compact modern desktop chrome while preserving the
native traffic-light safe area.

### Workspace navigation hierarchy

Knovum is a single-purpose knowledge-space application. The Spaces sidebar is the leading application
navigation surface and sits directly against the native window edge; there is no application-module rail.
Settings is an application preference destination available through the command palette and a system shortcut;
it is not a persistent row in the Spaces sidebar.

The window toolbar owns native window chrome and tab navigation. Its compact
tab strip exposes the Spaces opened in the current desktop session, with back/forward history, an explicitly
scoped global Space search and a trailing New Space action. The Spaces sidebar remains the durable Space browser,
while the sidebar's current-Space Outline owns document structure. The sidebar footer exposes Recently Deleted
directly. Space context menus own Space management; a compact bottom-right content overlay owns Task Center
presentation. Neither tasks nor document actions are appended to the tab strip.
This keeps application preferences, space navigation and document/object navigation distinct without
presenting the product as a collection of unrelated tools.

The toolbar is a full-width window drag surface. Only concrete interactive controls opt back into pointer
input; empty layout space remains draggable even when it lies inside a feature toolbar's grid area.

The leading toolbar region matches the live sidebar width and contains only the sidebar visibility control.
Outline and feature actions begin at the workspace edge. Resizing the sidebar updates both regions from one
shell-owned width value.

The Spaces sidebar uses a consistent 224px default width in both pinned and transient-preview states. A
pinned sidebar remains resizable between 200px and 320px. Its resize feedback is painted on the workspace
surface boundary, never inside the navigation background or the window gutter. The contextual Inspector is an
independently animated trailing overlay on the canvas. Object and Space properties use a compact floating Inspector
inset from the upper trailing edge, separated by spacing and a hairline rather than elevation. Long-form Ask and
Source modes use a full-height resizable trailing dock that participates in the work-area split, keeping the
canvas visible instead of covering it. Neither presentation participates in the navigation splitter or changes
the sidebar width, preserving the user's spatial frame.

The primary workspace surface is flush with the toolbar bottom. It is flush with the leading sidebar edge while
the sidebar is pinned; when the sidebar is collapsed, an 8px leading window inset matches the trailing and bottom
insets. Hover preview remains an overlay and does not change this geometry. The surface uses a 12px workspace
radius and one real hairline border; elevation does not substitute for any edge.

### Space canvas object geometry

Space canvas objects use **semantic desktop geometry**, never one universal card size. Initial size, resize bounds and fixed/resizable behavior are owned centrally by `src/features/spaces/canvas/node-size.ts`; node components must not invent independent min/max values.

Canvas nodes are desktop objects, not floating web cards. Every movable content node uses one opaque surface, an 8px object radius, one quiet hairline and the shared `shadow-canvas-object` elevation. That elevation combines a tight contact shadow, a short downward cast and one low-opacity ambient layer, creating the grounded paper-on-canvas depth used by spatial desktop tools without borrowing popover height. It remains identical at rest, hover, selection and drag, so interaction never produces a web-card lift. Hover may change only paint properties such as background or border color; the canvas object's one hover step is its boundary hairline deepening to `border-border-hover` at an unchanged width, and it must never change geometry, transform, font weight, padding or icon-box dimensions. Ordinary object selection does not draw a rectangular accent outline: the contextual toolbar and capability-specific resize handles identify the active object. Transparent Text is the deliberate exception and receives the minimal boundary treatment defined below. Sections remain spatial containers and therefore have neither card surface nor object shadow. Type-specific meaning belongs in the icon, label and content—not colored card chrome.

- Notes and Markdown use compact reading widths; code is deliberately wider; websites, files and containers reserve enough area for a useful preview without becoming dashboard cards.
- Image and video nodes give the full card to the visual content; their names and metadata belong to selection and Inspector, while transient work status may sit over the image. Files retain a fixed-height caption strip for recognition at every detail level. Visual resize preserves the intrinsic image or video ratio without reserving caption height.
- Credentials, aliases, smart queries and portals are compact desktop utilities with fixed geometry. They do not grow into arbitrary blank rectangles.
- Audio is a fixed compact transport surface. Video chooses portrait, square or landscape node geometry from its display aspect; resizing preserves that proportion while transport controls overlay the viewport.
- Images initialize from intrinsic pixel geometry. Standard photo/video ratios are preserved directly; extreme panoramas or long screenshots remain fully visible with `object-contain` inside bounded desktop-safe geometry so labels and resize handles stay usable.
- Documents initialize from the page ratio their system render reported, so a portrait page stays portrait and a landscape deck stays landscape inside the same bounded geometry. A near-square render is a generic icon plate rather than a page and keeps the default file card. Unlike media, a document card stays freely resizable: its content is `object-contain`, so resizing cannot distort the page.
- Sections are the only intentionally large spatial container and have their own broad resize envelope.

Every imported visual asset, and every plain document the system renderer could draw, carries intrinsic width/height through the Rust import boundary. A node may appear with a neutral processing placeholder while bytes are copied, but once intrinsic geometry is known the canvas must recompute its designed initial geometry rather than preserving an arbitrary placeholder ratio.

### Space canvas adaptive detail

Canvas zoom changes information density, not node geometry. The viewport resolves one shared detail profile:

| Profile | Preview item budget | Meaning                                       |
| ------- | ------------------- | --------------------------------------------- |
| compact | 2                   | recognition and primary identity              |
| summary | 4                   | recognition plus a short structured preview   |
| full    | 10                  | the normal non-interactive reading projection |

The item budget is semantic: Tiptap and Markdown count root blocks, subtitles count cues, code counts lines,
archives count entries, and structured objects count fields. Markdown is projected from source block ranges before
rendering; rendered rich DOM must never be line-clamped as the primary density mechanism. Media and fixed utility
nodes keep their type-owned representation instead of pretending every kind contains a list.

Viewport detail and interaction are independent state. Activating a node switches it from `preview` to
`interactive` content without rewriting the viewport profile; editors, complete source and embedded controls may
then reveal the canonical content. Zooming alone must not persist a frame, derived height or content mutation.
Text may keep an invisible canonical measurement plane while presenting a projected preview, so display density
cannot feed back into persisted geometry.

Every canvas node kind declares its projection strategy exhaustively. New kinds must choose a strategy when they
enter the node union; there is no generic compatibility fallback and no component-local zoom threshold. The
viewport owns the shared profile, the node type owns physical layout and content projection, and the interaction
state owns whether the projection or canonical content is shown.

### Space canvas object interaction model

Unavailable actions remain discoverable unless irrelevant to the selection. Dense toolbars and menus keep their normal row geometry and disclose a short reason on hover or keyboard focus; panels place one focusable help control beside the disabled execution action. Setup help may link to the relevant settings pane. Do not place persistent explanatory paragraphs between controls or repeat help icons on every option. The common help control owns Reka focus and popover behavior, while the feature supplies localized reason text and optional navigation.

Canvas objects follow a desktop direct-manipulation model. Content, selection chrome and spatial geometry are separate responsibilities:

- `SpaceNodeFrame` owns a fixed outer geometry root with `overflow: visible` and a clipped inner content surface;
- Konva content-node shapes are transparent interaction geometry; the DOM content surface is the sole painter of
  the node background, boundary and micro-elevation. Sections remain Konva-painted spatial backgrounds;
- the content surface never changes geometry, border width, font weight, transform or elevation for hover, focus, selection or drag; its stable micro-elevation comes only from `shadow-canvas-object`, and its one hover change is the boundary `border-color`;
- selection, resize and relation controls live in an interaction overlay outside the clipped content surface;
- the interaction overlay is absolutely positioned and cannot participate in node measurement;
- resize and relation coordinates are zero-size anchors fixed to the object boundary; visible marks and pointer targets are centered pseudo-elements, so inverse zoom scaling cannot move the anchor;
- node dimensions come from persisted data plus the central size policy, never from a hover-dependent DOM measurement;
- loading, analysis results, transcripts, chapters and other enrichment may replace content inside an allocated region but must not change the node's outer dimensions.

The pointer state machine is explicit:

1. Hover identifies the pointer target through the cursor and by deepening the object's boundary hairline one paint step. It does not repaint object content or move any geometry.
2. A single click selects the object and reveals one stable accent selection outline.
3. Dragging anywhere on non-interactive object content moves the selection. Selection and connector chrome hide at drag start, remain absent throughout preview and snapping, and return only after the final frame has been applied locally.
4. Double-click performs the type's primary open/edit action.
   Audio and video are the exception: selection immediately enables their embedded transport controls, while playback still requires an explicit play action. Media never uses a double-click merely to reveal a second play button. Only the play button and active transport accept DOM pointer input; the remaining media surface stays available to Konva for dragging. The video transport uses the regular media-control density; the fixed-height audio object uses the compact density. Both remain in object space so canvas zoom scales the complete transport and its hit targets together.
5. Resize handles appear only for a selected resizable object. Most objects have no Transformer border; transparent Text uses a screen-stable 1px accent border so its editable bounds remain legible. Proportional visual nodes expose four corner handles; freely resizable nodes expose four corners plus four edge midpoints; Text exposes left/right width handles and one proportional bottom-right handle, with no other corner or independent height handles. Each 7px accent circle overlaps its node boundary by one quarter and carries a surface-colored separation ring; its transparent pointer target is at least 20px square. Konva owns the enabled anchors, resize cursor, aspect constraint and transform lifecycle.
   Text side-resize is reflow: width changes while measured content derives height. Text bottom-right resize is scale: one gesture baseline scales the complete typography plane, including inline marks, line spacing, list indentation and insets. Preview, commit and history treat frame plus typography scale as one layout update; content measurement rejects stale width/scale observations instead of correcting them later.
6. Relation-capable objects use the Konva adapter's connector projection and hit layer. Knovum owns semantic relation records; the adapter owns endpoint geometry, hit testing and interaction feedback.
7. Selection and drag indicators are painted by the overlay, so selecting one object cannot rerasterize sibling object content.
8. In Select mode, an unmodified left drag on empty canvas draws a partial-intersection rubber band, and a plain click clears the selection. Pan mode makes left drags pan, including over knowledge nodes; the middle button, held Space and two-finger scrolling also pan through the same viewport controller. Pinch zooms. The top-center compact tool group exposes Select, Pan, Note and Text; Outline belongs to the sidebar, Space management belongs to native context menus, and zoom and Saved Views remain bottom-left. The shell-owned Task Center appears at the content area's bottom-right and opens upward; node-local acquisition progress remains on the canvas. Placement tools use a crosshair and return to Select after creating empty inline-editable knowledge at the clicked world coordinate. Selection chrome and content controls are inactive during Pan or placement. The canvas engine's own delete key stays disabled so deletion remains a Knovum command.

9. Empty-canvas cursors state the current meaning: arrow at rest, crosshair only while the rubber band is being drawn, grabbing while the viewport pans. Objects keep grab and grabbing.
10. Dragging reports its own alignment. The moving frame snaps to nearby object anchors and to the center of the surrounding content bounds, and one hairline marks the anchor it caught. When the gaps on both sides of the moving frame are equal, end-capped measurement spans carrying their shared distance replace that hairline. Snap and measurement share one constant screen-space tolerance, so the same drag feels identical at every zoom level; all guides clear when the drag ends and none of them is persisted geometry.

A single selected content node or Section shows one screen-stable action bar centered on the object's bounds and clamped to the viewport, above its bounds when there is room and below otherwise. Its opaque overlay has the existing panel radius, neutral hairline, compact label type and restrained control elevation. The bar measures the available viewport width: primary actions use icon and label while they fit, then transition to icon-only controls with accessible names and hover titles. Type-specific formatting, image tools and duplication remain visible, while structural commands and derivations remain under More. The bar clears the full pointer area of a selected node's connector controls, not merely the object edge. The final segment after More contains the universal icon-only **Details** and **Download** controls, separated by a full-height hairline. Download opens the native save dialog and writes that node's supported representation to a local file. Edited images download as the rendered PNG, including crop, rotation, flip, tonal adjustments and eraser strokes. A Section downloads a structured outline, not an asset package; credential downloads contain non-secret metadata only. The bar is selection chrome, so unselected nodes never carry persistent download or overflow buttons. Multi-selection keeps its separate compact structural toolbar.

The image action bar keeps direct image actions visible. Adjust, crop, flip/rotate and eraser use one temporary edit session: entering a tool replaces the action bar with that tool's panel, changes preview on the node, Apply writes one history entry, and Cancel discards the draft. Comment mode likewise occupies the action area; upscale, multi-angle and layer-splitting popovers have one shared open state so only one is open at a time. Crop exposes a dimmed outside area, direct drag handles, and drag-to-draw selection over the full source image. Its side panel groups W/aspect lock/H, collapsible, single-row general and social presets with aspect glyphs and reference dimensions, and Cancel/Done. The aspect lock is one session value shared by numeric fields and canvas handles. Adjust uses one scrollable Light, Color, Detail and Effects list; the four icons scroll to sections and track the visible section. The strict v5 edit contract contains the Light, Color, Detail and Effects controls, and Vue preview and Rust output both render them. Unsupported generation actions remain inspectable; only their execution controls are disabled. Active eraser and crop sessions enable direct pointer interaction on the selected image. The transform mode uses a compact replacement toolbar. The eraser panel keeps its undo action in the header, brush size in a single neutral slider, and Apply disabled until a stroke changes. The multi-angle popover offers subject/camera modes, a draggable angle guide, rotation, tilt and three framing levels. Its thumbnail renders the managed original with the committed `imageEdit`, matching the input Rust later materializes. The guide is an interaction aid, never a generated result. Its controls become bounded model instructions in Rust. Layer splitting offers automatic four-layer output or a custom layer count and optional separation guidance; the UI never quotes an invented credit cost. The WebView owns interactive preview only. Rust reads the authorized managed asset and applies the strict `imageEdit` contract for downloaded PNGs, local background removal and cloud inputs, then validates and publishes output. Upscale and multi-angle generation create derived nodes through the configured image provider. Layer splitting creates independent transparent image nodes only when its dedicated provider credential is configured. Unavailable cloud capabilities state the missing configuration instead of presenting a successful no-op.

Group and Section interaction uses the same compact desktop language while keeping their different roles visible. A selected Group uses a quiet dashed world-space boundary, title chip and screen-stable rotate/scale handles; double-click enters its editing scope and the scope path remains visible above the canvas. A Section uses a low-chroma background, 40px title header, content count and dedicated resize handles. Its collapse control hides descendants without changing their Structure/Layout. During a valid drag, the destination Section receives one accent drop preview; a collapsed destination temporarily expands only after the 600ms dwell and returns to its persisted presentation when the preview is cancelled. Selection toolbars expose explicit dissolve, reparent, fit, collapse and subtree-delete meanings instead of inferring ownership from geometry.

Resize is a capability, not a default decoration. Image and video resize with their media viewport ratio preserved. Notes, Markdown, websites, files, code, folders, archives and knowledge objects resize within type-specific bounds. Audio, credentials, aliases, queries and portals remain fixed compact utilities. Sections use their dedicated container resize treatment.

Space library covers are Canvas Overviews: they fit all visible authored objects, sections and relations into one fixed 16:9 image, independent of the current pan and zoom. The cover renderer uses the same document projection and color tokens as the editor but draws a simplified, non-interactive Konva scene; DOM content and workbench controls are never captured. Covers refresh after content or scene-geometry changes, not viewport navigation.

Relations connect knowledge-bearing objects, not every selectable canvas utility:

| Object role                  | Kinds                                                     | Relation capability | Reason                                                                                                |
| ---------------------------- | --------------------------------------------------------- | ------------------- | ----------------------------------------------------------------------------------------------------- |
| Authored knowledge           | note, text, markdown, object                              | Yes                 | The object contains an assertion, explanation or authored source.                                     |
| Imported/reference knowledge | website, image, video, audio, file, code, folder, archive | Yes                 | The object is a retrievable evidence or source boundary.                                              |
| Secure infrastructure        | credential                                                | No                  | A secret is configuration, never knowledge-graph content.                                             |
| Spatial organization         | Structure Group/Section, text                             | No                  | Containers organize content; free text labels the canvas without asserting a knowledge relation.      |
| Projection/navigation        | portal                                                    | No                  | Portals navigate to another Space, Saved View or node without creating a duplicate relation identity. |

This distinction must be visible and predictable. Relation-capable selected objects show anchors and expose the relation command. Non-capable objects show neither; their details identify them as organization, navigation or secure configuration objects. There is no disabled invisible handle and no handle that appears only because the pointer happened to cross a narrow edge.

Rendered relations use the Konva adapter's relation projection. Endpoint geometry remains adapter-owned canvas state and is never copied into the semantic relation model; the Space domain persists only relation identity, kind, status and provenance. User-selected relation colors are sparse canvas-presentation overrides backed by the `--knovum-canvas-relation-*` tokens. Neutral is represented only by the absence of an override. Parallel semantic relations between the same visible endpoints use stable, separated curve lanes so their labels and hit targets remain distinct.

### Space canvas object layouts

Audio and video use one media-session lifecycle. A natural playback completion unloads the active
transport, resets its timeline and returns the node to its preview representation; replay is an
explicit new session that starts from the beginning.

Each node layout represents the object's job instead of reusing a universal header/body/footer card:

| Kind           | Primary canvas representation                                   | Persistent chrome                                                           | Enrichment placement                                                                    |
| -------------- | --------------------------------------------------------------- | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| image          | The image itself, fully visible within intrinsic-ratio geometry | None; name and metadata belong to selection/Inspector                       | Visual description, OCR and tags in Inspector                                           |
| video          | Fixed-ratio media viewport with centered play affordance        | None; transport overlays the viewport, name and metadata live in Inspector | Transcript, summary and chapters in Inspector; chapter marks may appear on the timeline |
| audio          | Compact horizontal waveform and transport                       | One title row; no empty preview region                                      | Transcript and chapters in Inspector                                                    |
| file/PDF       | Document thumbnail or readable first-page preview               | One title row; never repeat the filename as subtitle                        | Extracted text and document structure in Inspector                                      |
| note           | Direct editable note surface                                    | No file-style header; title is the first content role                       | AI/knowledge metadata in Inspector                                                      |
| text           | Transparent free-standing canvas typography                     | None; selection alone reveals typography controls                           | None                                                                                    |
| markdown       | Reading/editing surface with document typography                | Compact document title, not a media/file card footer                        | Outline and generated knowledge in Inspector                                            |
| website        | Bookmark preview with favicon, title and domain                 | Domain is secondary identity; URL is not repeated in a footer               | Extracted sections in Inspector                                                         |
| code           | Monospaced source preview                                       | Filename plus language                                                      | Symbols and extracted knowledge in Inspector                                            |
| folder/archive | Compact file-list object                                        | Name plus item count/size                                                   | Full contents in details/Inspector                                                      |
| object         | Structured knowledge object                                     | Type and status are semantic labels                                         | Properties and relations in Inspector                                                   |
| credential     | Fixed secure utility row                                        | Configuration state only                                                    | Secret value never appears on canvas                                                    |
| portal         | Fixed navigation utility row                                    | Target Space, Saved View or node and navigation affordance                  | Target details outside the node                                                         |
| section        | Spatial grouping container                                      | Compact title strip                                                         | Member content remains independent                                                      |

Canvas nodes display only information required for recognition and direct manipulation. Rich analysis is contextual detail, not extra card content. A transcript becoming ready, chapters being generated or OCR completing must not append a new region, create blank space or change the object's position or size.

Document resources belong to selection detail. The Inspector presents deduplicated visual resources as a compact
thumbnail grid; selecting a thumbnail opens the shared dialog viewer with every occurrence, source location,
nearby context and optional analysis. Office resources are identified as extracted embedded images. PDF resources
are identified as rendered page regions or recognition pages and are never described as original embedded files.
Resource visibility depends on document parsing and native rendering, not on AI configuration.

## Window and shell layout

Knovum is one desktop knowledge-space application. Its stable composition is:

```text
OS window
└── Knovum Knowledge Space
    ├── Unified Toolbar
    │   ├── workspace navigation controls
    │   ├── active Space controls
    │   └── task / inspector controls
    └── Knowledge Space Split View
        ├── optional Feature Sidebar
        ├── Workspace
        └── optional contextual Inspector
```

The toolbar is part of the knowledge-space workflow. It contains Space navigation, canvas actions,
task progress and contextual controls; it is not an application switcher. Settings temporarily
replaces the workspace and returns to the last knowledge-space route.

In the Spaces app, one Space is one knowledge context and one infinite canvas; there is no intermediate Board entity. The single leading sidebar switches between the Spaces Browser and the current Space Navigator; these views never appear as adjacent columns. The Browser contains Inbox and one active Spaces list ordered by most recently opened. The Navigator contains Outline and Scenes, preserves the sidebar width, and provides an explicit return to the Browser. Recents and Favorites are not separate user-facing concepts. Archived Spaces leave the workspace sidebar and are managed from Settings, where they can be restored or permanently deleted; there is no second Recently Deleted stage for Spaces. Tags are classification/filtering, not a peer view mode. The trailing Inspector is optional selection detail, defaults closed, and never opens merely because selection changed. High-frequency selection actions stay in the contextual canvas selection bar.

The default canvas layout shows the Spaces Sidebar and canvas. Navigator replaces the Browser within that same resizable sidebar rather than opening another pane. Inspector starts collapsed and yields before the leading sidebar when width becomes constrained.

There is no permanent global status bar. Task progress appears contextually in the top toolbar;
offline state appears only when offline. Toolbar controls are limited to knowledge-space or truly
application-wide commands. The command palette remains available from the application menu and
`Command/Ctrl+K`, but it has no misleading search icon in the toolbar. Content search states its
scope, such as “Find in Space.” Global content search is available through `Command/Ctrl+Shift+F` and the
command palette instead of a second persistent magnifying-glass button in the sidebar. Import adds content
to the active Space and belongs in the Add menu. Space package import/export/backup are contextual data
operations exposed through the Space overflow menu; they are not application preferences.

The sidebar and inspector persist one versioned Spaces layout. Settings does not read or write that splitter state.

The splitter persistence key is versioned with the pane topology. A shell redesign that changes which pane owns titlebar chrome must advance that version instead of replaying stale collapsed/width state into a new layout contract.

On macOS, the unified toolbar owns the entire traffic-light band. Its leading controls begin after one native-control exclusion zone and never jump when the Feature Sidebar opens or closes. All splitters begin below the toolbar, so neither their lines nor resize hit targets cross the native controls.

The unified toolbar, leading sidebar boundary and Inspector header use the same window hairline token. Do not combine an inset toolbar shadow with independent panel borders; the shell must read as one aligned split view.

### Platform-owned window chrome

Share hierarchy and command semantics across platforms; do not force identical titlebar geometry.

- **macOS:** keep native traffic lights and the native overlay titlebar. Never hand-place or redraw the traffic lights. The shell owns an exclusion zone derived from whether leading application navigation is actually visible; when a Feature Sidebar collapses, toolbar content remains outside the native control region. Empty chrome can use `data-tauri-drag-region`.
- **Windows:** retain native caption buttons and Windows window behavior. macOS traffic-light padding must not leak into Windows geometry; content may reveal Mica where supported.

Interactive WebView controls remain ordinary controls, not drag regions. Feature code must not guess window-button coordinates.

## Interaction

- search fields embedded in navigation surfaces use `bg-raised` with one quiet border; they read as a light desktop control rather than a gray browser form field;
- text fields, search fields, textareas and pop-up buttons use one visible boundary only: a neutral border at rest and the `border-focus-ring` color when focused. Do not stack a border, control shadow, focus ring and descendant outline on the same control;
- adjacent peer navigation rows use a 2px vertical gap. Selected and hover backgrounds must never merge into one continuous block;
- all scrolling surfaces read the single `--knovum-scrollbar-size` token and use the shared thin overlay-scrollbar profile: transparent track, rounded thumb, quiet resting contrast and stronger hover contrast. `UiScrollArea` is preferred for normal product regions; unavoidable overflow surfaces inherit the same global profile;
- command palette is the global command discovery surface;
- application shortcuts do not steal OS text-editing keys from inputs, textareas, selects or editable content unless a command explicitly declares itself safe in editable contexts;
- frequent macOS commands also exist in the macOS application menu;
- `UiKbd` renders platform-correct modifiers;
- icon-only actions require an accessible label and tooltip when the icon is not self-evident;
- right-click context menus on non-editable desktop workspace surfaces use Tauri's native `Menu.popup()` path, not a WebView Context Menu component; this rule covers canvas objects/background, Spaces, Outline, Scenes, Tags and Relations. Editable text is not intercepted so the platform text-editing menu remains available;
- transient outcomes use `vue-sonner`; decisions use Reka Dialog/AlertDialog;
- background jobs use the Task Center bridge and OS progress;
- reduced-motion and forced-colors are handled globally;
- content is selectable by default; shell/navigation chrome opts out with `select-none`.

## UI primitives

Reka UI owns behavior: focus, keyboard navigation, ARIA, portals, collision and primitive state. `shared/ui` owns only durable Knovum visual compositions.

Do not wrap Reka merely to rename props/events. For richer WebView-owned structures such as Tree, Dropdown Menu, Combobox and Toolbar, compose Reka directly until a repeated Knovum visual composition is proven. Right-click desktop context menus are platform-owned through Tauri Menu and are not a Reka surface.

Choose component ownership by interaction contract, not markup similarity:

1. Use an existing `shared/ui` component when its meaning, keyboard behavior, state and density match the control. Do not recreate its visual classes inside a feature.
2. Keep a control in its feature when it depends on domain geometry, commands or editing state. Repeated controls within one feature share a feature-owned visual contract while composing Reka behavior directly.
3. Promote a feature control to `shared/ui` only when at least two independent features need the same meaning, behavior and visual treatment. A second use with a different interaction contract is not proof of reuse.

Review desktop controls in their default, hover, pressed, focused, disabled, invalid and busy states. Verify keyboard operation, tooltip and accessible names for icon actions, long localized labels, compact widths and reduced motion. Popup surfaces remain anchored to their control and use collision handling; canvas geometry tools own their placement separately from ordinary trigger popovers.

Canvas selection surfaces declare their interaction kind: ordinary actions, compact editing controls, or a sustained inspector. Actions and compact controls attach above or below the selected object; inspectors attach to its left or right. The canvas selection anchor owns placement for every node type. It reads the current selection and visible canvas objects itself, excluding selected and hidden objects, so individual node tools only declare their interaction kind and spacing. Placement considers the safe canvas viewport and nearby objects; a clear position on the preferred axis wins, followed by a clear position on the other axis. When every adjacent position intersects another object, choose the position with the least overlap and keep editing controls scrollable within the available canvas space. A panel never changes ownership or becomes a trailing sidebar merely because its anchor is crowded.

## Native materials

The current macOS visual profile uses Tauri's transparent WebView + `sidebar` window effect. This requires `app.macOSPrivateApi: true`; Tauri documents that this prevents Mac App Store acceptance. Knovum currently optimizes for direct/team distribution. If Mac App Store distribution becomes a requirement, record a new ADR and remove the private-material profile rather than silently weakening it.

Windows uses the official Mica window effect and WebView2 Fluent overlay scrollbars when available. No third-party vibrancy package is used.
