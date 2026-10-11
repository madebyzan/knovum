use super::{super::stream::Publisher, Loaded};
use crate::{
    cancellation::Cancellation,
    onnx::{self, Values},
};
use ort::value::DynValue;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

pub(super) fn generate(
    engine: &mut Loaded,
    codes: &[Vec<i32>],
    text: &[i32],
    seed: u64,
    publisher: &mut Publisher,
    cancel: &Cancellation,
) -> Result<(), String> {
    let mut rows = Vec::new();
    let templates = &engine.manifest.prompt_templates;
    for &token in templates
        .user_prompt_prefix_token_ids
        .iter()
        .chain(std::iter::once(&6))
    {
        push_text(&mut rows, token)?;
    }
    for frame in codes {
        rows.push(8);
        rows.extend_from_slice(frame);
    }
    push_text(&mut rows, 7)?;
    for &token in templates
        .user_prompt_after_reference_token_ids
        .iter()
        .chain(text)
        .chain(&templates.assistant_prompt_prefix_token_ids)
        .chain(std::iter::once(&6))
    {
        push_text(&mut rows, token)?;
    }
    let count = rows.len() / 17;
    if count > 2048 {
        return Err("invalid_input: MOSS prompt exceeds its context budget".into());
    }
    let feeds = Values::from([
        (
            "input_ids".into(),
            onnx::tensor(vec![1, count as i64, 17], rows)?,
        ),
        (
            "attention_mask".into(),
            onnx::tensor(vec![1, count as i64], vec![1i32; count])?,
        ),
    ]);
    let mut output = onnx::run(&mut engine.prefill, feeds, cancel)?;
    let mut hidden = last_hidden(onnx::take(&mut output, "global_hidden")?)?;
    let mut past = cache(output)?;
    let mut seen = vec![0i32; 16 * 1024];
    let mut pending = Vec::with_capacity(8);
    let mut random = Pcg64::seed_from_u64(seed);
    for index in 0..1200 {
        cancel.check()?;
        let feeds = Values::from([
            ("global_hidden".into(), onnx::tensor(vec![1, 768], hidden)?),
            (
                "repetition_seen_mask".into(),
                onnx::tensor(vec![1, 16, 1024], seen.clone())?,
            ),
            (
                "assistant_random_u".into(),
                onnx::tensor(vec![1], vec![random.random::<f32>()])?,
            ),
            (
                "audio_random_u".into(),
                onnx::tensor(
                    vec![1, 16],
                    (0..16).map(|_| random.random::<f32>()).collect::<Vec<_>>(),
                )?,
            ),
        ]);
        let mut output = onnx::run(&mut engine.frame, feeds, cancel)?;
        let should_continue = onnx::take(&mut output, "should_continue")?;
        let (_, flag) = should_continue
            .try_extract_tensor::<i32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if flag.len() != 1 || !matches!(flag[0], 0 | 1) {
            return Err("invalid_output: MOSS stop decision differs".into());
        }
        if flag[0] == 0 {
            if !pending.is_empty() {
                engine.codec.publish(&pending, publisher, cancel)?;
            }
            return Ok(());
        }
        let frame = onnx::take(&mut output, "frame_token_ids")?;
        let (_, frame) = frame
            .try_extract_tensor::<i32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if frame.len() != 16 || frame.iter().any(|token| !(0..1024).contains(token)) {
            return Err("invalid_output: MOSS generated an invalid codec frame".into());
        }
        for (channel, &token) in frame.iter().enumerate() {
            seen[channel * 1024 + token as usize] = 1;
        }
        pending.push(frame.to_vec());
        if pending.len() == 8 {
            engine.codec.publish(&pending, publisher, cancel)?;
            pending.clear();
        }
        let mut row = Vec::with_capacity(17);
        row.push(9);
        row.extend_from_slice(frame);
        past.insert("input_ids".into(), onnx::tensor(vec![1, 1, 17], row)?);
        past.insert(
            "past_valid_lengths".into(),
            onnx::tensor(vec![1], vec![(count + index) as i32])?,
        );
        let mut output = onnx::run(&mut engine.decode, past, cancel)?;
        hidden = last_hidden(onnx::take(&mut output, "global_hidden")?)?;
        past = cache(output)?;
    }
    Err("resource_exhausted: MOSS generation reached its 1200-frame limit".into())
}
fn push_text(rows: &mut Vec<i32>, token: i32) -> Result<(), String> {
    if !(0..16384).contains(&token) {
        return Err("model_corrupt: MOSS prompt token is invalid".into());
    }
    rows.push(token);
    rows.extend([1024; 16]);
    Ok(())
}
fn last_hidden(value: DynValue) -> Result<Vec<f32>, String> {
    let (shape, values) = value
        .try_extract_tensor::<f32>()
        .map_err(|error| format!("invalid_output: {error}"))?;
    if shape.len() != 3
        || shape[0] != 1
        || shape[1] < 1
        || shape[1] > 2048
        || shape[2] != 768
        || values.iter().any(|value| !value.is_finite())
    {
        return Err("invalid_output: MOSS global hidden shape differs".into());
    }
    Ok(values[values.len() - 768..].to_vec())
}
fn cache(values: Values) -> Result<Values, String> {
    if values.len() != 24 {
        return Err("invalid_output: MOSS global cache inventory differs".into());
    }
    let mut cache = Values::new();
    for (name, value) in values {
        let suffix = name
            .strip_prefix("present_")
            .ok_or("invalid_output: MOSS global cache name differs")?;
        cache.insert(format!("past_{suffix}"), value);
    }
    Ok(cache)
}
