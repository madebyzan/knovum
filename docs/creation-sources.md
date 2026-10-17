# Creation sources

How a Space resource reaches a model when the user asks for something new. One document for the
whole path: what the user selects, what the model receives, and who decides between the two.

## The rule

> Every node on the canvas is knowledge. Anything a user can select, they can create from.

There are four things a user creates: a **document**, an **image**, a **video**, an **audio**
reading. Everything else is a detail of how that happens. There is no separate "edit an image"
request: attaching pixels and asking for a change is the same request as asking for a new image.

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

## Pinning a selection

Each selected resource is pinned to the revision the user is looking at, on the basis that made it
readable:

| Basis       | Pinned value        | Used for                                          |
| ----------- | ------------------- | ------------------------------------------------- |
| `asset`     | the file's bytes    | images, which are readable as pixels on their own |
| `knowledge` | the indexed content | text, documents, websites, transcripts            |

The basis is decided once, where the selection is resolved, and then travels with the input through
the Run and into the derivation record. No handler assumes a basis of its own, which is what used to
make an unanalyzed image look unusable.

## Capability negotiation

An image-generation adapter declares what it accepts instead of the shared layer guessing: how many
reference images, and how many bytes each may be. The shared layer refuses what the adapter did not
declare, and the caller renders within the declared budget instead of discovering a provider limit
through a failed request. No adapter silently drops an attachment.

Verified anchors for the adapters that accept references today:

- Gemini image models accept reference images per model (3 for the 2.5 Flash image model, up to 14
  for the 3.x image models), around 7 MB per file.
- The OpenAI-family shapes (Grok edits, Doubao Seedream, OpenRouter image references, SiliconFlow
  Qwen-Image-Edit), DashScope's qwen-image protocol and Kling's omni-image models each accept one
  image alongside the prompt.

A model with no declared reference support is a text-only image model: its sources fall back to
Text, and the creation entry stays available.

### Transport: a file reference beats base64

Where a provider has a file API the pixels are uploaded and the request carries the reference —
Gemini `fileData`, Grok `file_id`, DashScope's uploaded URL. Inline base64 is used only where the
API offers nothing else (Doubao, OpenRouter, SiliconFlow, Kling), because it inflates the payload by
about a third. The choice belongs to the adapter, never to the calling feature.

## Preparing a source

Before bytes leave the process, every source goes through one shared preparation:

1. render the current pixels, applying the node's pending image edit
2. verify the content revision again, so a queued Run cannot send an image the user changed
3. check the media type against what the renderer accepts
4. downscale when a file exceeds the declared budget, rather than failing the Run
5. own the temporary file until the request completes

Rendering happens outside the write transaction: it is CPU work, and its correctness rests on the
content revision check, not on a database lock.

## What the user sees

One creation dialog: what you need, in words; the output switch (document / image / video); the few
options that output actually has; the executor. The selected resources are listed as the material.
There is no style picker and no per-output dialog — the requirement text says what the style is.

When a source still needs preparation, the Run says so as a stage on the output node that was
already reserved when the user clicked. A disabled menu entry is reserved for the cases that cannot
be fixed by waiting: a selection larger than a Run can pin, or a selection with nothing a model can
read, each stated plainly.

## Reuse

A derived node records its Action, pinned parameters, sources and execution identity. That is
everything a repeat needs, so reuse reads the result rather than capturing a workflow in advance.
