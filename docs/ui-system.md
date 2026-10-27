# Knovum desktop design system

Audio processing is exposed by the owning canvas node's action bar. Narration/audiobook and video
dubbing editors open from node actions with the source node fixed; the voice a project speaks with
is chosen inside that operation, and nothing else about voices lives there. Voices are created,
inspected and removed in Settings only, so an audio operation that has no voice to offer says so
once and links there rather than carrying a second library. A speech project belongs to its node
and operation; there is no duplicate of it to create or switch between.
Do not duplicate speech generation, voice cloning, source separation or transcription in the selection
inspector. Structural selection actions, such as grouping, belong to the selection action bar.
Do not add an independent speech sidebar destination or a Space-wide video/manuscript selector
for these operations.

Resource settings are organized around models and user capabilities. Python profiles and dependency
installation are application-owned implementation details, without manual environment controls.
The initiating feature prepares dependencies automatically and owns progress/cancellation through
the Task Center. Download-region/source selection belongs to the catalog and publication configuration.

Tailwind class detection is scoped to the frontend `src/` tree through the stylesheet's standard
`source()` option. Native build inputs, packaging evidence and repository tooling are not UI class
sources; they must not affect CSS output or frontend compilation cost.

## Product language

Knovum is a desktop workbench. The target is Apple-like platform sensibility plus Linear-like information density, not a web dashboard and not a visual clone of either product.

The design language is **quiet, layered, platform-aware and keyboard-first**:

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
once in Tailwind `@theme static` namespaces. CSS, utility classes and canvas readers consume the same
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
| Public shape/elevation/motion | Tailwind radius/shadow/ease/blur namespaces          | `--shadow-popover`                    | utilities and CSS                             |
| Component                     | `--knovum-{component}[-{part}]-{property}[-{state}]` | `--knovum-sidebar-item-bg-selected`   | owning composition                            |
| Domain                        | `--knovum-space-{object}-{property}[-{variant}]`     | `--knovum-space-grid-color`           | Spaces only                                   |

`fg` means foreground for text/icons; `bg` means background in private component/domain attributes.
`stroke` covers boundaries, SVG strokes, outlines and focus rings. These abbreviations are explicit and
consistent; do not introduce synonyms or repeat category words. Preserve owner and role when they
carry meaning. A CSS token need not repeat the property already encoded by a framework namespace.

Tailwind owns the base color palette. Use built-in --color-{hue}-{shade} variables; do not create a
parallel product reference palette. Override built-in values only for an explicit product-wide adjustment.
Shared --color-content-{hue} roles serve user-selected text, relation strokes and annotations (600 light,
400 dark). --color-content-{hue}-subtle uses the built-in 200 shade for colored backgrounds and highlights
in both appearances. Notes and grouping fills share these backgrounds. Highlighted text is black on
these light fills in either appearance. Content colors carry no status
meaning; UI accent/danger/warning roles remain separate decisions sourced from the same base palette.

States mean the named state: `hover` is pointer hover, `pressed` is transient active press, `selected` is
persistent selection and `inactive` is the explicit inactive component role. Open menus and recent-item
feedback do not consume pressed paint. Neutral control paint increases from rest to hover to pressed in
both appearances. Ghost controls start transparent and use the same hover/pressed roles. A committing
action is the solid neutral (`surface-inverse`/`fg-inverse`, hover `surface-inverse-hover`): one
`UiButton variant="primary"`, or `UiIconButton tone="primary"` where the action is an icon alone — the
circular send/stop/play disc. Accent is reserved for links, selection, focus and progress; a confirm
button painted in the brand hue competes with the content and stops marking anything. Call sites must
not re-paint a primary control, and a surface has at most one. Solid destructive actions retain their
documented opacity treatment.
Do not create a complete state matrix where no consumer needs it.

Status foreground, boundary and background decisions remain independent. Repeated weak warning surfaces
use `surface-warning-subtle`, and their interactive hover uses `surface-warning-subtle-hover`; consumers
must not reimplement these roles as a solid warning hue with an arbitrary percentage. Other opacity
modifiers remain valid for instance-specific composition, not as substitutes for an existing role.

Typography uses size/line-height pairs: caption, label, body, title and display are each role's default;
caption-sm, body-lg and title-sm are the only alternate sizes. Add a size suffix only for a real
alternate style, not to complete a hypothetical sm/md/lg matrix. Text-size modes override both built-in
variables without changing the role names. Label and body differ in leading even when size matches.
Use title-sm for compact section, dialog and tool headings; body-lg belongs to reading content and
larger descriptive text. HTML heading level describes document structure, not a font-size slot.
Product font/tracking tokens contain only authored deviations from Tailwind.

Use `size-3`, `size-3.5`, `size-4`, `size-4.5`, `size-5` for 12–20px icons. Use Tailwind spacing/container
widths for fixed geometry and `duration-100`, `duration-150`, `duration-220` for ordinary motion.
Density-sensitive controls and platform-control exclusion retain product variables; shared interaction and toolbar heights use Tailwind spacing extensions. Tags use the same interaction height as other single-line navigation. Settings entries share `--knovum-setting-min-height` (52px compact / 56px comfortable), retain py-3, and grow when content needs more room.
Settings and extension lists share `UiSettingsSection` for their raised surface, border and panel radius.
Extension rows use the same body-lg labels and caption descriptions as preference rows. Category tabs
use neutral control surfaces; page actions use secondary buttons, with the primary neutral reserved for
committing an operation. Empty search results retain concise, left-aligned guidance.
Empty settings collections use `UiSettingsEmptyState` inside `UiSettingsSection`, with a centered
icon above the message and generous vertical padding, following the Archived Spaces treatment.
Avoid repeating the page title as a section title.
Settings panes keep 24px between peer sections. Appearance owns theme, text size, density and language;
Notifications is a separate General destination. AI routing separates the application default from
capability overrides. Local models use one settings surface per capability, with compact name/status,
download size, installed size and trailing actions; avoid nested panel frames and shaded table headings.
The global voice library uses these settings surfaces and compact rows, with one inline creation
or profile inspector open at a time. Voices have no Space selector or copy-to-Space action.
Adding a voice is a settings title action, so the library carries no section header repeating the
page title. Creation leads with the voice source as a segmented control shared with the speech
editors: the source decides the entire form, so it is a visible choice rather than a value inside a
dropdown, and switching it follows the unsaved-draft decision.
Every row opens with its own audition control, because a voice cannot be recognised from a name:
pressing it speaks a sample line in that voice and the row draws the waveform of the audio actually
being heard, taken from the playing audio rather than animated beside it. A preset stores an engine,
a language and a speed — never a recording — so the sample is synthesised on demand, and the library
holds one audition session: starting a row stops the row that was speaking, and pressing the
speaking row stops it. While a voice speaks, the other rows' controls are unavailable rather than
silently queued.
Voice creation uses labelled fields: the preset choice expands within its grid column, language keeps a
compact column, and preview/save actions share one form footer, with preview controls leading and the
committing action trailing. An unnamed preset voice shows the chosen preset's name as its placeholder,
which is the name it will be saved under. Conversation extension selection lives
in the composer toolbar; its anchored popover uses shared checkboxes, bounded scrolling and a separate
management footer. Empty extension categories are omitted.
Use rounded-sm for grids, rounded-md for controls, rounded-lg for canvas objects and rounded-xl for
workspaces/dialogs/outer sections. Only 10px panels and 9px inner sections extend the built-in radius scale.
Fixed switch thumbs use bg-white and video-label scrims use bg-black/45 in their owning component.

Aliases require a deliberate semantic relationship: inverse surfaces pair with foreground roles;
business colors never borrow a status meaning merely to reuse a hue. Note text is always solid black on colored paper, while free canvas text uses the normal foreground.
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

The contract uses [Tailwind's built-in theme mechanism](https://tailwindcss.com/docs/theme), with semantic
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
| `bg-surface-stage`                            | appearance-independent dark media stage and film strip |
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
- panel motion runs on one clock, `useWorkspaceEdges`. It animates panel _state_ - pinned, visible, open - with
  one interpolation, and every edge that has to line up derives from the same frame value: the reserved slot
  widths, the sidebar surface's position (a transform, so the sidebar never reflows) and the toolbar
  columns. A sash drag changes a width, not a state, so it lands in the same frame with no motion to suspend;
  reduced motion keeps every state and drops the interpolation. CSS keeps only fades with nothing to align
  (elevation, scrim, pane content) on `--knovum-panel-motion-*`, which a test holds equal to the clock;
- panel edges land on whole CSS pixels in every frame. The work area's origin moves with the leading edge,
  and engines place composited layers - the Konva canvases and the grid - on whole pixels while DOM text
  keeps the fraction. Measured in WebKit at 2x, an origin at 0.5px moved card text by 0.49 device pixels
  and the grid by 0; at 1px, text by 1.0 and the grid by 2. A fractional origin therefore shifts card text
  against its own frame and the grid by a different sub-pixel amount every frame, which reads as the canvas
  shaking while it slides; at whole pixels every layer agrees. A CSS transition cannot be told to land on
  pixels, so geometry never animates through one;
- the canvas never reacts to panel motion: nothing inside a sidebar reflows or mounts during it (fixed-width
  content; the right pane only fades as it mounts), nothing compensates the canvas camera for the moving
  host, and the stage surface - every Konva raster, the grid and the culled region - is bounded by the
  window rather than the host, so a moving work-area edge is a clip. Measured on the real canvas with 1,200
  notes, a toggle used to reallocate three layer canvases and mount or unmount about 36 cards inside the
  motion; it now does neither. The curve starts gently: a front-loaded curve covers most of a 240px travel
  in two frames and reads as a snap;
- a sidebar is the region its own width sizes, and that width is plain application state in pixels. The shell
  does not route it through a percent-based panel solver: converting to percent and back is lossy, so a panel
  resting on its minimum can report less than its minimum and have its commit silently rejected, leaving the
  committed width and the reserved space disagreeing. The sash is a real element that takes pointer capture
  for the gesture, writes that one number, and clamps at the geometry instead of collapsing - hiding is the
  toggle's job, so a drag never removes the handle being dragged, which is the only way a divider can survive
  its own gesture. Keyboard resizing uses the same path. An n-panel percent solver is still the right tool
  where the layout genuinely is n panels, as in the video editing workbench;
- the left sidebar has one surface for both of its roles. Its slot is only the layout width it reserves; the
  surface is fixed at the sidebar's width and placed by transform at the sidebar's own edge. Pinning from a
  preview grows the slot while that edge stays put, so the sidebar holds still as the work area makes room;
  closing moves both by the same frame value, so the sidebar's edge stays on the canvas edge. Pinning never
  moves the subtree between hosts, which would reset its scroll position and make every virtualized row
  re-measure inside the first frame of the transition;
- the first workspace tab starts exactly on the canvas's leading edge. The toolbar's leading column is
  `minmax(auto, max(gutter, <edge>))`: never narrower than the navigation cluster, otherwise ending where
  the sidebar slot ends - the same expression on the same frame - so the tabs wait for the canvas edge and
  then ride it. The tab column adds no inset of its own, and the trailing column ends where the canvas ends when the
  right pane is open. Without a pinned sidebar the work area keeps one gutter of inset on every side;
- a sidebar toggle states its state through `UiPanelToggleIcon`, one geometry shared by both sidebars and
  mirrored to its window edge: a window frame with a solid pane inset from it by the same padding on every
  side and concentric corners, the way a sidebar sits inside its window. Open, the pane is full width;
  closed, the same block narrows to a rail against the window edge, so both states read as one object and
  the difference is how much room the panel takes. The width itself animates on the panel's duration and
  curve - not a scale, which would flatten the corners mid-motion. The icon carries the state, so the button
  takes no pressed background on top of it. Swapping or cross-fading two different line glyphs inside an
  18px box paints both strokes over each other for the length of the transition, which reads as a flash;
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
while the sidebar's current-Space Outline owns document structure. The sidebar footer exposes Settings
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
the document editor owns its heading Outline locally. Source opens as a temporary drill-in and returns to its originating
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

| Profile  | Preview item budget | Meaning                                       |
| -------- | ------------------- | --------------------------------------------- |
| compact  | 2                   | recognition and primary identity              |
| standard | 4                   | recognition plus a short structured preview   |
| full     | 10                  | the normal non-interactive reading projection |

The item budget is semantic: Tiptap and Markdown count root blocks, subtitles count cues, code counts lines,
archives count entries, and structured objects count fields. A preview also allows 1024 characters per item;
Tiptap limits nested nodes to 32 per item. Markdown parses a cached, bounded source prefix of at most the full
profile character budget, then selects root-block ranges. A very large single block may end at the character
budget; Markdown's renderer closes its rendered structure and the preview displays an ellipsis instead of
inventing a remaining-block count. Rendered rich DOM must never be line-clamped as the primary density mechanism. Media and fixed utility
nodes keep their type-owned representation instead of pretending every kind contains a list.

Viewport detail and interaction are independent state. Activating a node switches it from `preview` to
`interactive` content without rewriting the viewport profile; editors, complete source and embedded controls may
then reveal the canonical content. Zooming alone must not persist a frame, derived height or content mutation.
Text mounts its invisible canonical measurement plane only until an accurate measurement is available.
Measurements are cached by immutable document identity, frame width, font size, typography scale, alignment
and loaded-font epoch; these inputs invalidate the cache. An active editor keeps its complete measurement
plane. Display density cannot feed back into persisted geometry.

Selected offscreen objects retain operation geometry without mounting their full content. The active inline
editor stays mounted when its object leaves the viewport or the workbench is covered. Other DOM content
suspends while Library, Settings or a full editor covers the canvas. Empty relationship, Section, group and
feedback layers allocate no Konva backing canvas; the drag layer remains available to synchronous drag start.

Canvas image previews use a feature-owned module Worker and transferable bitmaps. The existing image edit
algorithm owns crop, transforms and tone; preview dispatch replaces queued work for the same surface and
prioritizes the active image. Interactive draft previews use a 512-pixel long edge, at most once per 100 ms;
a settled preview follows after 180 ms. Settled backing resolution follows viewport zoom with DPR at most 2
and at most two million pixels. Cropping/removal coordinates remain in logical image space. The previous
bitmap stays visible while the next render runs, and the last consumer releases the Worker. Canonical Rust
materialization and export retain their full-resolution policy.

Outline uses Reka tree virtualization while keeping button TreeItems, hierarchy, type-ahead and rename
leases. Virtual rows occupy the viewport width, retain 2px peer spacing and scroll only vertically;
long titles stay truncated and preserve their one-way hover animation. Reka's TreeVirtualizer reads
its immediate scrolling parent, so this region uses the global scrollbar profile rather than nesting
another scroll viewport. Rows report their border-box height, including inline rename errors. The Space
library uses TanStack virtual rows; focused and renamed rows remain mounted, and arrow
keys/Home/End reveal the destination before transferring focus. Search input updates immediately and
filters after a 150 ms pause. Full text/video editors and recovery surfaces load on demand. Long-document
statistics and heading derivation follow input after a 200 ms pause; save and session commit remain immediate.

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

A single selected content node or Section shows one screen-stable action bar centered on the object's bounds and clamped to the viewport, above its bounds when there is room and below otherwise. Its opaque overlay has the existing panel radius, neutral hairline, compact label type and restrained control elevation. The bar measures the available viewport width: primary actions use icon and label while they fit, then transition to icon-only controls with accessible names and hover titles. Type-specific formatting, image tools and duplication remain visible, while structural commands and derivations remain under More. The bar clears the full pointer area of a selected node's connector controls and a Group's rotation handle using the shared canvas chrome metrics, not merely the object edge. A selected node always leads with one labelled action of its own: the kind's open or edit surface when it has one, otherwise asking the conversation about it. A node with neither — a credential, whose card carries its own editor — leads with its access controls instead of a decorative action, so the bar never opens with the overflow trigger. The final segment after More contains the universal icon-only **Details** and **Download** controls, separated by a full-height hairline, and nothing else is ever appended after it. Download opens the native save dialog and writes that node's supported representation to a local file. Edited images download as the rendered PNG, including crop, rotation, flip and tonal adjustments. A Section downloads a structured outline, not an asset package; credential downloads contain non-secret metadata only. The trailing access group holds how a node is read rather than changed — details and download — and a kind may lead it with its own reading: a derived picture that lays over its input adds compare-with-original there, icon-only, never among the actions that make something. The bar is selection chrome, so unselected nodes never carry persistent download or overflow buttons. Multi-selection keeps its separate compact structural toolbar.

The image action bar keeps direct image actions visible. Adjust, crop and flip/rotate use one temporary edit session: entering a tool replaces the action bar with that tool's panel, changes preview on the node, Apply writes one history entry, and Cancel discards the draft. AI object removal publishes a separate derived image: Qwen and GPT Image models use a temporary brush mask; supported Gemini, Grok Imagine, Doubao Seedream, SiliconFlow Qwen edit, Kling Omni and OpenRouter image models use an object description. Cancel discards the draft without changing the source node. Comment mode likewise occupies the action area; upscale, multi-angle and prompt-removal popovers have one shared open state so only one is open at a time. Crop exposes a dimmed outside area, direct drag handles, and drag-to-draw selection over the full source image. Its side panel groups W/aspect lock/H, collapsible, single-row general and social presets with aspect glyphs and reference dimensions, and Cancel/Done. The aspect lock is one session value shared by numeric fields and canvas handles. Adjust uses one scrollable Light, Color, Detail and Effects list; the four icons scroll to sections and track the visible section. The strict v6 edit contract contains the Light, Color, Detail and Effects controls, and Vue preview and Rust output both render them. Unsupported generation actions remain inspectable; only their execution controls are disabled. Active brush-removal and crop sessions enable direct pointer interaction on the selected image. The transform mode uses a compact replacement toolbar. The brush-removal panel keeps undo in the header, brush size in a neutral slider, and Generate disabled until a mask stroke exists and the image provider is ready. The multi-angle popover offers subject/camera modes, a draggable angle guide, rotation, tilt and three framing levels. Its thumbnail renders the managed original with the committed `imageEdit`, matching the input Rust later materializes. The guide is an interaction aid, never a generated result. Its controls become bounded model instructions in Rust. Picking one object by hand and letting a whole picture come apart are different actions and are named as such. **Select object** is an interactive local session: pointer prompts produce one mask, which is either cut out as its own image or filled in where the object was. Layer splitting takes no prompt at all. They sit next to each other in the decomposition family — remove background, erase, select object, split layers — and share the local selection, depth and inpainting models. Layer splitting is a direct image action with no options: discovery decides how many layers the image carries, each layer is published at its own cropped size, and a layer that carries no content of its own is left out. The WebView owns interactive preview only. Rust reads the authorized managed asset and applies the strict `imageEdit` contract for downloaded PNGs, local background removal and cloud inputs, then validates and publishes output. Upscale, multi-angle and AI object removal create derived nodes through the configured image provider. Layer splitting creates independent transparent image nodes with the local image models and needs no credential. Unavailable cloud capabilities state the missing configuration instead of presenting a successful no-op.

Group and Section interaction uses the same compact desktop language while keeping their different roles visible. A selected Group uses a quiet dashed world-space boundary, title chip and screen-stable rotate/scale handles; double-click enters its editing scope and the scope path remains visible above the canvas. A Section uses a low-chroma background, 40px title header, content count and dedicated resize handles. Its collapse control hides descendants without changing their Structure/Layout. During a valid drag, the destination Section receives one accent drop preview; a collapsed destination temporarily expands only after the 600ms dwell and returns to its persisted presentation when the preview is cancelled. Selection toolbars expose explicit dissolve, reparent, fit, collapse and subtree-delete meanings instead of inferring ownership from geometry. Section actions use the same bar placement, density, Details and Download composition as content nodes, with container-specific collapse, fit and focus controls. Details inspects the selected container or mixed structure roots and their contained content; it does not fall through to Space health. Fit is unavailable for empty or locked Sections. Section/Group labels use the canonical font and label/caption tokens, and Section background images honor cover, ratio and repeat.

Content, Group and Section translation previews operate on the effective selection roots. One ephemeral Structure/Layout snapshot projects the moving backgrounds, descendants, relations and selection bounds, and remains visible until the edit transaction is accepted. A frame resize changes the Section boundary while preserving descendant world positions. A Section is selected or moved through its header and border; its unfilled border excludes the interior from Konva hit filling, so contained content and empty-space marquee selection remain directly accessible. The header is painted before the complete screen-stable 1px outline. Section corners use the corresponding diagonal resize cursor, which remains active during resizing and clears on completion, cancellation, or disabled interaction. Drag membership uses the moving selection’s world-space center for both preview and commit. Entry and exit show an accent wash, dashed target boundary, and explicit release instruction; collapsed Sections first expose only the visible header as a target, show entry feedback immediately, and expand after dwell or on drop onto the header. Cancel restores the gesture before native drag termination can submit an edit; selection bars stay hidden throughout direct manipulation and its commit handoff. Clicking a selected root reduces selection on release, while dragging preserves the mixed selection.

Resize is a capability, not a default decoration. Image and video resize with their media viewport ratio preserved. Notes, Markdown, websites, files, code, folders, archives and knowledge objects resize within type-specific bounds. Audio, credentials, aliases, queries and portals remain fixed compact utilities. Sections use their dedicated container resize treatment.

Space library covers are Canvas Overviews: they fit all visible authored objects, sections and relations into one fixed 16:9 image, independent of the current pan and zoom. The cover renderer uses the same document projection and color tokens as the editor but draws a simplified, non-interactive Konva scene; DOM content and workbench controls are never captured. Covers refresh after content or scene-geometry changes, not viewport navigation. A session transition commits
user content, layout and persistence before starting a cover capture of that committed scene. Image decoding
and derived-cover publication continue outside the navigation barrier. Publication is serialized, belongs to
the captured Space, and discards deleted, superseded or changed sources. Saved View captures decode only
image sources intersecting their viewport.

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
two-column virtualized masonry gallery. Thumbnails reserve their source aspect ratio before loading; unusually
tall or wide resources use bounded, uncropped previews. Resizing the sidebar recalculates column geometry,
and keyboard focus keeps the current thumbnail and adjacent tab targets mounted. Selecting a thumbnail opens
the shared dialog viewer with every occurrence, source location and
nearby context, and exports the picture through the same format menu as a node. Office resources are identified
as extracted embedded images. PDF resources are identified as rendered page regions or scanned pages and are
never described as original embedded files. Resource visibility depends on document parsing and system
rendering, never on AI configuration.

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
replaces the workspace and returns to the last knowledge-space route with editor sessions retained.

In the Spaces app, one Space is one knowledge context and one infinite canvas; there is no intermediate Board entity. The leading sidebar owns Space navigation and the canvas structure Outline; the latter selects and reveals canvas objects. The current document's heading Outline stays inside the editor: a quiet minibar at the upper right expands on hover or explicit keyboard activation into a compact 288px overlay. The minibar shows at most 16 orientation marks, preferring H1/H2 when at least three exist; the expanded directory includes every heading, highlights the current section and scrolls within a bounded height without a visible scrollbar. It does not resize the document or open the workspace right sidebar. Documents without headings omit the minibar; passive orientation marks appear at the xl breakpoint, while explicit keyboard activation can open the outline at any width. Recents and Favorites are not separate user-facing concepts. Archived Spaces leave the workspace sidebar and are managed from Settings, where they can be restored or permanently deleted. Tags are classification/filtering, not a peer view mode. The right sidebar defaults closed and never opens merely because selection changed. High-frequency selection actions stay in the contextual canvas selection bar.

AI writing actions belong to the selection bubble menu. The document toolbar has no AI writing entry.

The default canvas layout shows the Spaces Sidebar and canvas. The right sidebar starts closed. Its Conversation and Details views share one pane; contextual image edit panels remain anchored to their canvas objects.

The workbench has exactly one tab strip, in the toolbar. One tab is one opened thing: a Space canvas,
a document or a video editor. There is no second strip inside the work surface and no fixed canvas
pseudo-tab, so a Space can be open with only a document tab. A new tab opens directly right of the
active tab and existing order never rearranges itself. Titles identify the Space or resource, with
type icons and a quiet unsaved marker. The Inbox canvas tab is permanent: it cannot be closed and
every close action keeps it, so the workbench is never empty. Opening a resource that already has a
tab focuses that tab instead of duplicating it. Reka owns tab keyboard/ARIA behavior; overflow
scrolls horizontally and the active tab remains visible. Closing a tab activates its right
neighbour, otherwise its left one, and keeps focus on the tab that became active. Reopening the most
recently closed tab restores it at its original position. The Spaces Browser stays a navigation
destination rather than a tab, and replaces the strip with its own toolbar while it is open.

The tab strip is the only place that closes an opened thing. Editor surfaces carry no back or close
affordance and no Escape-to-close shortcut: their headers hold editing state and editing actions
only, and leaving an editor means activating another tab.

Each tab reserves one trailing slot that holds either the unsaved marker or, while the tab is
hovered or its close button is focused, the close button. Autosave therefore never changes how much
room the label has, so a saving document cannot make its tab title reflow.

Open editor resources show a compact editing indicator, separate from the user-authored lock icon.
A protected selection disables deletion and cross-Space transfer, explains that its editor tabs must
be closed first, and offers an explicit action to the affected editor. Keyboard deletion/cut reports
the same reason with a navigation action; protected history replay reports the reason and preserves
the current history entry. Layout controls remain available. No operation silently skips protected
members of a multi-selection or closes their tabs as a side effect.

`Control+Tab` / `Control+Shift+Tab` cycle the tab strip; `$mod+W` closes the active tab unless it is
the permanent Inbox tab; `$mod+1` through `$mod+8` select that tab and `$mod+9` selects the last one.
`Alt+ArrowLeft` / `Alt+ArrowRight` move through tab activation history. All shortcuts enter through
AppCommand, including the native Close Tab menu action.

The canvas Outline, Conversation/Details pane and canvas commands are available only on the active
canvas surface. Document and video editors retain Space navigation but own their outlines, histories,
search and editing controls. Document citations open a document-local source preview; returning to
canvas or another editor clears that preview while retaining the document draft and history. The shell's requested right-sidebar visibility survives
surface changes, but the pane and its toggle are absent where the capability is unavailable.

A canvas is initially empty only when authored Structure and current-Space captures are absent, and
no placement tool is active. Empty Sections, Groups and hidden/offscreen objects count as authored
content. A canvas without authored objects accepts creation, selection and import but does not accept
wheel pan/zoom or drag panning; it preserves the camera for Undo. The inactive retained canvas is hidden
and inert, with input, canvas overlays and contextual commands suspended. Surface suspension cancels
transient gestures without using Escape's deselection or Group-exit behavior.

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
- single-line input commit/cancel events ignore Enter and Escape while IME composition is active; feature forms consume the shared semantic events instead of attaching raw commit key handlers;
- closing a tab or full-surface editor explicitly returns focus to the newly active tab, the new-Space control, or the underlying canvas so keyboard work can continue without a pointer click;
- frequent macOS commands also exist in the macOS application menu;
- `UiKbd` renders platform-correct modifiers;
- icon-only actions require an accessible label and tooltip when the icon is not self-evident;
- right-click context menus on non-editable desktop workspace surfaces use Tauri's native `Menu.popup()` path, not a WebView Context Menu component; this rule covers canvas objects/background, Spaces, Outline, Scenes, Tags and Relations. Editable text is not intercepted so the platform text-editing menu remains available;
- transient outcomes use `vue-sonner`; destructive and discard confirmations use the shared in-app `UiConfirmAction` dialog, while file picking and saving use the native OS dialog. A confirmation decides the fate of the surface that raised it, so `UiDialog` carries explicit stacking bands (`DIALOG_LAYER_CLASSES`) and the confirmation owns the band above every dialog: portalled content is inserted at the teleport anchor its component created on mount, so an equal band would bury the confirmation under the dialog that requested it. Confirming a discard belongs to moves the user did not ask for, such as switching the edited record inside a dialog; an explicit Cancel, Escape or close button is already the user abandoning that draft and closes immediately;
- background jobs use the Task Center bridge and OS progress;
- reduced-motion and forced-colors are handled globally;
- content is selectable by default; shell/navigation chrome opts out with `select-none`.

## UI primitives

Reka UI owns behavior: focus, keyboard navigation, ARIA, portals, collision and primitive state. `shared/ui` owns only durable Knovum visual compositions.

Do not wrap Reka merely to rename props/events. For richer WebView-owned structures such as Tree, Dropdown Menu, Combobox and Toolbar, compose Reka directly until a repeated Knovum visual composition is proven. Right-click desktop context menus are platform-owned through Tauri Menu and are not a Reka surface. Reka dialog content declares the active application-command scope: background commands are suppressed while a modal is open, with explicit per-dialog command allowances only when the modal owns that action.

Choose component ownership by interaction contract, not markup similarity:

1. Use an existing `shared/ui` component when its meaning, keyboard behavior, state and density match the control. Do not recreate its visual classes inside a feature.
2. Keep a control in its feature when it depends on domain geometry, commands or editing state. Repeated controls within one feature share a feature-owned visual contract while composing Reka behavior directly.
3. Promote a feature control to `shared/ui` only when at least two independent features need the same meaning, behavior and visual treatment. A second use with a different interaction contract is not proof of reuse.

Review desktop controls in their default, hover, pressed, focused, disabled, invalid and busy states. Verify keyboard operation, tooltip and accessible names for icon actions, long localized labels, compact widths and reduced motion. Popup surfaces remain anchored to their control and use collision handling; canvas geometry tools own their placement separately from ordinary trigger popovers.
Popover and dropdown triggers must own their positioning anchor directly. Do not nest a Tooltip trigger around a popup trigger or enable `UiIconButton`'s Tooltip inside one: nested positioning primitives can prevent the popup from anchoring to its button and move it to a viewport corner. Icon-only popup triggers retain an accessible label and may use a title hint when needed.

Canvas selection chrome has separate geometric contracts. A short action bar attaches above the selected object's visible bounds, or below when the upper position does not fit. It can temporarily cover unrelated canvas objects; their positions never influence its placement. The canvas selection anchor uses only the projected selection, fixed chrome bounds, viewport bounds and the measured surface size. It keeps the bar centered on a visible part of the selection, clamps it to the safe viewport and preserves its current side near a flip threshold. If neither exterior side fits, the bar attaches inside the selected visible edge; it never falls back to an unrelated screen corner. A relation uses its projected curve midpoint, including its zero-size point bounds. For a wide multi-selection whose box center is empty, use an actual visible member as the anchor. Triggered menus and popovers stay attached to their buttons; sustained edit panels prefer the selected object's sides and scroll internally when space is limited. Measure their unconstrained inner content independently of the outer scroll viewport so switching modes recalculates placement and height instead of retaining the previous limit. The height offered to an editor is the free viewport band after excluding fixed chrome, not the current measured height of its toolbar or loading state; loaded content can grow within that band. Direct object manipulation hides the action bar until the gesture ends. A panel never becomes a trailing sidebar merely because its anchor is crowded.

Node action bars measure their expanded and icon-only widths before display. The shortest desktop work area keeps the "More" trigger and common inspect/download tail reachable. When icon-only image actions exceed the available width, original-image editing, commenting and the decomposition family move into "More" together; generation popovers remain on their visible triggers so their anchors stay valid. The generic node bar moves video editing, speech operations and transcription into "More" at the same density boundary, while the node's own leading action stays in the bar at every width. Image, audio, video and other node bars share the same labelled action controls and compact icon mode. Business operations precede "More"; the separated trailing group contains only inspect and download.

Choosing a format is part of saving a resource rather than a menu of its own: the trailing download action opens the native save dialog with every format the resource admits, as owned by [resource-export.md](resource-export.md), and the chosen destination names the format. A long export reports progress through one notice that can cancel it. Unsupported formats are not offered.

Speech interactions follow the image tools’ contextual geometry: short bars attach above or below the selected media; sustained editors prefer its sides and scroll internally. Vocal/background separation starts directly from its action. Its first invocation starts the task and opens a compact contextual status bar. Returning to a running separation opens status without cancelling; cancellation is an explicit control in that bar. Closing the bar keeps Task Center observation and execution alive. Unavailable sources and the ten-minute limit have visible explanations. The two audio nodes it will produce appear on the canvas when the task starts, connected to their source and showing their own progress, and settle in place when it finishes. Their existing node controls own playback and export; no result-review bar or add-node confirmation is shown. Cloning separates interval selection, model/language settings, and audition text into mutually exclusive, button-anchored controls. Its bar uses the same measured label/icon density as other node bars; source preparation, audition results and close remain reachable at compact widths. Reference intervals share one selection composition with precise inputs. Audition parameters stay mounted through generation, and candidate names survive closing their popover; cancellation locks while its request is pending. Errors appear beside the active bar instead of only in a help tooltip.

Short dubbing, audiobook and video dubbing open their editing surface directly from the node action, without an intermediate edit button or parameter popover. Sustained speech editors use the same 12px selection gap, centered side placement, rounded panel, `bg-surface-overlay` and `shadow-popover` as image inspectors. The operation is the compact header title; the selected canvas node owns the source context, so inspectors do not repeat its filename in another context line. Segmented pages use the image tools’ neutral control surface and raised, bordered active state, with content padding separating them from the header. Cloning model and language fields stack in their button-anchored settings popover so neither depends on squeezing a normal select into a narrow grid column. Video delivery is an expandable group available once a saved script exists; generated stems and multi-track export appear only when their corresponding output exists.

Audiobook separates manuscript, role voices and settings; video separates script and mixing; short dubbing keeps its small settings section inline. Translating a dubbing script belongs with that script, not on a page of its own: it states what it does to the lines, offers the target language and the translation model, and commits the script before the Run starts, because a Run translates a saved revision. Translation is a model operation; Desktop Agents stay with conversations, which is where their session-shaped work belongs. Page changes retain drafts and hide inactive audition scopes. Each project owns one `UiScrollArea`, including virtualized script rows; generation, revision saving and run status stay outside that viewport. Generation is the stable primary action and saves a valid pending revision before synthesis. Submitting generation hands the work to the Task Center and to the output node already waiting on the canvas, so the composer closes instead of holding a copy of either. A finished project is therefore not replayed inside the composer: its audio is the node's to play and export, and the composer keeps only what the node cannot say — the signal check over the generated speech. Running work exposes an explicit Cancel action rather than a row of disabled primary commands. Chapter and paragraph actions sit with the manuscript, and project/library actions remain directly visible. Editor content uses flat groups and the existing small controls, with labelled, aligned metadata fields and quiet icon actions for ordering and deletion. These operations have no backdrop or detached sidebar.

State is owned outside transient "More" menus. Parameter content may remain mounted while hidden when it owns a draft; closing its popover preserves that draft without retaining focus. Popovers extend the existing overlay visibility scope to their content; hidden audition players pause, and nested overlays close. Selection/source changes close the operation surface, and project close uses the existing save/discard leave participant. Multi-selection and relations use compact icon controls and expose their secondary actions in the shared selection menu. These changes preserve each action's meaning and order; a hidden duplicate must not remain in both the bar and its menu.

Speech media previews compose the existing playback controller, `SpaceMediaTimeline`, and Knovum buttons rather than browser-native media chrome. Voice cloning and design proceed without a voice-use declaration checkbox; valid inputs and the existing audition workflow determine action availability. Reference cloning uses a keyboard-accessible two-thumb interval with precise decimal fields; the real media duration and existing 5–30 second validation own preparation admission. Model explanations are disclosed on demand, while unavailable-model errors stay visible. Opening a dubbing operation resumes its most recently updated project for that node and type, or initializes a real source-bound draft once the source is ready. Project naming and a separate creation form are not prerequisites. The first audiobook draft imports the node manuscript, and the first video draft uses an available transcript without overwriting edited or saved scripts. Narration converts Markdown to spoken text through MarkdownIt. Voice-library and add-voice actions remain visible beside the editor; an empty library has an explicit explanation instead of a blank disabled select. Starting another draft is an optional visible action, and saved drafts are identified by title and date. Generation is the stable primary action and first saves a valid changed draft; a failed write prevents synthesis. Automatic source imports and default voice assignment are initialization, so opening and closing without editing needs no confirmation. Actual edits use the unsaved-draft decision: complete edits offer saving; incomplete edits offer continuing or discarding, without an unavailable Save action. Closing still follows that decision, and leaving during a write prevents delayed generation after disposal. The selected canvas object remains the visible source context, so the video editor provides compact source playback without a second video preview. Manuscript and timed script editing precede mixing, output and metadata settings. Those settings use independent expandable sections that retain drafts and active tasks when collapsed; invalid settings expand to expose the correction. The editor header, close action and generation/cancellation commands remain reachable while content uses one `UiScrollArea`; timed script rows virtualize against that same viewport and retain the focused row.

AI image tools separate interaction from execution. Brush repair uses the selected local model; text-described removal and multi-angle generation retain their explicitly configured remote target. Opening the eraser, painting a mask, changing brush size or entering a removal description requires a ready image and exclusive node-operation ownership; it does not require model credentials or a successful capability query. Generation confirmation checks the explicitly selected input mode, its actual execution capability and input constraints, and explains why submission is unavailable. Mask and description removal share one draft session and submission surface; switching modes is a user choice and never an automatic fallback. Existing node-operation admission continues to prevent dragging or concurrent edits while either mode is active.

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
reloads. It lists work that has nowhere else to be watched — preparation, indexing, derivations,
exports. Work with its own surface stays off the list: a conversation Run is watched in the
conversation it is answering, which already shows that it is running, stops it and reports how it
ended, so a row would be a second and worse copy of that surface. The store still holds it, because
being in flight is what holds back an update restart; what it does not earn is a row, a badge count
or a duplicate failure message. It presents measured bytes for downloads and indeterminate
verification/installation;
it does not invent a verification percentage or own installation lifetime. Cancel releases one
request, and retry starts resource preparation without automatically repeating a business mutation.
Settings uses that same preparation path for optional pre-download and model removal; it is a
management surface, not a prerequisite for using a capability. Application launch does not download
optional models or tools.

Local models have their own Settings entry, separate from AI connections and execution routing.
The page starts with a capability overview grouped into knowledge, audio and images. Each capability
uses `UiSettingsSection` and a keyboard-accessible Reka disclosure that shows its configured or fixed
model, actual download state, total installed size and active preparation count. Expanding exposes
model comparisons and download/select/remove actions; default configuration and downloaded files
remain separate concepts. Downloading never implicitly changes the configured model. Voice selection
explains its scope for new reference voices; existing voices retain their original model.

Search covers model names, descriptions and capability names across all groups. Search and download
filters expand matching capabilities without changing the user's normal disclosure state; clearing
filters restores that state. Empty results have one page-level explanation and a clear-filters action
that returns focus to search. Background query failures stay visible without unmounting cached model
controls or resetting disclosure state. Model operations lock competing actions while pending;
cancellation stays pending until the preparation actually settles. Failures reveal their capability.
Replacing a model action keeps keyboard focus within that model without stealing dialog focus.
Downloads show measured transfer percentages; verification and installation remain indeterminate,
using the same stage policy as Task Center. Hidden capabilities and model controls remain mounted
to retain active preparation. Settings owns scrolling, with no embedded
sidebar or fixed-height scroll frame. Task Center remains the cross-view owner of preparation progress
and cancellation.

## Product semantic selection rules

Tokens represent an existing shared policy, not every distinct value or potential future customization.
Prefer Tailwind's primitive scales for ordinary geometry; product density extensions use its built-in
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
focus remain distinct. HTML, ARIA and Reka disabled controls use opacity-40; this is a component rule,
not a new color token. Disabled controls keep the desktop default cursor. Informational capability
badges and loading indicators are not disabled controls.

Spaces owns `src/features/spaces/space-tokens.css`, imported once by app.css so DOM and
runtime canvas paints remain available together. It owns canvas grid and object elevation, including appearance overrides. Shared content colors belong
to the product theme. The remaining feature variables are not exported through @theme and must not be used
outside Spaces. Built-in Tailwind palette variables are available to token definitions and content composition.

Typography roles remain caption, label, body, title and display, with only existing meaningful size
variants. Keep caption-sm for keyboard hints/debug metadata; use label for operation text, body/body-lg
for reading, and title-sm for compact headings. Do not add size slots to make a symmetrical matrix.

`UiInput` emits composition-safe `commit`/`cancel` events with the original keyboard event.
Consumers that own Enter/Escape actions use explicit `.prevent` modifiers; an ordinary input keeps
native form submission and dialog behavior.

### Canvas generators

Image, video and document generation is an in-canvas surface, never a dialog. A generator is a
world-space frame sized to the output, with its label and border counter-scaled through
`--canvas-zoom`, and one composer attached beneath it: references, an auto-growing prompt, then a
footer with `+` (add references), the output's own options (aspect ratio or duration), a compact
executor field and one primary _Generate_ action.

- **It behaves like a canvas object.** Pressing the frame selects it (and puts the canvas selection
  down), dragging moves it, Delete/Backspace removes it, Esc leaves it. There is no close button:
  leaving a generator that has neither a prompt nor references removes it, the way an empty text
  box disappears; one with a draft stays as a frame until it is deleted or generates. A generator
  just opened focuses its prompt; pressing an existing frame does not, so Delete acts on the frame.
- **References come from the canvas.** `+` offers _Pick from canvas_ and _Upload file…_. Picking
  keeps the composer open with an accent strip above it; the canvas's own selection (click,
  Shift-click, marquee) is the gesture, each pick is added, content a model can never read is
  refused by name, and Esc or _Done_ finishes. An uploaded file is imported beside the frame,
  without moving the camera, and is a reference from that moment. A reference shows a thumbnail
  for a picture and a named chip for anything else; one still being prepared shows a spinner, one
  that cannot be read a warning, and either keeps _Generate_ disabled.
- **Input belongs to the overlay.** `CanvasWorldOverlay` is a sibling of the stage, not inside
  Konva's content element: presses on a generator never start a stage gesture, yet still reach the
  document, so selects and menus inside the composer dismiss on an outside press like everywhere
  else (never `stopPropagation` on `pointerdown` for that). Keys and context menus stop at the
  overlay; only the wheel is forwarded to the stage so the canvas pans and zooms over a frame.
- The selection toolbar is hidden while a generator is active so two composers never compete. The
  canvas add menu's _Generate_ group holds Image, Video and Document; the open-ended Agent path is
  the Space conversation, not a menu entry.

### Agent conversation

The conversation list renders **turns**, not messages. One turn is one exchange: what a person
asked, the plan and tools the run used, the text it wrote, the nodes it added and the sources it
cited. Grouping it there is also what keeps an answer in flight inside the list — the turn carries
the text that is still arriving, so nothing is drawn beside the list and then moved into it when
the stored message lands, which is what made the view jump as an answer settled.

A turn is drawn from its **transcript**: what the run said and did, in the order it happened — the
words it wrote, the tools it called between them, and the stored answers
(`conversation-transcript.ts` → `conversation-timeline.ts`). Consecutive calls fold into one group,
so the words that announce a search sit above it and the words that report on it sit below. The
transcript comes from the live stream while the run speaks (`content_delta`, `content_boundary` for
a new message, `tool_started` marking where a call fell) and from what the run recorded once it
settles: a desktop Agent's ledger, where its updates are kept in arrival order up to its last call
and its stored answer takes over; the built-in Agent's stored messages, where each tool result
follows the answer that called it. Both are recorded in the order the Agent produced, so a settled
run reads back with the shape it had live, and the stream is dropped only once both the stored
answers and the ledger have loaded. Content only ever arrives at the end; nothing on screen moves
when a tool starts, ends, or the run settles. What a call is — title, status, context, result — is
read from the ledger by call id in every case; the stream only says where it happened.

A tool group is closed until the person opens it, and only they close it again. It never opens for
a call and folds when the call ends: that moved everything below it twice per tool. While the run
works at a group it says so in its own summary line, so no second line appears between two calls
only to vanish when the next one begins. The one waiting line — `preparing` (no run yet, models
may still be loading) or `generating` — exists only while the turn has nothing else to show, in the
turn that waits, never as a notice at the end of the transcript; the first block takes its place.
Only the answer that ends a turn offers copy, branch and regenerate.

Active turn status lines (preparing, generating, active tool execution, and in-progress plan steps)
pair the 3-stroke breathing mark with neutral foreground shimmer in a stable `size-3.5` leading
slot so settling into a checkmark never shifts the label horizontally.

The composer carries exactly one action for the answer in flight. Sending becomes stopping while a
run is producing, with pausing beside it for the runs that can be resumed; there is no second stop
control in a bar of its own. The three conditions that used to be one `busy` flag are kept apart:
a conversation that cannot take a question yet disables sending, a run being produced replaces it
with stopping, and a request that has not been accepted yet shows that control disabled rather than
offering to stop something that cannot be stopped.

Everything the exchange has to say that is not transcript — why it is waiting, that it is paused,
what failed — sits in one strip against the composer. A failure in the scroller can be scrolled
away from; somebody who scrolled up to read must not be the last to know the answer stopped.

Jumping back to the newest turn floats over the transcript instead of taking a row under it, so
appearing and disappearing never moves the composer.

### Agent extensions

Extensions use four searchable, counted categories: Plugins, Apps, MCP, and Skills.
Plugins are complete installed capability packages; Skills lists only independent personal instructions.
Apps lists package-declared integrations and manages the connection and reviewed access in a
dedicated inspection view. Connecting a service is one action, not a list: there is no name to
invent, no account to choose and nothing to bind afterwards, and reconnecting renews the
authorization in place. Disconnecting is offered beside it. It links to the owning plugin for overall activation and removal; it does not offer
component switches or another custom MCP form. MCP lists raw protocol services, both user-configured
and package-owned. A packaged service inherits its plugin enablement and cannot be toggled separately.

The discoverable catalog includes optional bundled extensions without requiring an external source.
Installed releases lead to their installed inspection rather than offering duplicate installation.
Localized bundled names describe user outcomes; external package names retain their publisher spelling.
Catalog browsing is a route state (`extensionAction=browse`) so Apps can lead directly to installation
and browser history returns predictably. Bundled packages can be uninstalled and remain removed.

Settings title actions use `UiSettingsHeaderActions`, which projects controls into the title-right
area using Vue Teleport. Each pane owns its actions, operation and dialog state; `SettingsView`
provides one header target that stays mounted across pane transitions. Desktop Agent inspection,
archived-space deletion, extension discovery/installation, MCP creation and personal Skill
creation/import all use this composition.

Extension categories and search share the following row; do not add a separate right-aligned
action row below search. Inspection and configuration views hide collection actions.

Use flat identity rows with quiet icons, names, one-line descriptions and explicit actions. Contents
and configuration open in a dedicated inspection view with a return action. Plugin contents inspect
Skills locally and link to their service configuration. Plugin enablement is the sole activation
control for all packaged components; do not duplicate them in personal Skills or offer child switches,
connection creation/deletion, or per-conversation component checkboxes. The conversation picker
selects complete plugins, personal Skills and standalone MCP connections. A missing required
component makes the package unavailable as a whole. Read-only instructions render as Markdown;
editing a packaged Skill requires copying it to personal ownership first.

Enablement is user intent, not a connection-health claim. Availability admits the current revision,
parent plugin and account; it does not claim a remote server is online. Endpoint/account setup and
tool authorization remain explicit: selecting a plugin never grants denied or unreviewed tools.
Standalone connections start disabled and tool permissions default to deny. No mock integrations,
inert trial buttons or pretend executable applications.

MCP configuration is an inline settings form constrained to `max-w-xl`, not a large modal. Back,
Cancel, browser history and settings navigation all use the settings leave coordinator to preserve
unsaved work. Identity, transport and authorization use compact sections; optional public headers
and environment variables use progressive disclosure. Reuse segmented STDIO/Streamable HTTP controls,
native executable/directory pickers and literal argument inputs. No raw JSON editors or nested cards.
Changing transport replaces its configuration and authorization requirements. Credentials remain in
the dedicated account authorization flow.

The extension tab, selected entity, service configuration, catalog browsing and creation intent live in the settings
route query (`extensionTab`, `extensionId`, `connectorId`, `extensionAction`). Search is local visual
state; installed content and availability use TanStack Query. Reuse Knovum semantic tokens, Reka
behavior and native installation/export dialogs.

### Local image controls

Image model choices extend the existing capability groups in Settings; they do not introduce another
flat download page. Upscaling chooses a target above the existing 2K/4K/8K menu; choosing a size starts
the operation. Eraser repair keeps its brush/description interaction and adds an explicit target for
mask mode. Smart selection uses one compact inspector on the shared selection surface, with point,
exclusion, rectangle and brush tools rendered on the source node. Its Extract/Repair actions publish
related outputs directly. Relative-depth generation is a direct node-menu action. Temporary masks,
encoding work and controls do not create detached canvas nodes. Geometry, side selection, gap, More
migration and focus restoration remain responsibilities of the existing bar and selection owners.

### Lightweight video workbench

The video editor is a compact same-window desktop workbench. Reka splitters own the library/stage/
selection-inspector columns and the stage/timeline division. Selection opens its owning inspector;
closing it restores ruler focus. The library is a tabbed rail of materials, text and transitions that
reuses managed Space imports and existing media players. The stage is a Konva scene: the composed
frame, its layers and its overlays are real scene nodes, so a selected clip is moved, corner-scaled
and rotated with transformer handles that keep their size at any stage zoom, a selected overlay is
dragged to any position, clicking a layer selects it, and the frame can be zoomed or fitted without
affecting output settings. Every manipulation commits to the project, which remains the only source
of truth; precise values stay available in the inspector for keyboard editing. The timeline exposes placed visual clips with filmstrip thumbnails, cut transitions, text
and independent audio lanes with waveforms, over a pixels-per-second scale with pointer-anchored
zoom, a shared playhead, millisecond timecode, output-frame stepping, pointer trim handles, clip
dragging and keyboard trim; dragged edges snap to clip edges, the playhead and the frame grid.
Clip settings include precise in/out fields, timeline position, volume, framing, free transform and
fades; audio settings distinguish source time from timeline time and expose the lane. Transition
settings expose the kind and a duration bounded by the media the cut can spare. Output resolution,
frame rate and background colour live in a labelled settings popover. Unavailable materials and
preparation errors remain visible, and a project whose stored payload cannot be read states that plainly and
offers a fresh edit of the unchanged source. All edits share the project's undo/redo
history; saving and native rendering stay in the workbench header.
