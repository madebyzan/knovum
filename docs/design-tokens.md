# Design token contract

`src/styles/tokens.css` is the only source of product token values. This document owns token definition,
naming, consumption and mode rules. `ui-system.md` owns visual language, component behavior and where
those visual roles belong. Neither document is a second palette or an executable token registry.

## One value owner

Public utility tokens are declared with direct values in Tailwind's native `@theme static` namespaces.
Use the same variable in CSS and JavaScript and its native utility in markup. Do not add a parallel
`--knovum-*` variable, a primitive-to-component alias ladder, a compatibility name or a fallback palette.
`static` deliberately retains variables that are consumed by Canvas rather than discoverable class names.

Private component geometry and feature paint use ordinary, directly assigned CSS custom properties in
`@layer theme`. They are not exported through `@theme` merely to manufacture a shorter utility name.
A private variable is an independent decision, not an alias of a public variable. Its declaration at
`:root` provides availability, not permission for every feature to consume it.

A computed relationship such as inactive foreground mixing is a calculation, not a second value source.
Equal literals in independent roles do not require deduplication. Typography size and line height are
separate properties of one role, not aliases. Mode overrides are values of the same token under another
condition, not a parallel implementation.

## Naming and ownership

Names use lowercase kebab-case. Keep a finite vocabulary per category; do not invent synonyms for an
existing role. Preserve the application property and owner where the namespace does not already express
it. Default semantic roles omit a `default` suffix; additional sizes use `sm` or `lg`, and pure size scales
use `xs`, `sm`, `md`, `lg`.

| Category | Canonical definition | Consumption |
| --- | --- | --- |
| Font family | `--font-sans`, `--font-mono` | `font-sans`, `font-mono` |
| UI typography | `--text-{role}`, `--text-{role}--line-height` | `text-{role}` |
| Tracking | `--tracking-{role}` | `tracking-{role}` |
| UI color | `--color-{role}` | Property-appropriate color utility or the variable |
| Content palette | `--color-canvas-{object}-{choice}` | Canvas presentation and its swatches only |
| Shape | `--radius-{role}` | `rounded-{role}` |
| Static elevation | `--shadow-{role}` | `shadow-{role}` |
| Material | `--blur-{role}` | Material composition |
| Timing curve | `--ease-{role}` | `ease-{role}` |
| Private geometry/paint | `--knovum-{owner}-{part/property}-{qualifier}` | Native variable utility or direct CSS |

The UI typography roles are `micro`, `caption`, `label`, `body`, `body-lg`, `title-sm`, `title`, and
`display`. Components choose the role; text-size preferences change its values. Document font sizes and
Canvas typography scaling are content decisions and must not be rewritten as the UI type scale.

Surfaces distinguish `chrome` (window/navigation), `canvas` (primary content), `surface` (embedded or
grouped content), `panel` (content-object panels), `raised` (local controls), `overlay` (floating content),
`field` (quiet input fill), and `scrim` (occlusion behind a modal). See `ui-system.md` for composition rules.
Foreground uses an emphasis hierarchy; border and divider roles describe boundaries, not text emphasis.

`hover` and `pressed` are transient interaction states; `selected` is persistent selection. The
`tab-inactive` role means an unselected tab, not an inactive OS window. Window activation is represented
only by the `data-window-active` mode. Disabled, focused and selected are not interchangeable states.

Canvas hue names are authored presentation choices, not status intents. Do not bind a green relation to
success or a yellow note to warning merely because values coincide. Inline rich-text color presets and
user-selected colors remain document content; they are not backup application themes.

Do not infer a new component token from the existence of a component. A shared component can select a
public radius or typography role directly. Ordinary spacing uses Tailwind's numeric scale. Fixed product
geometry that varies with density or platform has an owned private token. Canvas coordinate transforms,
zoom math, persisted sizes and instance-specific geometry remain in their domain model.

## Runtime composite shadows

Tailwind's shadow compiler expands literal `@theme` shadows into utility declarations. A later CSS
variable override does not change that expanded literal. Consequently a mode-dependent composite must
not also be declared in the compile-time shadow scale.

The mode-dependent `--shadow-canvas-object` value is declared once in the root mode layer, with a dark
appearance override. Its single public `shadow-canvas-object` utility applies Tailwind's native
`shadow-(--shadow-canvas-object)` utility. There is no second value variable, hand-written shadow
compositor or competing utility implementation. This preserves Tailwind's ring/shadow composition while
keeping the entire shadow live at runtime. Static elevation roles remain native `@theme` definitions.

The toolbar's neutral shadow is a zero shadow rather than `none`: it participates in a comma-separated
shadow list with the toolbar's material highlight. `none` is not a valid member of that list.

## Modes and precedence

Defaults and overrides live in the same `theme` cascade layer. Root mode selectors use `:root:where(...)`
so specificity does not silently defeat source order. Priority is base values, independent text/platform/
density overrides, appearance, window activity, then accessibility preferences. Intersections that need
special behavior are explicit selectors, not fallback chains.

Density changes interaction geometry, not typography or native titlebar geometry. Text size changes UI
size/line-height pairs. Platform changes native exclusion geometry. Window activity changes only its
specified paint roles. Increased contrast wins over ordinary inactive foreground weakening; inactive
object boundaries still do not show active hover feedback. Reduced transparency makes overlays opaque.
Motion and forced-colors presentation continue to be handled by the base stylesheet.

When adding a mode, review every consumer that snapshots computed values. CSS consumers update through
the cascade; Canvas must reread on every relevant environment change. Its palette reads synchronously
from CSS, refreshes after mount, watches theme/window activity and relevant media queries, and disposes
its listeners with the owning scope. Missing required tokens throw. Do not substitute hard-coded colors,
empty strings or an initial duplicate palette.

## Changes and verification

A token change includes its definition, all consumers, mode overrides, tests and affected documentation.
Renames are replacements: delete the old name in the same change. No compatibility aliases, dual readers,
maintained migration maps or silent defaults are part of this contract.

Before adding a token, identify its owner, application property, allowed consumers, mode behavior and why
existing roles are insufficient. Keep the definition beside its category. Do not add another registry or
value-generating layer to repeat this information.

`src/styles/__tests__/tokens.test.ts` checks direct definitions, typography pairs, retained runtime
variables, compiled utility behavior, composite-shadow boundaries, equal-specificity mode selectors and
unresolved literal private references in production source. Existing `shared/ui` tests verify that `cn`
merges the public font-size, radius and shadow roles in their actual utility groups. Canvas palette tests
cover initialization, required values, environment changes and cleanup.

Run the repository's own installed dependency versions and full checks before merging. Browser regression
must include light/dark, text size, density, platform, window activity, increased contrast and reduced
transparency, plus computed Canvas object shadows. A local CSS fixture or syntax-only check is not a
substitute for the application build, typecheck, unit suite or desktop visual verification.
