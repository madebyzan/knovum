# Creation sources

How a Space resource reaches a model when the user asks for something new. One document for the
whole path: what the user selects, what the model receives, and who decides between the two.

## The rule

> Every node on the canvas is knowledge. Anything a user can select, they can create from.

Availability is a property of the **Action and the file type**, never of an asynchronous background
job. "This image has not been analyzed yet" is a reason to prepare during the Run, not a reason to
disable a menu entry.

## Three forms a source can take

A selected node enters a model in one of three forms. The form is chosen by capability negotiation,
not by the UI:

| Form            | What travels                                    | When it is chosen                                  |
| --------------- | ----------------------------------------------- | -------------------------------------------------- |
| **Original**    | the actual bytes (rendered PNG, document file)  | the target accepts binary input of that media type |
| **Text**        | extracted or indexed text, retrieved when large | the source is textual, or the target is text-only  |
| **Description** | cached visual/media analysis                    | fallback only: the target cannot read the original |

Preference order is Original → Text → Description. A description is a lossy substitute for pixels
the model could have seen; it is never preferred over the original, and it is never a precondition.

### Why this order matters

Sending an image through its own description costs an extra model call, waits for a background job,
and loses everything the description did not mention. Sending the pixels costs nothing extra and
preserves identity, composition, palette and text in the image — which is the whole point of
"make me a variant of these".

## Roles, not slots

An image attached to an image generation can mean three different things. The request carries the
role so the adapter can map it onto its API:

| Role          | Meaning                           | Default rule                             |
| ------------- | --------------------------------- | ---------------------------------------- |
| `edit-target` | "change this image"               | exactly one image selected, image output |
| `reference`   | "look like this" / "include this" | several images selected                  |
| `grounding`   | "the facts are in here"           | documents, notes, websites, transcripts  |

A role is a request field, not a UI index. The user never sees "input 1".

## Capability negotiation

An image-generation adapter declares what it accepts instead of the shared layer guessing:

- maximum reference images
- transport (inline data URL or uploaded file reference)
- accepted media types
- per-file and per-request byte budget
- maximum outputs per request

Verified anchors for the adapters that support references today:

- OpenAI image edits accept an `images` array of file IDs or data URLs — up to 16 images for the
  GPT image models — and up to 10 outputs per request, with a ~20 MB cap per data URL.
- Gemini image models accept reference images inline, with a per-model ceiling (3 for the 2.5 Flash
  image model, up to 14 for the 3.x image models) and roughly 7 MB per inline file.

Every other adapter declares the conservative limit its API is verified to support, and the shared
layer refuses what the adapter did not declare. No adapter silently drops an attachment.

### Transport

Inline base64 is simple but inflates payloads by about a third, so it is used while the request
stays inside the declared budget. Above it, the adapter's file-upload capability carries the bytes
and the request references the uploaded ID. The choice belongs to the adapter's declaration, never
to the calling feature.

## Preparing a source

Before bytes leave the process, every source goes through one shared preparation:

1. render the current pixels, applying the node's pending image edit
2. verify the content revision again, so a queued Run cannot send an image the user changed
3. check the media type against the adapter's declaration
4. downscale when a file exceeds the budget, rather than failing the Run
5. own the temporary file until the request completes

## What the user sees

One creation dialog. A requirement field, the attached sources with their role and readiness, an
output switch (document / image / video), and the output options the Action's parameter schema
declares. No per-output dialog, no per-output parameter list written by hand.

When a source still needs preparation, the Run says so as a stage — "understanding sources", then
"generating" — on the output node that was already reserved when the user clicked. A disabled menu
entry is reserved for the cases that cannot be fixed by waiting: a missing asset file, an
unsupported media type, or no configured model for the capability, each stated with the action that
resolves it.

## Reuse

A derived node records its Action, pinned parameters, source roles and execution identity. That is
everything a repeat needs, so reuse reads the result:

- **Generate again** — same requirement, same settings, new attempt.
- **Create from this** — opens the dialog pre-filled, with the sources swappable.

Nothing is stored for this to work, and it works the first time, without a capture step.
