use serde::Deserialize;
use serde_json::{json, Value};

use super::{self as indexing, KnowledgeChunkSeed};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaKnowledge {
    source_revision: String,
    transcript: Transcript,
    insights: Option<Insights>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Transcript {
    id: String,
    revision: u32,
    duration_ms: i64,
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Segment {
    id: String,
    revision: u32,
    start_ms: i64,
    end_ms: i64,
    text: String,
}

#[derive(Deserialize)]
struct Insights {
    summary: String,
    chapters: Vec<Chapter>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Chapter {
    start_ms: i64,
    end_ms: i64,
    title: String,
    summary: String,
}

pub(crate) fn chunks(data: &Value) -> Result<Vec<KnowledgeChunkSeed>, String> {
    let Some(value) = data.get("mediaKnowledge") else {
        return Ok(Vec::new());
    };
    let knowledge: MediaKnowledge = serde_json::from_value(value.clone())
        .map_err(|error| format!("Validated mediaKnowledge could not be decoded: {error}"))?;
    let mut seeds = Vec::new();

    if let Some(insights) = &knowledge.insights {
        append(
            &mut seeds,
            "text",
            &insights.summary,
            &knowledge.source_revision,
            json!({ "kind": "media", "startMs": 0, "endMs": knowledge.transcript.duration_ms }),
            json!({ "role": "summary" }),
        );
        for (index, chapter) in insights.chapters.iter().enumerate() {
            append(
                &mut seeds,
                "chapter",
                &format!("{}\n{}", chapter.title, chapter.summary),
                &knowledge.source_revision,
                json!({ "kind": "media", "startMs": chapter.start_ms, "endMs": chapter.end_ms }),
                json!({ "role": "chapter", "index": index }),
            );
        }
    }

    for (index, segment) in knowledge.transcript.segments.iter().enumerate() {
        append(
            &mut seeds,
            "transcript",
            &segment.text,
            &knowledge.source_revision,
            json!({ "kind": "media", "startMs": segment.start_ms, "endMs": segment.end_ms }),
            json!({
                "role": "transcript",
                "index": index,
                "transcriptId": knowledge.transcript.id,
                "transcriptRevision": knowledge.transcript.revision,
                "segmentId": segment.id,
                "segmentRevision": segment.revision,
            }),
        );
    }
    Ok(seeds)
}

fn append(
    target: &mut Vec<KnowledgeChunkSeed>,
    kind: &'static str,
    text: &str,
    source_revision: &str,
    location: Value,
    metadata: Value,
) {
    for (part, mut chunk) in
        indexing::chunks(kind, text.trim(), source_revision, location, metadata)
            .into_iter()
            .enumerate()
    {
        if let Some(object) = chunk.metadata.as_object_mut() {
            object.insert("part".into(), Value::from(part as u64));
        }
        target.push(chunk);
    }
}

#[cfg(test)]
mod tests;
