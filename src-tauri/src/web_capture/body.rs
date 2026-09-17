use reqwest::Response;

pub(crate) async fn read_body_prefix(
    response: &mut Response,
    limit: usize,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut body = Vec::with_capacity(limit.min(128 * 1024));
    while body.len() < limit {
        let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? else {
            break;
        };
        let remaining = limit - body.len();
        if chunk.len() > remaining {
            body.extend_from_slice(&chunk[..remaining]);
            return Ok((body, chunk[remaining..].to_vec()));
        }
        body.extend_from_slice(&chunk);
    }
    Ok((body, Vec::new()))
}

pub(crate) async fn read_body_remainder(
    response: &mut Response,
    body: &mut Vec<u8>,
    limit: usize,
) -> Result<(), String> {
    if body.len() > limit {
        return Err("Website image exceeds the size limit".into());
    }
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if body.len() + chunk.len() > limit {
            return Err("Website image exceeds the size limit".into());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(())
}
