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

Content rendering failures use a compact, borderless state inside the affected work area with
a neutral icon, a short explanation and an explicit reopen action. Navigation and unrelated
controls remain available. Do not replace the whole workbench with a diagnostic card or expose
technical stacks. Expected operation errors stay with the initiating control or Task Center.

## Surface hierarchy

The visual system has four layers:

1. **Window / navigation chrome** — the unified toolbar and feature sidebar share one quiet opaque plane.
2. **Content** — document, canvas, diff, conversation and package documentation. Content uses an opaque `bg-surface-canvas` surface.
3. **Elevated** — menus, popovers, command palette and transient task UI use `bg-surface-overlay`, a strong hairline boundary and restrained `shadow-popover` separation.
4. **Modal** — dialogs use `bg-surface-overlay` + `shadow-dialog` and a restrained scrim.

Elevation belongs primarily to floating layers. Space canvas objects use only their dedicated micro-elevation:
a short contact shadow that separates movable content from the infinite canvas without making it read as a
popover. They never borrow panel, popover or dialog elevation.

Do not use blur or glass effects inside normal content cards. Native material belongs to navigation/control layers.

## Tailwind CSS v4 contract

Tailwind supplies spacing, fixed dimensions, default radii, font weights and ordinary durations.
`src/styles/tokens.css` extends this foundation only for authored product decisions. Public tokens live
once in native `@theme static` namespaces. CSS, utility classes and canvas readers consume the same
variables; there is no parallel `--knovum-color-*` / typography API or inline adapter. Static emission
keeps required variables available even when a utility is not used. Mode selectors override those names.

### Token architecture and naming

DTCG standardizes token types and interchange, not one mandatory naming vocabulary. The project uses
lowercase kebab-case and Tailwind's standard namespaces. Role names below are the product contract.

| Layer                         | Grammar                                              | Example                               | Consumer                                      |
| ----------------------------- | ---------------------------------------------------- | ------------------------------------- | --------------------------------------------- |
| Public surface                | `--color-surface-{role}[-{emphasis}][-{state}]`      | `--color-surface-danger-subtle-hover` | `bg-surface-danger-subtle-hover`              |
| Public foreground             | `--color-fg-{role}`                                  | `--color-fg-secondary`                | `text-fg-secondary`                           |
| Public boundary               | `--color-stroke-{role}[-{state}]`                    | `--color-stroke-default-hover`        | `border-stroke-default-hover`, rings/outlines |
| Public typography             | `--text-{role}[-{size}]` and `--line-height` suffix  | `--text-body--line-height`            | text utilities and CSS                        |
| Public shape/elevation/motion | native radius/shadow/ease/blur namespaces            | `--shadow-popover`                    | utilities and CSS                             |
| Component                     | `--knovum-{component}[-{part}]-{property}[-{state}]` | `--knovum-sidebar-item-bg-selected`   | owning composition                            |
| Domain                        | `--knovum-space-{object}-{property}[-{variant}]`     | `--knovum-space-grid-color`           | Spaces only                                   |

`fg` means foreground for text/icons; `bg` means background in private component/domain attributes.
`stroke` covers boundaries, SVG strokes, outlines and focus rings. These abbreviations are explicit and
consistent; do not introduce synonyms or repeat category words. Preserve owner and role when they
carry meaning. A CSS token need not repeat the property already encoded by a framework namespace.

Tailwind owns the base color palette. Use native --color-{hue}-{shade} variables; do not create a
parallel product reference palette. Override native values only for an explicit product-wide adjustment.
Shared --color-content-{hue} roles serve user-selected text, relation strokes and annotations (600 light,
400 dark). --color-content-{hue}-subtle uses the native 200 shade for colored backgrounds and highlights
in both appearances. Notes and grouping fills share these backgrounds. Highlighted text is black on
these light fills in either appearance. Content colors carry no status
meaning; UI accent/danger/warning roles remain separate decisions sourced from the same base palette.

States mean the named state: `hover` is pointer hover, `pressed` is transient active press, `selected` is
persistent selection and `inactive` is the explicit inactive component role. Open menus and recent-item
feedback do not consume pressed paint. Neutral control paint increases from rest to hover to pressed in
both appearances. Ghost controls start transparent and use the same hover/pressed roles. Primary actions
use accent-hover/accent-pressed. Solid destructive actions retain their documented opacity treatment.
Do not create a complete state matrix where no consumer needs it.

Status foreground, boundary and background decisions remain independent. Repeated weak warning surfaces
use `surface-warning-subtle`, and their interactive hover uses `surface-warning-subtle-hover`; consumers
must not reimplement these roles as a solid warning hue with an arbitrary percentage. Other opacity
modifiers remain valid for instance-specific composition, not as substitutes for an existing role.

Typography uses size/line-height pairs: caption, label, body, title and display are each role's default;
caption-sm, body-lg and title-sm are the only alternate sizes. Add a size suffix only for a real
alternate style, not to complete a hypothetical sm/md/lg matrix. Text-size modes override both native
variables without changing the role names. Label and body differ in leading even when size matches.
Use title-sm for compact section, dialog and tool headings; body-lg belongs to reading content and
larger descriptive text. HTML heading level describes document structure, not a font-size slot.
Native font/tracking tokens contain only authored deviations from Tailwind.

Use `size-3`, `size-3.5`, `size-4`, `size-4.5`, `size-5` for 12–20px icons. Use Tailwind spacing/container
widths for fixed geometry and `duration-100`, `duration-150`, `duration-220` for ordinary motion.
Density-sensitive controls and native platform exclusion retain product variables; shared interaction and toolbar heights use native spacing extensions. Tags use the same interaction height as other single-line navigation. Settings entries share `--knovum-setting-min-height` (52px compact / 56px comfortable), retain py-3, and grow when content needs more room.
Use rounded-sm for grids, rounded-md for controls, rounded-lg for canvas objects and rounded-xl for
workspaces/dialogs/outer sections. Only 10px panels and 9px inner sections extend the native radius scale.
Fixed switch thumbs use bg-white and video-label scrims use bg-black/45 in their owning component.

Aliases require a deliberate semantic relationship: inverse surfaces pair with foreground roles;
business colors never borrow a status meaning merely to reuse a hue. Note text is always native black on colored paper, while free canvas text uses the normal foreground.
Notes allow background selection, not node-level or selection-level text colors. Their editor omits the
Color extension, and both editing and preview render without inline color support. Font size, bold,
italic, underline, lists and alignment remain available.
Keep private feature attributes out of global utility namespaces. Delete unused or redundant declarations;
do not retain legacy aliases, fallback palettes or parallel theme APIs.

Mode selectors are orthogonal: appearance controls paint, text size controls typography, density controls
geometry, platform controls native-window exclusion, window activation softens interaction, and
accessibility media queries adjust contrast/transparency. Required canvas paint fails visibly if missing.

```html
<div class="bg-surface-canvas text-body text-fg-primary">
  <button class="h-(--knovum-control-height-md) rounded-md bg-surface-accent text-fg-on-accent">...</button>
</div>
```

Prefer semantic utility classes over arbitrary references when a public utility exists. Arbitrary values
are reserved for instance geometry or primitive-owned runtime values. Components own their visual
composition. Scoped CSS must not create aliases through @apply; genuine scoped theme composition
requires an explicit @reference to the main stylesheet.

The contract uses [Tailwind's native theme mechanism](https://tailwindcss.com/docs/theme), with semantic
role separation informed by [DTCG](https://www.designtokens.org/TR/2025.10/format/).

## Color roles

| Utility                                       | Role                                                   |
| --------------------------------------------- | ------------------------------------------------------ |
| `bg-surface-chrome`                           | unified window, sidebar and toolbar plane              |
| `bg-surface-canvas`                           | opaque primary content                                 |
| `bg-surface-muted`                            | recessed previews, embedded content and quiet fields   |
| `bg-surface-panel`                            | structural content panels, sidebars and canvas objects |
| `bg-surface-raised`                           | raised fields, compact controls and elevated content   |
| `bg-surface-overlay`                          | floating menus, popovers and overlay chrome            |
| `bg-surface-inverse` / `text-fg-inverse`      | contrasting action/badge surface pair                  |
| `bg-surface-document`                         | appearance-independent white HTML document             |
| `text-fg-on-image` / `border-stroke-on-image` | white media-overlay contrast                           |
| `bg-(--knovum-sidebar-item-bg-hover)`         | navigation hover                                       |
| `bg-(--knovum-sidebar-item-bg-selected)`      | navigation selection                                   |
| `border-stroke-divider`                       | rare structural separator                              |
| `border-(--knovum-window-border-color)`       | crisp window/workspace edge                            |
| `text-fg-primary`                             | primary text                                           |
| `text-fg-secondary`                           | secondary text                                         |
| `text-fg-tertiary`                            | tertiary metadata                                      |
| `text-fg-accent` / `bg-surface-accent`        | restrained primary accent                              |
| `--knovum-space-measurement-color`            | canvas measurements and spacing guides                 |
| `border-stroke-default-hover`                 | one deeper object boundary on hover                    |

Surface selection is structural: canvas is the base content plane; chrome owns desktop navigation;
panel owns content-bearing structural regions; muted owns recessed/embedded regions; raised owns small
fields and elevated content; overlay owns transient floating layers. A text input may use muted when
embedded in a recessed region or raised when it needs a distinct field plane. Select by context, not by
which neutral happens to look similar. Canvas overview rendering uses the same panel role as live objects.

Features must not invent their own gray palette or generic accent palette. Measurement guides use their own
token because a span that reads as selection blue or destructive red would misstate what the drag is doing.
`accent` marks primary interaction; `danger` covers destructive actions and failures; `success` marks positive
outcomes; `warning` marks caution. Subtle backgrounds are supporting tints, never foreground colors.

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

- transient sidebar: horizontal transform over 180ms and opacity over 150ms in both directions;
- scrim: opacity only, 150ms enter and exit;
- view swaps: at most 150ms with a 4px directional offset;
- controls: 100–150ms paint transitions; toolbar controls never change geometry when pressed;
- command-initiated panel visibility animates the split width and matching toolbar region over 180ms; direct
  splitter dragging and window resizing track their actual dimensions without interpolation. The canvas stage
  uses a bounded size step around its visible host to reduce backing reallocations during resize;
- the pinned left sidebar stays inside its splitter panel; only its temporary preview overlays the canvas.
  The sidebar content keeps its chosen width while the panel reveals it, so labels do not reflow through narrow
  intermediate widths. The canvas keeps the same left inset and rounded corner in both states; with the sidebar
  pinned, workspace tabs align to that inset;
- never animate canvas-object geometry or other layout dimensions, and never interpolate a persisted
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

| Utility           | Default | Use                                                    |
| ----------------- | ------: | ------------------------------------------------------ |
| `text-caption-sm` |    11px | keyboard hints/debug metadata only                     |
| `text-caption`    |    12px | secondary labels and descriptions                      |
| `text-label`      |    13px | controls, rows and navigation                          |
| `text-body`       |    13px | compact body content                                   |
| `text-body-lg`    |    14px | reading content and larger descriptive text            |
| `text-title-sm`   |    15px | subview/section title                                  |
| `text-title`      |    17px | workspace/settings title                               |
| `text-display`    |    24px | settings root, onboarding and exceptional empty states |

`Settings -> Text size` changes the values behind these roles. Features continue using the same utilities.

## Geometry and density

Global roles include:

- `h-(--knovum-control-height-sm)`, `h-(--knovum-control-height-md)`, `h-(--knovum-control-height-lg)`;
- `h-(--knovum-titlebar-height)` for integrated titlebar/toolbar geometry;
- `h-interaction` / `min-h-interaction` for menu candidates, sidebar entries, trees and outlines;
- `h-toolbar` for editor toolbars and task-center headers;
- `min-h-(--knovum-setting-min-height)` for preference fields, connection summaries and model installation settings;
- `size-3` through `size-5` for symbols. Toolbar symbols use `size-4.5` (18px) inside 32px controls.

Menu options, selection candidates and navigation share `h-interaction`: 30px compact / 34px
comfortable. This is one product density decision, not independent component knobs. Use `min-h-interaction`
for entries that may wrap. Editor toolbars and task-center headers share `h-toolbar`: 44px compact / 48px
comfortable. Command-palette options retain their richer, independently owned layout. Collapsible headings
use content line-height plus py-2, without a fixed interaction height.

`data-platform` changes titlebar/chrome geometry where the operating systems differ. `data-density` changes interaction density without changing typography hierarchy. `--knovum-titlebar-height` is the stable native titlebar band and intentionally does not shrink with content density; density-sensitive in-workspace toolbars use their own toolbar/control geometry.
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
directly. Space context menus own Space management; a compact toolbar utility control owns Task Center
presentation. Neither tasks nor document actions are appended to the tab strip.
This keeps application preferences, space navigation and document/object navigation distinct without
presenting the product as a collection of unrelated tools.

One unified 40px toolbar spans the window above the workspace split. Its empty area is the Tauri drag target;
interactive controls alone receive pointer input. A subtle highlight separates the chrome from the inset work area.
Resizing the panes updates the corresponding toolbar regions from the splitter's actual layout without animation.

The leading toolbar region shares the left navigation panel's width and keeps navigation and history controls in
place when the sidebar is previewed. With the sidebar pinned, Space tabs align with the main work surface's
leading edge. Leading and right chrome actions follow the established compact control size. The right pane's
view picker is part of its content header rather than the window toolbar.

The canvas and right pane each own an inset, rounded surface with a border and the same bottom gutter. The
right pane's view picker, content header, body and composer live inside that one surface. The window toolbar
remains a separate native-chrome plane above both surfaces. During expansion, the pane reveals right-anchored
content at its preferred width instead of compressing the content.

The Spaces sidebar uses a consistent 224px default width in both pinned and transient-preview states. A
pinned sidebar remains resizable between 200px and 320px. Its resize feedback is painted on the workspace
surface boundary, never inside the navigation background or the window gutter. Closing it preserves the
navigation affordance and hover preview; it is application navigation, not a context panel. The right workspace sidebar is a
resizable pane in the workspace split; it keeps the canvas visible instead of covering it. One compact header
contains the current view picker, conversation or content title, and local actions. The picker menu can add views
without compressing that header. The visibility toggle stays at the window toolbar's top-right corner in both states.
Agent conversation provides task input, attachments, target and extension selection without a
mode selector. A conversation can move from discussion to execution through user messages.
Task lists are shown only when the Agent uses SDK Todo; simple answers need no plan or approval
step. Extension choices follow configured tool policies, not an inferred conversation intent.

That toggle opens Conversation as the default view. Explicit object actions open Details;
the document editor can open its heading Outline. Source opens as a temporary drill-in and returns to its originating
view. Selection alone never changes the active view. The right pane does not change the leading sidebar width.
Closing it releases workspace width and suspends its content until reopened; it has no hover preview.
An empty Conversation offers short prompts that fill and focus the composer without sending. Composer
references stay next to the text they accompany: canvas references identify their Space source and can
preview visual content, while local files retain a separate file identity. Both can be removed before sending.

The Details view uses a content-first hierarchy: editable identity and direct actions, the
content preview and type-specific details, derivation and knowledge relationships, related content, then
collapsible file metadata. Related content is discovered from the current Space's semantic index rather than
persisted as a relation; selecting a result reveals it on the canvas without creating a line. The UI separates
unindexed content, indexing, no matches and retrieval failure. It does not display synthetic similarity
percentages. Details uses the sidebar's scrolling area rather than a settings-page layout.

The primary workspace surface is inset 8px from the window bottom and right edge, and from the left edge when
navigation is collapsed. A single outer frame owns its quiet border, radius and shallow elevation. The canvas
and right context pane split inside that frame; the context pane draws only their internal hairline. Hover
preview remains an overlay below the toolbar and does not change split geometry.

### Space canvas object geometry

Space canvas objects use **semantic desktop geometry**, never one universal card size. `src/features/spaces/canvas/node-size.ts` owns provisional size and resize bounds; Rust's `src-tauri/src/spaces/nodes/content_size.rs` determines persisted image, video and document dimensions from intrinsic content within the content write transaction. Node components must not invent independent min/max values.

Canvas nodes are desktop objects, not floating web cards. Every movable content node uses one opaque surface, an 8px object radius, one quiet hairline and the shared `--knovum-space-node-box-shadow` elevation. That elevation combines a tight contact shadow, a short downward cast and one low-opacity ambient layer, creating the grounded paper-on-canvas depth used by spatial desktop tools without borrowing popover height. It remains identical at rest, hover, selection and drag, so interaction never produces a web-card lift. Hover may change only paint properties such as background or border color; the canvas object's one hover step is its boundary hairline deepening to `border-stroke-default-hover` at an unchanged width, and it must never change geometry, transform, font weight, padding or icon-box dimensions. Ordinary object selection does not draw a rectangular accent outline: the contextual toolbar and capability-specific resize handles identify the active object. Transparent Text is the deliberate exception and receives the minimal boundary treatment defined below. Sections remain spatial containers and therefore have neither card surface nor object shadow. Type-specific meaning belongs in the icon, label and content—not colored card chrome.

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
- the content surface never changes geometry, border width, font weight, transform or elevation for hover, focus, selection or drag; its stable micro-elevation comes only from `shadow-(--knovum-space-node-box-shadow)`, and its one hover change is the boundary `border-color`;
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
8. In Select mode, an unmodified left drag on empty canvas draws a partial-intersection rubber band, and a plain click clears the selection. Pan mode makes left drags pan, including over knowledge nodes; the middle button, held Space and two-finger scrolling also pan through the same viewport controller. Pinch zooms. The top-center compact tool group exposes Select, Pan, Note and Text; Outline belongs to the sidebar, Space management belongs to native context menus, and zoom and Saved Views remain bottom-left. The shell-owned Task Center stays in the unified toolbar beside the right sidebar toggle and opens a global task list with a current-Space filter. Each node-owned task identifies its Space and node and can open that node. Node-local acquisition and enrichment state remains on the canvas. Placement tools use a crosshair and return to Select after creating empty inline-editable knowledge at the clicked world coordinate. Selection chrome and content controls are inactive during Pan or placement. The canvas engine's own delete key stays disabled so deletion remains a Knovum command.

9. Empty-canvas cursors state the current meaning: arrow at rest, crosshair only while the rubber band is being drawn, grabbing while the viewport pans. Objects keep grab and grabbing.
10. Dragging reports its own alignment. The moving frame snaps to nearby object anchors and to the center of the surrounding content bounds, and one hairline marks the anchor it caught. When the gaps on both sides of the moving frame are equal, end-capped measurement spans carrying their shared distance replace that hairline. Snap and measurement share one constant screen-space tolerance, so the same drag feels identical at every zoom level; all guides clear when the drag ends and none of them is persisted geometry.

A single selected content node or Section shows one screen-stable action bar centered on the object's bounds and clamped to the viewport, above its bounds when there is room and below otherwise. Its opaque overlay has the existing panel radius, neutral hairline, compact label type and restrained control elevation. The bar measures the available viewport width: primary actions use icon and label while they fit, then transition to icon-only controls with accessible names and hover titles. Type-specific formatting, image tools and duplication remain visible, while structural commands and derivations remain under More. The bar clears the full pointer area of a selected node's connector controls, not merely the object edge. The final segment after More contains the universal icon-only **Details** and **Download** controls, separated by a full-height hairline. Download opens the native save dialog and writes that node's supported representation to a local file. Edited images download as the rendered PNG, including crop, rotation, flip and tonal adjustments. A Section downloads a structured outline, not an asset package; credential downloads contain non-secret metadata only. The bar is selection chrome, so unselected nodes never carry persistent download or overflow buttons. Multi-selection keeps its separate compact structural toolbar.

The image action bar keeps direct image actions visible. Adjust, crop and flip/rotate use one temporary edit session: entering a tool replaces the action bar with that tool's panel, changes preview on the node, Apply writes one history entry, and Cancel discards the draft. AI object removal publishes a separate derived image: Qwen and GPT Image models use a temporary brush mask; supported Gemini, Grok Imagine, Doubao Seedream, SiliconFlow Qwen edit, Kling Omni and OpenRouter image models use an object description. Cancel discards the draft without changing the source node. Comment mode likewise occupies the action area; upscale, multi-angle, layer-splitting and prompt-removal popovers have one shared open state so only one is open at a time. Crop exposes a dimmed outside area, direct drag handles, and drag-to-draw selection over the full source image. Its side panel groups W/aspect lock/H, collapsible, single-row general and social presets with aspect glyphs and reference dimensions, and Cancel/Done. The aspect lock is one session value shared by numeric fields and canvas handles. Adjust uses one scrollable Light, Color, Detail and Effects list; the four icons scroll to sections and track the visible section. The strict v6 edit contract contains the Light, Color, Detail and Effects controls, and Vue preview and Rust output both render them. Unsupported generation actions remain inspectable; only their execution controls are disabled. Active brush-removal and crop sessions enable direct pointer interaction on the selected image. The transform mode uses a compact replacement toolbar. The brush-removal panel keeps undo in the header, brush size in a neutral slider, and Generate disabled until a mask stroke exists and the image provider is ready. The multi-angle popover offers subject/camera modes, a draggable angle guide, rotation, tilt and three framing levels. Its thumbnail renders the managed original with the committed `imageEdit`, matching the input Rust later materializes. The guide is an interaction aid, never a generated result. Its controls become bounded model instructions in Rust. Layer splitting offers automatic four-layer output or a custom layer count and optional separation guidance; the UI never quotes an invented credit cost. The WebView owns interactive preview only. Rust reads the authorized managed asset and applies the strict `imageEdit` contract for downloaded PNGs, local background removal and cloud inputs, then validates and publishes output. Upscale, multi-angle and AI object removal create derived nodes through the configured image provider. Layer splitting creates independent transparent image nodes only when its dedicated provider credential is configured. Unavailable cloud capabilities state the missing configuration instead of presenting a successful no-op.

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

Rendered relations use the Konva adapter's relation projection. Endpoint geometry remains adapter-owned canvas state and is never copied into the semantic relation model; the Space domain persists only relation identity, kind, status and provenance. User-selected relation colors are sparse canvas-presentation overrides backed by the shared `--color-content-*` roles. Neutral is represented only by the absence of an override. Parallel semantic relations between the same visible endpoints use stable, separated curve lanes so their labels and hit targets remain distinct.

### Space canvas object layouts

Audio and video use one media-session lifecycle. A natural playback completion unloads the active
transport, resets its timeline and returns the node to its preview representation; replay is an
explicit new session that starts from the beginning.

Each node layout represents the object's job instead of reusing a universal header/body/footer card:

| Kind           | Primary canvas representation                                   | Persistent chrome                                                          | Enrichment placement                                                                    |
| -------------- | --------------------------------------------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| image          | The image itself, fully visible within intrinsic-ratio geometry | None; name and metadata belong to selection/Inspector                      | Visual description, OCR and tags in Inspector                                           |
| video          | Fixed-ratio media viewport with centered play affordance        | None; transport overlays the viewport, name and metadata live in Inspector | Transcript, summary and chapters in Inspector; chapter marks may appear on the timeline |
| audio          | Compact horizontal waveform and transport                       | One title row; no empty preview region                                     | Transcript and chapters in Inspector                                                    |
| file/PDF       | Document thumbnail or readable first-page preview               | One title row; never repeat the filename as subtitle                       | Extracted text and document structure in Inspector                                      |
| note           | Direct editable note surface                                    | No file-style header; title is the first content role                      | AI/knowledge metadata in Inspector                                                      |
| text           | Transparent free-standing canvas typography                     | None; selection alone reveals typography controls                          | None                                                                                    |
| markdown       | Reading/editing surface with document typography                | Compact document title, not a media/file card footer                       | Outline and generated knowledge in Inspector                                            |
| website        | Bookmark preview with favicon, title and domain                 | Domain is secondary identity; URL is not repeated in a footer              | Extracted sections in Inspector                                                         |
| code           | Monospaced source preview                                       | Filename plus language                                                     | Symbols and extracted knowledge in Inspector                                            |
| folder/archive | Compact file-list object                                        | Name plus item count/size                                                  | Full contents in details/Inspector                                                      |
| object         | Structured knowledge object                                     | Type and status are semantic labels                                        | Properties and relations in Inspector                                                   |
| credential     | Fixed secure utility row                                        | Configuration state only                                                   | Secret value never appears on canvas                                                    |
| portal         | Fixed navigation utility row                                    | Target Space, Saved View or node and navigation affordance                 | Target details outside the node                                                         |
| section        | Spatial grouping container                                      | Compact title strip                                                        | Member content remains independent                                                      |

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
    │   └── task / right sidebar controls
    └── Knowledge Space Split View
        ├── optional Feature Sidebar
        ├── Workspace
        └── optional resizable right sidebar
```

The toolbar is part of the knowledge-space workflow. It contains Space navigation, canvas actions,
task progress and contextual controls; it is not an application switcher. Settings temporarily
replaces the workspace and returns to the last knowledge-space route.

In the Spaces app, one Space is one knowledge context and one infinite canvas; there is no intermediate Board entity. The leading sidebar owns Space navigation and the canvas structure Outline; the latter selects and reveals canvas objects. The current document's heading Outline belongs to the right sidebar because it navigates only that document. Recents and Favorites are not separate user-facing concepts. Archived Spaces leave the workspace sidebar and are managed from Settings, where they can be restored or permanently deleted; there is no second Recently Deleted stage for Spaces. Tags are classification/filtering, not a peer view mode. The right sidebar defaults closed and never opens merely because selection changed. High-frequency selection actions stay in the contextual canvas selection bar.

The default canvas layout shows the Spaces Sidebar and canvas. The right sidebar starts closed. Its Conversation, Details and document Outline views share one pane; contextual image and workflow edit panels remain anchored to their canvas objects.

There is no permanent global status bar. Task progress appears contextually in the top toolbar;
offline state appears only when offline. Toolbar controls are limited to knowledge-space or truly
application-wide commands. The command palette remains available from the application menu and
`Command/Ctrl+K`, but it has no misleading search icon in the toolbar. Content search states its
scope, such as “Find in Space.” Global content search is available through `Command/Ctrl+Shift+F` and the
command palette instead of a second persistent magnifying-glass button in the sidebar. Import adds content
to the active Space and belongs in the Add menu. Space package import/export/backup are contextual data
operations exposed through the Space overflow menu; they are not application preferences.

The leading and right sidebar widths remain stable while the workspace is open. Settings does not control these panes.

On macOS, the unified toolbar owns the entire traffic-light band. Its leading controls begin after one native-control exclusion zone and never jump when the Feature Sidebar opens or closes. All splitters begin below the toolbar, so neither their lines nor resize hit targets cross the native controls.

The window chrome and the inset workspace surfaces define separate depth planes. Canvas and right pane retain
their own rounded borders and a shared bottom inset, with a narrow gutter between them.

### Platform-owned window chrome

Share hierarchy and command semantics across platforms; do not force identical titlebar geometry.

- **macOS:** keep native traffic lights and the native overlay titlebar. Never hand-place or redraw the traffic lights. The shell owns an exclusion zone derived from whether leading application navigation is actually visible; when a Feature Sidebar collapses, toolbar content remains outside the native control region. Empty chrome can use `data-tauri-drag-region`.
- **Windows:** retain native caption buttons and Windows window behavior. macOS traffic-light padding must not leak into Windows geometry; content may reveal Mica where supported.

Interactive WebView controls remain ordinary controls, not drag regions. Feature code must not guess window-button coordinates.

## Interaction

- search fields embedded in navigation surfaces use `bg-surface-raised` with one quiet border; they read as a light desktop control rather than a gray browser form field;
- text fields, search fields, textareas and pop-up buttons use one visible boundary only: a neutral border at rest and the `border-stroke-focus` color when focused. Do not stack a border, control shadow, focus ring and descendant outline on the same control;
- adjacent peer navigation rows use a 2px vertical gap. Selected and hover backgrounds must never merge into one continuous block;
- all scrolling surfaces read the single `--knovum-scrollbar-size` token and use the shared thin overlay-scrollbar profile: transparent track, rounded thumb, quiet resting contrast and stronger hover contrast. `UiScrollArea` is preferred for normal product regions; unavoidable overflow surfaces inherit the same global profile;
- command palette is the global command discovery surface;
- application shortcuts do not steal OS text-editing keys from inputs, textareas, selects or editable content unless a command explicitly declares itself safe in editable contexts;
- frequent macOS commands also exist in the macOS application menu;
- `UiKbd` renders platform-correct modifiers;
- icon-only actions require an accessible label and tooltip when the icon is not self-evident;
- right-click context menus on non-editable desktop workspace surfaces use Tauri's native `Menu.popup()` path, not a WebView Context Menu component; this rule covers canvas objects/background, Spaces, Outline, Scenes, Tags and Relations. Editable text is not intercepted so the platform text-editing menu remains available;
- transient outcomes use `vue-sonner`; destructive and discard confirmations use the shared in-app `UiConfirmAction` dialog, while file picking and saving use the native OS dialog;
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
Popover and dropdown triggers must own their positioning anchor directly. Do not nest a Tooltip trigger around a popup trigger or enable `UiIconButton`'s Tooltip inside one: nested positioning primitives can prevent the popup from anchoring to its button and move it to a viewport corner. Icon-only popup triggers retain an accessible label and may use a title hint when needed.

Canvas selection chrome has separate geometric contracts. A short action bar attaches above the selected object's visible bounds, or below when the upper position does not fit. It can temporarily cover unrelated canvas objects; their positions never influence its placement. The canvas selection anchor uses only the projected selection, fixed chrome bounds, viewport bounds and the measured surface size. It keeps the bar centered on a visible part of the selection, clamps it to the safe viewport and preserves its current side near a flip threshold. If neither exterior side fits, the bar attaches inside the selected visible edge; it never falls back to an unrelated screen corner. A relation uses its projected curve midpoint, including its zero-size point bounds. For a wide multi-selection whose box center is empty, use an actual visible member as the anchor. Triggered menus and popovers stay attached to their buttons; sustained edit panels prefer the selected object's sides and scroll internally when space is limited. Measure their unconstrained inner content independently of the outer scroll viewport so switching modes recalculates placement and height instead of retaining the previous limit. Direct object manipulation hides the action bar until the gesture ends. A panel never becomes a trailing sidebar merely because its anchor is crowded.

Node action bars measure their expanded and icon-only widths before display. The shortest desktop work area keeps the "More" trigger and common inspect/download tail reachable. When icon-only image actions exceed the available width, original-image editing and commenting move into "More" together; generation popovers remain on their visible triggers so their anchors stay valid. The generic node bar moves its primary action and video editing into "More" at the same density boundary. Multi-selection and relations use compact icon controls and expose their secondary actions in the shared selection menu. These changes preserve each action's meaning and order; a hidden duplicate must not remain in both the bar and its menu.

AI image tools separate local input preparation from remote execution. Opening the eraser, painting a mask, changing brush size or entering a removal description requires a ready image and exclusive node-operation ownership; it does not require model credentials or a successful capability query. Generation confirmation checks the explicitly selected input mode, its actual execution capability and input constraints, and explains why submission is unavailable. Mask and description removal share one draft session and submission surface; switching modes is a user choice and never an automatic fallback. Existing node-operation admission continues to prevent dragging or concurrent edits while either mode is active.

## Native materials

The current macOS visual profile uses Tauri's transparent WebView + `sidebar` window effect. This requires `app.macOSPrivateApi: true`; Tauri documents that this prevents Mac App Store acceptance. Knovum currently optimizes for direct/team distribution. If Mac App Store distribution becomes a requirement, record a new ADR and remove the private-material profile rather than silently weakening it.

Windows uses the official Mica window effect and WebView2 Fluent overlay scrollbars when available. No third-party vibrancy package is used.

## First-use local resources

A user starts a product action, rather than navigating to Settings to install its dependencies.
Transcription and URL media import display resource preparation in the action's existing progress
surface. Semantic search has an explicit activation action: keyword results remain usable while
resources are prepared, and indexing is presented as a separate actual stage before semantic results
are enabled. Switching Spaces during preparation does not replay the old query in the new Space.

Task Center projects Rust-owned preparation snapshots with the same job identity across WebView
reloads. It presents measured bytes for downloads and indeterminate verification/installation;
it does not invent a verification percentage or own installation lifetime. Cancel releases one
request, and retry starts resource preparation without automatically repeating a business mutation.
Settings uses that same preparation path for optional pre-download and model removal; it is a
management surface, not a prerequisite for using a capability. Application launch does not download
optional models or tools.

## Product semantic selection rules

Tokens represent an existing shared policy, not every distinct value or potential future customization.
Prefer Tailwind's primitive scales for ordinary geometry; product density extensions use its native
spacing namespace. Shared UI components own state combinations. New tokens need a concrete role,
consumer boundary and reason an existing rule cannot serve it.

| Surface | Selection rule                                                                                  |
| ------- | ----------------------------------------------------------------------------------------------- |
| canvas  | main document/workspace content plane                                                           |
| chrome  | persistent desktop navigation and toolbar chrome                                                |
| panel   | structural content containers and canvas objects                                                |
| muted   | embedded previews, recessed content and quiet fields; no parallel inset role                    |
| raised  | resting controls and selected tabs that stand above their parent surface                        |
| overlay | transient menus, popovers and dialogs; use material-overlay when translucent material is needed |

Use shadow-popover for small transient panels and shadow-dialog for modal/command-palette depth.
Control hover/press paint is an interaction state, not an alternative container surface. Selection and
focus remain distinct. Native, ARIA and Reka disabled controls use opacity-40; this is a component rule,
not a new color token. Disabled controls keep the desktop default cursor. Informational capability
badges and loading indicators are not disabled controls.

Spaces owns `src/features/spaces/presentation/space-tokens.css`, imported once by app.css so DOM and
runtime canvas paints remain available together. It owns canvas grid and object elevation, including appearance overrides. Shared content colors belong
to the product theme. The remaining feature variables are not exported through @theme and must not be used
outside Spaces. Native Tailwind palette variables are available to token definitions and content composition.

Typography roles remain caption, label, body, title and display, with only existing meaningful size
variants. Keep caption-sm for keyboard hints/debug metadata; use label for operation text, body/body-lg
for reading, and title-sm for compact headings. Do not add size slots to make a symmetrical matrix.
