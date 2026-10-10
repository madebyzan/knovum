---
name: knovum-meeting-notes
description: Turn an existing audio or video transcript into grounded meeting notes, decisions, action items, or a media study guide with timestamp references.
---

# Meeting and media notes

Use this skill when the request concerns a recorded meeting, interview, lecture, or other timed media. Respond in the user's language and preserve the source language for quoted passages.

## Establish the source

Read the selected media node and its current revision through the advertised node-reading tool (`read-node` is its business Action ID; external MCP exposes `get_node`). Confirm that a usable timed transcript exists. A filename, description, or visual thumbnail is not a transcript. If it is absent or failed, explain that the source needs transcription through the media node's toolbar before notes can be grounded. Do not invent a transcription Action or silently summarize unrelated sources instead.

Read relevant transcript segments before writing. Keep source node IDs, revisions, and timestamps with the evidence. Label uncertain or inaudible passages. Identify speakers only when the transcript or authorized source explicitly identifies them; do not infer identity from a voice.

## Structure the output

- Meetings: purpose, concise topic summaries, explicit decisions, open questions, and action items. For each action item, include the task, the stated owner, the stated due date, and supporting timestamp. Mark missing owners or dates as unspecified. A suggestion is not an agreed decision.
- Interviews: questions, the speaker's stated positions, noteworthy quotations with timestamps, and unresolved follow-ups.
- Lectures: key ideas, supporting explanations, terms, and study questions. Distinguish the speaker's claims from independently established facts.

Do not fabricate attendance, consensus, deadlines, quotations, or events after the recording. Avoid copying the entire transcript into the notes.

For Ask or Plan, return notes or a publication proposal. For an authorized Execute request, the built-in Agent uses the admitted `summarize-meeting` Action with the actual Transcript input role and current parameter schema, making the structure and evidence requirements explicit in `summaryPurpose`. An external Agent produces the notes with its own runtime and saves the final Markdown or requested document artifact in the task output directory; it cannot call Knovum inference Actions. Review the notes against the transcript. Preserve source node IDs, revisions and timestamp references in the document, and claim publication only from a completed Action or task output receipt.

## On-demand resources

Read `references/transcript-review.md` when its review criteria apply. Use `templates/meeting-record.md` when a structured working record helps this task. Load only the resource needed for the current step; adapt its structure and omit unused sections.
